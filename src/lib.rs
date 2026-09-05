//! Codecov MCP library: HTTP client and rmcp tools (stdio or Streamable HTTP).

pub mod client;
pub mod env_file;
pub mod report;
pub mod server;
pub mod tool_args;

#[cfg(test)]
mod test_env;

pub use env_file::load_dotenv;
pub use server::{run, run_http, CodecovMcp, DEFAULT_HTTP_LISTEN};
