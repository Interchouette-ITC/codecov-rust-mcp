//! JSON-schema parameter structs for rmcp `Parameters<T>` tool handlers.

use schemars::JsonSchema;
use serde::Deserialize;

/// Shared owner/repo/branch/sha selectors for Codecov repo tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoArgs {
    /// GitHub owner or org (e.g. `Interchouette-ITC`).
    pub owner: String,
    /// Repository name (e.g. `rustashop`).
    pub repo: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    pub branch: Option<String>,
    /// Commit SHA; when set, takes precedence over `branch` on the API.
    #[serde(default)]
    pub sha: Option<String>,
}

/// Arguments for `codecov_miss_files`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MissFilesArgs {
    /// GitHub owner or org.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    pub branch: Option<String>,
    /// Commit SHA.
    #[serde(default)]
    pub sha: Option<String>,
    /// Max files to return (default 30). Use `0` for uncapped.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Arguments for `codecov_file_report`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileReportArgs {
    /// GitHub owner or org.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Repository-relative file path.
    pub path: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    pub branch: Option<String>,
    /// Commit SHA.
    #[serde(default)]
    pub sha: Option<String>,
}
