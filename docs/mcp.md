# codecov-rust-mcp

MCP server for the [Codecov](https://docs.codecov.com/) API v2 over **stdio**. Agents can inspect a pull request's patch coverage, coverage totals, missed files, and per-file line reports.

Service is fixed to GitHub (`github` in the Codecov API path).

## Tools

| Tool                  | Parameters                                                           | Purpose                                                                                                                     |
| --------------------- | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `codecov_pr_report`   | `owner`, `repo`, `pull_id`                                           | PR patch totals and exact missed/partial added lines, queried directly from Codecov                                         |
| `codecov_totals`      | `owner`, `repo`, optional `branch`, optional `sha`                   | Whole-commit coverage totals                                                                                                |
| `codecov_miss_files`  | `owner`, `repo`, optional `branch`, optional `sha`, optional `limit` | Whole-commit totals plus files with misses, sorted descending; `limit` default `30`; `0` = uncapped                         |
| `codecov_file_report` | `owner`, `repo`, `path`, optional `branch`, optional `sha`            | Whole-file line coverage for one repository-relative path                                                                   |

When both `branch` and `sha` are set, Codecov uses `sha` (commit) preference on the API query string.

## Get PR coverage directly from Codecov

When given a PR URL, extract its owner, repository, and PR number and call
`codecov_pr_report`. For example, for
`https://github.com/Interchouette-ITC/rangular/pull/97`, use
`owner="Interchouette-ITC"`, `repo="rangular"`, and `pull_id=97`. The tool calls
Codecov's [pull comparison endpoint](https://docs.codecov.com/reference/repos_compare_retrieve.md)
with that PR number; it does not fetch from GitHub or require resolving the head SHA.
It returns Codecov's authoritative patch coverage totals plus per-file added lines
that are missed or partially covered, including line numbers and source text. Use
those lines to identify where tests need to be added or expanded.

For `codecov_pr_report`, `missed_lines_reported` and `patch.misses` are Codecov's
patch-miss totals; the returned per-line details should match, and
`line_details_complete` says whether they do. A coverage status of `0` means
covered, `1` missed, and `2` partially covered; null means unmeasured and must not
be called a miss. Removed lines are excluded from new-code findings.

The other tools are commit/file reports, not patch reports. In their
`line_coverage` pairs, `[line_number, 0]` means covered, `[line_number, 1]` missed,
and `[line_number, 2]` partial; the value is a status, not a hit count. Do not
substitute whole-file or whole-commit misses for PR patch misses.

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

| Path           | How                                                                                                                                                                |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| From source    | `cargo +stable install --path . --force`                                                                                                                           |
| Prebuilt Linux | [GitHub Releases](https://github.com/Interchouette-ITC/codecov-rust-mcp/releases) asset `codecov-rust-mcp-*-x86_64-unknown-linux-gnu`                              |
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
