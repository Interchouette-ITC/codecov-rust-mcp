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
    /// Slash-delimited repository-relative file path; dot segments and control characters are rejected.
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
        validate_repository_relative_path(&self.path)
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

fn validate_repository_relative_path(path: &str) -> Result<(), String> {
    validate_nonempty("path", path)?;
    let bytes = path.as_bytes();
    let drive_absolute = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');
    let invalid = path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || drive_absolute
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."));
    if invalid {
        return Err(
            "path must be a slash-delimited repository-relative path without control characters or empty, `.` or `..` segments"
                .into(),
        );
    }
    Ok(())
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
    fn validates_file_paths_without_normalizing_them() {
        let args = |path: &str| FileReportArgs {
            owner: "owner".into(),
            repo: "repo".into(),
            path: path.into(),
            branch: None,
            sha: None,
        };
        assert!(args("src/dir with spaces/λ.rs").validate().is_ok());

        for path in [
            "/absolute/path",
            "C:/absolute/path",
            "../outside",
            "src/../outside",
            "src/./file.rs",
            "src//file.rs",
            "src/file.rs/",
            r"src\file.rs",
            "src/\nfile.rs",
        ] {
            assert!(
                args(path).validate().is_err(),
                "accepted invalid path {path:?}"
            );
        }
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
