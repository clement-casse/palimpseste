# palimpseste-wasm

The engines compiled to WebAssembly for the PWA. The rules run in the
browser, on every edit (ADR APP-LOCAL-RULES). This crate only maps what the
engines compute, and the French names they give it, to plain data. It
applies no rule; of its own, it names only the constant term of a formula
("Base") and an adjustment ("Ajustement de ...").

It depends on `co-kernel`, `co-contemporain`, `co-fantasy` and
`palimpseste-document` (with the `ts` feature), plus `serde`, `tsify` and
`wasm-bindgen`. It builds as a `cdylib` for the browser and as an `rlib` so
the native tests can run.

## The bindings

Three stateless functions over plain data (ADR APP-BINDINGS):

| JavaScript | Rust | Returns |
|---|---|---|
| `emptyDocument(engine)` | `empty_document_js` | a `Document` with no choice made |
| `catalog(engine)` | `catalog_js` → `catalog()` | the content the engine offers: profils, peuples, voies, every capacité by identifier (`abilities`), weapons, the steps of a level-up or spending, ... with identifiers and French names |
| `evaluate(document)` | `evaluate_js` → `evaluate()` | a `Sheet`: the status of each creation step and of each level-up or spending (`progression`), characteristics, values with their breakdown, violations (waived or not) placed in their step, unused waivers, and engine-specific details, the offers of each level-up or spending among them |

No Rust object crosses the boundary, so JavaScript never frees memory by
hand. Values cross as `tsify::Ts<T>`, which turns a value the front end got
wrong into a thrown `JsError` instead of a panic. Do not use the
deprecated `into_wasm_abi` / `from_wasm_abi` attributes, which leak memory
when (de)serialization fails.

## Modules

| File | Holds |
|---|---|
| `lib.rs` | the `#[wasm_bindgen]` exports and the native tests |
| `catalog.rs` | `Catalog` (`FantasyCatalog`, `ContemporainCatalog`) and its entries, built from the engines' `ALL`, `id()`, `name()` and public content tables |
| `sheet.rs` | `Sheet` and its lines (`StepStatus`, `CharacteristicLine`, `ValueLine`, `ContributionLine`, `ReportedViolation`, ...), and `evaluate`: the values of each engine's `Value::FIXED`, named by `Value::name()`, `label` and `term_label` |

Identifiers in the catalogue and the sheet are those of the choices
document, so the front end can write them back as they are.

## Building

```sh
mise run wasm:build
```

This runs `cargo build -p palimpseste-wasm --target wasm32-unknown-unknown
--release`, then `wasm-bindgen --target web`. The result (an ES module, its
`.d.ts` and the `.wasm` file) lands in `app/frontend/src/wasm/`. These
files are **generated**: never edit or commit them. Every `frontend:*`
task rebuilds them first.

The `wasm-bindgen-cli` version in `mise.toml` must equal the `wasm-bindgen`
crate version locked in `Cargo.lock`. Bump them together, and update the
`Dockerfile`, which pins the same version.

## Recipes

**Exposing a new engine value:** add it to the engine's `Value::FIXED` and
name it in `Value::name()`; the sheet lists it with no change here.

**Exposing a new engine detail:** add it to the engine's `Details`, mapped
from the engine's public API. Keep the mapping mechanical: if it needs a
rule or a name, it belongs in the engine. Then rebuild and use the
generated type in the front end.

**Exposing new content:** add it to the engine's catalogue struct in
`catalog.rs`, built from the engine's `ALL` and accessors. Never hard-code
a list.

## Testing

The native unit tests in `lib.rs` evaluate documents for both engines.
They cover values, breakdown sources, violations, waivers and adjustments,
without a browser. The front end's Vitest suite then loads the real
`.wasm` file (`app/frontend/src/test/setup.ts`).

```sh
mise run check            # Rust: fmt, lint, test, doc-test
mise run frontend:check   # rebuilds the bindings and type-checks the PWA against them
mise run frontend:test
```
