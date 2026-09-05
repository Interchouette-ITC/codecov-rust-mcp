# codecov-rust-mcp

Rust **MCP** server for the [Codecov](https://docs.codecov.com/) API v2 over **stdio**: coverage totals, miss files, and file reports.

Source: [Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp)

Docker Hub: [`interchouette/codecov-rust-mcp`](https://hub.docker.com/r/interchouette/codecov-rust-mcp)
Org GHCR: `ghcr.io/interchouette-itc/codecov-rust-mcp`

Size-optimized multi-stage build → distroless `cc-debian13` (Debian 13, non-root; no apt/perl/shell). TLS via rustls (no OpenSSL package). CA certs included for outbound Codecov HTTPS.

## What’s inside

| Binary | Role |
| --- | --- |
| `codecov-rust-mcp` | MCP stdio server (`codecov_totals`, `codecov_miss_files`, `codecov_file_report`) |

## Quick start

```bash
docker pull interchouette/codecov-rust-mcp:latest

# MCP over stdio (attach stdin; pass Access token)
docker run --rm -i -e CODECOV_TOKEN interchouette/codecov-rust-mcp:latest
```

Example MCP host config (any client that can spawn a process):

```json
{
  "mcpServers": {
    "codecov": {
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "-e",
        "CODECOV_TOKEN",
        "interchouette/codecov-rust-mcp:latest"
      ]
    }
  }
}
```

Set `CODECOV_TOKEN` in the host environment (Codecov **Access** token, not a CI upload token).

## Tags

| Tag | Meaning |
| --- | --- |
| `:dev` | Latest development image (manual / on-demand push) |
| `:X.Y.Z` | Release matching `Cargo.toml` / GitHub Release `vX.Y.Z` |
| `:latest` | Moves with each release |

## Docs

- Repo docs: [github.com/Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp)
- Operator guide: [`docs/mcp.md`](https://github.com/Interchouette-ITC/codecov-rust-mcp/blob/dev/docs/mcp.md)
- Docker details: [`docker/README.md`](https://github.com/Interchouette-ITC/codecov-rust-mcp/blob/dev/docker/README.md)
- Website: [interchouette.net](https://interchouette.net/)

## License

Apache-2.0. See the repository.
