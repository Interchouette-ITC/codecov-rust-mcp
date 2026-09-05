# codecov-mcp

<p align="center">
  <a href="https://github.com/Interchouette-ITC/codecov-rust-mcp/actions/workflows/ci.yml"><img src="https://github.com/Interchouette-ITC/codecov-rust-mcp/actions/workflows/ci.yml/badge.svg?branch=dev" alt="CI" /></a>
  <a href="https://codecov.io/gh/Interchouette-ITC/codecov-rust-mcp"><img src="https://codecov.io/gh/Interchouette-ITC/codecov-rust-mcp/branch/dev/graph/badge.svg" alt="codecov" /></a>
</p>

MCP server for the [Codecov](https://docs.codecov.com/) API v2 over **stdio**. Exposes three read tools so agents can check coverage totals, list files with misses, and inspect a single file report.

Canonical repo: [Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp).

Service is fixed to GitHub (`github` in the Codecov API path).

Licensed under the [Apache License 2.0](../LICENSE) (Copyright Interchouette 2026).

## Docs

| Doc | Description |
| --- | --- |
| [`mcp.md`](mcp.md) | Operator guide: tools, auth, env, Make, Cursor, examples |
| [`LICENSE`](../LICENSE) | Apache-2.0 |

## Tools

| Tool | Purpose |
| --- | --- |
| `codecov_totals` | Commit coverage totals for a GitHub `owner` / `repo` (optional `branch` or `sha`) |
| `codecov_miss_files` | Commit `totals` + files with `misses > 0` (sorted desc) + `returned` count (`limit` default 30; `0` = uncapped) |
| `codecov_file_report` | Line coverage for one repository-relative `path` |

## Quick start

```bash
cargo +stable install --path . --force
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/codecov-mcp"
cp .env.example "${XDG_CONFIG_HOME:-$HOME/.config}/codecov-mcp/.env"
# edit that file: set CODECOV_TOKEN (Access token, not upload token)
```

Cursor: [`mcp.json.example`](../mcp.json.example) (no checkout path; token from user config `.env`).
