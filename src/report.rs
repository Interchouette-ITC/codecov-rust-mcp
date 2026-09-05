//! Coverage report types and miss-file helpers.

use serde::{Deserialize, Serialize};

/// Aggregated coverage totals for a commit or file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportTotals {
    #[serde(default)]
    pub files: i64,
    #[serde(default)]
    pub lines: i64,
    #[serde(default)]
    pub hits: i64,
    #[serde(default)]
    pub misses: i64,
    #[serde(default)]
    pub partials: i64,
    #[serde(default)]
    pub coverage: Option<serde_json::Value>,
    #[serde(default)]
    pub branches: i64,
    #[serde(default)]
    pub methods: i64,
}

/// One file entry inside a coverage report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFile {
    pub name: String,
    pub totals: ReportTotals,
    #[serde(default)]
    pub line_coverage: Vec<serde_json::Value>,
}

/// Full Codecov totals response (commit totals plus per-file breakdown).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub totals: ReportTotals,
    #[serde(default)]
    pub files: Vec<ReportFile>,
    #[serde(default)]
    pub commit_file_url: Option<String>,
}

/// Compact miss summary for agent-facing miss lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissFile {
    pub name: String,
    pub misses: i64,
    pub lines: i64,
    pub hits: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<serde_json::Value>,
}

/// Miss-file list plus commit totals and how many files were returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissFilesResult {
    pub totals: ReportTotals,
    pub files: Vec<MissFile>,
    pub returned: usize,
}

/// Files with `misses > 0`, sorted by misses descending.
///
/// When `limit` is `0`, no truncate is applied. Otherwise the list is capped to `limit`.
#[must_use]
pub fn miss_files_from_report(report: &CoverageReport, limit: usize) -> Vec<MissFile> {
    let mut out: Vec<MissFile> = report
        .files
        .iter()
        .filter(|f| f.totals.misses > 0)
        .map(|f| MissFile {
            name: f.name.clone(),
            misses: f.totals.misses,
            lines: f.totals.lines,
            hits: f.totals.hits,
            coverage: f.totals.coverage.clone(),
        })
        .collect();
    out.sort_by(|a, b| b.misses.cmp(&a.misses).then_with(|| a.name.cmp(&b.name)));
    if limit > 0 && out.len() > limit {
        out.truncate(limit);
    }
    out
}

/// Builds a [`MissFilesResult`] from a coverage report.
#[must_use]
pub fn miss_files_result(report: &CoverageReport, limit: usize) -> MissFilesResult {
    let files = miss_files_from_report(report, limit);
    let returned = files.len();
    MissFilesResult {
        totals: report.totals.clone(),
        files,
        returned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn file(name: &str, misses: i64) -> ReportFile {
        ReportFile {
            name: name.to_string(),
            totals: ReportTotals {
                misses,
                lines: 10,
                hits: 10 - misses,
                coverage: Some(json!(90.0)),
                ..ReportTotals::default()
            },
            line_coverage: Vec::new(),
        }
    }

    #[test]
    fn miss_files_filters_sorts_and_limits() {
        let report = CoverageReport {
            totals: ReportTotals {
                misses: 20,
                ..ReportTotals::default()
            },
            files: vec![
                file("a.rs", 0),
                file("b.rs", 5),
                file("c.rs", 12),
                file("d.rs", 3),
                file("e.rs", 5),
            ],
            commit_file_url: None,
        };
        let result = miss_files_result(&report, 2);
        assert_eq!(result.returned, 2);
        assert_eq!(result.totals.misses, 20);
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.files[0].name, "c.rs");
        assert_eq!(result.files[0].misses, 12);
        assert_eq!(result.files[0].coverage, Some(json!(90.0)));
        assert_eq!(result.files[1].name, "b.rs");
        assert_eq!(result.files[1].misses, 5);

        // Equal misses: name tie-break (`then_with`), then Debug on the result.
        let tied = miss_files_from_report(&report, 0);
        assert_eq!(tied.iter().filter(|f| f.misses == 5).count(), 2);
        let names: Vec<&str> = tied
            .iter()
            .filter(|f| f.misses == 5)
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["b.rs", "e.rs"]);
        let debug = format!("{result:?}");
        assert!(debug.contains("returned"));
    }

    #[test]
    fn miss_files_empty_when_no_misses() {
        let report = CoverageReport {
            totals: ReportTotals::default(),
            files: vec![file("ok.rs", 0)],
            commit_file_url: None,
        };
        let result = miss_files_result(&report, 30);
        assert!(result.files.is_empty());
        assert_eq!(result.returned, 0);
    }

    #[test]
    fn miss_files_limit_zero_returns_all() {
        let report = CoverageReport {
            totals: ReportTotals::default(),
            files: vec![file("a.rs", 1), file("b.rs", 2), file("c.rs", 3)],
            commit_file_url: None,
        };
        let files = miss_files_from_report(&report, 0);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].name, "c.rs");
        let result = miss_files_result(&report, 0);
        assert_eq!(result.returned, 3);
        assert_eq!(result.files.len(), 3);
    }
}
