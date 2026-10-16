# 0008. Backend logging

Date: 2026-10-16.
Status: accepted.

## Context

The backend (ADR 0005, ADR 0007) wrote its few errors to stderr with
`eprintln!` and said nothing else: not where it listens, not which
requests it serves, not why it rejects one. Trying the application end to
end needs that.

## Decisions

- **`tracing`, in `app/backend` only.** The backend logs through
  `tracing`, the ecosystem axum and tower-http already speak, and prints
  through `tracing-subscriber` (`fmt`, `env-filter`). `RUST_LOG` filters
  what is printed; the default is `info`. The kernel, the engines,
  `palimpseste-document` and `palimpseste-wasm` do not log.
- **Requests are traced by tower-http's `TraceLayer`** (`trace` feature):
  one span per request with its method and URI, and its status and
  latency when it finishes, at `info`; server errors at `error`.
- **What the API logs**: at startup, the database, the front end
  directory (a warning when it holds no `index.html`) and the address; each
  change to the store (user and character created, character renamed or
  deleted, document stored with its version) at `info`; a stale `If-Match`
  at `info`; a request rejected as unprocessable, with its reason, or
  without `If-Match` at `warn`; a store error at `error`. Ids are logged,
  never documents.

## Consequences

- Asset requests are logged too; `RUST_LOG=info,tower_http=warn` keeps only
  the API's own events.
