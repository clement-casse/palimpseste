# Application

How the parts fit: the engines run in the browser, the backend stores,
and a choices document passes between them.

| Path | Package | Role |
|---|---|---|
| `crates/co-kernel`, `crates/co-contemporain`, `crates/co-fantasy` | | the rules, no I/O |
| `app/document` | `palimpseste-document` | the choices document and its conversion to the engines |
| `app/wasm` | `palimpseste-wasm` | the engines for the browser |
| `app/frontend` | pnpm package | the PWA: edits and evaluates characters |
| `app/backend` | `palimpseste-backend` | serves, signs in, checks and stores |

## Boundaries

**APP-LAYOUT: Rules under `crates/`, applications under `app/`.** The
Rust packages above form the Cargo workspace; `app/frontend` stays outside
it.

**APP-BOUNDARIES: Pure rules.** The kernel and the engines have no I/O, no
async code and no `serde`. `serde` is allowed in `app/backend`,
`app/document` and `app/wasm`; `tokio` and async code in `app/backend`
only. *Why:* the rules stay deterministic, and every platform links the
same crates.

**APP-LOCAL-RULES: The rules run in the browser.** The front end holds the
document being edited and calls the engines, compiled to WebAssembly, on
every edit: values, breakdowns, violations, step status and the options to
offer all come from them, recomputed in microseconds. A page loads a
character once and saves its whole document; nothing else crosses the
network. The engines' Rust code is the single source of rule behaviour.

**APP-STORE-ONLY: The backend never applies a rule.** It serves the
application, signs users in, and stores characters as sent, rule-breaking
ones included, since the Game Master can override every rule. It rejects
only what it cannot read (ADR BE-CHECKS).

**APP-REST: REST and JSON, on one origin.** The API is REST with JSON
bodies under `/api`, on the origin that serves the application (no CORS).
A document is replaced whole, its version in `ETag` and `If-Match`.
*Why:* HTTP gives the PWA URL caching and concurrency control for free.
GraphQL sends everything as a `POST` to one endpoint, which a service
worker cannot cache by URL, and has no input unions for per-engine
documents; Connect would restate the Rust types in `.proto` files and has
no official Rust server.

## The choices document

**APP-DOCUMENT: Versioned, per engine, naming content by identifier.**
`Document` is tagged by `engine` (`contemporain`, `fantasy`) and carries a
`version` (now 2); each engine variant holds the choices, the adjustments
and the waivers, mirroring the engine builders. Version 2 added the
progression: COF2 `levelUps` (the capacités acquired and forgotten, the
orphan exchange) and `abilityOptions`; COC2 `experience`, `xpSpendings`
(capacités bought, trait changes), `raisedCharacteristics` and
`acquiredSequels`. `specialities` (a capacité's identifier and the
speciality written for it, ADR ENG-SPECIALITIES) came later within
version 2, without a bump: an older build drops them (decided
2026-10-25, with no stored document to protect). COC2 `bonusDieTargets` (a voie and
the capacité its rank 5's dé bonus goes to, ADR ENG-COC-BONUS-DIE) came
the same way (decided 2026-11-12). A capacité is named by its identifier (`cof.rage.2`,
ADR ENG-VALUE-IDS), the key its name, description and effects will later
attach to. Only the current version is read: a format change bumps the
version and the backend migrates or deletes the stored documents (ADR
BE-STORE), so an old build meeting a new document refuses it instead of
dropping what it does not know. Content is named by the
engines' identifiers (ADR ENG-CATALOGUE), so the content enums are not
mirrored and the engines stay free of `serde`. A document converts one
way, to an engine `Character`: it is the source of truth, and the engines
need no getters on choices.

**APP-READABLE: Converting is checking.** A document is unreadable only
for an unsupported version, an unknown content identifier or adjustment
target, or a blank author, reason, rule, waiver subject or item name;
never for a broken rule. Waiver rules and subjects are checked for shape
only: a waiver matching nothing is reported as unused, never rejected.

**APP-DOCUMENT-CRATE: One reader for browser and server.**
`palimpseste-document` holds the document types and their conversion, and
is shared by `app/wasm` and `app/backend`. It also reads the equipment in
play (`Equipment`, ADR ENG-IN-PLAY), which only the browser keeps; the
backend never stores it. Its `tsify` derives sit behind
the `ts` feature, which only `palimpseste-wasm` enables, so the backend
does not link `wasm-bindgen`.

## WebAssembly

**APP-BINDINGS: Stateless functions over plain data.** `palimpseste-wasm`
exposes the rolls (ADR APP-ROLLS), `emptyDocument(engine)`, `catalog(engine)`,
`evaluate(document)` (values with breakdowns, violations by step, step
status) and `evaluateInPlay(document, equipment)`: the same steps and
violations, the values with the equipment in play, and the findings of
the rules that hold in play (`inPlay`, ADR ENG-IN-PLAY). The bindings
only translate: names, labels and the values listed come from the
engines (ADR ENG-CATALOGUE), and the bindings name only a formula's
constant term and an adjustment. No Rust object crosses the boundary, so
JavaScript never frees memory by hand. Types are described to TypeScript by `tsify` and passed as
`tsify::Ts`, so a wrong value throws instead of panicking. *Why not the
`into_wasm_abi`/`from_wasm_abi` attributes:* they are deprecated, and leak
memory when (de)serialization fails. Each violation carries
where the engine places it (ADR ENG-COF-PLACEMENT, ENG-COC-PLACEMENT): a
creation step, or a step of a level-up or spending, numbered in
`progression`; `sheet.steps` count the creation's own, and
`sheet.progression` gives each level-up or spending with the status of its
steps. The details carry what each level-up or spending offers within the
rules, the capacités held with where they were acquired (a capacité kept
through another, the voie du mage's rank 1, names it in `keptThrough` and
is placed where it was acquired), and the options they offer. A capacité
is always named by its identifier: the catalog's `abilities` label each
one, and a breakdown line gives the identifier of its source (`sourceId`),
so the capacités' names, descriptions and effects attach later to the same
key.

**APP-ROLLS: The bindings draw the dice, the engines resolve them.**
`rollOffer(document, play, roll)` returns what a roll offers (ADR
ENG-ROLLS) and `roll(document, play, request)` rolls it. A roll is named
by the identifier a sheet line carries (`roll` on a characteristic, an
attack value or a weapon's DM, `attackRoll` on a DM for the attack before
it), and its options by their position in the offer; `Play` is what the
device knows of the character in play, the equipment worn and the COC2
damage taken, which the engines never store. `roll` computes the dice of
the request, draws each face from `crypto.getRandomValues`, imported
through `wasm-bindgen`, and hands the faces to the engine: a byte beyond
the last multiple of the die's faces is drawn again, so every face is as
likely. The Rust function behind it takes the source of random bytes as a
parameter, which the native tests fix. The outcome is plain data made to
be shown and kept as it is: every die with the one kept, each line of the
total with its French label, the options left unpicked. Of their own, the
bindings name the lines that are no content (the allies helping, the
Game Master's modifier, a critical, the competence cap). *Why in the
bindings:* the front end only presents, and the engines stay pure (ADR
APP-BOUNDARIES, KER-ROLLS); *why no crate:* `wasm-bindgen` already reaches
the browser's generator, and no other randomness is needed.

**APP-WASM-BUILD: `cargo` and `wasm-bindgen-cli`.** `cargo` builds the
`wasm32-unknown-unknown` target, and `wasm-bindgen --target web` writes an
ES module and its `.d.ts` into `app/frontend/src/wasm/`, generated and
never committed (`mise run wasm:build`). The `wasm-bindgen-cli` version in
`mise.toml` equals the `wasm-bindgen` crate version in `Cargo.lock`.
*Why not `wasm-pack`:* one tool fewer.

## Open

- Offline editing: documents kept in IndexedDB, edits queued and sent once
  online, and a session that expires while offline.
- Size: `wasm-opt` and compressed responses (the `.wasm` file is about
  300 kB, 90 kB gzipped).
- Who may waive: any signed-in user signs waivers with their nickname, as
  text. The Game Master's authority, and waivers tied to a user, wait for
  parties.
