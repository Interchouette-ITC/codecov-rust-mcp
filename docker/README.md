# Docker (codecov-rust-mcp)

Secure runtime image for the MCP binary (Streamable HTTP by default).

| Item                | Value                                                                     |
| ------------------- | ------------------------------------------------------------------------- |
| Hub                 | `interchouette/codecov-rust-mcp`                                          |
| Org GHCR            | `ghcr.io/interchouette-itc/codecov-rust-mcp`                              |
| Dockerfile          | [`Dockerfile`](Dockerfile)                                                |
| Hub Overview source | [`DOCKERHUB.md`](DOCKERHUB.md) (sync to Hub; do not invent Overview copy) |
| HTTP                | `0.0.0.0:8690` → `/mcp`                                                   |

## Build / push

```bash
make docker-build          # :$(version) + :latest (Hub name)
make docker-build-dev      # :dev + :latest (+ GHCR name tags)
make docker-push-dev-hub   # after login
```

CI: `.github/workflows/docker-build-push-dev.yml` (push to `dev` touching docker/src/Cargo.\*, or `workflow_dispatch`). Release workflow pushes `:X.Y.Z` + `:latest`.

Image is multi-stage → `gcr.io/distroless/cc-debian13:nonroot`. No shell as PID 1.

## Run

```bash
# HTTP (default in image)
docker run --rm -p 8690:8690 -e CODECOV_TOKEN interchouette/codecov-rust-mcp:latest

# stdio
docker run --rm -i -e CODECOV_TOKEN -e MCP_HTTP=false interchouette/codecov-rust-mcp:latest
```
