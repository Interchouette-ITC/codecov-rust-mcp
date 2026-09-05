# codecov-rust-mcp

MCP server for the [Codecov](https://docs.codecov.com/) API v2 over **stdio**. Agents use three read tools to inspect coverage totals, list files with misses, and fetch a single file report.

Service is fixed to GitHub (`github` in the Codecov API path).

## Tools

| Tool                  | Parameters                                                           | Purpose                                                                                                                     |
| --------------------- | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `codecov_totals`      | `owner`, `repo`, optional `branch`, optional `sha`                   | Commit coverage totals for a GitHub owner/repo                                                                              |
| `codecov_miss_files`  | `owner`, `repo`, optional `branch`, optional `sha`, optional `limit` | Commit `totals` plus files with `misses > 0` (sorted descending) and `returned` count. `limit` default `30`; `0` = uncapped |
| `codecov_file_report` | `owner`, `repo`, `path`, optional `branch`, optional `sha`           | Line coverage for one repository-relative file path                                                                         |

When both `branch` and `sha` are set, Codecov uses `sha` (commit) preference on the API query string.

## Auth

Authentication is **bearer token only**. Put the token in `$HOME/.config/codecov-rust-mcp/.env` (or a checkout `.env`). The binary loads it via `dotenvy` (existing process env wins).

| Requirement  | Detail                                                       |
| ------------ | ------------------------------------------------------------ |
| Env / `.env` | `CODECOV_TOKEN` must be set                                  |
| Header       | `Authorization: bearer <token>` on every Codecov API request |
| Not used     | OAuth, GitHub login, or other Codecov account flows          |

Without `CODECOV_TOKEN`, tool calls fail with a clear error. The stdio process can still start (useful for transport smoke tests).

## Environment

| Variable          | Required | Description                                                                                                       |
| ----------------- | -------- | ----------------------------------------------------------------------------------------------------------------- |
| `CODECOV_TOKEN`   | yes      | Codecov **Access** token (`Authorization: bearer …`). Use account Settings → Access, not a per-repo upload token. |
| `CODECOV_API_URL` | no       | API base (default `https://api.codecov.io/api/v2`)                                                                |

## Make targets

| Target         | Action                                           |
| -------------- | ------------------------------------------------ |
| `make build`   | Debug build (`cargo +stable`)                    |
| `make release` | Release build                                    |
| `make lint`    | `fmt --check` + clippy (pedantic / nursery deny) |
| `make test`    | Unit + integration tests                         |
| `make ci`      | `lint` + `test` + `doc`                          |
| `make deny`    | `cargo deny check`                               |
| `make audit`   | `cargo audit`                                    |
| `make doc`     | rustdoc → `docs/api-rust/` (Pages deploy source) |
| `make run`     | Build and run the stdio server                   |

```bash
make lint
make test
make ci
```

## Install

| Path           | How                                                                                                                                              |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| From source    | `cargo +stable install --path . --force`                                                                                                         |
| Prebuilt Linux | [GitHub Releases](https://github.com/Interchouette-ITC/codecov-rust-mcp/releases) asset `codecov-rust-mcp-*-x86_64-unknown-linux-gnu`            |
| Docker         | `docker pull interchouette/codecov-rust-mcp:latest` then `docker run -p 8690:8690 -e CODECOV_TOKEN …` (HTTP `/mcp`; see [`docker/README.md`](../docker/README.md)) |

## MCP clients

Works with any host that speaks MCP **stdio** or **Streamable HTTP** (agents, IDEs, CLIs).

### stdio (default)

1. Install the binary on `PATH`: `cargo +stable install --path . --force`
2. Put `CODECOV_TOKEN` in `$HOME/.config/codecov-rust-mcp/.env` (or a checkout `.env`). Never commit tokens.
3. Register the server like [`mcp.json.example`](../mcp.json.example): `"command": "codecov-rust-mcp"`.

### Streamable HTTP

```bash
codecov-rust-mcp --http --listen 0.0.0.0:8690
# or: MCP_HTTP=true CODECOV_MCP_ADDR=0.0.0.0:8690 codecov-rust-mcp
```

Endpoint: `http://<host>:8690/mcp`. Docker image defaults to this mode (port **8690**). See [`docker/README.md`](../docker/README.md).

## Example

Print coverage totals (requires `CODECOV_TOKEN`):

```bash
cp .env.example .env   # set CODECOV_TOKEN
cargo +stable run --example totals -- OWNER REPO BRANCH
```

Pass your GitHub `OWNER`, `REPO`, and `BRANCH`. See `examples/totals.rs` for the binary's built-in defaults when args are omitted.

## Thanks

Thanks to [GitHub](https://github.com/) for hosting and Actions, and to [Codecov](https://about.codecov.io/) for coverage reports and the [API](https://docs.codecov.com/) this server wraps.

## See also

- Docs index: [`README.md`](README.md)
- Overview: [`OVERVIEW.md`](OVERVIEW.md)
- Contributing: [`CONTRIBUTING.md`](CONTRIBUTING.md)

## License

**Apache-2.0** (Apache License, Version 2.0). See [`LICENSE`](../LICENSE).
