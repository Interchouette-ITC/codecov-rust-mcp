//! HTTP client for the Codecov API v2.

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use crate::report::{miss_files_result, CoverageReport, MissFilesResult, ReportFile};

/// Encode path/query components; keep unreserved `-_.` but encode `/`.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'.');

/// Default Codecov API v2 base (no trailing slash).
pub const DEFAULT_API_URL: &str = "https://api.codecov.io/api/v2";

/// Errors from Codecov client construction or HTTP calls.
#[derive(Debug)]
pub enum ClientError {
    /// `CODECOV_TOKEN` missing or empty.
    MissingToken,
    /// Underlying HTTP transport failure.
    Http(reqwest::Error),
    /// Non-success HTTP status from Codecov.
    Api { status: u16, body: String },
    /// Response JSON could not be decoded.
    Json(serde_json::Error),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingToken => {
                write!(f, "CODECOV_TOKEN is missing or empty")
            }
            Self::Http(err) => write!(f, "HTTP error: {err}"),
            Self::Api { status, body } => {
                let snippet = truncate(body, 400);
                write!(f, "Codecov API {status}: {snippet}")
            }
            Self::Json(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::MissingToken | Self::Api { .. } => None,
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

/// Reject empty / whitespace-only tokens.
///
/// # Errors
///
/// Returns [`ClientError::MissingToken`] when `s` is empty after trim.
pub fn require_token(s: &str) -> Result<(), ClientError> {
    if s.trim().is_empty() {
        Err(ClientError::MissingToken)
    } else {
        Ok(())
    }
}

/// Codecov API client (Bearer token + base URL).
#[derive(Clone)]
pub struct CodecovClient {
    token: String,
    api_base: String,
    http: reqwest::Client,
}

impl std::fmt::Debug for CodecovClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodecovClient")
            .field("has_token", &!self.token.is_empty())
            .field("api_base", &self.api_base)
            .finish_non_exhaustive()
    }
}

impl CodecovClient {
    /// Builds a client with an explicit token and API base URL.
    ///
    /// # Panics
    ///
    /// Panics only if the default `reqwest::Client` cannot be constructed
    /// (should not happen in normal environments).
    #[must_use]
    pub fn new(token: impl Into<String>, api_base: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        let token = token.into();
        let value = format!("bearer {token}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static("bearer")),
        );
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("reqwest client");
        Self {
            token,
            api_base: trim_trailing_slash(api_base.into()),
            http,
        }
    }

    /// Builds a client from `CODECOV_TOKEN` and optional `CODECOV_API_URL`.
    ///
    /// Loads an optional `.env` file first (does not override existing process env).
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::MissingToken`] when the token env var is unset or empty.
    pub fn from_env() -> Result<Self, ClientError> {
        crate::env_file::load_dotenv();
        let token = std::env::var("CODECOV_TOKEN").unwrap_or_default();
        require_token(&token)?;
        let api_base =
            std::env::var("CODECOV_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        Ok(Self::new(token, api_base))
    }

    /// API base URL without trailing slash.
    #[must_use]
    pub fn api_base(&self) -> &str {
        &self.api_base
    }

    /// Builds the totals URL for a GitHub repo.
    #[must_use]
    pub fn totals_url(
        api_base: &str,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        sha: Option<&str>,
    ) -> String {
        let base = trim_trailing_slash(api_base.to_string());
        let mut url = format!("{base}/github/{}/repos/{}/totals/", enc(owner), enc(repo));
        append_branch_sha(&mut url, branch, sha);
        url
    }

    /// Builds the file report URL for a path inside a GitHub repo.
    #[must_use]
    pub fn file_report_url(
        api_base: &str,
        owner: &str,
        repo: &str,
        path: &str,
        branch: Option<&str>,
        sha: Option<&str>,
    ) -> String {
        let base = trim_trailing_slash(api_base.to_string());
        // No trailing slash after the file path: Codecov treats `parser.rs/` as a
        // different key and returns 404 for line coverage.
        let mut url = format!(
            "{base}/github/{}/repos/{}/file_report/{}",
            enc(owner),
            enc(repo),
            enc(path)
        );
        append_branch_sha(&mut url, branch, sha);
        url
    }

    /// Fetches commit coverage totals (including per-file breakdown).
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Http`], [`ClientError::Api`], or [`ClientError::Json`].
    pub async fn totals(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        sha: Option<&str>,
    ) -> Result<CoverageReport, ClientError> {
        let url = Self::totals_url(&self.api_base, owner, repo, branch, sha);
        self.get_json(&url).await
    }

    /// Files with misses, sorted descending, capped by `limit` (`0` = uncapped).
    ///
    /// # Errors
    ///
    /// Same as [`Self::totals`].
    pub async fn miss_files(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        sha: Option<&str>,
        limit: usize,
    ) -> Result<MissFilesResult, ClientError> {
        let report = self.totals(owner, repo, branch, sha).await?;
        Ok(miss_files_result(&report, limit))
    }

    /// Fetches line coverage for a single file path.
    ///
    /// # Errors
    ///
    /// Same as [`Self::totals`].
    pub async fn file_report(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: Option<&str>,
        sha: Option<&str>,
    ) -> Result<ReportFile, ClientError> {
        let url = Self::file_report_url(&self.api_base, owner, repo, path, branch, sha);
        self.get_json(&url).await
    }

    pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, ClientError> {
        let response = self.http.get(url).send().await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(ClientError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(serde_json::from_str(&body)?)
    }
}

fn enc(s: &str) -> String {
    utf8_percent_encode(s, COMPONENT).to_string()
}

fn trim_trailing_slash(mut s: String) -> String {
    while s.ends_with('/') {
        s.pop();
    }
    s
}

fn append_branch_sha(url: &mut String, branch: Option<&str>, sha: Option<&str>) {
    let mut sep = '?';
    if let Some(branch) = branch.filter(|b| !b.is_empty()) {
        url.push(sep);
        url.push_str("branch=");
        url.push_str(&enc(branch));
        sep = '&';
    }
    if let Some(sha) = sha.filter(|s| !s.is_empty()) {
        url.push(sep);
        url.push_str("sha=");
        url.push_str(&enc(sha));
    }
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)] // env mutex must span HTTP mocks
mod tests {
    use super::*;
    use crate::test_env::{env_lock, restore_env};
    use serde_json::json;
    use std::error::Error;
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
            "line_coverage": [[1, 1], [2, 0]]
        })
    }

    #[test]
    fn require_token_rejects_empty() {
        assert!(matches!(require_token(""), Err(ClientError::MissingToken)));
        assert!(matches!(
            require_token("   "),
            Err(ClientError::MissingToken)
        ));
        assert!(require_token("tok").is_ok());
    }

    #[test]
    fn client_error_display_and_source() {
        let missing = ClientError::MissingToken;
        assert!(missing.to_string().contains("CODECOV_TOKEN"));
        assert!(missing.source().is_none());

        let api = ClientError::Api {
            status: 500,
            body: "short".into(),
        };
        assert_eq!(api.to_string(), "Codecov API 500: short");
        assert!(api.source().is_none());

        let long_body: String = "x".repeat(450);
        let truncated = ClientError::Api {
            status: 502,
            body: long_body,
        };
        let display = truncated.to_string();
        assert!(display.starts_with("Codecov API 502: "));
        assert_eq!(
            display.chars().count(),
            "Codecov API 502: ".chars().count() + 400
        );

        let json_err = ClientError::from(serde_json::from_str::<()>("x").unwrap_err());
        assert!(json_err.to_string().contains("JSON error"));
        assert!(json_err.source().is_some());
    }

    #[tokio::test]
    async fn client_error_http_display_and_from() {
        let err = reqwest::Client::new()
            .get("http://127.0.0.1:9/")
            .send()
            .await
            .expect_err("connection should fail");
        let http = ClientError::from(err);
        assert!(http.to_string().contains("HTTP error"));
        assert!(http.source().is_some());
    }

    #[test]
    fn debug_hides_token() {
        let client = CodecovClient::new("secret-token", DEFAULT_API_URL);
        let debug = format!("{client:?}");
        assert!(debug.contains("has_token: true"));
        assert!(!debug.contains("secret-token"));
        assert!(debug.contains(DEFAULT_API_URL));
    }

    #[test]
    fn totals_url_encodes_and_queries() {
        let url = CodecovClient::totals_url(
            DEFAULT_API_URL,
            "Interchouette-ITC",
            "rustashop",
            Some("dev"),
            None,
        );
        assert_eq!(
            url,
            "https://api.codecov.io/api/v2/github/Interchouette-ITC/repos/rustashop/totals/?branch=dev"
        );
    }

    #[test]
    fn totals_url_with_sha() {
        let url = CodecovClient::totals_url(
            "https://api.codecov.io/api/v2/",
            "o",
            "r",
            Some("main"),
            Some("abc123"),
        );
        assert_eq!(
            url,
            "https://api.codecov.io/api/v2/github/o/repos/r/totals/?branch=main&sha=abc123"
        );
    }

    #[test]
    fn file_report_url_encodes_path_without_trailing_slash() {
        let url = CodecovClient::file_report_url(
            DEFAULT_API_URL,
            "o",
            "r",
            "crates/foo/src/lib.rs",
            Some("dev"),
            None,
        );
        assert_eq!(
            url,
            "https://api.codecov.io/api/v2/github/o/repos/r/file_report/crates%2Ffoo%2Fsrc%2Flib.rs?branch=dev"
        );
        assert!(
            !url.contains("lib.rs/"),
            "trailing slash after path breaks Codecov file_report lookups: {url}"
        );
    }

    #[test]
    fn new_stores_trimmed_base() {
        let client = CodecovClient::new("tok", "https://api.codecov.io/api/v2/");
        assert_eq!(client.api_base(), DEFAULT_API_URL);
        assert!(!client.token.is_empty());
    }

    #[tokio::test]
    async fn totals_miss_files_and_file_report_ok() {
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
        let report = client
            .totals("o", "r", Some("dev"), None)
            .await
            .expect("totals");
        assert_eq!(report.totals.misses, 2);
        assert_eq!(report.files.len(), 1);

        let misses = client
            .miss_files("o", "r", None, Some("abc"), 10)
            .await
            .expect("miss_files");
        assert_eq!(misses.returned, 1);
        assert_eq!(misses.files[0].name, "a.rs");

        let file = client
            .file_report("o", "r", "a.rs", Some("dev"), None)
            .await
            .expect("file_report");
        assert_eq!(file.name, "a.rs");
        assert_eq!(file.line_coverage.len(), 2);
    }

    #[tokio::test]
    async fn get_json_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let client = CodecovClient::new("tok", server.uri());
        let url = format!("{}/github/o/repos/r/totals/", client.api_base());
        match client.get_json::<serde_json::Value>(&url).await {
            Err(ClientError::Api { status, body }) => {
                assert_eq!(status, 500);
                assert_eq!(body, "boom");
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_json_invalid_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
            .mount(&server)
            .await;

        let client = CodecovClient::new("tok", server.uri());
        let url = format!("{}/github/o/repos/r/totals/", client.api_base());
        let json_err = client.get_json::<serde_json::Value>(&url).await;
        assert!(matches!(json_err, Err(ClientError::Json(_))));
    }

    #[tokio::test]
    async fn get_json_http_transport_error() {
        let client = CodecovClient::new("tok", "http://127.0.0.1:9");
        let err = client
            .get_json::<serde_json::Value>("http://127.0.0.1:9/github/o/repos/r/totals/")
            .await;
        assert!(matches!(err, Err(ClientError::Http(_))));
    }

    #[tokio::test]
    async fn from_env_uses_token_and_api_url() {
        let _guard = env_lock();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r".*/totals/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_totals_json()))
            .mount(&server)
            .await;

        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        std::env::set_var("CODECOV_TOKEN", "env-tok");
        std::env::set_var("CODECOV_API_URL", server.uri());

        let client = CodecovClient::from_env().expect("from_env");
        assert_eq!(client.api_base(), server.uri().trim_end_matches('/'));
        let report = client.totals("o", "r", None, None).await.expect("totals");
        assert_eq!(report.totals.hits, 8);

        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }

    #[test]
    fn new_falls_back_on_invalid_auth_header() {
        // Newlines are invalid in HTTP header values.
        let client = CodecovClient::new("bad\ntoken", "https://api.codecov.io/api/v2/");
        assert_eq!(client.api_base(), DEFAULT_API_URL);
        assert!(!client.token.is_empty());
    }

    #[test]
    fn from_env_defaults_api_url() {
        let _guard = env_lock();
        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        std::env::set_var("CODECOV_TOKEN", "default-url-tok");
        std::env::remove_var("CODECOV_API_URL");
        let client = CodecovClient::from_env().expect("from_env");
        assert_eq!(client.api_base(), DEFAULT_API_URL);
        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }

    #[tokio::test]
    async fn miss_files_propagates_totals_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500).set_body_string("nope"))
            .mount(&server)
            .await;
        let client = CodecovClient::new("tok", server.uri());
        let err = client.miss_files("o", "r", None, None, 10).await;
        assert!(matches!(err, Err(ClientError::Api { .. })));
    }

    #[test]
    fn from_env_missing_token() {
        let _guard = env_lock();
        let prev_token = std::env::var("CODECOV_TOKEN").ok();
        let prev_url = std::env::var("CODECOV_API_URL").ok();
        // Empty (not removed): load_dotenv must not refill from a checkout `.env`.
        std::env::set_var("CODECOV_TOKEN", "");
        std::env::remove_var("CODECOV_API_URL");
        assert!(matches!(
            CodecovClient::from_env(),
            Err(ClientError::MissingToken)
        ));
        restore_env("CODECOV_TOKEN", prev_token);
        restore_env("CODECOV_API_URL", prev_url);
    }
}
