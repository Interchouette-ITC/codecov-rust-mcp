//! `codecov-rust-mcp` - Codecov MCP server over stdio.

use anyhow::Result;
use clap::Parser;
use codecov_rust_mcp::{load_dotenv, CodecovMcp};
use rmcp::{transport::stdio, ServiceExt};

#[derive(Debug, Parser)]
#[command(
    name = "codecov-rust-mcp",
    about = "Codecov MCP server (stdio): coverage totals, miss files, file reports",
    version
)]
struct Cli {}

fn init_logging() {
    // Keep stdio MCP quiet: many hosts treat any stderr line as an error.
    // Default warn; override with RUST_LOG when debugging.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    init_logging();
    let _cli = Cli::parse();
    tracing::info!("codecov-rust-mcp starting (stdio)");
    let server = CodecovMcp;
    let service = server
        .serve(stdio())
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    service
        .waiting()
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    Ok(())
}
