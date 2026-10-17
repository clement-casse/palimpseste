---
name: backend-structure
description: Authoritative, opinionated guide for laying out and growing `app/backend` (the `palimpseste-backend` crate). Use when adding a use case, a repository method, an HTTP route, a domain rule, a model type, an adapter or a configuration option to the backend, when reviewing backend code, or when answering "where should X go?" in it. The conventions are deliberate compromises between hexagonal architecture (after howtocodeit's hexagonal Rust guide) and this codebase's constraints, recorded in docs/adr/backend.md. Apply them as they are. The only reason to deviate is a real technical impossibility.
---

# Backend structure

This skill is the day-to-day reference for working inside `app/backend/`.
It is **opinionated by design**: it encodes trade-offs already settled in
[docs/adr/backend.md](../../../docs/adr/backend.md). `domain.rs`,
layered folders, an `XxxApiServer` pass-through trait, a `Result<T>` alias
and error enums with a variant per technology are rejected; do not bring
them back under another name.

The point is **maintainability through unsurprising structure**: once a
reader has accepted the dependency-injection discipline, every file is
where they expect it.

When this skill and `docs/adr/backend.md` disagree, the document wins
and this skill is updated.

---

## 1. The stance

The backend aims for an **acceptable compromise** between hexagonal
architecture and pragmatic Rust, for *this* application. It honours every
load-bearing principle of
[howtocodeit's hexagonal Rust guide](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust):
the domain at the centre, traits as seams, adapters as translators,
dependencies pointing inward, dependency injection enforcing testability.
It makes two compromises the guide does not:

1. **One `Arc<dyn PalimpsesteService>`, held by the HTTP adapter.** Inside
   the domain, the service holds its repository and identity providers as
   **type parameters** (zero cost, the guide's pattern). The adapter erases
   the service to `Arc<dyn>` once, so the axum state and handlers are not
   generic over every dependency. Mocks plug in through the same generics
   for service tests, and through `Arc::new(MockPalimpsesteService::new())`
   for adapter tests.
2. **A single `Underlying(#[from] anyhow::Error)` catch-all** in `Error`,
   **for technical failures only**. Business outcomes are named
   exhaustively; `Underlying` carries what is outside the domain language
   (the database, a provider unreachable, a configuration error). See §7.

Every other decision is the guide's. When in doubt, do what it would do.

**Dependency injection is non-negotiable.** Every dependency crossing a
layer travels as a trait. The service never builds its repository; an
adapter never builds the service; nothing reads the environment or a
global. Composition happens in `main.rs` only.

---

## 2. What the crate is

`palimpseste-backend` is **one bounded context**, packaged as a library
(`lib.rs`) driven by a binary (`main.rs`). The backend serves one
application and is not meant to be split or scaled out, so there is one use
case trait, one repository trait, one `Error`.

Two consequences for everything you do:

1. **The headline of the crate is the domain.** Open `lib.rs` and you see
   `PalimpsesteService`, `Error`, `ActionContext` and the re-exported model
   types. Nothing else competes for attention.
2. **Implementations are internal.** `PalimpsesteServiceImpl`, `SqliteRepo`,
   `OidcProviders`, `HttpApi` exist so that `main.rs` can compose a server.
   Keep the public surface minimal:

| Public from `lib.rs` | Reason |
|---|---|
| Model types (`User`, `Nickname`, `Character`, `Version`, `PendingSignIn`, ...) | The traits traffic in them. |
| `PalimpsesteService` + `Error` + `ActionContext` | The contract. |
| `PalimpsesteServiceImpl::new` | `main.rs` wires it. |
| `PalimpsesteRepository` + `SqliteRepo::open` | The persistence seam and its adapter. |
| `IdentityProviders` + `OidcProviders::new` + `ProviderConfig` | The sign-in seam and its adapter. |
| `HttpApi::new` + `HttpApi::router` | `main.rs` serves it. |
| Anything else | **No.** `pub(crate)` or module-private. |

Do not re-export helpers, internal value objects, conversions or "for
convenience" types. If `main.rs` or a trait signature does not need it, it
is not public. The crate is a library: `missing_docs` applies to everything
public.

---

## 3. The canonical layout

```
app/backend/
├── Cargo.toml
├── tests/fixtures/        # test-only data (the issuer's RSA key)
└── src/
    ├── main.rs            # composition root: clap Config, tracing, wiring, serve
    ├── lib.rs             # PalimpsesteService, Error, ActionContext, re-exports
    ├── user.rs            # model: UserId, User, Nickname, Profile, Identity
    ├── session.rs         # model: SessionToken, TokenHash, SESSION_TTL
    ├── sign_in.rs         # model: ProviderId, PendingSignIn, VerifiedIdentity, SignInEnd, ...
    ├── character.rs       # model: CharacterId, CharacterName, Version, Character
    ├── service.rs         # PalimpsesteServiceImpl<R: PalimpsesteRepository, P: IdentityProviders>
    ├── repo.rs            # PalimpsesteRepository trait
    ├── repo/
    │   └── sqlite.rs      # SqliteRepo: schema, migrations, queries, spawn_blocking
    ├── provider.rs        # IdentityProviders trait
    ├── provider/
    │   └── oidc.rs        # OidcProviders, ProviderConfig (openidconnect)
    ├── api.rs             # hat: pub mod http;
    ├── api/
    │   ├── http.rs        # HttpApi: router, error → status, session extractor, views, handlers
    │   └── http/
    │       ├── sign_in.rs # /api/auth/*: redirects, cookies
    │       └── frontend.rs# the built PWA, cache headers
```

Hard rules:

- **No `mod.rs`.** A module with children is `<name>.rs` next to
  `<name>/`: `repo.rs` + `repo/sqlite.rs`, never `repo/mod.rs`.
- **No layered folders** (`domain/`, `application/`, `infrastructure/`,
  `ports/`, `adapters/`). The model is files at the root; adapters sit in
  their role directory.
- **Role directories only:** `repo/`, `provider/`, `api/`. Each groups a
  trait (or, for `api`, a declaration-only hat) with the adapters that
  fulfil it. A need that fits none gets a new role (§5).
- **No file named `domain.rs`, `types.rs`, `models.rs`, `entities.rs`,
  `utils.rs`** (§4).
- **`main.rs` holds no logic** beyond reading the configuration, building
  each piece in dependency order, logging the startup, and serving.

---

## 4. Model files

The model is what makes the backend distinct. Treat its files accordingly.

### Naming: by concept, not by category

Each model file is named after the **concept it describes**. "domain" is
banned, and so are its aliases (`types.rs`, `models.rs`, `entities.rs`):
the name says *which* concept.

| File | Holds |
|---|---|
| `user.rs` | `UserId`, `User`, `Nickname`, `Profile`, `Identity`, nickname candidates |
| `session.rs` | `SessionToken`, `TokenHash`, `SESSION_TTL`, random tokens |
| `sign_in.rs` | `ProviderId`, `ProviderSummary`, `SignInId`, `Authorization`, `PendingSignIn`, `ProviderAnswer`, `VerifiedIdentity` (nickname and label from claims), `Linking`, `SignInEnd`, `LinkedIdentity`, `SIGN_IN_TTL` |
| `character.rs` | `CharacterId`, `CharacterName`, `Version`, `Character` |

### Rules live in the model

A rule about one value is a constructor of its value object:
`Nickname::parse` trims, refuses a blank or too long nickname;
`CharacterName::parse` refuses a blank name. A rule deriving one value from
another is a function or method next to the type
(`VerifiedIdentity::wanted_nickname`, `nickname_candidates`). These are
pure and unit-tested in their file.

Model types carry **no `serde`** and no framework type. Their JSON shape is
the HTTP adapter's (`UserView`, `CharacterView`). Fields the repository
must build without re-validating (`Nickname(pub(crate) String)`) are
`pub(crate)`, never `pub`.

### One concept per file, split as it grows

Split a model file when it becomes hard to scan, hard to test alone, or
mixes concepts, before it hurts, not after. Name the new file after the
concept it takes away. Stay flat at the root until the domain itself is
hierarchical; then promote an umbrella (`<concept>.rs` + `<concept>/`).

### The cognitive-load test

1. Can a contributor opening the crate cold tell where a concept lives
   without a map?
2. If not, is the map obvious from `tree src/`?
3. Will both still hold after the next two reasonable additions?

Three yeses: keep it. A no: restructure *first*, in its own change.

---

## 5. Adapters: roles, hats, and translation only

Adapters are **translators**: they convert what crosses the crate's
boundary into and out of the model. They own no business logic. A
non-trivial conditional in an adapter (choosing a path, enforcing a rule,
deciding what to do when X) belongs in `service.rs` or the model.

### Roles

| Role | Direction | Trait | Adapter |
|---|---|---|---|
| `repo/` | outbound: persistence | `PalimpsesteRepository` | `sqlite.rs`: `SqliteRepo` |
| `provider/` | outbound: identity providers | `IdentityProviders` | `oidc.rs`: `OidcProviders` |
| `api/` | inbound: transports | none (hat only) | `http.rs`: `HttpApi` |

A need that fits none (sending mail, a queue) **gets a new role**: a trait
in `<role>.rs`, an adapter in `<role>/<technology>.rs`, the same discipline.
Do not stuff it into an existing role.

### The hat file

`api.rs` declares and does nothing else:

```rust
//! Technical adapters serving [`crate::PalimpsesteService`].

pub mod http;
```

`repo.rs` and `provider.rs` hold their trait, the `pub mod` of their
adapters and the re-export of the adapter type. No logic.

### What each adapter does

- **`api/http.rs`** holds `Arc<dyn PalimpsesteService>`. Per request:
  1. translate the input into model types (`Nickname::parse`,
     `CharacterId::from`, `if_match` → `Version`, the session cookie →
     `SignedIn` → `ActionContext`);
  2. call `state.service.<use case>(&ctx, ...)`;
  3. translate the result: model → view (`CharacterView`), `Error` →
     status through `Rejection`, `Underlying` → a bare `500` with the cause
     logged, never shown;
  4. **nothing else**: no repository call, no rule.
  `HttpApi::router()` gives the whole application router (`/api`, the
  front end, tracing); `main.rs` only serves it. HTTP-only conditions stay
  here (`428` without `If-Match`, `ETag` formatting, cookie flags,
  redirect targets).
- **`repo/sqlite.rs`** owns the SQL: the schema and its migrations
  (idempotent, applied by `SqliteRepo::open`), the queries, and running
  them on `spawn_blocking` behind one connection (`call`). It decides only
  what needs a transaction: compare-and-swap of a version, find-or-create,
  uniqueness. Such outcomes become named variants (§7).
- **`provider/oidc.rs`** owns the protocol: discovery, the authorization
  URL with PKCE, `state` and `nonce`, the code exchange, ID token
  verification. It returns raw claims (`VerifiedIdentity`); what they mean
  (the nickname, the label) is the model's.

There is no `XxxApiServer` trait between an adapter and the service: it
would only forward calls.

---

## 6. Configuration and composition

`main.rs` is the composition root:

1. `Config::parse()` with clap (`derive`, `env`): each option has a flag,
   its `PALIMPSESTE_*` variable and a default, and appears in `--help`.
   Settings whose variable names are computed (the per-provider
   `PALIMPSESTE_OIDC_<ID>_*`) are read by a function next to `Config`,
   with an error naming the variable.
2. Initialise tracing (`RUST_LOG`).
3. Build in dependency order: `OidcProviders::new` → `SqliteRepo::open` →
   `PalimpsesteServiceImpl::new(repo, providers)` →
   `HttpApi::new(Arc::new(service), ...)`.
4. Bind and serve.

`main` returns `anyhow::Result<()>`; each startup step adds a
`.context(...)`. Adapters take plain values (`&Path`, `&str`,
`Vec<ProviderConfig>`) and **never read the environment**.

There are **no Cargo features**: one binary is the only deployment.

---

## 7. Errors

The reference is
[howtocodeit's error handling guide](https://www.howtocodeit.com/guides/the-definitive-guide-to-rust-error-handling);
this section spells out the compromise.

### One enum, named `Error`

`lib.rs` defines one `thiserror` enum named `Error`: no prefix
(`BackendError` is forbidden), no `type Result<T>` alias. Every trait method
(service, repository, providers) returns `Result<T, Error>`.

### The audience test is the only criterion

> **Is the user supposed to see and react to this error, or is it a
> technical failure for whoever runs the server?**

- **User-facing → a named variant**, exhaustive and matchable:
  `NoSession`, `UnknownProvider`, `BlankNickname`, `NicknameTooLong`,
  `ProfileTooLong`, `NicknameTaken`, `IdentityTaken`, `CharacterNotFound`,
  `BlankName`, `UnreadableDocument`, `OtherEngine`, `Stale`, `SignInFailed`.
  The user can change what they do: pick another nickname, reload, sign in
  again.
- **Technical → `Error::Underlying(anyhow::Error)`**: the database, a
  provider unreachable, a configuration error, a stored document that no
  longer reads. Nothing the user did causes it. The HTTP adapter answers a
  bare `500` and logs the chain (`{cause:#}`).

Do not reason about "is it the database" or "is it I/O". Reason about who
the message is for.

### A flow that always ends in a user outcome

Signing in always ends by sending the browser back to the app, told
whether it worked. So the service turns every failure of the flow,
technical or not, into `SignInFailed { linking, cause }`: a user outcome
("sign-in failed, try again", on the login or profile page, chosen by
`linking`) carrying the technical cause for the logs, never for the user.
Use this shape, a named variant with an `anyhow::Error` cause, only for
flows whose every failure has one user-facing ending.

### The boundary: `.context("...")?`

In an adapter, a technical failure enters the domain through `anyhow`:

```rust
// repo/sqlite.rs
let result = tokio::task::spawn_blocking(move || query(&mut lock(&connection)))
    .await
    .context(what)?; // JoinError → anyhow → Error::Underlying
Ok(result.context(what)?) // rusqlite::Error → anyhow → Error::Underlying
```

- **The message is for developers**: it names what was being done
  ("replacing a document", "discovering provider dex").
- **`.context("...")?` is the form.** A bare `?` loses the breadcrumb;
  `.map_err(|_| ...)` loses the source.

### Business-shaped technical signals become named variants

A version that moved under a compare-and-swap is `Error::Stale(version)`;
an identity already owned by another user is `Error::IdentityTaken`; a
nickname another user holds is `Error::NicknameTaken`. The repository
detects them inside its transaction and returns the variant, not
`Underlying`.

### Anti-patterns to refuse

- A business failure in `Underlying` ("character not found" is a variant).
- A technical variant (`Error::Database(String)`, `Error::Io`,
  `Error::Internal`, `Error::Other`): use `Underlying`.
- A `?` across the boundary without `.context("...")`.
- `anyhow::Result` from a trait method.
- An adapter's error type (`rusqlite::Error`, `reqwest::Error`) in a trait
  signature.
- Matching on an error's `Display` to decide behaviour. (Using it as a
  response body is fine.)
- An `Underlying` cause, or any developer message, in a response body.

### When to panic

Never for a recoverable error. A configuration the server cannot start
without is an `Err` returned from `main`. Inside the library, return
`Error`. (`unwrap`, `expect` and `panic!` are denied outside tests by the
workspace lints.)

---

## 8. One context, and when that changes

Everything is one bounded context: there is no cross-service dependency to
manage. If a concern appears that has its own lifecycle, language and data
(campaigns, parties), **do not** add a second trait inside this crate or a
module named after it with its own layout. Propose the split (a second
context in this crate, or a second crate) as a change to
ADR BE-ONE-CONTEXT; AGENTS.md forbids new crates without one.

---

## 9. Trait shape and dependency injection

The three traits are decorated alike:

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PalimpsesteService: Send + Sync + 'static {
    async fn characters(&self, ctx: &ActionContext) -> Result<Vec<Character>, Error>;
}
```

- `#[async_trait]`: required for `Arc<dyn PalimpsesteService>`.
- `#[cfg_attr(test, mockall::automock)]` **before** `#[async_trait]`: the
  mocks (`MockPalimpsesteService`, `MockPalimpsesteRepository`,
  `MockIdentityProviders`) exist under `cfg(test)` only.
- Bounds are exactly `Send + Sync + 'static`; not `Clone`.
- **Every use case acting for a signed-in user takes
  `ctx: &ActionContext` first.** Sign-in use cases, which have no user yet,
  do not. The repository takes the `UserId` explicitly and scopes every
  query by it: that is how a user reaches only their data.
- Prefer owned or `Copy` parameters to `Option<&T>` in traits (mockall).

```rust
pub struct PalimpsesteServiceImpl<R: PalimpsesteRepository, P: IdentityProviders> {
    repo: R,
    providers: P,
}
```

Dependencies are **type parameters**, not `Arc<dyn ...>`. Use cases are
**methods of `PalimpsesteService`**: no use-case file, no command objects.
The trait *is* the use-case surface.

There is **no authorizer** yet: with no roles, ownership through
`ActionContext` is the whole policy. An `authorizer` role (trait
`PalimpsesteAuthorizer`, domain-typed checks, a permissive implementation)
arrives with parties or roles, by a change to ADR BE-ACTION-CONTEXT.

---

## 10. Testing

Run everything through mise: `mise run check` (fmt, clippy, nextest, doc
tests); `mise run test` for the tests alone. Four shapes:

- **Model tests**, in each model file: value objects and derivations, no
  I/O.
- **Service tests** (`service.rs`): `MockPalimpsesteRepository` and
  `MockIdentityProviders` with expectations, then
  `PalimpsesteServiceImpl::new(repo, providers)` and the use case.
- **Repository tests** (`repo/sqlite.rs`): `SqliteRepo::in_memory()`, or a
  temporary file for persistence and migrations, through the trait.
- **HTTP tests** (`api/http*.rs`):
  - with `MockPalimpsesteService` for one mapping (an `Underlying` is a
    bare `500`);
  - end to end, with the real service on `SqliteRepo::in_memory()`, and
    for sign-in a local OIDC issuer (`api/http/sign_in.rs`). These pin the
    HTTP contract the front end relies on: statuses, bodies, cookies,
    redirects, `ETag`. Change their assertions only with a change to
    ADR BE-API or ADR BE-CHECKS.

Do not hand-write mocks: `automock` generates them.

---

## 11. Recipes

### Add a use case

1. Add the method to `PalimpsesteService` in `lib.rs`; `ctx: &ActionContext`
   first if it acts for a signed-in user.
2. If it needs new persistence, add the method to `PalimpsesteRepository`
   (`repo.rs`) and implement it in `repo/sqlite.rs` **first**, with its
   repository test.
3. Implement it in `service.rs`; log the domain event (ids, never
   documents, tokens or claims, ADR BE-LOGS); add a service test with mocks.
4. Expose it in `api/http.rs`: a route, a handler translating in and out,
   a view if it returns data; add an end-to-end test.
5. Update the API table (ADR BE-API) in docs/adr/backend.md.

### Add a domain rule

A rule about one value goes in its value object's constructor in the model
file, returning a new named `Error` variant. A rule spanning values or
needing data goes in `service.rs`. The HTTP adapter maps the new variant
to a status in `Rejection`. If the rule needs atomicity, the repository
detects the condition in its transaction and returns the named variant.

### Add a model type or split a model file

Put the type in the file of its concept, or a new file named after it;
declare the module in `lib.rs` and re-export only what a trait signature or
`main.rs` uses.

### Add a column or a table

Change `SCHEMA` in `repo/sqlite.rs` and add a migration to `MIGRATIONS`,
applied when its column is missing, so existing databases open; add a
test migrating a database of the previous schema.

### Add a configuration option

Add a field to `Config` in `main.rs` with `#[arg(long, env =
"PALIMPSESTE_...", default_value = ...)]` and a doc comment (it is the
`--help` text); pass the value to the constructor that needs it.

### Add an identity provider protocol or another outbound adapter

A new protocol behind the same trait: `provider/<protocol>.rs`, chosen in
`main.rs`. A new kind of dependency: a new role (§5), held by the service as
a new type parameter, mocked under `cfg(test)`.

### Add a transport

`api/<transport>.rs` holding `Arc<dyn PalimpsesteService>`, declared in
`api.rs`, wired in `main.rs`.

---

## 12. The opinionated stance, and its single escape hatch

Refuse, pointing to ADR BE-LAYOUT or this skill:

- a `domain.rs`, `types.rs`, `models.rs`, a `mod.rs`;
- layered folders, hexagonal vocabulary in names (`ports`, `adapters`,
  `inbound`, `outbound`);
- an `XxxApiServer` trait, a `Result<T>` alias, a prefixed error enum;
- a rule in a handler, an SQL query or the OIDC code;
- `serde` on a model type;
- an adapter reading the environment, or wiring outside `main.rs`;
- a technical `Error` variant, or a user-facing condition in `Underlying`.

There is **room for judgement on quantity** (how many model files, when to
promote a directory, when to split `api/http.rs`), never on the categories.

**The single escape hatch** is a real *technical* impossibility: something
the compiler, Cargo or a library cannot do while the conventions hold.
When it fires:

1. Do not code around it with a "small exception".
2. Propose the change to docs/adr/backend.md, with the failure (compiler
   error, ...) that justifies it.
3. Once accepted, update this skill so the exception becomes the rule.

Preference, elegance, "it would be cleaner if..." are not triggers. Unsure
whether a problem is technical or stylistic? It is stylistic.

---

## 13. Pointers

- [docs/adr/backend.md](../../../docs/adr/backend.md): this structure
  (Structure), the routes and what they reject (API), SQLite, sign-in,
  sessions, users and identities, configuration, what is logged.
- [docs/adr/application.md](../../../docs/adr/application.md): why the
  backend never applies a rule, the choices document it stores.
- [AGENTS.md](../../../AGENTS.md): scope, design principles, Rust
  conventions.
- [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust)
  and [The Definitive Guide to Rust Error Handling](https://www.howtocodeit.com/guides/the-definitive-guide-to-rust-error-handling):
  the references, followed except for the compromises of §1.
