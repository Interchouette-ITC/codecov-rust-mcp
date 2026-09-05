# codecov-mcp

MCP server for the [Codecov](https://docs.codecov.com/) API v2 over **stdio**. Exposes three read tools so agents can check coverage totals, list files with misses, and inspect a single file report.

Canonical repo: [Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp).

Service is fixed to GitHub (`github` in the Codecov API path).

Licensed under the [Apache License 2.0](../LICENSE) (Copyright Interchouette 2026).

## Docs

| Doc                     | Description                                              |
| ----------------------- | -------------------------------------------------------- |
| [`mcp.md`](mcp.md)      | Operator guide: tools, auth, env, Make, Cursor, examples |
| [`LICENSE`](../LICENSE) | Apache-2.0                                               |

## Tools

| Tool                  | Purpose                                                                                                         |
| --------------------- | --------------------------------------------------------------------------------------------------------------- |
| `codecov_totals`      | Commit coverage totals for a GitHub `owner` / `repo` (optional `branch` or `sha`)                               |
| `codecov_miss_files`  | Commit `totals` + files with `misses > 0` (sorted desc) + `returned` count (`limit` default 30; `0` = uncapped) |
| `codecov_file_report` | Line coverage for one repository-relative `path`                                                                |

## Quick start

```bash
cp .env.example .env   # set CODECOV_TOKEN (Access token, not upload token)
make release
make run
```

Cursor: [`mcp.json.example`](../mcp.json.example) runs the release binary; token comes from `.env`, not from `mcp.json`.
