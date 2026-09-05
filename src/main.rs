//! `codecov-rust-mcp` - Codecov MCP server (stdio or Streamable HTTP).
//!
//! ```bash
//! codecov-rust-mcp
//! codecov-rust-mcp --http
//! codecov-rust-mcp --http --listen 0.0.0.0:8690
//! MCP_HTTP=true codecov-rust-mcp
//! ```

use anyhow::Result;
use clap::Parser;
use codecov_rust_mcp::{
    load_dotenv,
    server::{run, run_http, DEFAULT_HTTP_LISTEN},
};

#[derive(Debug, Parser)]
#[command(
    name = "codecov-rust-mcp",
    about = "Codecov MCP server (stdio or Streamable HTTP): coverage totals, miss files, file reports",
    version
)]
struct Cli {
    /// Serve Streamable HTTP instead of stdio.
    #[arg(
        long,
        env = "MCP_HTTP",
        value_parser = clap::builder::BoolishValueParser::new()
    )]
    http: bool,

    /// HTTP bind address when `--http` is set (also: `CODECOV_MCP_ADDR`).
    #[arg(long, env = "CODECOV_MCP_ADDR", default_value = DEFAULT_HTTP_LISTEN)]
    listen: String,
}

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
    let cli = Cli::parse();

    if cli.http {
        tracing::info!(addr = %cli.listen, "codecov-rust-mcp starting (HTTP)");
        run_http(&cli.listen)
            .await
            .map_err(|err| anyhow::anyhow!("{err}"))?;
    } else {
        tracing::info!("codecov-rust-mcp starting (stdio)");
        run().await.map_err(|err| anyhow::anyhow!("{err}"))?;
    }
    Ok(())
}
