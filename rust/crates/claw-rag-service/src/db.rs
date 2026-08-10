//! `SQLite` storage for chunks and embedding vectors.

use std::path::Path;

use rusqlite::{params, Connection};

const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    text TEXT NOT NULL,
    UNIQUE(path, ordinal)
);
CREATE TABLE IF NOT EXISTS embeddings (
    chunk_id INTEGER PRIMARY KEY,
    dim INTEGER NOT NULL,
    vec BLOB NOT NULL,
    FOREIGN KEY (chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS files (
    path TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    mtime_ms INTEGER NOT NULL,
    indexed_at_ms INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS document_records (
    path TEXT PRIMARY KEY,
    file_type TEXT NOT NULL,
    domain TEXT NOT NULL,
    equipment TEXT NOT NULL,
    source_type TEXT NOT NULL,
    original_path TEXT NOT NULL,
    parser_status TEXT NOT NULL,
    reserved_media INTEGER NOT NULL,
    metadata_json TEXT NOT NULL,
    FOREIGN KEY (path) REFERENCES files(path) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_chunks_path ON chunks(path);
CREATE INDEX IF NOT EXISTS idx_document_records_domain ON document_records(domain);
CREATE INDEX IF NOT EXISTS idx_document_records_equipment ON document_records(equipment);
";

const FTS_SCHEMA: &str = r"
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    chunk_id UNINDEXED,
    path UNINDEXED,
    text
);
";

pub fn open_db(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        r"
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
",
    )
    .map_err(|e| e.to_string())?;
    conn.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
    let _ = conn.execute_batch(FTS_SCHEMA);

    Ok(conn)
}

#[allow(dead_code)]
pub fn truncate_index(conn: &Connection) -> Result<(), String> {
    let _ = conn.execute("DELETE FROM chunks_fts", []);
    conn.execute_batch("DELETE FROM embeddings; DELETE FROM chunks; DELETE FROM files;")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn file_is_unchanged(
    conn: &Connection,
    path: &str,
    content_hash: &str,
    size_bytes: i64,
    mtime_ms: i64,
) -> Result<bool, String> {
    let mut stmt = conn
        .prepare("SELECT content_hash, size_bytes, mtime_ms FROM files WHERE path=?1 LIMIT 1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![path]).map_err(|e| e.to_string())?;
    if let Some(r) = rows.next().map_err(|e| e.to_string())? {
        let h: String = r.get(0).map_err(|e| e.to_string())?;
        let sz: i64 = r.get(1).map_err(|e| e.to_string())?;
        let mt: i64 = r.get(2).map_err(|e| e.to_string())?;
        return Ok(h == content_hash && sz == size_bytes && mt == mtime_ms);
    }
    Ok(false)
}

pub fn indexed_chunk_count_for_path(conn: &Connection, path: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE path=?1",
        params![path],
        |r| r.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn document_record_exists(conn: &Connection, path: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT 1 FROM document_records WHERE path=?1 LIMIT 1",
        params![path],
        |_| Ok(()),
    )
    .map(|()| true)
    .or_else(|e| {
        if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
            Ok(false)
        } else {
            Err(e.to_string())
        }
    })
}

pub fn embedding_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM embeddings", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn document_record_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM document_records", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn upsert_file_meta(
    conn: &Connection,
    path: &str,
    content_hash: &str,
    size_bytes: i64,
    mtime_ms: i64,
    indexed_at_ms: i64,
) -> Result<(), String> {
    conn.execute(
        r"
INSERT INTO files(path, content_hash, size_bytes, mtime_ms, indexed_at_ms)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(path) DO UPDATE SET
  content_hash=excluded.content_hash,
  size_bytes=excluded.size_bytes,
  mtime_ms=excluded.mtime_ms,
  indexed_at_ms=excluded.indexed_at_ms
",
        params![path, content_hash, size_bytes, mtime_ms, indexed_at_ms],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_file_and_chunks(conn: &Connection, path: &str) -> Result<(), String> {
    // Delete chunks first (embeddings cascade); then remove file meta.
    let _ = conn.execute("DELETE FROM chunks_fts WHERE path=?1", params![path]);
    conn.execute("DELETE FROM chunks WHERE path=?1", params![path])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM document_records WHERE path=?1", params![path])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM files WHERE path=?1", params![path])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_document_record(
    conn: &Connection,
    path: &str,
    metadata: &crate::document::DocumentMetadata,
) -> Result<(), String> {
    let metadata_json = serde_json::to_string(metadata).map_err(|e| e.to_string())?;
    conn.execute(
        r"
INSERT INTO document_records(
    path,
    file_type,
    domain,
    equipment,
    source_type,
    original_path,
    parser_status,
    reserved_media,
    metadata_json
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
ON CONFLICT(path) DO UPDATE SET
  file_type=excluded.file_type,
  domain=excluded.domain,
  equipment=excluded.equipment,
  source_type=excluded.source_type,
  original_path=excluded.original_path,
  parser_status=excluded.parser_status,
  reserved_media=excluded.reserved_media,
  metadata_json=excluded.metadata_json
",
        params![
            path,
            metadata.file_type,
            metadata.domain,
            metadata.equipment,
            metadata.source_type,
            metadata.original_path,
            metadata.parser_status,
            if metadata.reserved_media {
                1_i64
            } else {
                0_i64
            },
            metadata_json
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_all_files(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT path FROM files")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn insert_chunk(
    conn: &Connection,
    path: &str,
    ordinal: i32,
    text: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO chunks (path, ordinal, text) VALUES (?1, ?2, ?3)",
        params![path, ordinal, text],
    )
    .map_err(|e| e.to_string())?;
    let chunk_id = conn.last_insert_rowid();
    let _ = conn.execute(
        "INSERT INTO chunks_fts (chunk_id, path, text) VALUES (?1, ?2, ?3)",
        params![chunk_id, path, text],
    );
    Ok(chunk_id)
}

pub fn insert_embedding(
    conn: &Connection,
    chunk_id: i64,
    dim: usize,
    vec: &[f32],
) -> Result<(), String> {
    let bytes = f32_slice_to_blob(vec);
    let dim_i64 = i64::try_from(dim).map_err(|_| "embedding dim too large".to_string())?;
    conn.execute(
        "INSERT INTO embeddings (chunk_id, dim, vec) VALUES (?1, ?2, ?3)",
        params![chunk_id, dim_i64, bytes],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn f32_slice_to_blob(v: &[f32]) -> Vec<u8> {
    let mut b = Vec::with_capacity(v.len() * 4);
    for x in v {
        b.extend_from_slice(&x.to_le_bytes());
    }
    b
}

pub fn blob_to_f32_vec(blob: &[u8], dim: usize) -> Option<Vec<f32>> {
    if blob.len() != dim * 4 {
        return None;
    }
    let mut v = Vec::with_capacity(dim);
    for chunk in blob.chunks_exact(4) {
        v.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Some(v)
}

#[derive(Debug, Clone)]
pub struct ChunkRow {
    pub id: i64,
    pub path: String,
    pub text: String,
    pub vec: Vec<f32>,
    pub file_type: Option<String>,
    pub domain: Option<String>,
    pub equipment: Option<String>,
    pub source_type: Option<String>,
    pub original_path: Option<String>,
    pub parser_status: Option<String>,
    pub reserved_media: bool,
}

pub fn load_all_indexed(conn: &Connection) -> Result<Vec<ChunkRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                c.id,
                c.path,
                c.text,
                e.dim,
                e.vec,
                d.file_type,
                d.domain,
                d.equipment,
                d.source_type,
                d.original_path,
                d.parser_status,
                d.reserved_media
             FROM chunks c
             INNER JOIN embeddings e ON e.chunk_id = c.id
             LEFT JOIN document_records d ON d.path = c.path",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while let Some(r) = rows.next().map_err(|e| e.to_string())? {
        let id: i64 = r.get(0).map_err(|e| e.to_string())?;
        let path: String = r.get(1).map_err(|e| e.to_string())?;
        let text: String = r.get(2).map_err(|e| e.to_string())?;
        let dim: i64 = r.get(3).map_err(|e| e.to_string())?;
        let blob: Vec<u8> = r.get(4).map_err(|e| e.to_string())?;
        let reserved_media = r
            .get::<_, Option<i64>>(11)
            .map_err(|e| e.to_string())?
            .unwrap_or(0)
            != 0;
        let dim = usize::try_from(dim).map_err(|_| "invalid embedding dim in db".to_string())?;
        let Some(vec) = blob_to_f32_vec(&blob, dim) else {
            continue;
        };
        out.push(ChunkRow {
            id,
            path,
            text,
            vec,
            file_type: r.get(5).map_err(|e| e.to_string())?,
            domain: r.get(6).map_err(|e| e.to_string())?,
            equipment: r.get(7).map_err(|e| e.to_string())?,
            source_type: r.get(8).map_err(|e| e.to_string())?,
            original_path: r.get(9).map_err(|e| e.to_string())?,
            parser_status: r.get(10).map_err(|e| e.to_string())?,
            reserved_media,
        });
    }
    Ok(out)
}

pub fn fts5_available(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='chunks_fts' LIMIT 1",
        [],
        |_| Ok(()),
    )
    .is_ok()
}

pub fn query_fts_scores(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(i64, f32)>, String> {
    if !fts5_available(conn) {
        return Ok(Vec::new());
    }
    let match_query = fts_match_query(query);
    if match_query.is_empty() {
        return Ok(Vec::new());
    }
    let limit_i64 = i64::try_from(limit).map_err(|_| "fts limit too large".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT chunk_id, bm25(chunks_fts) AS rank
             FROM chunks_fts
             WHERE chunks_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![match_query, limit_i64], |row| {
            let chunk_id: i64 = row.get(0)?;
            let rank: f32 = row.get(1)?;
            Ok((chunk_id, 1.0 / (1.0 + rank.abs())))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

fn fts_match_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|term| {
            term.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c as u32 > 127)
                .collect::<String>()
        })
        .filter(|term| !term.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join(" OR ")
}

pub fn chunk_count(conn: &Connection) -> Result<i64, String> {
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(n)
}
