//! Integration tests for the three Codecov tools (HTTP client against wiremock).

use codecov_rust_mcp::client::CodecovClient;
use serde_json::json;
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_totals_json() -> serde_json::Value {
    json!({
        "totals": {
            "files": 2,
            "lines": 20,
            "hits": 16,
            "misses": 4,
            "partials": 0,
            "coverage": 80.0,
            "branches": 0,
            "methods": 0
        },
        "files": [
            {
                "name": "src/lib.rs",
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
            },
            {
                "name": "src/ok.rs",
                "totals": {
                    "files": 0,
                    "lines": 10,
                    "hits": 10,
                    "misses": 0,
                    "partials": 0,
                    "coverage": 100.0,
                    "branches": 0,
                    "methods": 0
                },
                "line_coverage": []
            }
        ]
    })
}

fn sample_file_report_json() -> serde_json::Value {
    json!({
        "name": "crates/foo/src/parser.rs",
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
        "line_coverage": [[129, 0], [136, 0], [353, 1]]
    })
}

#[tokio::test]
async fn codecov_totals_hits_totals_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/github/Interchouette-ITC/repos/rangular/totals/"))
        .and(query_param("branch", "dev"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_totals_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = CodecovClient::new("tok", server.uri());
    let report = client
        .totals("Interchouette-ITC", "rangular", Some("dev"), None)
        .await
        .expect("totals");

    assert_eq!(report.totals.misses, 4);
    assert_eq!(report.files.len(), 2);
}

#[tokio::test]
async fn codecov_miss_files_filters_sorted_misses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r".*/totals/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_totals_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = CodecovClient::new("tok", server.uri());
    let misses = client
        .miss_files("Interchouette-ITC", "rangular", Some("dev"), None, 10)
        .await
        .expect("miss_files");

    assert_eq!(misses.returned, 1);
    assert_eq!(misses.files[0].name, "src/lib.rs");
    assert_eq!(misses.files[0].misses, 2);
    assert_eq!(misses.totals.misses, 4);
}

#[tokio::test]
async fn codecov_file_report_omits_trailing_slash_on_path() {
    let server = MockServer::start().await;
    let file_path = "crates/foo/src/parser.rs";
    let encoded = "crates%2Ffoo%2Fsrc%2Fparser.rs";

    Mock::given(method("GET"))
        .and(path(format!(
            "/github/Interchouette-ITC/repos/rangular/file_report/{encoded}"
        )))
        .and(query_param("branch", "dev"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_file_report_json()))
        .expect(1)
        .mount(&server)
        .await;

    // Trailing-slash form must not match the mock (would 404 if still used).
    Mock::given(method("GET"))
        .and(path(format!(
            "/github/Interchouette-ITC/repos/rangular/file_report/{encoded}/"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .expect(0)
        .mount(&server)
        .await;

    let client = CodecovClient::new("tok", server.uri());
    let file = client
        .file_report(
            "Interchouette-ITC",
            "rangular",
            file_path,
            Some("dev"),
            None,
        )
        .await
        .expect("file_report");

    assert_eq!(file.name, "crates/foo/src/parser.rs");
    assert_eq!(file.line_coverage.len(), 3);

    let requests = server.received_requests().await.expect("requests");
    assert_eq!(requests.len(), 1);
    let path = requests[0].url.path();
    assert!(
        path.ends_with("parser.rs"),
        "expected path without trailing slash, got {path}"
    );
    assert!(
        !path.ends_with("parser.rs/"),
        "trailing slash after file path must not be requested: {path}"
    );
}
