# Security policy

## Supported versions

Security fixes target the latest tip of the `dev` branch and any tagged releases
published from this repository.

## Reporting a vulnerability

Do **not** open a public GitHub issue for an unfixed vulnerability.

Prefer a private [GitHub Security Advisory](https://github.com/Interchouette-ITC/codecov-rust-mcp/security/advisories/new)
on this repository when available. Otherwise email
[contact@interchouette.net](mailto:contact@interchouette.net) with a clear
description, impact, and reproduction steps when possible.

We will acknowledge receipt and follow up. Do not expect a fixed SLA.

## Tokens

Never commit Codecov Access tokens, upload tokens, or other secrets. Keep them
in process env or a local `.env` (see [`.env.example`](../.env.example)).
