# Contributing to codecov-mcp

## Code changes

- Prefer one concern per PR.
- Run `make lint`, `make test`, `make audit`, and `make deny` before push (or `make ci` when you also need rustdoc).
- Conventional commits: `feat: …`, `fix: …`, `docs: …`, `ci: …`, etc.
- PR body follows [`pull_request_template.md`](pull_request_template.md) (**Summary** + **Test plan** only).
- Follow the [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) and [`SECURITY.md`](SECURITY.md).

## Local gates

| Target | Action |
| --- | --- |
| `make lint` | `fmt --check` + clippy (pedantic / nursery deny) |
| `make test` | Unit + integration tests |
| `make audit` | `cargo audit` |
| `make deny` | `cargo deny check` |
| `make doc` | rustdoc → `docs/api-rust/` |
| `make ci` | `lint` + `test` + `doc` |

## Auth for live API smoke

Use a Codecov **Access** token in `$HOME/.config/codecov-mcp/.env` (or a checkout `.env`). Never commit tokens. See [`mcp.md`](mcp.md).

## Questions

Open a GitHub issue on [Interchouette-ITC/codecov-rust-mcp](https://github.com/Interchouette-ITC/codecov-rust-mcp).
