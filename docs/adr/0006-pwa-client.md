# 0006. PWA client: REST API, engines compiled to WebAssembly

Date: 2026-10-16.
Status: accepted.

## Context

The application users install is a Progressive Web App. A user signs in
and creates, edits and deletes characters of any Chroniques Oubliées
engine. The engines (ADR 0001 to 0004) are Rust crates with no I/O; the
backend (ADR 0005) stores users and characters. Two questions follow: where
the rules run, and how the client talks to the server.

## Decisions

- **The rules run in the browser.** The engines are compiled to
  WebAssembly and called from the front end. Derived values, breakdowns and
  violations are computed on the client, as the user edits, and work
  offline. The server stores choices and never judges them: a character
  that breaks a rule is still saved, since the Game Master can override
  every rule. It rejects only a document it cannot read.
- **A REST API with JSON bodies.** Characters are resources under `/api`
  (list, read, create, replace, delete), served by the axum backend.
  - Not GraphQL: the client always loads a whole character and never needs
    field selection across a graph. GraphQL would send every request as a
    POST to one endpoint, which a service worker cannot cache or replay by
    URL. It would also add a second schema, with one union per engine for
    the choices, and GraphQL has no input unions.
  - Not Connect (protobuf RPC): its `.proto` schema would duplicate the Rust
    types, which are the contract, and map Rust enums carrying data to
    verbose `oneof`s. There is no official Connect server for Rust.
  - REST gets the PWA features from HTTP itself: URL caching in the service
    worker, `ETag` and `If-Match` for concurrent edits (see Follow-ups).
- **New crate `app/wasm`, package `palimpseste-wasm`.** It depends on the
  kernel and the engines and exposes them to JavaScript with
  `wasm-bindgen`. A binding is a stateless function over plain data:
  choices in, report or values out. No Rust object crosses the boundary,
  so JavaScript never frees memory by hand, and an engine call recomputes
  everything (cheap: microseconds).
- **Types cross the boundary through `serde` and `tsify`.** `tsify`
  generates the TypeScript declarations from the Rust types. Arguments and
  results are wrapped in `tsify::Ts`, so a value JavaScript got wrong
  throws an error instead of panicking. The ABI attributes
  `into_wasm_abi`/`from_wasm_abi` are deprecated: they leak memory when
  (de)serialization fails. `palimpseste-wasm` may use `serde`; the kernel
  and the engines stay free of it. The crate maps the kernel types to its
  own serializable mirrors (violation, subject, category, severity).
- **Built with `cargo` and `wasm-bindgen-cli`, not `wasm-pack`.** `cargo`
  builds the `wasm32-unknown-unknown` target, and `wasm-bindgen --target
  web` writes an ES module with its `.d.ts` into
  `app/frontend/src/wasm/`, which is generated and not committed. One tool
  fewer than with `wasm-pack`. The CLI version, pinned in `mise.toml`,
  must equal the `wasm-bindgen` crate version in `Cargo.lock`.
- **Front end in `app/frontend`: TypeScript, Svelte 5, Vite,
  `vite-plugin-pwa`.** The app keeps choices as state and derives
  everything else from the engine. Svelte runes say this directly
  (`$derived(evaluate(choices))`) and `bind:value` suits a form-heavy
  editor. React would need hooks with hand-kept dependency lists, and
  `value`/`onChange` on every field. Svelte compiles to a small runtime:
  the app's JavaScript is 19 kB gzipped, against 72 kB with React in the
  first bootstrap. Costs accepted: a smaller component ecosystem than
  React's, and coding agents that know Svelte 5 runes less well than React
  (review generated components for Svelte 4 syntax). Plain Svelte, not
  SvelteKit: the backend serves static files, so SvelteKit's server
  rendering and routing are not needed. The bindings depend on no
  framework.
- **TypeScript 6 for the front end.** Type-checking `.svelte` files needs
  `svelte-check`, which supports TypeScript 5 and 6 only, not the native
  TypeScript 7. Move to 7 when `svelte-check` supports it.
- **Vite and `vite-plugin-pwa`.** Vite bundles the `.wasm` file as a
  hashed asset without a plugin. `vite-plugin-pwa` (Workbox, `generateSW`,
  auto-update) writes the manifest and a service worker that precaches the
  app, `.wasm` included, and never answers `/api` with the app shell.
  Node 24 (LTS) and pnpm are pinned in `mise.toml`.
- **The backend serves the built client.** Same origin as the API, so no
  CORS. Files under `/assets/` are named after their content and served as
  `immutable` for a year. Every other file (`index.html`, `sw.js`, the
  manifest, the icons) is served `no-cache`, so a new build reaches users.
  A missing asset is `404`, not `index.html`, so a stale or wrong path is
  never cached as the app shell. Unknown paths under `/api` are `404`. In
  development, Vite serves the client and forwards `/api` to the backend.
- **First binding: the report of a new character.** `newCharacterReport(engine)`
  returns the violations of a character with no choice made. The backend
  stores no choices yet, so this is the true report of every character it
  holds, and it proves the chain from engine to page without settling the
  choices format.

## Tasks

| Task | Effect |
|---|---|
| `mise run wasm:build` | compiles `palimpseste-wasm` and generates the bindings |
| `mise run frontend:dev` | Vite dev server on port 5173, `/api` forwarded to the backend |
| `mise run frontend:check` | type-checks the front end (`svelte-check`) |
| `mise run frontend:build` | builds the PWA into `app/frontend/dist` |
| `mise run serve` | builds the PWA and serves it with the API on port 3000 |

## Follow-ups

Each needs its own decision:

- **Choices document format.** The format for a character's choices, used
  by the API and by the bindings. Either a DTO crate shared by
  `app/backend` and `app/wasm`, mapped to the engine builders, or a `serde`
  feature on the engines. The engines hold large content enums (78 COF2
  voies, weapon and trait tables), so mirroring them in a DTO duplicates
  them. A middle way: derive `serde` behind a feature on the content enums
  only (stable game names), with the document structure in a DTO crate,
  and a version field in the stored document.
- **Character CRUD.** `GET`/`PUT`/`DELETE /api/characters/{id}`: a whole
  document per `PUT`, with a version in an `ETag` and `If-Match`
  (`412` on conflict).
- **Offline editing.** Characters kept in IndexedDB, edits queued and sent
  when online.
- **Authentication.** For a client on the same origin, an `HttpOnly`,
  `SameSite` session cookie (directly or at the end of an OIDC flow), not
  tokens in `localStorage`.
- **Persistence** of users and characters.
- **Size.** `wasm-opt` and compressed responses: the `.wasm` file is
  295 kB, 91 kB gzipped.

## Consequences

- The workspace gains `app/wasm`, and the repository gains `app/frontend`,
  a pnpm package outside the Cargo workspace.
- `mise run check` still covers only Rust; `frontend:check` is separate,
  because it needs a WebAssembly build and Node.
- The engine Rust code is the single source of rule behaviour on every
  platform. The server can link the same crates natively if it ever needs
  to read choices.
