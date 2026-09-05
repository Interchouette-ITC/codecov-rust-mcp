//! MCP server (`rmcp`) for Codecov over stdio.

#![allow(clippy::unused_async)]

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::client::CodecovClient;
use crate::tool_args::{FileReportArgs, MissFilesArgs, RepoArgs};

/// MCP server handle exposing Codecov coverage tools.
#[derive(Clone, Default)]
pub struct CodecovMcp;

fn text_ok(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(text.into())])
}

fn mcp_err(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(msg.into(), None)
}

fn client_from_env() -> Result<CodecovClient, McpError> {
    CodecovClient::from_env().map_err(|err| mcp_err(err.to_string()))
}

fn to_json_text<T: serde::Serialize>(value: &T) -> Result<String, McpError> {
    serde_json::to_string_pretty(value).map_err(|err| mcp_err(err.to_string()))
}

#[tool_router]
impl CodecovMcp {
    /// Returns commit coverage totals for a GitHub repository.
    #[tool(
        description = "Codecov commit coverage totals for a GitHub owner/repo (optional branch or sha)"
    )]
    async fn codecov_totals(
        &self,
        Parameters(RepoArgs {
            owner,
            repo,
            branch,
            sha,
        }): Parameters<RepoArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = client_from_env()?;
        let report = client
            .totals(&owner, &repo, branch.as_deref(), sha.as_deref())
            .await
            .map_err(|err| mcp_err(err.to_string()))?;
        Ok(text_ok(to_json_text(&report.totals)?))
    }

    /// Returns commit totals plus files with missed lines, sorted by misses descending.
    #[tool(
        description = "Codecov miss files for a GitHub owner/repo: totals + files sorted by misses desc + returned count (limit default 30; 0 = uncapped)"
    )]
    async fn codecov_miss_files(
        &self,
        Parameters(MissFilesArgs {
            owner,
            repo,
            branch,
            sha,
            limit,
        }): Parameters<MissFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = client_from_env()?;
        let limit = usize::try_from(limit.unwrap_or(30)).unwrap_or(30);
        let result = client
            .miss_files(&owner, &repo, branch.as_deref(), sha.as_deref(), limit)
            .await
            .map_err(|err| mcp_err(err.to_string()))?;
        Ok(text_ok(to_json_text(&result)?))
    }

    /// Returns line coverage for one repository-relative path.
    #[tool(description = "Codecov line coverage report for one file path in a GitHub owner/repo")]
    async fn codecov_file_report(
        &self,
        Parameters(FileReportArgs {
            owner,
            repo,
            path,
            branch,
            sha,
        }): Parameters<FileReportArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = client_from_env()?;
        let report = client
            .file_report(&owner, &repo, &path, branch.as_deref(), sha.as_deref())
            .await
            .map_err(|err| mcp_err(err.to_string()))?;
        Ok(text_ok(to_json_text(&report)?))
    }
}

/// Serves MCP over stdio until the client disconnects.
///
/// # Errors
///
/// Returns transport or protocol errors from `rmcp`.
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = CodecovMcp;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[tool_handler]
#[allow(clippy::unused_async_trait_impl)]
impl ServerHandler for CodecovMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(rmcp::model::Implementation::new(
                "codecov",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Codecov coverage tools: codecov_totals, codecov_miss_files, codecov_file_report. Requires CODECOV_TOKEN; optional CODECOV_API_URL.",
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_version_matches_crate() {
        let info = CodecovMcp.get_info();
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.server_info.name.as_str(), "codecov");
    }
}
