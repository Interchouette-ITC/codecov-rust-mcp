# codecov-rust-mcp

Rust **MCP** server for the [Codecov](https://docs.codecov.com/) API v2: coverage totals, miss files, and file reports. **stdio** locally; **Streamable HTTP** in Docker.

Source: [Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp)

Docker Hub: [`interchouette/codecov-rust-mcp`](https://hub.docker.com/r/interchouette/codecov-rust-mcp)
Org GHCR: `ghcr.io/interchouette-itc/codecov-rust-mcp`

Size-optimized multi-stage build → distroless `cc-debian13` (Debian 13, non-root; no apt/perl/shell). TLS via rustls (no OpenSSL package). CA certs included for outbound Codecov HTTPS.

## What’s inside

| Binary             | Role                                                                       |
| ------------------ | -------------------------------------------------------------------------- |
| `codecov-rust-mcp` | MCP server (`codecov_totals`, `codecov_miss_files`, `codecov_file_report`) |

## Quick start

```bash
docker pull interchouette/codecov-rust-mcp:latest

# Streamable HTTP on :8690 → /mcp
docker run -d -p 8690:8690 -e CODECOV_TOKEN interchouette/codecov-rust-mcp:latest

# stdio instead
docker run --rm -i -e CODECOV_TOKEN -e MCP_HTTP=false interchouette/codecov-rust-mcp:latest
```

AI clients that support Streamable HTTP can use `http://localhost:8690/mcp` when the port is published.

## Environment

| Env                | Default                 | Meaning                                          |
| ------------------ | ----------------------- | ------------------------------------------------ |
| `CODECOV_TOKEN`    | (required)              | Codecov **Access** token (not a CI upload token) |
| `CODECOV_API_URL`  | Codecov API v2          | Optional API base override                       |
| `MCP_HTTP`         | `true` in image         | Serve Streamable HTTP instead of stdio           |
| `CODECOV_MCP_ADDR` | `0.0.0.0:8690` in image | HTTP bind address                                |

## Tags

| Tag       | Meaning                                                                      |
| --------- | ---------------------------------------------------------------------------- |
| `:dev`    | Tip of `dev` (CI push / workflow_dispatch); does **not** move `:latest`      |
| `:X.Y.Z`  | Release matching `Cargo.toml` / GitHub Release `vX.Y.Z`                      |
| `:latest` | Last GitHub Release only                                                     |

## Docs

- Repo docs: [github.com/Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp)
- Operator guide: [`docs/mcp.md`](https://github.com/Interchouette-ITC/codecov-rust-mcp/blob/dev/docs/mcp.md)
- Docker details: [`docker/README.md`](https://github.com/Interchouette-ITC/codecov-rust-mcp/blob/dev/docker/README.md)
- Website: [interchouette.net](https://interchouette.net/)

## License

Apache-2.0. See the repository.
