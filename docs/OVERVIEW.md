# Overview

**codecov-rust-mcp** is a small Rust [MCP](https://modelcontextprotocol.io/) server
(**stdio** or **Streamable HTTP**). It wraps the [Codecov](https://docs.codecov.com/) API v2 so
agents and IDEs can read coverage without scraping HTML or parsing local
`lcov` by hand.

## What it does

| Tool | Role |
| --- | --- |
| `codecov_totals` | Commit coverage totals for a GitHub `owner` / `repo` |
| `codecov_miss_files` | Files with missed lines, sorted by misses |
| `codecov_file_report` | Line coverage for one repository-relative path |

Service is fixed to GitHub (`github` in the Codecov API path). Auth is a Codecov
**Access** bearer token (`CODECOV_TOKEN`).

## Layout

| Path | Role |
| --- | --- |
| `src/` | Library + `codecov-rust-mcp` binary (rmcp tools, HTTP client) |
| `docs/` | Hub, operator guide, contributing, security |
| `docker/` | Distroless image + Hub Overview (`DOCKERHUB.md`) |
| `examples/` | Manual smoke helpers |
| `.github/workflows/` | CI, coverage upload, rustdoc Pages, release binaries + images |

## Where to go next

| Doc | Purpose |
| --- | --- |
| [`README.md`](README.md) | Hub: badges, tools, quick start |
| [`mcp.md`](mcp.md) | Operator guide (env, Make, MCP clients) |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | PR and local gate habits |
| [Rust API (rustdoc)](https://interchouette-itc.github.io/codecov-rust-mcp/) | Generated API docs |
| [GitHub Releases](https://github.com/Interchouette-ITC/codecov-rust-mcp/releases) | Prebuilt Linux binary |
