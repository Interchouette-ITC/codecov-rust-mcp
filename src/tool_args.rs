//! JSON-schema parameter structs for rmcp `Parameters<T>` tool handlers.

use schemars::JsonSchema;
use serde::Deserialize;

/// Shared owner/repo/branch/sha selectors for Codecov repo tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoArgs {
    /// GitHub owner or org (e.g. `Interchouette-ITC`).
    #[schemars(length(min = 1))]
    pub owner: String,
    /// Repository name (e.g. `rustashop`).
    #[schemars(length(min = 1))]
    pub repo: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    #[schemars(length(min = 1))]
    pub branch: Option<String>,
    /// Commit SHA; when set, takes precedence over `branch` on the API.
    #[serde(default)]
    #[schemars(length(min = 1))]
    pub sha: Option<String>,
}

impl RepoArgs {
    /// Validates all repository selectors before making an API request.
    ///
    /// # Errors
    ///
    /// Returns an error when a required selector is empty or whitespace-only.
    pub fn validate(&self) -> Result<(), String> {
        validate_repo_fields(
            &self.owner,
            &self.repo,
            self.branch.as_deref(),
            self.sha.as_deref(),
        )
    }
}

/// Arguments for `codecov_miss_files`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MissFilesArgs {
    /// GitHub owner or org.
    #[schemars(length(min = 1))]
    pub owner: String,
    /// Repository name.
    #[schemars(length(min = 1))]
    pub repo: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    #[schemars(length(min = 1))]
    pub branch: Option<String>,
    /// Commit SHA.
    #[serde(default)]
    #[schemars(length(min = 1))]
    pub sha: Option<String>,
    /// Max files to return (default 30). Use `0` for uncapped; this limits output, not the Codecov report fetched.
    #[serde(default)]
    pub limit: Option<u32>,
}

impl MissFilesArgs {
    /// Validates all repository selectors before making an API request.
    ///
    /// # Errors
    ///
    /// Returns an error when a required selector is empty or whitespace-only.
    pub fn validate(&self) -> Result<(), String> {
        validate_repo_fields(
            &self.owner,
            &self.repo,
            self.branch.as_deref(),
            self.sha.as_deref(),
        )
    }
}

/// Arguments for `codecov_file_report`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileReportArgs {
    /// GitHub owner or org.
    #[schemars(length(min = 1))]
    pub owner: String,
    /// Repository name.
    #[schemars(length(min = 1))]
    pub repo: String,
    /// Repository-relative file path.
    #[schemars(length(min = 1))]
    pub path: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    #[schemars(length(min = 1))]
    pub branch: Option<String>,
    /// Commit SHA.
    #[serde(default)]
    #[schemars(length(min = 1))]
    pub sha: Option<String>,
}

impl FileReportArgs {
    /// Validates all file-report selectors before making an API request.
    ///
    /// # Errors
    ///
    /// Returns an error when a required selector is empty or whitespace-only.
    pub fn validate(&self) -> Result<(), String> {
        validate_repo_fields(
            &self.owner,
            &self.repo,
            self.branch.as_deref(),
            self.sha.as_deref(),
        )?;
        validate_nonempty("path", &self.path)
    }
}

/// Arguments for `codecov_pr_report`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PullReportArgs {
    /// GitHub owner or org (e.g. `Interchouette-ITC`).
    #[schemars(length(min = 1))]
    pub owner: String,
    /// Repository name (e.g. `rangular`).
    #[schemars(length(min = 1))]
    pub repo: String,
    /// Pull request number from the PR URL (e.g. `97`).
    #[schemars(range(min = 1))]
    pub pull_id: u64,
}

impl PullReportArgs {
    /// Validates the pull request selector before making an API request.
    ///
    /// # Errors
    ///
    /// Returns an error when the repository selectors are empty or the pull ID is zero.
    pub fn validate(&self) -> Result<(), String> {
        validate_repo_fields(&self.owner, &self.repo, None, None)?;
        if self.pull_id == 0 {
            return Err("pull_id must be greater than zero".into());
        }
        Ok(())
    }
}

fn validate_repo_fields(
    owner: &str,
    repo: &str,
    branch: Option<&str>,
    sha: Option<&str>,
) -> Result<(), String> {
    validate_nonempty("owner", owner)?;
    validate_nonempty("repo", repo)?;
    if let Some(branch) = branch {
        validate_nonempty("branch", branch)?;
    }
    if let Some(sha) = sha {
        validate_nonempty("sha", sha)?;
    }
    Ok(())
}

fn validate_nonempty(name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{name} must not be empty"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_required_and_optional_selectors() {
        let args = RepoArgs {
            owner: " ".into(),
            repo: "repo".into(),
            branch: Some("main".into()),
            sha: None,
        };
        assert_eq!(args.validate().unwrap_err(), "owner must not be empty");

        let args = RepoArgs {
            owner: "owner".into(),
            repo: "repo".into(),
            branch: Some(" ".into()),
            sha: None,
        };
        assert_eq!(args.validate().unwrap_err(), "branch must not be empty");

        let args = RepoArgs {
            owner: "owner".into(),
            repo: "repo".into(),
            branch: None,
            sha: Some(String::new()),
        };
        assert_eq!(args.validate().unwrap_err(), "sha must not be empty");

        let args = FileReportArgs {
            owner: "owner".into(),
            repo: "repo".into(),
            path: String::new(),
            branch: None,
            sha: None,
        };
        assert_eq!(args.validate().unwrap_err(), "path must not be empty");

        let args = PullReportArgs {
            owner: "owner".into(),
            repo: "repo".into(),
            pull_id: 0,
        };
        assert_eq!(
            args.validate().unwrap_err(),
            "pull_id must be greater than zero"
        );
    }

    #[test]
    fn schemas_include_string_and_pull_id_constraints() {
        let repo_schema = serde_json::to_value(schemars::schema_for!(RepoArgs))
            .expect("serialize repo args schema");
        assert_eq!(repo_schema["properties"]["owner"]["minLength"], 1);
        assert_eq!(repo_schema["properties"]["repo"]["minLength"], 1);
        for field in ["branch", "sha"] {
            assert_eq!(repo_schema["properties"][field]["minLength"], 1);
        }

        let pull_schema = serde_json::to_value(schemars::schema_for!(PullReportArgs))
            .expect("serialize pull args schema");
        assert_eq!(pull_schema["properties"]["pull_id"]["minimum"], 1);
    }
}
