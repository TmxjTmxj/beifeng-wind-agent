//! `claw-rag-service` — HTTP API + `ingest` subcommand.

mod cli;
mod handlers;
mod http;
mod state;

use std::path::PathBuf;

use clap::Parser;
use claw_rag_service::{
    default_knowledge_base_path, dispatch_skill, document_record_count, format_skill_output,
    generate_fault_analysis_result, generate_wind_inspection_advice, generate_wind_report,
    generate_wind_risk_assessment, open_db, query_fault_graph_file, query_index, run_ingest,
    set_global_wind_rules_config, suggestions_for_multi_component_query, EmbedConfig,
    FaultAnalysisInput, GraphQuery, MemoryService, QueryRequest, QueryResponse,
    ReportHistoryRecord, ScadaCsvConnector, SkillQueryResponse, WindReportGenerateInput,
    WindRulesConfig,
};

use crate::{
    cli::{Cli, Cmd, FaultAnalysisArgs},
    http::serve_http,
};

pub(crate) fn resolve_embed_config() -> Result<EmbedConfig, String> {
    if let Some(c) = EmbedConfig::mock_from_env() {
        return Ok(c);
    }
    EmbedConfig::from_env()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load `.env` if present (walks up parent directories).
    // This is a convenience for local development; CI/production should set real env vars.
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let rules = WindRulesConfig::load(&cli.config).unwrap_or_else(|e| {
        eprintln!(
            "wind-rules: use embedded defaults, could not load {}: {e}",
            cli.config.display()
        );
        WindRulesConfig::default_embedded()
    });
    set_global_wind_rules_config(rules);

    match cli.command {
        Some(Cmd::Ingest(a)) => {
            let cfg = resolve_embed_config()?;
            let client = reqwest::Client::new();
            let mut roots = a.workspace;
            if let Some(knowledge_base) = a.knowledge_base {
                roots.push(knowledge_base);
            } else if roots.is_empty() && default_knowledge_base_path().is_dir() {
                roots.push(default_knowledge_base_path());
            }
            let st = run_ingest(&roots, &a.db, &cfg, &client).await?;
            let conn = open_db(&a.db)?;
            let document_records = document_record_count(&conn)?;
            eprintln!(
                "ingest: files={} chunks={} embeddings={} document_records={}",
                st.files_indexed, st.chunks_total, st.embeddings_written, document_records
            );
        }
        Some(Cmd::IngestScada(a)) => {
            let connector = ScadaCsvConnector {
                data_dir: a.data_dir,
            };
            let points = connector.load_latest(a.turbine_id.as_deref())?;
            println!("{}", connector.to_context_summary(&points));
        }
        Some(Cmd::Query(a)) => {
            let cfg = resolve_embed_config()?;
            let client = reqwest::Client::new();
            let response = query_index(
                &a.db,
                &client,
                &cfg,
                &QueryRequest {
                    query: a.query,
                    top_k: a.top_k,
                    turbine_id: None,
                    component: None,
                    symptom: None,
                    scada_context: None,
                    domain: a.domain,
                    equipment: a.equipment,
                    file_type: a.file_type,
                    source_type: a.source_type,
                    reserved_media: a.reserved_media,
                    search_mode: a.mode,
                },
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Some(Cmd::GraphQuery(a)) => {
            let response = query_fault_graph_file(
                &a.graph,
                &GraphQuery {
                    component: a.component,
                    symptom: a.symptom,
                    risk_level: a.risk_level,
                    inspection_method: a.inspection_method,
                    limit: a.limit,
                },
            )?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Some(Cmd::AdviceQuery(a)) => {
            let graph_response = query_fault_graph_file(
                &a.graph,
                &GraphQuery {
                    component: a.component.clone(),
                    symptom: a.symptom.or_else(|| Some(a.query.clone())),
                    risk_level: None,
                    inspection_method: None,
                    limit: 3,
                },
            )?;
            let hits = if a.db.is_file() {
                match resolve_embed_config() {
                    Ok(cfg) => {
                        let client = reqwest::Client::new();
                        query_index(
                            &a.db,
                            &client,
                            &cfg,
                            &QueryRequest {
                                query: a.query.clone(),
                                top_k: a.top_k,
                                turbine_id: None,
                                component: a.component.clone(),
                                symptom: None,
                                scada_context: None,
                                domain: a.component,
                                equipment: None,
                                file_type: None,
                                source_type: None,
                                reserved_media: None,
                                search_mode: a.mode,
                            },
                        )
                        .await?
                        .hits
                    }
                    Err(e) => {
                        eprintln!("advice-query: skip RAG hits: {e}");
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };
            let advice = generate_wind_inspection_advice(&a.query, &hits, &graph_response.matches);
            println!("{}", serde_json::to_string_pretty(&advice)?);
        }
        Some(Cmd::RiskQuery(a)) => {
            let graph_response = query_fault_graph_file(
                &a.graph,
                &GraphQuery {
                    component: a.component.clone(),
                    symptom: a.symptom.or_else(|| Some(a.query.clone())),
                    risk_level: None,
                    inspection_method: None,
                    limit: 3,
                },
            )?;
            let hits = if a.db.is_file() {
                match resolve_embed_config() {
                    Ok(cfg) => {
                        let client = reqwest::Client::new();
                        query_index(
                            &a.db,
                            &client,
                            &cfg,
                            &QueryRequest {
                                query: a.query.clone(),
                                top_k: a.top_k,
                                turbine_id: None,
                                component: a.component.clone(),
                                symptom: None,
                                scada_context: None,
                                domain: a.component,
                                equipment: None,
                                file_type: None,
                                source_type: None,
                                reserved_media: None,
                                search_mode: a.mode,
                            },
                        )
                        .await?
                        .hits
                    }
                    Err(e) => {
                        eprintln!("risk-query: skip RAG hits: {e}");
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };
            let advice = generate_wind_inspection_advice(&a.query, &hits, &graph_response.matches);
            let risk_assessment = generate_wind_risk_assessment(&advice, &hits);
            println!("{}", serde_json::to_string_pretty(&risk_assessment)?);
        }
        Some(Cmd::FaultAnalysis(a)) => {
            let input = FaultAnalysisInput {
                problem: a.problem.clone(),
                component: a.component.clone(),
                symptom: a.symptom.clone(),
            };
            let response = fault_analysis_query_response(&a).await?;
            let result = generate_fault_analysis_result(&input, &response);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(Cmd::ReportGenerate(a)) => {
            let report_type = a.report_type.clone();
            let input = FaultAnalysisInput {
                problem: a.problem.clone(),
                component: a.component.clone(),
                symptom: a.symptom.clone(),
            };
            let fault_args = FaultAnalysisArgs {
                db: a.db,
                graph: a.graph,
                problem: a.problem.clone(),
                component: a.component.clone(),
                symptom: a.symptom.clone(),
                mode: a.mode,
                top_k: a.top_k,
            };
            let response = fault_analysis_query_response(&fault_args).await?;
            let fault_analysis = generate_fault_analysis_result(&input, &response);
            let report = generate_wind_report(
                &WindReportGenerateInput {
                    problem: a.problem,
                    component: a.component,
                    symptom: a.symptom,
                    report_type: Some(a.report_type),
                    title: a.title,
                },
                &fault_analysis,
                &a.reports_dir,
            )?;
            append_report_history(&report, fault_args.problem.as_str(), None, &report_type);
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Some(Cmd::SkillQuery(a)) => {
            let skill_type = dispatch_skill(&a.query, a.component.as_deref());
            let input = FaultAnalysisInput {
                problem: a.query.clone(),
                component: a.component.clone(),
                symptom: a.symptom.clone(),
            };
            let fault_args = FaultAnalysisArgs {
                db: a.db,
                graph: a.graph,
                problem: a.query.clone(),
                component: a.component.clone(),
                symptom: a.symptom.clone(),
                mode: a.mode,
                top_k: a.top_k,
            };
            let response = fault_analysis_query_response(&fault_args).await?;
            let fault_analysis = generate_fault_analysis_result(&input, &response);
            let skill_output = format_skill_output(
                &skill_type,
                &fault_analysis,
                &response.advice,
                &response.risk_assessment,
                &response.graph_suggestions,
            );
            let result = SkillQueryResponse {
                skill_name: skill_type.to_string(),
                query: a.query,
                component: input.component,
                fault_analysis,
                skill_output,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(Cmd::Serve(s)) => {
            serve_http(s.db).await?;
        }
        None => {
            let db = PathBuf::from(
                std::env::var("CLAW_RAG_DB").unwrap_or_else(|_| "beifeng/data/wind.sqlite".into()),
            );
            serve_http(db).await?;
        }
    }

    Ok(())
}

fn append_report_history(
    report: &claw_rag_service::WindReportGeneration,
    problem: &str,
    turbine_id: Option<&str>,
    report_type: &str,
) {
    let service = MemoryService::new(MemoryService::default_root());
    let record = ReportHistoryRecord {
        report_id: format!("report-{}", claw_rag_service::now_timestamp()),
        turbine_id: turbine_id.unwrap_or("UNKNOWN").to_string(),
        report_type: report_type.to_string(),
        problem: problem.to_string(),
        risk_level: report.fault_analysis.risk_level.clone(),
        report_path: report.report_path.clone(),
        created_at: claw_rag_service::now_timestamp(),
        notes: Some("auto-recorded by report generation runtime".to_string()),
    };
    if let Err(e) = service.append_report_record(&record) {
        eprintln!("memory-json: skip append report history: {e}");
    }
}

async fn fault_analysis_query_response(
    a: &FaultAnalysisArgs,
) -> Result<QueryResponse, Box<dyn std::error::Error + Send + Sync>> {
    if a.db.is_file() {
        match resolve_embed_config() {
            Ok(cfg) => {
                let client = reqwest::Client::new();
                return Ok(query_index(
                    &a.db,
                    &client,
                    &cfg,
                    &QueryRequest {
                        query: a.problem.clone(),
                        top_k: a.top_k,
                        turbine_id: None,
                        component: a.component.clone(),
                        symptom: a.symptom.clone(),
                        scada_context: None,
                        domain: a.component.clone(),
                        equipment: None,
                        file_type: None,
                        source_type: None,
                        reserved_media: None,
                        search_mode: a.mode,
                    },
                )
                .await?);
            }
            Err(e) => {
                eprintln!("fault-analysis: skip RAG hits: {e}");
            }
        }
    }

    // 使用多组件查询来识别 query 中的所有相关组件
    let graph_suggestions = suggestions_for_multi_component_query(&a.graph, &a.problem)?;
    let advice = generate_wind_inspection_advice(&a.problem, &[], &graph_suggestions);
    let risk_assessment = generate_wind_risk_assessment(&advice, &[]);
    Ok(QueryResponse {
        hits: Vec::new(),
        graph_suggestions,
        advice,
        risk_assessment,
        phase: "fault-analysis-multi-component".to_string(),
        search_mode: a.mode,
        fts5_enabled: false,
    })
}
