//! Print JSON coverage totals for a GitHub owner/repo/branch.
//!
//! Usage: `cargo run --example totals -- [owner] [repo] [branch]`
//! Defaults: `Interchouette-ITC` `rustashop` `dev`.
//! Requires `CODECOV_TOKEN`.

use codecov_rust_mcp::client::CodecovClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut args = std::env::args().skip(1);
    let owner = args
        .next()
        .unwrap_or_else(|| "Interchouette-ITC".to_string());
    let repo = args.next().unwrap_or_else(|| "rustashop".to_string());
    let branch = args.next().unwrap_or_else(|| "dev".to_string());

    let client = CodecovClient::from_env()?;
    let report = client
        .totals(&owner, &repo, Some(branch.as_str()), None)
        .await?;
    println!("{}", serde_json::to_string_pretty(&report.totals)?);
    Ok(())
}
