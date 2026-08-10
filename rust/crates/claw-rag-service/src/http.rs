use std::{path::PathBuf, sync::Arc};

use claw_rag_service::{MemoryService, SqliteWindMemory};

use crate::{handlers::rag_router, resolve_embed_config, state::AppState};

pub(crate) async fn serve_http(
    db: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = resolve_embed_config()?;
    let memory = SqliteWindMemory::new(db.clone());
    if let Err(e) = memory.ensure_schema() {
        eprintln!("memory: skip schema init: {e}");
    }
    let state = Arc::new(AppState {
        db_path: db,
        client: reqwest::Client::new(),
        cfg,
        memory,
        json_memory: MemoryService::new(MemoryService::default_root()),
    });

    let app = rag_router(state.clone());

    let port: u16 = std::env::var("CLAW_RAG_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8787);
    let host: std::net::IpAddr = std::env::var("CLAW_RAG_HOST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    let addr = std::net::SocketAddr::from((host, port));
    eprintln!(
        "claw-rag-service db={} listen=http://{addr}",
        state.db_path.display()
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
