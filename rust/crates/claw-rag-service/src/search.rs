//! Hybrid search over indexed chunks.

use std::collections::BTreeMap;
use std::path::Path;

use reqwest::Client;

use crate::db::{fts5_available, load_all_indexed, open_db, query_fts_scores, ChunkRow};
use crate::embed::{cosine_similarity, embed_batch, EmbedConfig};
use crate::graph::{
    default_graph_path, suggestions_for_multi_component_query, suggestions_for_query,
};
use crate::{
    generate_wind_inspection_advice, generate_wind_risk_assessment, QueryRequest, QueryResponse,
    RagHit, ScoreBreakdown, SearchMode,
};

pub async fn query_index(
    db_path: &Path,
    client: &Client,
    cfg: &EmbedConfig,
    req: &QueryRequest,
) -> Result<QueryResponse, String> {
    if !db_path.is_file() {
        let graph_suggestions = graph_suggestions(req);
        let mut advice =
            generate_wind_inspection_advice(&req.query, &Vec::new(), &graph_suggestions);
        add_scada_context(&mut advice, req);
        let risk_assessment = generate_wind_risk_assessment(&advice, &[]);
        return Ok(QueryResponse {
            hits: Vec::new(),
            graph_suggestions,
            advice,
            risk_assessment,
            phase: "1-sqlite-no-db".to_string(),
            search_mode: req.search_mode,
            fts5_enabled: false,
        });
    }

    let conn = open_db(db_path)?;
    let fts_enabled = fts5_available(&conn);

    let mut rows = load_all_indexed(&conn)?;
    rows.retain(|row| row_matches_filters(row, req));

    if rows.is_empty() {
        let graph_suggestions = graph_suggestions(req);
        let mut advice =
            generate_wind_inspection_advice(&req.query, &Vec::new(), &graph_suggestions);
        add_scada_context(&mut advice, req);
        let risk_assessment = generate_wind_risk_assessment(&advice, &[]);
        return Ok(QueryResponse {
            hits: Vec::new(),
            graph_suggestions,
            advice,
            risk_assessment,
            phase: "1-sqlite-empty".to_string(),
            search_mode: req.search_mode,
            fts5_enabled: fts_enabled,
        });
    }

    let vector_scores = if uses_vector(req.search_mode) {
        vector_scores(client, cfg, req, &rows).await?
    } else {
        BTreeMap::new()
    };

    let keyword_scores = if uses_keyword(req.search_mode) {
        keyword_scores(&conn, req, &rows)?
    } else {
        BTreeMap::new()
    };
    drop(conn);

    let max_vector = vector_scores
        .values()
        .copied()
        .fold(0.0_f32, f32::max)
        .max(1.0);
    let max_keyword = keyword_scores
        .values()
        .copied()
        .fold(0.0_f32, f32::max)
        .max(1.0);

    let mut scored = rows
        .iter()
        .map(|row| {
            let vector_score = if uses_vector(req.search_mode) {
                normalize_01(*vector_scores.get(&row.id).unwrap_or(&0.0) / max_vector)
            } else {
                0.0
            };
            let keyword_score = if uses_keyword(req.search_mode) {
                normalize_01(*keyword_scores.get(&row.id).unwrap_or(&0.0) / max_keyword)
            } else {
                0.0
            };
            let metadata_score = metadata_score(row, req);
            let final_score = match req.search_mode {
                SearchMode::Vector => vector_score,
                SearchMode::Keyword => keyword_score,
                SearchMode::Hybrid => {
                    0.65 * vector_score + 0.25 * keyword_score + 0.10 * metadata_score
                }
            };
            (
                final_score,
                row,
                ScoreBreakdown {
                    vector_score,
                    keyword_score,
                    metadata_score,
                    final_score,
                },
            )
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.path.cmp(&b.1.path))
    });

    let top = req.top_k.min(64) as usize;
    let hits: Vec<RagHit> = scored
        .into_iter()
        .take(top)
        .map(|(_, row, breakdown)| hit_from_row(row, breakdown))
        .collect();
    let graph_suggestions = graph_suggestions(req);
    let mut advice = generate_wind_inspection_advice(&req.query, &hits, &graph_suggestions);
    add_scada_context(&mut advice, req);
    let risk_assessment = generate_wind_risk_assessment(&advice, &hits);

    Ok(QueryResponse {
        hits,
        graph_suggestions,
        advice,
        risk_assessment,
        phase: "3-hybrid".to_string(),
        search_mode: req.search_mode,
        fts5_enabled: fts_enabled,
    })
}

fn graph_suggestions(req: &QueryRequest) -> Vec<crate::GraphSuggestion> {
    let query_context = contextual_query(req);

    // 优先使用多组件查询逻辑，以支持复合故障场景
    let suggestions = suggestions_for_multi_component_query(&default_graph_path(), &query_context);

    // 如果多组件查询返回空结果，回退到常规查询
    match suggestions {
        Ok(suggestions) if !suggestions.is_empty() => suggestions,
        _ => {
            let graph_query = if let Some(symptom) = req.symptom.as_deref() {
                format!("{query_context} {symptom}")
            } else {
                query_context
            };
            suggestions_for_query(
                &default_graph_path(),
                &graph_query,
                req.domain.as_deref().or(req.component.as_deref()),
                req.equipment.as_deref(),
            )
            .unwrap_or_default()
        }
    }
}

fn contextual_query(req: &QueryRequest) -> String {
    if let Some(scada_context) = req.scada_context.as_deref() {
        let scada_context = scada_context.trim();
        if !scada_context.is_empty() {
            return format!("{}\nSCADA上下文：{}", req.query, scada_context);
        }
    }
    req.query.clone()
}

fn add_scada_context(advice: &mut crate::WindInspectionAdvice, req: &QueryRequest) {
    if let Some(context) = req.scada_context.as_deref() {
        let context = context.trim();
        if !context.is_empty() {
            advice.add_additional_context(format!("SCADA上下文：{context}"));
        }
    }
}

async fn vector_scores(
    client: &Client,
    cfg: &EmbedConfig,
    req: &QueryRequest,
    rows: &[ChunkRow],
) -> Result<BTreeMap<i64, f32>, String> {
    let query_text = contextual_query(req);
    let qvecs = embed_batch(client, cfg, std::slice::from_ref(&query_text)).await?;
    let q = qvecs
        .into_iter()
        .next()
        .ok_or_else(|| "no query embedding".to_string())?;
    let expected = rows[0].vec.len();
    if q.len() != expected {
        return Err(format!(
            "embedding dimension mismatch: index uses dim {} but query embedding has {} (same model/env as ingest required)",
            expected,
            q.len()
        ));
    }
    Ok(rows
        .iter()
        .map(|row| {
            (
                row.id,
                normalize_01((cosine_similarity(&q, &row.vec) + 1.0) / 2.0),
            )
        })
        .collect())
}

fn keyword_scores(
    conn: &rusqlite::Connection,
    req: &QueryRequest,
    rows: &[ChunkRow],
) -> Result<BTreeMap<i64, f32>, String> {
    let mut scores = BTreeMap::new();

    let query_text = contextual_query(req);
    for (chunk_id, score) in query_fts_scores(conn, &query_text, 256)? {
        scores.insert(chunk_id, score);
    }

    for row in rows {
        let fallback = simple_keyword_score(&query_text, &row.text);
        let current = scores.get(&row.id).copied().unwrap_or(0.0);
        if fallback > current {
            scores.insert(row.id, fallback);
        }
    }

    Ok(scores)
}

fn simple_keyword_score(query: &str, text: &str) -> f32 {
    let text_lower = text.to_ascii_lowercase();
    let terms = keyword_terms(query);
    if terms.is_empty() {
        return 0.0;
    }
    let matches = terms
        .iter()
        .filter(|term| text_lower.contains(term.as_str()))
        .count();
    matches as f32 / terms.len() as f32
}

fn keyword_terms(query: &str) -> Vec<String> {
    let lower = query.to_ascii_lowercase();
    let mut terms = lower
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if terms.is_empty() && !query.trim().is_empty() {
        terms.push(query.trim().to_ascii_lowercase());
    }
    if terms.len() == 1 {
        let compact = terms[0].clone();
        for token in ["叶片", "裂纹", "复检", "齿轮箱", "油温", "轴承", "功率曲线"]
        {
            if compact.contains(token) && !terms.iter().any(|term| term == token) {
                terms.push(token.to_string());
            }
        }
    }
    terms
}

fn row_matches_filters(row: &ChunkRow, req: &QueryRequest) -> bool {
    optional_eq(row.domain.as_deref(), req.domain.as_deref())
        && optional_eq(row.equipment.as_deref(), req.equipment.as_deref())
        && optional_eq(row.file_type.as_deref(), req.file_type.as_deref())
        && optional_eq(row.source_type.as_deref(), req.source_type.as_deref())
        && req
            .reserved_media
            .is_none_or(|reserved| row.reserved_media == reserved)
}

fn optional_eq(actual: Option<&str>, expected: Option<&str>) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    actual.is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn metadata_score(row: &ChunkRow, req: &QueryRequest) -> f32 {
    let mut total = 0.0;
    let mut matched = 0.0;

    if req.domain.is_some() {
        total += 1.0;
        if optional_eq(row.domain.as_deref(), req.domain.as_deref()) {
            matched += 1.0;
        }
    }
    if req.equipment.is_some() {
        total += 1.0;
        if optional_eq(row.equipment.as_deref(), req.equipment.as_deref()) {
            matched += 1.0;
        }
    }
    if req.file_type.is_some() {
        total += 1.0;
        if optional_eq(row.file_type.as_deref(), req.file_type.as_deref()) {
            matched += 1.0;
        }
    }
    if req.source_type.is_some() {
        total += 1.0;
        if optional_eq(row.source_type.as_deref(), req.source_type.as_deref()) {
            matched += 1.0;
        }
    }

    if total == 0.0 {
        0.0
    } else {
        matched / total
    }
}

fn hit_from_row(row: &ChunkRow, breakdown: ScoreBreakdown) -> RagHit {
    let source_path = row
        .original_path
        .clone()
        .unwrap_or_else(|| row.path.clone());
    RagHit {
        path: row.path.clone(),
        snippet: truncate_snippet(&row.text, 480),
        score: Some(breakdown.final_score),
        chunk_text: row.text.clone(),
        source_path,
        file_type: row.file_type.clone(),
        domain: row.domain.clone(),
        equipment: row.equipment.clone(),
        source_type: row.source_type.clone(),
        parser_status: row.parser_status.clone(),
        score_breakdown: breakdown,
    }
}

fn uses_vector(mode: SearchMode) -> bool {
    matches!(mode, SearchMode::Vector | SearchMode::Hybrid)
}

fn uses_keyword(mode: SearchMode) -> bool {
    matches!(mode, SearchMode::Keyword | SearchMode::Hybrid)
}

fn normalize_01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn truncate_snippet(s: &str, max_chars: usize) -> String {
    let n = s.chars().count();
    if n <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + "..."
}
