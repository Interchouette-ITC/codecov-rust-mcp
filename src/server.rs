//! MCP server (`rmcp`) for Codecov over stdio.

#![allow(clippy::unused_async)]

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};

use crate::client::CodecovClient;
use crate::tool_args::{FileReportArgs, MissFilesArgs, RepoArgs};

/// MCP server handle exposing Codecov coverage tools.
#[derive(Clone, Default)]
pub struct CodecovMcp;

pub(crate) fn text_ok(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(text.into())])
}

pub(crate) fn mcp_err(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(msg.into(), None)
}

pub(crate) fn client_from_env() -> Result<CodecovClient, McpError> {
    CodecovClient::from_env().map_err(|err| mcp_err(err.to_string()))
}

pub(crate) fn to_json_text<T: serde::Serialize>(value: &T) -> Result<String, McpError> {
    serde_json::to_string_pretty(value).map_err(|err| mcp_err(err.to_string()))
}

/// Fetches totals via an existing client and returns pretty JSON text.
pub(crate) async fn totals_with_client(
    client: &CodecovClient,
    args: RepoArgs,
) -> Result<CallToolResult, McpError> {
    let report = client
        .totals(
            &args.owner,
            &args.repo,
            args.branch.as_deref(),
            args.sha.as_deref(),
        )
        .await
        .map_err(|err| mcp_err(err.to_string()))?;
    Ok(text_ok(to_json_text(&report.totals)?))
}

/// Fetches miss files via an existing client and returns pretty JSON text.
pub(crate) async fn miss_files_with_client(
    client: &CodecovClient,
    args: MissFilesArgs,
) -> Result<CallToolResult, McpError> {
    let limit = usize::try_from(args.limit.unwrap_or(30)).unwrap_or(30);
    let result = client
        .miss_files(
            &args.owner,
            &args.repo,
            args.branch.as_deref(),
            args.sha.as_deref(),
            limit,
        )
        .await
        .map_err(|err| mcp_err(err.to_string()))?;
    Ok(text_ok(to_json_text(&result)?))
}

/// Fetches a file report via an existing client and returns pretty JSON text.
pub(crate) async fn file_report_with_client(
    client: &CodecovClient,
    args: FileReportArgs,
) -> Result<CallToolResult, McpError> {
    let report = client
        .file_report(
            &args.owner,
            &args.repo,
            &args.path,
            args.branch.as_deref(),
            args.sha.as_deref(),
        )
        .await
        .map_err(|err| mcp_err(err.to_string()))?;
    Ok(text_ok(to_json_text(&report)?))
}

#[tool_router]
impl CodecovMcp {
    /// Returns commit coverage totals for a GitHub repository.
    #[tool(
        description = "Codecov commit coverage totals for a GitHub owner/repo (optional branch or sha)"
    )]
    async fn codecov_totals(
        &self,
        Parameters(args): Parameters<RepoArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = client_from_env()?;
        totals_with_client(&client, args).await
    }

    /// Returns commit totals plus files with missed lines, sorted by misses descending.
    #[tool(
        description = "Codecov miss files for a GitHub owner/repo: totals + files sorted by misses desc + returned count (limit default 30; 0 = uncapped)"
    )]
    async fn codecov_miss_files(
        &self,
        Parameters(args): Parameters<MissFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = client_from_env()?;
        miss_files_with_client(&client, args).await
    }

    /// Returns line coverage for one repository-relative path.
    #[tool(description = "Codecov line coverage report for one file path in a GitHub owner/repo")]
    async fn codecov_file_report(
        &self,
        Parameters(args): Parameters<FileReportArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = client_from_env()?;
        file_report_with_client(&client, args).await
    }
}

#[cfg(test)]
impl CodecovMcp {
    async fn call_totals(&self, args: RepoArgs) -> Result<CallToolResult, McpError> {
        self.codecov_totals(Parameters(args)).await
    }

    async fn call_miss_files(&self, args: MissFilesArgs) -> Result<CallToolResult, McpError> {
        self.codecov_miss_files(Parameters(args)).await
    }

    async fn call_file_report(&self, args: FileReportArgs) -> Result<CallToolResult, McpError> {
        self.codecov_file_report(Parameters(args)).await
    }
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
#[allow(clippy::await_holding_lock)] // env mutex must span HTTP mocks
mod tests {
    use super::*;
    use crate::test_env::{env_lock, restore_env};
    use serde_json::json;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_totals_json() -> serde_json::Value {
        json!({
            "totals": {
                "files": 1,
                "lines": 10,
                "hits": 8,
                "misses": 2,
                "partials": 0,
                "coverage": 80.0,
                "branches": 0,
                "methods": 0
            },
            "files": [{
                "name": "a.rs",
                "totals": {
                    "files": 0,
                    "lines": 10,
                    "hits": 8,
                    "misses": 2,
                    "partials": 0,
                    "coverage": 80.0,
                    "branches": 0,
                    "methods": 0
                },
                "line_coverage": []
            }]
        })
    }

    fn sample_file_report_json() -> serde_json::Value {
        json!({
            "name": "src/lib.rs",
            "totals": {
                "files": 0,
                "lines": 4,
                "hits": 3,
                "misses": 1,
                "partials": 0,
                "coverage": 75.0,
                "branches": 0,
                "methods": 0
            },
            "line_coverage": [[1, 1]]
        })
    }

    #[test]
    fn mcp_server_version_matches_crate() {
        let info = CodecovMcp.get_info();
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.server_info.name.as_str(), "codecov");
    }

    #[test]
    fn text_ok_and_mcp_err_helpers() {
        let ok = text_ok("hello");
        assert!(ok.is_error.is_none() || ok.is_error == Some(false));
        let err = mcp_err("bad");
        assert!(err.to_string().contains("bad"));
    }

    struct Boom;

    impl serde::Serialize for Boom {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("boom"))
        }
    }

    #[test]
    fn to_json_text_ok_and_err() {
        let text = to_json_text(&json!({"a": 1})).expect("json");
        assert!(text.contains('1'));
        let err = to_json_text(&Boom);
        assert!(err.is_err());
    }

    #[test]
    fn client_from_env_missing_token() {
        let _guard = env_lock();
        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        std::env::set_var("CODECOV_TOKEN", "");
        std::env::remove_var("CODECOV_API_URL");
        assert!(client_from_env().is_err());
        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }

    #[tokio::test]
    async fn client_from_env_success_with_wiremock() {
        let _guard = env_lock();
        let server = MockServer::start().await;
        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        std::env::set_var("CODECOV_TOKEN", "server-tok");
        std::env::set_var("CODECOV_API_URL", server.uri());
        let client = client_from_env().expect("client");
        assert_eq!(client.api_base(), server.uri().trim_end_matches('/'));
        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }

    #[tokio::test]
    async fn helpers_with_client_cover_paths() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r".*/totals/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_totals_json()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex(r".*/file_report/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_file_report_json()))
            .mount(&server)
            .await;

        let client = CodecovClient::new("tok", server.uri());
        let totals = totals_with_client(
            &client,
            RepoArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: Some("dev".into()),
                sha: None,
            },
        )
        .await
        .expect("totals");
        assert!(totals.is_error.is_none() || totals.is_error == Some(false));

        let misses = miss_files_with_client(
            &client,
            MissFilesArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
                limit: Some(5),
            },
        )
        .await
        .expect("misses");
        assert!(misses.is_error.is_none() || misses.is_error == Some(false));

        let file = file_report_with_client(
            &client,
            FileReportArgs {
                owner: "o".into(),
                repo: "r".into(),
                path: "src/lib.rs".into(),
                branch: Some("dev".into()),
                sha: None,
            },
        )
        .await
        .expect("file");
        assert!(file.is_error.is_none() || file.is_error == Some(false));
    }

    #[tokio::test]
    async fn helpers_map_api_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503).set_body_string("down"))
            .mount(&server)
            .await;
        let client = CodecovClient::new("tok", server.uri());
        let args = RepoArgs {
            owner: "o".into(),
            repo: "r".into(),
            branch: None,
            sha: None,
        };
        assert!(totals_with_client(&client, args).await.is_err());
        assert!(miss_files_with_client(
            &client,
            MissFilesArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
                limit: Some(1),
            },
        )
        .await
        .is_err());
        assert!(file_report_with_client(
            &client,
            FileReportArgs {
                owner: "o".into(),
                repo: "r".into(),
                path: "a.rs".into(),
                branch: None,
                sha: None,
            },
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn tools_fail_without_token() {
        let _guard = env_lock();
        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        std::env::set_var("CODECOV_TOKEN", "");
        std::env::remove_var("CODECOV_API_URL");

        let mcp = CodecovMcp;
        assert!(mcp
            .call_totals(RepoArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
            })
            .await
            .is_err());
        assert!(mcp
            .call_miss_files(MissFilesArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
                limit: None,
            })
            .await
            .is_err());
        assert!(mcp
            .call_file_report(FileReportArgs {
                owner: "o".into(),
                repo: "r".into(),
                path: "a.rs".into(),
                branch: None,
                sha: None,
            })
            .await
            .is_err());

        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }

    #[tokio::test]
    async fn miss_files_limit_none_defaults() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r".*/totals/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_totals_json()))
            .mount(&server)
            .await;
        let client = CodecovClient::new("tok", server.uri());
        let ok = miss_files_with_client(
            &client,
            MissFilesArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
                limit: None,
            },
        )
        .await
        .expect("misses");
        assert!(ok.is_error.is_none() || ok.is_error == Some(false));
    }

    #[tokio::test]
    async fn tools_invoke_via_env_client() {
        let _guard = env_lock();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r".*/totals/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_totals_json()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex(r".*/file_report/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_file_report_json()))
            .mount(&server)
            .await;

        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        std::env::set_var("CODECOV_TOKEN", "tool-tok");
        std::env::set_var("CODECOV_API_URL", server.uri());

        let mcp = CodecovMcp;
        let totals = mcp
            .call_totals(RepoArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
            })
            .await
            .expect("totals tool");
        assert!(totals.is_error.is_none() || totals.is_error == Some(false));

        let misses = mcp
            .call_miss_files(MissFilesArgs {
                owner: "o".into(),
                repo: "r".into(),
                branch: None,
                sha: None,
                limit: None,
            })
            .await
            .expect("misses tool");
        assert!(misses.is_error.is_none() || misses.is_error == Some(false));

        let file = mcp
            .call_file_report(FileReportArgs {
                owner: "o".into(),
                repo: "r".into(),
                path: "src/lib.rs".into(),
                branch: None,
                sha: None,
            })
            .await
            .expect("file tool");
        assert!(file.is_error.is_none() || file.is_error == Some(false));

        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }
}
