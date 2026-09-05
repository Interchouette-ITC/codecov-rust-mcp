# Docker (codecov-rust-mcp)

Secure runtime image for the MCP stdio binary.

| Item | Value |
| --- | --- |
| Hub | `interchouette/codecov-rust-mcp` |
| Org GHCR | `ghcr.io/interchouette-itc/codecov-rust-mcp` |
| Dockerfile | [`Dockerfile`](Dockerfile) |
| Hub Overview source | [`DOCKERHUB.md`](DOCKERHUB.md) (sync to Hub; do not invent Overview copy) |

## Build

```bash
make docker-build          # :$(version) + :latest (Hub name)
make docker-build-dev      # :dev + :latest (+ GHCR name tags)
make docker-run            # interactive stdio smoke (needs CODECOV_TOKEN)
```

Image is multi-stage → `gcr.io/distroless/cc-debian13:nonroot`. No shell as PID 1.

## Run (stdio)

```bash
docker run --rm -i -e CODECOV_TOKEN interchouette/codecov-rust-mcp:latest
```

No published port. MCP clients that support Docker stdio should use `docker run --rm -i …` as the command (see Hub Overview / [`DOCKERHUB.md`](DOCKERHUB.md)).
