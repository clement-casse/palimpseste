# 0007. Character editing: choices document, local rules, a checking store

Date: 2026-10-16.
Status: accepted.

## Context

The PWA (ADR 0006) lists characters and shows the report of a character
with no choice. Users must now edit a character following the rules of
its engine, on a page of its own. ADR 0006 left the choices document
format and character CRUD as follow-ups; ADR 0005 kept the backend in
memory. Campaign initialization (D-25) stays out of this delivery.

## Decisions

- **The rules run locally.** The browser holds the choices being edited
  and calls the engines compiled to WebAssembly on every edit: values,
  breakdowns, violations, step status and the options to offer all come
  from the engines. The page loads a character once and saves its whole
  document; nothing else crosses the network.
- **The backend serves, checks and stores.** It serves the application,
  persists users and characters, and checks that what it receives is well
  formed. It never applies a rule: a character breaking any rule is stored
  as sent, since the Game Master can override every rule. Later features
  (friends, parties) will give it more to do; nothing here prepares them.
- **D1. A versioned choices document per engine.** `Document { version,
  engine }` where the engine variant holds the choices, the adjustments
  and the waivers, mirroring the engine builders. Content is referred to
  by the engines' stable string identifiers (`cof.epee-longue`,
  `coc.crime`), so the content enums are not mirrored and the engines stay
  free of `serde`. The document converts one way, to an engine
  `Character`: the document is the source of truth, so the engines need no
  getters on choices.
- **D2. New crate `app/document`, package `palimpseste-document`.** It
  holds the document types and their conversion to the engines' characters,
  and is shared by `app/wasm` and `app/backend`, so browser and server read
  a document with the same code. `tsify` derives sit behind a `ts` feature
  that only `palimpseste-wasm` enables, so the backend does not link
  `wasm-bindgen`. `palimpseste-document` may use `serde`.
- **D3. Engines expose their content.** Each content enum the document
  names gets `ALL`, `id()`, `from_id()` and `name()` (the French display
  name). `Profil`, `Domaine` and `Series` get identifiers. The content
  tables the creation rules read become public read-only accessors, so the
  page offers the options the rules allow without restating them. Options
  depending on the current choices (the voies a peuple slot accepts) are
  `Character` methods.
- **D4. Creation steps are engine content.** Each engine has a `Step` enum
  in the book order (COF.CRE.STEPS, COC.CRE.STEPS) and places each
  violation in a step. Steps guide: every step can be opened at any time,
  and every choice can be made in any order.
- **D5. Violation messages live in the front end**, keyed by rule
  identifier and filled with the violation's parameters. A rule without a
  message shows its identifier.
- **D6. What the backend checks.** Malformed JSON or an unknown request
  shape is `400`; an unknown user or character `404`; an unknown engine, an
  unsupported document version, a document of another engine than the
  character's, a document the conversion cannot read (unknown content
  identifier or adjustment target), an empty name, author, reason, rule,
  waiver subject or item name is `422`. Waiver rules and subjects are
  checked for shape only: the engines name subjects freely (content,
  budgets, values), and a waiver matching nothing is reported as unused,
  never rejected. A `PUT` of a document needs `If-Match` (`428`
  without it, `412` on a stale version). Bounds, rule breaches and missing
  choices are accepted: the backend never computes violations.
- **D7. Persistence in SQLite through `rusqlite`** (`bundled`), called
  from `tokio::task::spawn_blocking`. Two tables, users and characters;
  the document is stored as JSON text next to a version counter used as
  the `ETag`. `rusqlite` over `sqlx`: a handful of queries, no database at
  build time.
- **Breaking a rule is never easier than respecting it.** The page offers
  the options within the rules first; anything else goes through an
  explicit out-of-rules choice. Nothing is disabled or deleted: changing an
  earlier choice keeps the later ones, which turn into violations. A waiver
  or an adjustment needs an author (the current user) and a reason; an
  `absolute` waiver asks for confirmation.

## API

| Method | Path | Result |
|---|---|---|
| `POST` | `/api/users/{user_id}/characters` `{name, engine}` | `201`, the character with the empty document of its engine |
| `GET` | `/api/characters/{id}` | the character and its document, version in `ETag` |
| `PUT` | `/api/characters/{id}/document` | `204` with the new `ETag`; see D6 for rejections |
| `PATCH` | `/api/characters/{id}` `{name}` | the renamed character |
| `DELETE` | `/api/characters/{id}` | `204` |

## Follow-ups

- Offline editing and sync (ADR 0006).
- Authentication, and who may enter a waiver: with no authentication any
  user signs waivers under their own name; the Game Master's authority is
  enforced once parties exist.
- The demi-elfe upbringing as a choice, and capacité names per voie and
  rank, each with a decision entry in the co-rules skill.

## Consequences

- The workspace gains `app/document`. The backend now depends on the
  engines, through `palimpseste-document` only, and on `rusqlite`.
- The engines' public API grows by their content catalogue and steps.
- `newCharacterReport` is replaced by `emptyDocument` and `evaluate`.
