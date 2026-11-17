//! JSON-schema parameter structs for rmcp `Parameters<T>` tool handlers.

use schemars::JsonSchema;
use serde::Deserialize;

// Spell out Unicode White_Space: ECMAScript \s differs from Rust's trim().
const NON_WHITESPACE_PATTERN: &str =
    r"[^\u0009-\u000D\u0020\u0085\u00A0\u1680\u2000-\u200A\u2028\u2029\u202F\u205F\u3000]";
const REPOSITORY_PATH_PATTERN: &str = r"^(?!/)(?![A-Za-z]:/)(?!.*\\)(?!.*[\x00-\x1F\x7F-\x9F])(?![\u0009-\u000D\u0020\u0085\u00A0\u1680\u2000-\u200A\u2028\u2029\u202F\u205F\u3000]*$)(?:(?!\.{1,2}(?:/|$))[^/]+)(?:/(?!\.{1,2}(?:/|$))[^/]+)*$";

/// Shared owner/repo/branch/sha selectors for Codecov repo tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoArgs {
    /// GitHub owner or org (e.g. `Interchouette-ITC`).
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub owner: String,
    /// Repository name (e.g. `rustashop`).
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub repo: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub branch: Option<String>,
    /// Commit SHA; when set, takes precedence over `branch` on the API.
    #[serde(default)]
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
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
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub owner: String,
    /// Repository name.
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub repo: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub branch: Option<String>,
    /// Commit SHA.
    #[serde(default)]
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
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
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub owner: String,
    /// Repository name.
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub repo: String,
    /// Slash-delimited repository-relative path; absolute paths, empty segments, dot segments, backslashes, and control characters are rejected.
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = REPOSITORY_PATH_PATTERN))]
    pub path: String,
    /// Branch head to query when `sha` is omitted.
    #[serde(default)]
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub branch: Option<String>,
    /// Commit SHA.
    #[serde(default)]
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub sha: Option<String>,
}

impl FileReportArgs {
    /// Validates all file-report selectors before making an API request.
    ///
    /// # Errors
    ///
    /// Returns an error when a required selector is empty or whitespace-only, or the path is not repository-relative.
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
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
    pub owner: String,
    /// Repository name (e.g. `rangular`).
    #[schemars(length(min = 1))]
    #[schemars(regex(pattern = NON_WHITESPACE_PATTERN))]
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
            " ",
        ] {
            assert!(
                args(path).validate().is_err(),
                "accepted invalid path {path:?}"
            );
        }
    }

    #[test]
    fn schemas_include_string_and_pull_id_constraints() {
        for schema in [
            schemars::schema_for!(RepoArgs),
            schemars::schema_for!(MissFilesArgs),
            schemars::schema_for!(FileReportArgs),
            schemars::schema_for!(PullReportArgs),
        ] {
            let schema = serde_json::to_value(schema).expect("serialize tool args schema");
            for field in ["owner", "repo", "branch", "sha"] {
                let Some(property) = schema["properties"].get(field) else {
                    continue;
                };
                assert_eq!(property["minLength"], 1, "{field}");
                assert_eq!(property["pattern"], NON_WHITESPACE_PATTERN, "{field}");
                if matches!(field, "branch" | "sha") {
                    assert_eq!(property["type"], serde_json::json!(["string", "null"]));
                    assert!(!schema["required"]
                        .as_array()
                        .expect("required fields")
                        .contains(&serde_json::json!(field)));
                }
            }
        }

        let pull_schema = serde_json::to_value(schemars::schema_for!(PullReportArgs))
            .expect("serialize pull args schema");
        assert_eq!(pull_schema["properties"]["pull_id"]["minimum"], 1);

        let file_schema = serde_json::to_value(schemars::schema_for!(FileReportArgs))
            .expect("serialize file args schema");
        assert_eq!(file_schema["properties"]["path"]["minLength"], 1);
        assert_eq!(
            file_schema["properties"]["path"]["pattern"],
            REPOSITORY_PATH_PATTERN
        );
    }

    #[test]
    fn schema_patterns_match_runtime_whitespace_validation() {
        let selector = fancy_regex::Regex::new(NON_WHITESPACE_PATTERN).expect("selector pattern");
        let path = fancy_regex::Regex::new(REPOSITORY_PATH_PATTERN).expect("path pattern");
        let whitespace = "\u{0009}\u{000A}\u{000B}\u{000C}\u{000D}\u{0020}\u{0085}\u{00A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}";
        let mut values = vec![
            String::new(),
            whitespace.into(),
            "owner".into(),
            " feature/branch ".into(),
            "\u{180E}".into(),
            "\u{200B}".into(),
            "\u{FEFF}".into(),
            "src/λ.rs".into(),
        ];
        for character in whitespace.chars() {
            values.push(character.to_string());
            values.push(format!("{character}value{character}"));
        }
        for value in values {
            assert_eq!(
                selector.is_match(&value).expect("match selector"),
                validate_nonempty("selector", &value).is_ok(),
                "selector: {value:?}"
            );
            assert_eq!(
                path.is_match(&value).expect("match path"),
                validate_repository_relative_path(&value).is_ok(),
                "path: {value:?}"
            );
        }
    }
}
