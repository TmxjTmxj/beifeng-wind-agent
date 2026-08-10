//! SQLite-backed memory for wind fault interactions.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FaultRecord {
    pub id: String,
    pub turbine_id: Option<String>,
    pub component: String,
    pub symptom: String,
    pub risk_level: String,
    pub created_at: String,
    pub query: String,
}

pub trait WindMemory: Send + Sync {
    fn record_fault_query(&self, record: &FaultRecord) -> Result<()>;
    fn query_recent_faults(&self, component: &str, limit: usize) -> Result<Vec<FaultRecord>>;
    fn query_turbine_history(&self, turbine_id: &str) -> Result<Vec<FaultRecord>>;
}

#[derive(Debug, Clone)]
pub struct SqliteWindMemory {
    pub db_path: PathBuf,
}

impl SqliteWindMemory {
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    pub fn ensure_schema(&self) -> Result<()> {
        let conn = open_memory_db(&self.db_path)?;
        ensure_fault_schema(&conn)?;
        Ok(())
    }
}

impl WindMemory for SqliteWindMemory {
    fn record_fault_query(&self, record: &FaultRecord) -> Result<()> {
        let conn = open_memory_db(&self.db_path)?;
        ensure_fault_schema(&conn)?;
        conn.execute(
            r"
INSERT INTO fault_queries(
    id, turbine_id, component, symptom, risk_level, query, created_at
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT(id) DO UPDATE SET
  turbine_id=excluded.turbine_id,
  component=excluded.component,
  symptom=excluded.symptom,
  risk_level=excluded.risk_level,
  query=excluded.query,
  created_at=excluded.created_at
",
            params![
                record.id,
                record.turbine_id,
                record.component,
                record.symptom,
                record.risk_level,
                record.query,
                record.created_at
            ],
        )?;
        Ok(())
    }

    fn query_recent_faults(&self, component: &str, limit: usize) -> Result<Vec<FaultRecord>> {
        let conn = open_memory_db(&self.db_path)?;
        ensure_fault_schema(&conn)?;
        let mut stmt = conn.prepare(
            r"
SELECT id, turbine_id, component, symptom, risk_level, created_at, query
FROM fault_queries
WHERE component = ?1
ORDER BY created_at DESC
LIMIT ?2
",
        )?;
        let rows = stmt.query_map(
            params![component, limit.max(1) as i64],
            fault_record_from_row,
        )?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("query recent fault records")
    }

    fn query_turbine_history(&self, turbine_id: &str) -> Result<Vec<FaultRecord>> {
        let conn = open_memory_db(&self.db_path)?;
        ensure_fault_schema(&conn)?;
        let mut stmt = conn.prepare(
            r"
SELECT id, turbine_id, component, symptom, risk_level, created_at, query
FROM fault_queries
WHERE turbine_id = ?1
ORDER BY created_at DESC
LIMIT 50
",
        )?;
        let rows = stmt.query_map(params![turbine_id], fault_record_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("query turbine fault history")
    }
}

#[must_use]
pub fn new_fault_record_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("fault-{millis}")
}

#[must_use]
pub fn now_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    secs.to_string()
}

pub fn summarize_recent_faults(records: &[FaultRecord]) -> Option<String> {
    if records.is_empty() {
        return None;
    }
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for record in records {
        *counts.entry(record.symptom.clone()).or_default() += 1;
    }
    let mut symptoms = counts.into_iter().collect::<Vec<_>>();
    symptoms.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let summary = symptoms
        .into_iter()
        .take(3)
        .map(|(symptom, count)| format!("{symptom}({count}次)"))
        .collect::<Vec<_>>()
        .join("，");
    Some(format!("历史同部件故障高频项：{summary}"))
}

fn open_memory_db(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create memory db dir {}", parent.display()))?;
        }
    }
    Connection::open(path).with_context(|| format!("open memory db {}", path.display()))
}

fn ensure_fault_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r"
CREATE TABLE IF NOT EXISTS fault_queries (
    id TEXT PRIMARY KEY,
    turbine_id TEXT,
    component TEXT NOT NULL,
    symptom TEXT,
    risk_level TEXT,
    query TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_fault_queries_component_created
ON fault_queries(component, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_fault_queries_turbine_created
ON fault_queries(turbine_id, created_at DESC);
",
    )
    .context("ensure memory fault schema")
}

fn fault_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FaultRecord> {
    Ok(FaultRecord {
        id: row.get(0)?,
        turbine_id: row.get(1)?,
        component: row.get(2)?,
        symptom: row.get(3)?,
        risk_level: row.get(4)?,
        created_at: row.get(5)?,
        query: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn records_and_queries_recent_faults() {
        let dir = tempdir().unwrap();
        let memory = SqliteWindMemory::new(dir.path().join("wind.sqlite"));
        memory
            .record_fault_query(&FaultRecord {
                id: "1".to_string(),
                turbine_id: Some("T-01".to_string()),
                component: "Gearbox".to_string(),
                symptom: "油温升高".to_string(),
                risk_level: "High".to_string(),
                created_at: "2026-06-05T10:00:00Z".to_string(),
                query: "齿轮箱油温升高".to_string(),
            })
            .unwrap();

        let recent = memory.query_recent_faults("Gearbox", 5).unwrap();
        assert_eq!(recent.len(), 1);
        assert!(summarize_recent_faults(&recent)
            .unwrap()
            .contains("油温升高"));

        let history = memory.query_turbine_history("T-01").unwrap();
        assert_eq!(history[0].component, "Gearbox");
    }
}
