# Backend

`app/backend` (`palimpseste-backend`) serves the application, signs users
in, and checks and stores their characters (ADR APP-STORE-ONLY). The
day-to-day guide to its layout is the backend-structure skill; where the
skill and this document disagree, this document wins and the skill is
updated.

## Structure

**BE-CRATE: One crate, library and binary.** axum, tokio and tower-http,
in a library (`lib.rs`) and a binary (`main.rs`) that is the composition
root only. No Cargo features: one binary is the only deployment.

**BE-ONE-CONTEXT: One bounded context.** One use-case trait,
`PalimpsesteService`. *Why:* the backend is small, serves one application
and is not meant to be split; a second context needs its own decision.

**BE-LAYOUT: Hexagonal, without its vocabulary.** After
[howtocodeit's guide](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust):

- `lib.rs`: `PalimpsesteService`, `Error`, `ActionContext`, and the
  re-exports forming the public API;
- model files named by concept, flat at the root (`user.rs`, `session.rs`,
  `sign_in.rs`, `character.rs`), whose value objects (`Nickname`,
  `Profile`, `CharacterName`, `Version`, ...) parse and hold the rules;
- `service.rs`: `PalimpsesteServiceImpl<R, P>`, generic over its
  repository and identity providers;
- one directory per adapter role, behind a declaration-only file:
  `repo.rs` and `repo/sqlite.rs`, `provider.rs` and `provider/oidc.rs`,
  `api.rs` and `api/http.rs`.

No `mod.rs`, no `domain/`, `infrastructure/` or `ports/`, no `types.rs`
or `models.rs`.

**BE-ADAPTERS: Adapters translate only.** JSON views, cookies, `ETag` and
`If-Match`, redirects and the front end live in `api/http*`; SQL and
`spawn_blocking` in `repo/sqlite.rs`; the OIDC protocol in
`provider/oidc.rs`. Rules live in the model and the service.

**BE-INJECTION: Wired in `main.rs`, tested against mocks.** The service
holds its repository and providers as type parameters; the HTTP adapter
holds it as `Arc<dyn PalimpsesteService>`; only `main.rs` builds and wires
them. Traits use `async-trait` (for `dyn`) and generate `mockall` mocks
under `cfg(test)`. Use cases are tested against mocks of the repository
and providers, the HTTP adapter against a mock of the service and end to
end, the SQLite repository alone, and sign-in against an issuer the test
serves (key in `tests/fixtures/issuer-key.pem`).

**BE-ACTION-CONTEXT: Ownership is the whole policy.** Every use case
acting for a signed-in user takes `ctx: &ActionContext` (the user's id)
first, and every repository query is scoped by it. *Why:* with no roles
nor parties, an authorizer would anticipate; it arrives with them.

**BE-ERRORS: One `Error` enum.** With `thiserror`, each condition the user
can react to is a named variant (`NicknameTaken`, `Stale`,
`SignInFailed`, ...); technical failures are `Underlying(anyhow::Error)`,
given a `.context(...)` where they leave an adapter. The HTTP adapter maps
variants to statuses and never shows an `Underlying` cause. No `Result<T>`
alias.

## API

**BE-API: The routes.** Every route but `/api/auth/*` answers `401`
without a live session; another user's character is `404`, as if it did
not exist; an unknown path under `/api` is `404`.

| Method | Path | Result |
|---|---|---|
| `GET` | `/api/auth/providers` | `[{id, name}]` |
| `GET` | `/api/auth/{provider}/login` | `303` to the provider, with the `palimpseste_login` cookie; `404` for an unknown provider |
| `GET` | `/api/auth/{provider}/link` | as `login`, to link an identity (BE-IDENTITIES); `303` to `/login` without a session |
| `GET` | `/api/auth/{provider}/callback` | `303` to `/` with the session cookie, to `/register` with the registration cookie for an identity no user has (BE-FIRST-SIGN-IN), or to `/login?signin=failed`; when linking, to `/profile?link=done`, `already`, `taken`, `provider` or `failed` |
| `GET` | `/api/auth/registration` | `{nickname, provider, label}`: the nickname suggested, the provider name (else the issuer) and the label; `404` without a live registration |
| `POST` | `/api/auth/registration` `{nickname, profile?}` | `201`, the user created, with the session cookie, the registration cookie removed; `404` without a live registration |
| `DELETE` | `/api/auth/registration` | `204`, the registration cancelled and its cookie removed |
| `POST` | `/api/auth/logout` | `204`, the session ended and its cookie removed |
| `GET` | `/api/me` | `{nickname, profile}` |
| `PATCH` | `/api/me` `{nickname?, profile?}` | the user, the fields given changed |
| `GET` | `/api/me/identities` | `[{provider, name, label}]`: the provider id (`null` when no longer configured) and name (else the issuer) |
| `GET` | `/api/me/characters` | the user's characters |
| `POST` | `/api/me/characters` `{name, engine}` | `201`, the character with its engine's empty document |
| `GET` | `/api/characters/{id}` | `{id, name, engine, document, version}`, the version in `ETag` |
| `PATCH` | `/api/characters/{id}` `{name}` | the renamed character |
| `DELETE` | `/api/characters/{id}` | `204` |
| `PUT` | `/api/characters/{id}/document` | `204`, the new version in `ETag` |

**BE-CHECKS: What the backend rejects.** Nothing else: bounds, broken
rules and missing choices are stored as sent, and the backend never
computes violations.

| Status | Request |
|---|---|
| `400` | malformed JSON |
| `422` | JSON of the wrong shape (an unknown field in `PATCH /api/me` or `POST /api/auth/registration`, an unknown engine, a negative rank); a blank or too long nickname or profile; a blank character name; a document of another engine than the character's, or unreadable (ADR APP-READABLE) |
| `409` | a nickname another user has; a registration of an identity another browser registered meanwhile |
| `428` | a document `PUT` without `If-Match` |
| `412` | a document `PUT` at a stale version, with the current `ETag` |

**BE-STORE: SQLite through `rusqlite`.** `rusqlite` (`bundled`), one
connection, each query on `spawn_blocking`. Tables `users`, `identities`,
`sessions`, `logins`, `registrations` and `characters`; a document is stored as JSON text
next to a version counter, its `ETag`. Opening the store creates the
tables and migrates an older schema in place, each migration running when
its column is missing. *Why not `sqlx`:* a handful of queries, and no
database at build time.

## Sign-in and users

**BE-OIDC: Any OpenID Connect provider.** Any provider with OIDC discovery
can be configured (Google, Microsoft, Keycloak, Authentik, Dex, ...);
GitHub, OAuth2 only, cannot. The flow is Authorization Code with PKCE
(S256), `state` and `nonce`, through `openidconnect` and its `reqwest`
client, following no redirect; the scopes are `openid` and, by default,
`profile email`. The ID token's signature, issuer, audience, expiry and
nonce are verified; the access token is not used. Discovery runs when a
sign-in starts and when it ends, never cached. *Why:* a rotated key needs
no invalidation, a provider down at startup does not stop the server, and
sign-ins are rare.

**BE-SIGN-IN: A sign-in is bound to its browser.** Starting one stores its
provider, `state`, `nonce` and PKCE verifier for 10 minutes (`logins`),
under a random id in the `palimpseste_login` cookie (`Path=/api/auth`).
The callback requires that cookie, takes the sign-in out of the store (it
completes at most once) and checks `state` against it, so a callback
started by someone else cannot sign a victim into the wrong account.
Expired sessions and sign-ins are purged when a sign-in starts.

**BE-SESSIONS: Server-side sessions.** A session is a random 32-byte token
in the `palimpseste_session` cookie (`HttpOnly`, `SameSite=Lax`, `Path=/`,
30 days, `Secure` under `https`). The store keeps only its SHA-256, with
the user and the expiry, so reading the database gives no usable session.
`SameSite=Lax` and JSON bodies keep other sites from acting through the
API. Cookies hold hex tokens and are read and written by hand.

**BE-USERS: A private id, a public nickname and profile.** The id
(`UserId`) keys a user's characters, sessions and identities, and never
leaves the server. The nickname is unique regardless of ASCII case (an
index `COLLATE NOCASE`); trimmed, it is not blank and holds at most 32
characters. The profile is free text, trimmed, at most 2000 characters,
empty until written. Their owner changes them. "Public" says what others
may see once friends or parties exist; no route shows a user to another
yet.

**BE-FIRST-SIGN-IN: The first sign-in creates an account, once asked.**
Signing in and signing up are one flow: the first sign-in of an identity no
user has creates no user. It starts a registration, kept for 30 minutes
(`registrations`): the identity, its label, and a suggested nickname,
from the `preferred_username`, else the `name` claim, else "Joueur", never
the email (a nickname is public), cut to 28 characters; when taken, the
first free of `nickname 2`, `nickname 3`, ... Its random token travels in
the `palimpseste_registration` cookie (`HttpOnly`, `SameSite=Lax`,
`Path=/api/auth/registration`), and the store keeps only its SHA-256, as
for sessions. The browser lands on `/register`, where the person chooses
their nickname and profile: completing the registration creates the user
and its identity in one transaction, and opens a session. Cancelling it
deletes it, leaving the identity free; so does letting it expire. A taken
nickname keeps the registration for another try. *Why not create the user
at once, flagged unregistered:* an abandoned sign-up would leave a user
holding a nickname, and every route would check the flag.

**BE-IDENTITIES: A user has one or more identities.** An identity is an
issuer and a subject, keyed by issuer rather than provider id so a
provider can be renamed. Its private label (the `email`, else the
`preferred_username`, else the `name`) is refreshed at each sign-in or
link, so its owner tells their identities apart. Any identity of a user
signs in to that user. A user has at most one identity per provider
(issuer): linking a second one is refused (`link=provider`), and an index
on `identities (user_id, issuer)` keeps it so. Linking is a sign-in started by a signed-in user:
the same flow and callback (one redirect URI per provider), with the user
recorded in `logins.link_user`; the callback then links the identity
instead of opening a session. Another user's identity is never moved nor
merged (`link=taken`).

## Operation

**BE-CONFIG: clap options, each with an environment variable.**

| Option | Variable | Default |
|---|---|---|
| `--addr` | `PALIMPSESTE_ADDR` | `127.0.0.1:3000` |
| `--frontend-dir` | `PALIMPSESTE_FRONTEND_DIR` | `app/frontend/dist` |
| `--database` | `PALIMPSESTE_DATABASE` | `palimpseste.sqlite3` |
| `--public-url` | `PALIMPSESTE_PUBLIC_URL` | `http://127.0.0.1:3000` |
| `--oidc-providers` | `PALIMPSESTE_OIDC_PROVIDERS` | none |

The public URL is where users reach the server: a provider redirects to
`{public-url}/api/auth/{id}/callback`, and `https` makes cookies `Secure`.
Each provider id (lower-case letters, digits and dashes) is set by
`PALIMPSESTE_OIDC_<ID>_NAME`, `_ISSUER`, `_CLIENT_ID`, `_CLIENT_SECRET`
and optionally `_SCOPES` (besides `openid`, default `profile email`;
Discord needs `identify email`), `<ID>` upper-cased with dashes as
underscores. `main.rs` reads these, as clap cannot declare computed names.
A missing variable, an invalid id or URL, or an id given twice stops the
server; with no provider, it starts and warns that nobody can sign in.
`--help` lists it all. To try it locally, register the redirect URI
`http://127.0.0.1:3000/api/auth/google/callback` at the provider, then:

```sh
PALIMPSESTE_OIDC_PROVIDERS=google \
PALIMPSESTE_OIDC_GOOGLE_NAME=Google \
PALIMPSESTE_OIDC_GOOGLE_ISSUER=https://accounts.google.com \
PALIMPSESTE_OIDC_GOOGLE_CLIENT_ID=... \
PALIMPSESTE_OIDC_GOOGLE_CLIENT_SECRET=... \
mise run serve
```

With `mise run frontend:dev`, Vite proxies `/api`: set the public URL to
Vite's origin and register that redirect URI.

**BE-LOGS: `tracing`, ids only.** The backend alone logs, through
`tracing`, printed by `tracing-subscriber` and filtered by `RUST_LOG`
(default `info`; `info,tower_http=warn` keeps only the application's
events).

- tower-http's `TraceLayer`: each request with its method, URI, status
  and latency at `info`, server errors at `error`.
- `main.rs`: the providers, the database and the address at startup, a
  warning when the front end directory has no `index.html`.
- The service: each domain event at `info` (registration started and
  cancelled, user created, signed in and out, identity linked, profile
  updated, character created, renamed or
  deleted, document stored with its version).
- The HTTP adapter: conflicts and stale versions at `info`; unprocessable
  requests with why, a missing `If-Match` and failed sign-ins at `warn`;
  internal errors at `error`.

Ids are logged, never documents, tokens or claims.

**BE-FRONTEND: The same router serves the front end.** Every path outside
`/api` is a file of the front end directory, falling back to `index.html`
so the PWA routes itself. Files under `/assets/` are named after their
content and cached `immutable` for a year; a missing one is `404`, so a
stale path is never cached as the app shell. Every other file
(`index.html`, the service worker, the manifest, the icons) is served
`no-cache`, so a new build reaches users.

## Open

- Unlinking an identity (keeping at least one), and merging two users.
- Showing users to each other, roles and an authorizer: with friends or
  parties.
- Structured error bodies, and the body of a `412`.
