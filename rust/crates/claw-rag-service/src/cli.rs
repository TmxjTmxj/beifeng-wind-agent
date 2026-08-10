use std::path::PathBuf;

use clap::{Parser, Subcommand};
use claw_rag_service::{default_graph_path, default_reports_dir, SearchMode};

#[derive(Parser)]
#[command(
    name = "claw-rag-service",
    about = "Workspace RAG index + HTTP query API"
)]
pub(crate) struct Cli {
    #[arg(long, default_value = "beifeng/config/wind_rules.toml")]
    pub(crate) config: PathBuf,
    #[command(subcommand)]
    pub(crate) command: Option<Cmd>,
}

#[derive(Subcommand)]
pub(crate) enum Cmd {
    /// Run HTTP server (default when no subcommand).
    Serve(ServeArgs),
    /// Index a workspace into `SQLite` (calls embedding API).
    Ingest(IngestArgs),
    /// Read SCADA CSV files and print the latest anomaly context.
    IngestScada(IngestScadaArgs),
    /// Query the RAG index from the CLI.
    Query(QueryArgs),
    /// Query the lightweight wind fault graph.
    GraphQuery(GraphQueryArgs),
    /// Query RAG + fault graph and generate rule-based wind inspection advice.
    AdviceQuery(AdviceQueryArgs),
    /// Query RAG + fault graph and generate rule-based wind risk assessment.
    RiskQuery(RiskQueryArgs),
    /// Run the Wind Fault Analysis workflow.
    FaultAnalysis(FaultAnalysisArgs),
    /// Generate a Markdown Wind O&M report from fault analysis.
    ReportGenerate(ReportGenerateArgs),
    /// Dispatch a skill and format skill-specific output.
    SkillQuery(SkillQueryArgs),
}

#[derive(Parser)]
pub(crate) struct ServeArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
}

#[derive(Parser)]
pub(crate) struct IngestArgs {
    /// Workspace roots to ingest. Repeat `--workspace` to ingest multiple repos (cross-repo RAG).
    #[arg(short, long)]
    pub(crate) workspace: Vec<PathBuf>,
    /// Wind Knowledge Hub root to ingest. Defaults to ./knowledge_base when no workspace is set and the directory exists.
    #[arg(long, env = "WIND_KNOWLEDGE_BASE")]
    pub(crate) knowledge_base: Option<PathBuf>,
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
}

#[derive(Parser)]
pub(crate) struct IngestScadaArgs {
    #[arg(long, default_value = "beifeng/data/scada_csv")]
    pub(crate) data_dir: PathBuf,
    #[arg(long)]
    pub(crate) turbine_id: Option<String>,
}

#[derive(Parser)]
pub(crate) struct QueryArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
    #[arg(long)]
    pub(crate) query: String,
    #[arg(long)]
    pub(crate) domain: Option<String>,
    #[arg(long)]
    pub(crate) equipment: Option<String>,
    #[arg(long)]
    pub(crate) file_type: Option<String>,
    #[arg(long)]
    pub(crate) source_type: Option<String>,
    #[arg(long)]
    pub(crate) reserved_media: Option<bool>,
    #[arg(long, default_value = "hybrid")]
    pub(crate) mode: SearchMode,
    #[arg(long, default_value_t = 8)]
    pub(crate) top_k: u32,
}

#[derive(Parser)]
pub(crate) struct GraphQueryArgs {
    #[arg(long, default_value_os_t = default_graph_path())]
    pub(crate) graph: PathBuf,
    #[arg(long)]
    pub(crate) component: Option<String>,
    #[arg(long)]
    pub(crate) symptom: Option<String>,
    #[arg(long)]
    pub(crate) risk_level: Option<String>,
    #[arg(long)]
    pub(crate) inspection_method: Option<String>,
    #[arg(long, default_value_t = 5)]
    pub(crate) limit: usize,
}

#[derive(Parser)]
pub(crate) struct AdviceQueryArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
    #[arg(long, default_value_os_t = default_graph_path())]
    pub(crate) graph: PathBuf,
    #[arg(long)]
    pub(crate) query: String,
    #[arg(long)]
    pub(crate) component: Option<String>,
    #[arg(long)]
    pub(crate) symptom: Option<String>,
    #[arg(long, default_value = "hybrid")]
    pub(crate) mode: SearchMode,
    #[arg(long, default_value_t = 8)]
    pub(crate) top_k: u32,
}

#[derive(Parser)]
pub(crate) struct RiskQueryArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
    #[arg(long, default_value_os_t = default_graph_path())]
    pub(crate) graph: PathBuf,
    #[arg(long)]
    pub(crate) query: String,
    #[arg(long)]
    pub(crate) component: Option<String>,
    #[arg(long)]
    pub(crate) symptom: Option<String>,
    #[arg(long, default_value = "hybrid")]
    pub(crate) mode: SearchMode,
    #[arg(long, default_value_t = 8)]
    pub(crate) top_k: u32,
}

#[derive(Parser)]
pub(crate) struct FaultAnalysisArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
    #[arg(long, default_value_os_t = default_graph_path())]
    pub(crate) graph: PathBuf,
    #[arg(long)]
    pub(crate) problem: String,
    #[arg(long)]
    pub(crate) component: Option<String>,
    #[arg(long)]
    pub(crate) symptom: Option<String>,
    #[arg(long, default_value = "hybrid")]
    pub(crate) mode: SearchMode,
    #[arg(long, default_value_t = 8)]
    pub(crate) top_k: u32,
}

#[derive(Parser)]
pub(crate) struct ReportGenerateArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
    #[arg(long, default_value_os_t = default_graph_path())]
    pub(crate) graph: PathBuf,
    #[arg(long)]
    pub(crate) problem: String,
    #[arg(long)]
    pub(crate) component: Option<String>,
    #[arg(long)]
    pub(crate) symptom: Option<String>,
    #[arg(long, default_value = "inspection_report")]
    pub(crate) report_type: String,
    #[arg(long)]
    pub(crate) title: Option<String>,
    #[arg(long, default_value = "hybrid")]
    pub(crate) mode: SearchMode,
    #[arg(long, default_value_t = 8)]
    pub(crate) top_k: u32,
    #[arg(long, default_value_os_t = default_reports_dir())]
    pub(crate) reports_dir: PathBuf,
}

#[derive(Parser)]
pub(crate) struct SkillQueryArgs {
    #[arg(long, env = "CLAW_RAG_DB", default_value = "beifeng/data/wind.sqlite")]
    pub(crate) db: PathBuf,
    #[arg(long, default_value_os_t = default_graph_path())]
    pub(crate) graph: PathBuf,
    #[arg(long)]
    pub(crate) query: String,
    #[arg(long)]
    pub(crate) component: Option<String>,
    #[arg(long)]
    pub(crate) symptom: Option<String>,
    #[arg(long, default_value = "hybrid")]
    pub(crate) mode: SearchMode,
    #[arg(long, default_value_t = 8)]
    pub(crate) top_k: u32,
}
