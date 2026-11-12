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

#[derive(Debug, Deserialize)]
pub struct PullComparison {
    pub base_commit: String,
    pub head_commit: String,
    pub totals: PullComparisonTotals,
    #[serde(default)]
    pub files: Vec<PullComparisonFile>,
}

#[derive(Debug, Deserialize)]
pub struct PullComparisonTotals {
    pub base: ReportTotals,
    pub head: ReportTotals,
    pub patch: ReportTotals,
}

#[derive(Debug, Deserialize)]
pub struct PullComparisonFile {
    pub name: ComparisonFileName,
    pub totals: PullComparisonTotals,
    #[serde(default)]
    pub lines: Vec<PullComparisonLine>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ComparisonFileName {
    Path(String),
    Versions {
        #[serde(default)]
        base: Option<String>,
        #[serde(default)]
        head: Option<String>,
    },
}

impl ComparisonFileName {
    fn path(&self) -> Option<&str> {
        match self {
            Self::Path(path) => Some(path),
            Self::Versions { base, head } => head.as_deref().or(base.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PullComparisonLine {
    pub value: String,
    pub number: ComparisonLineNumbers,
    pub coverage: ComparisonLineCoverage,
    pub added: bool,
    pub removed: bool,
}

#[derive(Debug, Deserialize)]
pub struct ComparisonLineNumbers {
    #[serde(default)]
    pub base: Option<i64>,
    #[serde(default)]
    pub head: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ComparisonLineCoverage {
    #[serde(default)]
    pub base: Option<i64>,
    #[serde(default)]
    pub head: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PullCoverageReport {
    pub pull_id: u64,
    pub base_commit: String,
    pub head_commit: String,
    pub patch: ReportTotals,
    pub files: Vec<PullCoverageFile>,
    pub missed_lines_reported: i64,
    pub missed_lines_returned: usize,
    pub line_details_complete: bool,
    pub line_coverage_legend: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PullCoverageFile {
    pub path: String,
    pub patch: ReportTotals,
    pub missed_lines: Vec<PullCoverageLine>,
    pub partial_lines: Vec<PullCoverageLine>,
    pub unmeasured_added_lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PullCoverageLine {
    pub line: i64,
    pub code: String,
}

/// Converts Codecov's pull comparison to an agent-oriented patch report.
///
/// # Errors
///
/// Returns an error when Codecov's comparison omits required file paths or line numbers,
/// or uses an unknown line coverage status.
pub fn pull_coverage_report(
    pull_id: u64,
    comparison: PullComparison,
) -> Result<PullCoverageReport, String> {
    let mut files = Vec::with_capacity(comparison.files.len());
    let mut missed_lines_returned = 0_usize;
    let mut partial_lines_returned = 0_i64;

    for file in comparison.files {
        let path = file
            .name
            .path()
            .filter(|path| !path.is_empty())
            .ok_or_else(|| "Codecov comparison returned a file without a path".to_string())?
            .to_string();
        let mut missed_lines = Vec::new();
        let mut partial_lines = Vec::new();
        let mut unmeasured_added_lines = 0;

        for line in file
            .lines
            .into_iter()
            .filter(|line| line.added && !line.removed)
        {
            let number = line
                .number
                .head
                .ok_or_else(|| format!("Codecov comparison returned an added line without a head line number in {path}"))?;
            let code = line
                .value
                .strip_prefix('+')
                .unwrap_or(&line.value)
                .to_string();
            match line.coverage.head {
                Some(0) => {}
                Some(1) => missed_lines.push(PullCoverageLine { line: number, code }),
                Some(2) => partial_lines.push(PullCoverageLine { line: number, code }),
                None => unmeasured_added_lines += 1,
                Some(status) => {
                    return Err(format!(
                        "Codecov comparison returned unknown line coverage status {status} in {path}:{number}"
                    ));
                }
            }
        }

        missed_lines_returned += missed_lines.len();
        partial_lines_returned += i64::try_from(partial_lines.len()).unwrap_or(i64::MAX);
        files.push(PullCoverageFile {
            path,
            patch: file.totals.patch,
            missed_lines,
            partial_lines,
            unmeasured_added_lines,
        });
    }
    files.sort_by(|left, right| {
        right
            .patch
            .misses
            .cmp(&left.patch.misses)
            .then_with(|| left.path.cmp(&right.path))
    });

    let line_details_complete = i64::try_from(missed_lines_returned)
        .is_ok_and(|count| count == comparison.totals.patch.misses)
        && partial_lines_returned == comparison.totals.patch.partials;
    let missed_lines_reported = comparison.totals.patch.misses;

    Ok(PullCoverageReport {
        pull_id,
        base_commit: comparison.base_commit,
        head_commit: comparison.head_commit,
        patch: comparison.totals.patch,
        files,
        missed_lines_reported,
        missed_lines_returned,
        line_details_complete,
        line_coverage_legend: "For changed lines: coverage 0 is covered, 1 is missed, 2 is partial. Unmeasured added lines have no numeric coverage value and are not counted as misses.",
    })
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

    fn pull_comparison(files: &serde_json::Value, misses: i64, partials: i64) -> PullComparison {
        let totals = json!({
            "files": 0,
            "lines": 0,
            "hits": 0,
            "misses": misses,
            "partials": partials,
            "coverage": 0.0,
            "branches": 0,
            "methods": 0
        });
        serde_json::from_value(json!({
            "base_commit": "base",
            "head_commit": "head",
            "totals": {
                "base": totals,
                "head": totals,
                "patch": totals
            },
            "files": files
        }))
        .expect("valid pull comparison fixture")
    }

    fn comparison_file(name: &serde_json::Value, lines: &serde_json::Value) -> serde_json::Value {
        let totals = json!({
            "files": 0,
            "lines": 0,
            "hits": 0,
            "misses": 0,
            "partials": 0,
            "coverage": 0.0,
            "branches": 0,
            "methods": 0
        });
        json!({
            "name": name,
            "totals": {
                "base": totals,
                "head": totals,
                "patch": totals
            },
            "lines": lines
        })
    }

    fn added_line(number: Option<i64>, status: Option<i64>) -> serde_json::Value {
        json!({
            "value": "+test line",
            "number": { "base": null, "head": number },
            "coverage": { "base": null, "head": status },
            "added": true,
            "removed": false
        })
    }

    #[test]
    fn pull_report_collects_added_line_statuses_and_sorts_files() {
        let comparison = pull_comparison(
            &json!([
                comparison_file(
                    &json!({ "base": "z.rs", "head": null }),
                    &json!([
                        added_line(Some(10), Some(1)),
                        added_line(Some(11), Some(2)),
                        added_line(Some(12), None),
                        added_line(Some(13), Some(0)),
                        {
                            "value": "-removed line",
                            "number": { "base": 14, "head": null },
                            "coverage": { "base": 1, "head": null },
                            "added": false,
                            "removed": true
                        }
                    ])
                ),
                comparison_file(&json!("a.rs"), &json!([added_line(Some(20), Some(1))]))
            ]),
            2,
            1,
        );

        let report = pull_coverage_report(97, comparison).expect("report");
        assert_eq!(report.files[0].path, "a.rs");
        assert_eq!(report.files[1].path, "z.rs");
        assert_eq!(report.files[1].missed_lines[0].line, 10);
        assert_eq!(report.files[1].partial_lines[0].line, 11);
        assert_eq!(report.files[1].unmeasured_added_lines, 1);
        assert_eq!(report.missed_lines_reported, 2);
        assert_eq!(report.missed_lines_returned, 2);
        assert!(report.line_details_complete);
    }

    #[test]
    fn pull_report_rejects_missing_paths_and_line_numbers() {
        let missing_path = pull_comparison(
            &json!([comparison_file(
                &json!({ "base": null, "head": null }),
                &json!([])
            )]),
            0,
            0,
        );
        assert!(pull_coverage_report(1, missing_path)
            .expect_err("missing path")
            .contains("without a path"));

        let missing_line = pull_comparison(
            &json!([comparison_file(
                &json!("src/lib.rs"),
                &json!([added_line(None, Some(1))])
            )]),
            1,
            0,
        );
        assert!(pull_coverage_report(1, missing_line)
            .expect_err("missing line number")
            .contains("without a head line number"));
    }

    #[test]
    fn pull_report_rejects_unknown_coverage_status_and_flags_incomplete_details() {
        let unknown_status = pull_comparison(
            &json!([comparison_file(
                &json!("src/lib.rs"),
                &json!([added_line(Some(10), Some(3))])
            )]),
            0,
            0,
        );
        assert!(pull_coverage_report(1, unknown_status)
            .expect_err("unknown status")
            .contains("unknown line coverage status 3"));

        let incomplete = pull_comparison(
            &json!([comparison_file(
                &json!("src/lib.rs"),
                &json!([added_line(Some(10), Some(1))])
            )]),
            2,
            1,
        );
        assert!(
            !pull_coverage_report(1, incomplete)
                .expect("incomplete report")
                .line_details_complete
        );
    }

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
