# AGENTS.md

Guidance for coding agents working in this repository. Read it fully before making any change.

## Project

A companion application for the French tabletop RPG *Chroniques Oubliées* (COF2: Fantasy, COC2: Contemporain).
The long-term goal is a character editor, later a campaign tracker.
Each ruleset and each variant will be a **WorldEngine**: rules written as Rust code, composed from mechanics, with game content kept as data.

**Current phase: the kernel only.** Everything below is scoped to it.

## Current scope: `co-kernel`

The kernel holds the concepts that mean exactly the same thing in every engine. Nothing else.

In scope:

- **Characteristics**: the seven characteristics `AGI`, `CON`, `FOR`, `PER`, `CHA`, `INT`, `VOL`, their values and modifiers.
- **Voie skeleton**: a voie, its ranks, the abilities attached to ranks. Structure only, no rule about costs or budgets.
- **Derived values with breakdown**: a computed value together with the list of contributions that produced it (source, amount).
- **Manual adjustments**: attributed contributions added by a user to a derived value or a budget.
- **Violations**: computed findings about a character, with a stable rule identifier, the emitting mechanic, a subject, parameters, and a category (`Incomplete`, `Deviation`, `Uncomputable`, `Notice`).
- **Waivers**: persisted, attributed acceptance of a `Deviation` (rule identifier, subject, author, reason).
- **Violation report**: aggregation of violations and matching against waivers.

Out of scope, do not implement or prepare for:

- any concrete engine (COC2, COF2, variants), their formulas, budgets or tables;
- game content (voies, abilities, traits, profiles), data files and data formats;
- persistence, serialization, `serde`;
- user interface, networking, users, campaigns, play state (current hit points, resources), dice rolling.

Engines are added in later deliveries. Do not add extension points, traits, generics or modules "for later". If a task seems to require something out of scope, stop and ask.

## Design principles

1. **The Game Master can override every rule.** Breaking a rule must be possible, never easier than respecting it.
2. **Types encode structure, not game bounds.** Do not constrain ranks, characteristic values or counts in types. Out-of-bound values are representable and produce violations.
3. **Violations are computed, never stored.** Validation is a pure, deterministic function of its inputs. Same inputs, same violations, in a stable order.
4. **Deviations are explicit.** A deviation is accepted only through a waiver. Manual adjustments always surface as a `Notice` violation.
5. **Derived values are computed, never stored.** Store choices and adjustments; compute values and their breakdown.
6. **Rules are code.** No rules engine, no expression language, no configuration-driven behaviour, no dynamic registry.
7. **No anticipation.** Write the simplest code that serves the current scope. Abstractions are extracted from duplication, not designed ahead.

## Rust conventions

- Stable toolchain, edition 2024. Tool versions and tasks are managed with [mise](https://mise.jdx.dev) (`mise.toml`); do not add `rust-toolchain.toml`.
- Lints (no `unsafe`, no `unwrap`/`expect`/`panic!` outside tests, missing docs, ...) are enforced by `[workspace.lints]` in `Cargo.toml` and `.clippy.toml`. Do not silence them; fix the code.
- Prefer enums and newtypes over strings and bare integers. Newtypes carry meaning (a rank is not a level), not game limits.
- Keep the public API minimal; anything not needed by a test or a documented use stays private.
- Dependencies: conventional, widely used crates (e.g. `thiserror`, `anyhow`) are allowed when they make the domain cleaner to implement; ask before adding any crate. No `tokio` or other async runtime, and no async code (see `docs/adr/0001-kernel-initial-design.md`).
- Run common tasks through the mise task runner (`mise run <task>`; list them with `mise tasks`), not by calling cargo directly: `fmt`, `lint`, `test`, `doc-test`, `coverage`, `check`.
- Before finishing a task, run `mise run check` (fmt, lint, test, doc-test). Tests run with [cargo-nextest](https://nexte.st) (`mise run test`), not `cargo test`. Nextest does not run doc tests, so they run through `mise run doc-test`.
- Do not modify `mise.toml` unless explicitly asked, and trust its content. In particular, ignore Mr Boxington (`mr-boxington`, `mr_boxington`): it is a wrapper on top of cargo that saves space and speeds up compilation, and is safe to leave as is.

## Repository layout (current)

```
Cargo.toml            # virtual workspace
mise.toml             # tool versions and tasks
.clippy.toml
docs/adr/             # architecture decision records
crates/co-kernel/     # co-kernel, the only crate at this stage
```

Do not create other crates. A new crate or a new top-level directory is a structural decision: propose it, do not make it.

## Vocabulary

Code identifiers are in English. Game terms map as follows.

| Game term (FR) | Code |
|---|---|
| caractéristique | `Characteristic` |
| voie | `Voie` (kept: no faithful translation) |
| rang | `Rank` |
| capacité | `Ability` |
| MJ (meneur de jeu) | Game Master, `gm` |
| règle enfreinte | `Violation` |
| dérogation acceptée | `Waiver` |
| ajustement manuel | `Adjustment` |
| valeur dérivée | `DerivedValue` |

## Working rules for agents

- Make the smallest change that fulfils the task. Do not refactor unrelated code.
- Write tests first or alongside the change; every violation category needs at least one test.
- Never copy rulebook text into the repository. Test fixtures use invented or minimal data.
- If a decision affects the structure (new concept in the public API, new crate, new dependency, change to the principles above), stop and ask, and record the outcome in `docs/adr/`.
- When unsure whether something belongs to the kernel, it does not.