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
        let mut url = format!(
            "{base}/github/{}/repos/{}/file_report/{}/",
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

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ClientError> {
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
mod tests {
    use super::*;

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
    fn file_report_url_encodes_path() {
        let url = CodecovClient::file_report_url(
            DEFAULT_API_URL,
            "o",
            "r",
            "crates/foo/src/lib.rs",
            Some("dev"),
            None,
        );
        assert!(url.contains("/file_report/"));
        assert!(url.contains("crates%2Ffoo%2Fsrc%2Flib.rs"));
        assert!(url.ends_with("?branch=dev"));
    }

    #[test]
    fn new_stores_trimmed_base() {
        let client = CodecovClient::new("tok", "https://api.codecov.io/api/v2/");
        assert_eq!(client.api_base(), DEFAULT_API_URL);
        assert!(!client.token.is_empty());
    }
}
