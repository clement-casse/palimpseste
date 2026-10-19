# palimpseste-backend

The HTTP server of Palimpseste. It serves the built PWA, signs users in
through OIDC providers, keeps their accounts, and stores their characters.
**It never applies a rule** (ADR APP-STORE-ONLY). A character that breaks
the rules is stored as sent. The only check is that its document is
readable by [`palimpseste-document`](../document) (ADR BE-CHECKS).

It is the only crate with async code: axum, tokio and tower-http. It
stores data in SQLite through `rusqlite`, and signs users in with
`openidconnect`. A bare `cargo run` at the workspace root starts it.

> The day-to-day layout guide is the
> [backend-structure skill](../../.claude/skills/backend-structure/SKILL.md),
> and the decisions are in [docs/adr/backend.md](../../docs/adr/backend.md)
> (`BE-*`). Read both before adding a use case, a route or a table. This
> README is the map, and they are the rules.

## Layout

One bounded context: a library (`lib.rs`) driven by a binary (`main.rs`).
It is hexagonal, but without the vocabulary (ADR BE-LAYOUT).

```
src/
├── main.rs              composition root: clap Config, tracing, wiring, serve
├── lib.rs               PalimpsesteService (the use cases), Error, ActionContext, re-exports
├── user.rs              model: UserId, User, Nickname, Profile, Identity
├── session.rs           model: SessionToken, TokenHash, SESSION_TTL
├── sign_in.rs           model: ProviderId, PendingSignIn, VerifiedIdentity, ...
├── registration.rs      model: RegistrationToken, Registration, PendingRegistration
├── character.rs         model: CharacterId, CharacterName, Version, Character
├── service.rs           PalimpsesteServiceImpl<R, P>: the use cases
├── repo.rs              PalimpsesteRepository trait   ─┐
├── repo/sqlite.rs       SqliteRepo: schema, migrations, queries
├── provider.rs          IdentityProviders trait        ├ outbound adapters
├── provider/oidc.rs     OidcProviders, ProviderConfig ─┘
├── api.rs               hat: pub mod http
└── api/http.rs          HttpApi: router, views, handlers, error → status  ─ inbound adapter
    └── http/
        ├── sign_in.rs   /api/auth/*: redirects, cookies
        └── frontend.rs  the built PWA, cache headers
tests/fixtures/          test-only data (the test issuer's RSA key)
```

Rules to follow:

- **Dependencies point inward.** Adapters translate and own no business
  logic. Model types carry no `serde`: their JSON shape is the HTTP
  adapter's views.
- **Dependency injection.** The service takes its repository and providers
  as type parameters. The HTTP adapter holds one
  `Arc<dyn PalimpsesteService>`. Everything is wired in `main.rs`, and
  nothing else reads the environment.
- **One `Error`** (`thiserror`). Business outcomes are named variants.
  Technical failures go to `Underlying(anyhow::Error)`.
- **Ownership is the whole policy.** Every use case receives an
  `ActionContext`, and a user reaches only their own data
  (ADR BE-ACTION-CONTEXT).
- No `mod.rs`, no `domain.rs`/`types.rs`/`utils.rs`, no layered folders.

## HTTP API

| Route | Methods | |
|---|---|---|
| `/api/auth/providers` | `GET` | the configured providers |
| `/api/auth/{provider}/login`, `/link`, `/callback` | `GET` | the OIDC flow (Authorization Code + PKCE, `state`, `nonce`) |
| `/api/auth/registration` | `GET`, `POST`, `DELETE` | the account pending at the first sign-in: read, create, cancel (ADR BE-FIRST-SIGN-IN) |
| `/api/auth/logout` | `POST` | ends the session |
| `/api/me` | `GET`, `PATCH` | the signed-in user's nickname and profile |
| `/api/me/identities` | `GET` | the identities linked, at most one per provider |
| `/api/me/characters` | `GET`, `POST` | list and create characters |
| `/api/characters/{id}` | `GET`, `PATCH`, `DELETE` | read, rename, delete |
| `/api/characters/{id}/document` | `PUT` | replace the document whole; `If-Match` required (`428`), a stale version answers `412` |
| `/healthz` | `GET` | `200` while the database answers, else `503`; no session, not traced |

Every route but `/api/auth/*` and `/healthz` needs a session, carried in
an `HttpOnly`, `SameSite=Lax` cookie, and answers `401` without one. Every
other path serves the PWA and falls back to `index.html` (ADR BE-FRONTEND).
See ADR BE-API and ADR BE-CHECKS for every status code.

## Running

```sh
mise run serve          # builds the PWA, then runs the server on http://127.0.0.1:3000
cargo run -p palimpseste-backend -- --help
```

Configuration is clap options, each with an environment variable
(ADR BE-CONFIG): `PALIMPSESTE_ADDR`, `PALIMPSESTE_FRONTEND_DIR`,
`PALIMPSESTE_DATABASE`, `PALIMPSESTE_PUBLIC_URL` and
`PALIMPSESTE_OIDC_PROVIDERS`. Each provider `<ID>` is set by
`PALIMPSESTE_OIDC_<ID>_NAME`, `_ISSUER`, `_CLIENT_ID`, `_CLIENT_SECRET` and
`_SCOPES`. Register `{public-url}/api/auth/{id}/callback` at the provider.
Keep secrets in an untracked `mise.local.toml`. With no provider, the
server starts and warns that nobody can sign in. Logs go through `tracing`
and are filtered by `RUST_LOG`. They carry ids only, never documents,
tokens or claims (ADR BE-LOGS).

The SQLite file is created if missing. Opening it migrates its schema,
versioned by `PRAGMA user_version`, in place; a database newer than the
server is refused (ADR BE-STORE).

## Recipes

Follow the backend-structure skill. In short:

- **A new use case:** a method on `PalimpsesteService` (`lib.rs`), its
  implementation in `service.rs`, any rule on a value as a constructor of
  its model type, and a handler and view in `api/http.rs`.
- **New persistence:** a method on `PalimpsesteRepository`, implemented in
  `repo/sqlite.rs` on `spawn_blocking`. A schema change is a migration
  appended to `MIGRATIONS`, run once per database, with a test that opens
  a database of the previous version.
- **A new configuration option:** a clap field with an `env` in `main.rs`,
  and a row in ADR BE-CONFIG.

## Testing

- **Model:** pure unit tests in each model file.
- **Service:** `service.rs` against `MockPalimpsesteRepository` and
  `MockIdentityProviders` (`mockall::automock`, test builds only).
- **HTTP adapter:** `api/http.rs` drives the router with `tower::oneshot`
  against `MockPalimpsesteService`. `api/http/sign_in.rs` runs the whole
  OIDC flow against a local test issuer, signed with
  `tests/fixtures/issuer-key.pem`.
- **SQLite:** `repo/sqlite.rs` runs on in-memory and temporary database
  files, migrations included.

```sh
mise run check          # fmt, lint, test (nextest), doc-test
```
