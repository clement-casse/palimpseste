# AGENTS.md

Guidance for coding agents working in this repository. Read it fully before making any change.

## Project

A companion application for the French tabletop RPG *Chroniques Oubliées* (COF2: Fantasy, COC2: Contemporain).
The long-term goal is a character editor, later a campaign tracker.
Each ruleset and each variant will be a **WorldEngine**: rules written as Rust code, composed from mechanics, with game content kept as data.

**Current phase: the kernel, and the COC2 and COF2 character creation engines.** Everything below is scoped to them.

## Current scope: `co-kernel`

The kernel holds the concepts that mean exactly the same thing in every engine. Nothing else.

In scope:

- **Characteristics**: the seven characteristics `AGI`, `CON`, `FOR`, `PER`, `CHA`, `INT`, `VOL`, their values and modifiers.
- **Voie skeleton**: a voie, its ranks, the abilities attached to ranks. Structure only, no rule about costs or budgets.
- **Derived values with breakdown**: a computed value together with the list of contributions that produced it (source, amount).
- **Manual adjustments**: attributed contributions added by a user to a derived value or a budget.
- **Violations**: computed findings about a character, with a stable rule identifier, the emitting mechanic, a subject, parameters, and a category (`Incomplete`, `Deviation` with its severity, `Uncomputable`, `Notice`).
- **Waivers**: persisted, attributed acceptance of a `Deviation` (rule identifier, subject, author, reason).
- **Violation report**: aggregation of violations and matching against waivers.

Out of scope for the kernel: anything engine-specific (formulas, budgets, tables, game content). It lives in the engine crates.

## Current scope: `co-contemporain`

The COC2 engine, core rules only (no variant, no universe), **character creation** only (see `docs/adr/0002-co-contemporain-creation-engine.md`).

In scope:

- **Creation choices**: age (and the age bracket derived from it), domaine, ranks acquired per voie, traits distinctifs with their points, characteristic points allocated.
- **Content read by the creation rules**: the 18 domaines with their voie slots, the 18 voies, the traits catalogue and sequels, the age-bracket limits, as enums and `match` tables.
- **Creation budgets and limits**: characteristic points, capacity points, maximum voies and rank, trait balance, characteristic bounds, domaine voies, social class.
- **Secondary characteristics**: Init, DEF, ATC and ATD per weapon category, contact DM bonus, CG, health scale length.

Out of scope, do not implement or prepare for:

- variants (COC2 3d6, ...) and universe rules (échelle d'évolution);
- combat resolution and weapon data (dice, bonus critique, critical range, arme de prédilection), protections, inventory and starting equipment;
- progression (XP, rank costs after creation, rank 5 characteristic increase);
- data files and data formats, persistence, serialization, `serde`;
- user interface, networking, users, campaigns, play state (health scale damage, resources), dice rolling.

## Current scope: `co-fantasy`

The COF2 engine, core rules only (no variant), **character creation at level 1** only (see `docs/adr/0003-co-fantasy-creation-engine.md`).

In scope:

- **Creation choices**: profil (and the family derived from it), peuple, value series and the value assigned to each characteristic, peuple bonus and malus, ranks acquired per voie, armour and shield.
- **Content read by the creation rules**: the 4 families, the 14 profils with their voies, key and magic characteristics and armour limits, the 8 peuples with their modifiers and voie de peuple, the 3 value series, the 78 voies reachable at level 1, the armour and shield table, as enums and `match` tables.
- **Creation limits**: series assignment and its advice, peuple modifiers, characteristic bounds, two profil voies and the voie de peuple, voie du mage, mage extra capacité, level required per rank, armour limits.
- **Derived values**: PV, DR (count and die), PC, PM, Init, DEF with armour and shield, contact, ranged and magic attack values, with the documented effects of rank 1-2 capacités (D-22) and the documented spells (D-23).

Out of scope, do not implement or prepare for:

- variants, prestige voies, hybrid profils;
- progression (levels above 1, capacity points, PV per level, orphan point, changement d'orientation);
- weapon data and DM, critical ranges, competence bonuses, languages and talent secondaire, capacités calling other capacités, armour malus to tests, starting equipment, final touch;
- data files and data formats, persistence, serialization, `serde`;
- user interface, networking, users, campaigns, play state (current PV, PC, PM, DR), dice rolling.

Other engines are added in later deliveries. Do not add extension points, traits, generics or modules "for later". If a task seems to require something out of scope, stop and ask.

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
- Builder methods (`with_*`, `without_*`) take their newtype parameters as `impl Into<Newtype>`, and those newtypes implement `From<primitive>` (infallible: newtypes carry no game bounds), so callers write `.with_age(32)`. Give each integer newtype exactly one `From<integer>` impl: a second one breaks inference of integer literals (`with_age(32)` falls back to `i32` and fails). Never let two parameters of one method convert from the same primitive, since swapping them would compile: this is why `Author` stays explicit in `Waiver::new` and `Adjustment::new`, next to a string `reason`. Constructors (`new`) keep their explicit parameters.
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
crates/co-kernel/     # co-kernel: concepts shared by every engine
crates/co-contemporain/ # COC2 engine: character creation
crates/co-fantasy/     # COF2 engine: character creation at level 1
```

Do not create other crates. A new crate or a new top-level directory is a structural decision: propose it, do not make it.

## Vocabulary

Code identifiers are in English. Game terms map as follows. Game content (names of domaines, voies, traits, sequels, social classes, profils, peuples, armours) keeps its French name without accents, as enum variants (`Voie::ArmesDeContact`, `Trait::ForceDeLaNature`).

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
| sévérité (indicative, règle, jamais) | `Severity` (`Recommended`, `Standard`, `Absolute`) |
| domaine | `Domaine` (kept) |
| tranche d'âge | `AgeBracket` |
| trait distinctif | `Trait` |
| séquelle | `Sequel` |
| classe sociale | `SocialClass` |
| échelle de santé | `HealthScale` |
| capacité de guérison (CG) | `HealingCapacity` |
| Init, DEF | `Initiative`, `Defence` |
| ATC, ATD | `Attack(AttackMode)` |
| profil | `Profil` (kept) |
| famille | `Family` |
| peuple | `Peuple` (kept) |
| série de valeurs | `Series` |
| points de vigueur (PV) | `VigourPoints` |
| dés de récupération (DR) | `RecoveryDice`, `RecoveryDie` |
| points de chance (PC) | `LuckPoints` |
| points de mana (PM) | `ManaPoints` |
| armure, bouclier | `Armour`, `Shield` |
| attaque au contact, à distance, magique | `Attack(AttackMode::Contact)`, `Ranged`, `Magic` |

## Working rules for agents

- Make the smallest change that fulfils the task. Do not refactor unrelated code.
- Write tests first or alongside the change; every violation category needs at least one test.
- Never copy rulebook text into the repository. Test fixtures use invented or minimal data; the book's worked characters (co-rules `examples/`) may be fixtures, as choices and numbers only.
- If a decision affects the structure (new concept in the public API, new crate, new dependency, change to the principles above), stop and ask, and record the outcome in `docs/adr/`.
- When unsure whether something belongs to the kernel, it does not.