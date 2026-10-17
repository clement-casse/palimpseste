# 0012. Backend structure: one bounded context, lib and bin

Date: 2026-10-17.
Status: accepted. Amends ADR 0005 (the backend crate).

## Context

The backend (ADR 0005, 0007, 0008, 0009, 0010) grew one decision at a time
in four files: `main.rs`, `api.rs` (routes and checks), `auth.rs` (OIDC,
cookies, sessions) and `store.rs` (SQLite). It had no structure of its own:

- domain rules sat in adapters: nickname, profile and name checks in HTTP
  handlers, nickname and label derivation in the OIDC code, the free
  nickname and the engine check in SQL;
- handlers called the store directly, and the sign-in callback mixed the
  store, the OIDC exchange and cookies;
- `api` and `auth` imported each other;
- `rusqlite::Result` crossed into handlers, sign-in failures were strings,
  business outcomes were `Ok` enums (`Replaced`, `ProfileUpdate`,
  `Linking::Taken`), and `ApiError::Internal` dropped the cause;
- `User` and `Character` were also the JSON views;
- configuration was read by hand-rolled `env::var` calls in two modules;
- every test went through the router, SQLite and, for sign-in, an issuer.

## Decisions

- **One bounded context.** The backend is small, serves one application and
  is not meant to be split or scaled out: it is one context, with one use
  case trait. A second context would be its own ADR.
- **One crate, lib and bin.** `palimpseste-backend` is a library
  (`lib.rs`) and a binary (`main.rs`). The library holds the model, the use
  cases and the adapters; the binary is the composition root only.
- **Layout after hexagonal architecture, without its vocabulary**, following
  [howtocodeit's guide](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust):
  - `lib.rs`: the `PalimpsesteService` trait (the use cases), `Error`,
    `ActionContext`, and the re-exports that form the public API;
  - model files named by concept, flat at the root: `user.rs`, `session.rs`,
    `sign_in.rs`, `character.rs`. Value objects (`Nickname`, `Profile`,
    `CharacterName`, `Version`, ...) parse and hold the rules;
  - `service.rs`: `PalimpsesteServiceImpl<R, P>`, generic over its
    repository and identity providers;
  - one directory per adapter role, each with a declaration-only hat:
    `repo.rs` + `repo/sqlite.rs` (outbound, persistence),
    `provider.rs` + `provider/oidc.rs` (outbound, identity providers),
    `api.rs` + `api/http.rs` (inbound, HTTP and the front end);
  - no `mod.rs`, no `domain/`, `infrastructure/` or `ports/` folders, no
    `types.rs` or `models.rs`.
- **Dependency injection.** The service holds its repository and providers
  as type parameters; the HTTP adapter holds the service as
  `Arc<dyn PalimpsesteService>`. Only `main.rs` builds and wires them. Traits
  use `async-trait` (for `dyn`) and generate `mockall` mocks under `cfg(test)`.
- **`ActionContext` instead of an authorizer.** Every use case acting for a
  signed-in user takes `ctx: &ActionContext` (the user's id) first; every
  repository query is scoped by it. With no roles or parties, an authorizer
  role would anticipate: it arrives with them.
- **One `Error` enum (`thiserror`).** Each variant the user can react to is
  named (`NicknameTaken`, `Stale`, `SignInFailed`, ...); technical failures
  go to `Underlying(anyhow::Error)`, tagged with `.context(...)` where they
  cross from an adapter. The HTTP adapter maps variants to statuses and
  never shows an `Underlying` cause. No `Result<T>` alias.
- **Adapters translate only.** JSON views, cookies, `ETag` and `If-Match`,
  redirects and the front end live in `api/http*`; SQL and blocking calls
  (`spawn_blocking`) in `repo/sqlite.rs`; the OIDC protocol in
  `provider/oidc.rs`. The rules they used to hold moved to the model and the
  service.
- **Configuration through clap** (`derive`, `env`): `--addr`,
  `--frontend-dir`, `--database`, `--public-url`, `--oidc-providers`, each
  with its existing `PALIMPSESTE_*` variable and default. The per-provider
  variables (ADR 0009) have names computed from the provider id, which
  clap cannot declare, so `main.rs` reads them, with the same error
  messages as before.
- **No Cargo features.** One binary is the only deployment.
- **New dependencies of `app/backend`:** `thiserror`, `anyhow`,
  `async-trait`, `clap`; `mockall` for tests.
- **The day-to-day guide** is the project skill
  `.claude/skills/backend-structure/SKILL.md`: where things go, and the
  recipes to add a use case, a rule or an adapter.

## Consequences

- The HTTP API, cookies, redirects, logs, database schema and environment
  variables are unchanged; the end-to-end tests moved with their
  assertions.
- Use cases are tested against mocks of the repository and providers, the
  HTTP adapter against a mock of the service, and the SQLite repository
  alone.
- Domain events are logged by the service, rejections and technical
  failures by the HTTP adapter (ADR 0008 still lists what is logged).
- `--help` documents the options and their environment variables.
