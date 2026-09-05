//! `codecov-mcp` - Codecov MCP server over stdio.

use anyhow::Result;
use clap::Parser;
use codecov_mcp::{load_dotenv, run};

#[derive(Debug, Parser)]
#[command(
    name = "codecov-mcp",
    about = "Codecov MCP server (stdio): coverage totals, miss files, file reports",
    version
)]
struct Cli {}

fn init_logging() {
    // Keep stdio MCP quiet: Cursor surfaces any stderr line as [error].
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
    tracing::info!("codecov-mcp starting (stdio)");
    run().await.map_err(|err| anyhow::anyhow!("{err}"))?;
    Ok(())
}
