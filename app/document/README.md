# palimpseste-document

The **choices document** of a character: what the player chose, with the
adjustments and waivers granted on it, as a versioned `serde` document per
engine (ADR APP-DOCUMENT). The browser (through
[`palimpseste-wasm`](../wasm)) and the server
([`palimpseste-backend`](../backend)) share it, so both read a document
the same way (ADR APP-DOCUMENT-CRATE).

It depends on `co-kernel`, `co-contemporain` and `co-fantasy`. The engines
stay free of `serde`: this crate never mirrors their content enums. It
names content by the engines' string identifiers instead
(`cof.weapon.epee-longue`, `coc.crime`).

## Modules

| File | Holds |
|---|---|
| `lib.rs` | `Document` (tagged by `engine`), `Engine`, `VERSION`, `Error`, and the shared entries: `Characteristic`, `PerCharacteristic`, `VoieEntry`, `ItemEntry`, `Target`, `AdjustmentEntry`, `Subject`, `WaiverEntry` |
| `contemporain.rs` | `ContemporainDocument`, `TraitEntry`, and `to_character()` → `co_contemporain::Character` |
| `fantasy.rs` | `FantasyDocument`, and `to_character()` → `co_fantasy::Character` |

## The shape

```json
{
  "engine": "fantasy",
  "version": 1,
  "profil": "cof.profil.barbare",
  "seriesValues": { "FOR": 3, "CON": 2 },
  "voies": [{ "id": "cof.rage", "ranks": [1] }],
  "items": [
    { "kind": "weapon", "id": "cof.weapon.epee-longue", "quantity": 1 },
    { "kind": "other", "name": "adventurer's bag", "quantity": 1 }
  ],
  "adjustments": [],
  "waivers": []
}
```

- Fields are `camelCase`. Every choice is optional or defaults to empty,
  so `Document::empty(engine)` is a valid document with no choice made.
- Characteristics are keyed by their abbreviation (`AGI`, `FOR`, ...).
- The full examples are the book's worked characters in
  `tests/fixtures/` (`lhagva.json`, `ionas.json`, `amelia-steele.json`).

## Converting is checking

A document converts one way, into an engine `Character`. The document is
the source of truth, and the engines need no getters on choices.
`Document::check()` is that conversion with the result thrown away. A
document is **unreadable** (`Error`) only when:

- its version is not `VERSION` (`UnsupportedVersion`);
- an identifier names no content of the engine, or names no adjustment
  target (`UnknownContent`);
- an author, reason, rule, waiver subject or item name is blank (`Empty`).

A broken rule never makes a document unreadable (ADR APP-READABLE). Waiver
rules and subjects are checked for shape only: a waiver that matches
nothing is reported as unused, never rejected.

## Recipes

**Adding a choice to an engine:** add the field to the engine's document,
with `#[serde(default)]` so that older documents still read. Convert it in
`to_character()` through the engine's `with_*` builder. Resolve
identifiers with the engine's `from_id`, and map a miss to
`Error::UnknownContent`. Extend the round-trip and unreadable-document
tests. A change that existing documents cannot read needs a new
`VERSION`.

**New content in an engine** needs no change here: identifiers resolve
through `from_id`.

## Features

- `ts`: derives `tsify::Tsify`, so `wasm-bindgen` can describe the types
  to TypeScript. Only `palimpseste-wasm` enables it, so the backend never
  links `wasm-bindgen`.

## Testing

`tests/documents.rs` converts the fixtures and checks the resulting
characters against the engines. It also tests empty documents and every
kind of unreadable document. The crate doc test parses a minimal document.

```sh
mise run check            # fmt, lint, test, doc-test
mise run frontend:check   # the TypeScript types generated from this crate
mise run frontend:test
```

Run the front-end tasks too after any change here: the PWA's types are
generated from these structs.
