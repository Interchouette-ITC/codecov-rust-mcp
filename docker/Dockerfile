# syntax=docker/dockerfile:1
#
# codecov-rust-mcp - MCP server for the Codecov API (Streamable HTTP in Docker).
# Multi-stage → distroless cc-debian13 (non-root). Build context: repository root.
#
#   docker build -f docker/Dockerfile -t interchouette/codecov-rust-mcp:latest .
#   make docker-build
#
# Default CMD: Streamable HTTP on :8690 (/mcp). Override for stdio:
#   docker run --rm -i -e CODECOV_TOKEN interchouette/codecov-rust-mcp:latest
#   (replace entrypoint args: omit --http)

ARG RUST_IMAGE=rust:slim-trixie
ARG RUNTIME_IMAGE=gcr.io/distroless/cc-debian13:nonroot

FROM ${RUST_IMAGE} AS builder

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    binutils \
    pkg-config \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ENV CARGO_TERM_COLOR=never \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    CARGO_INCREMENTAL=0

COPY Cargo.toml Cargo.lock ./

# Warm dependency layer (dummy sources; real tree overwrites next).
RUN mkdir -p src \
  && printf 'fn main() {}\n' > src/main.rs \
  && printf '// docker dep cache\n' > src/lib.rs \
  && cargo build --release --locked --bin codecov-rust-mcp \
  && rm -rf src \
       target/release/deps/codecov_rust_mcp* \
       target/release/.fingerprint/codecov-rust-mcp* \
       target/release/codecov-rust-mcp \
       target/release/codecov-rust-mcp.d

COPY src ./src

RUN touch src/lib.rs src/main.rs \
  && cargo build --release --locked --bin codecov-rust-mcp \
  && strip -s target/release/codecov-rust-mcp \
  && cp target/release/codecov-rust-mcp /codecov-rust-mcp

FROM ${RUNTIME_IMAGE} AS runtime

ARG APP_VERSION=1.0.0

WORKDIR /app

COPY --from=builder /codecov-rust-mcp /usr/local/bin/codecov-rust-mcp
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

USER nonroot:nonroot

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    SSL_CERT_DIR=/etc/ssl/certs \
    MCP_HTTP=true \
    CODECOV_MCP_ADDR=0.0.0.0:8690

EXPOSE 8690

LABEL org.opencontainers.image.title="codecov-rust-mcp" \
      org.opencontainers.image.description="MCP server for the Codecov API (stdio or Streamable HTTP)" \
      org.opencontainers.image.vendor="Interchouette" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.version="${APP_VERSION}" \
      org.opencontainers.image.source="https://github.com/Interchouette-ITC/codecov-rust-mcp"

ENTRYPOINT ["/usr/local/bin/codecov-rust-mcp"]
