# AGENTS.md

Guidance for coding agents working in this repository. Read it fully before making any change.

## Project

A companion application for the French tabletop RPG *Chroniques Oubliées* (COF2: Fantasy, COC2: Contemporain).
The long-term goal is a character editor, later a campaign tracker.
Each ruleset and each variant will be a **WorldEngine**: rules written as Rust code, composed from mechanics, with game content kept as data.

**Current phase: the kernel, the COC2 and COF2 character creation engines, COF2 levels and COC2 experience.** Everything below is scoped to them.

## Current scope: `co-kernel`

The kernel holds the concepts that mean exactly the same thing in every engine. Nothing else.

In scope:

- **Characteristics**: the seven characteristics `AGI`, `CON`, `FOR`, `PER`, `CHA`, `INT`, `VOL`, their abbreviations, values and modifiers.
- **Voies**: ranks, voie and ability identifiers, and the checks on the ranks acquired in a voie (CO.VOIE.STRUCTURE, CO.VOIE.ORDER). Structure only, no rule about costs or budgets.
- **Items**: the name of a free text item and a quantity carried (ADR KER-ITEMS).
- **Derived values with breakdown**: a computed value together with the list of contributions that produced it (source, amount).
- **Manual adjustments**: attributed contributions added by a user to a derived value or a budget.
- **Violations**: computed findings about a character, with a stable rule identifier, the emitting mechanic, a subject, parameters, and a category (`Incomplete`, `Deviation` with its severity, `Uncomputable`, `Notice`).
- **Waivers**: persisted, attributed acceptance of a `Deviation` (rule identifier, subject, author, reason).
- **Violation report**: aggregation of violations and matching against waivers.
- **Dice**: the dice to roll for a value, as a list of dice (`Dice`, `Die`), described but never rolled (ADR KER-DICE).

Out of scope for the kernel: anything engine-specific (formulas, budgets, tables, game content). It lives in the engine crates.

## Current scope: `co-contemporain`

The COC2 engine, core rules only (no variant, no universe): **character creation, and the experience spent after it** (see `docs/adr/engines.md`).

In scope:

- **Creation choices**: age (and the age bracket derived from it), domaine, ranks acquired per voie, traits distinctifs with their points, characteristic points allocated, items carried (weapons, free text items), protections worn.
- **Content read by the creation rules**: the 18 domaines with their voie slots, the 18 voies, the traits catalogue and sequels, the age-bracket limits, the weapon and protection tables, as enums and `match` tables.
- **Creation budgets and limits**: characteristic points, capacity points, maximum voies and rank, trait balance, characteristic bounds, domaine voies, social class, starting equipment of the weapon voies.
- **Experience** (ADR ENG-COC-EXPERIENCE): the XP earned (`Experience`) and each `XpSpending` in order, the ranks and trait changes it buys and the XP left (`Value::Experience`); rank n costs n XP, one rank of a voie per spending, six voies, the social class raised and Toxicomane cured with XP (COC.PRG.COST, ONE-RANK, LIMITS, XP-USE); the creation budgets and limits read the creation choices only.
- **Characteristic growth** (ADR ENG-COC-RANK-FIVE): the characteristic each rank 5 raises, chosen when the voie offers two or three (COC.PRG.CARAC, D-20).
- **Sequels acquired in play** (ADR ENG-COC-SEQUELS): recorded apart from the trait Séquelle, Diminué lowering the current CON or INT and shortening the health scale.
- **Secondary characteristics**: Init, DEF, ATC and ATD per weapon category, contact DM bonus, CG, health scale length, protection RD and malus, weapon damage dice and DM, bonus critique and critical range, with the rank-4 capacités of the four combat voies.
- **Health-scale states** (ADR ENG-HEALTH-STATES): the steps of a full échelle de santé and its states (step, name, malus), content the character sheet displays. The damage taken is not the engine's.
- **Equipment in play** (ADR ENG-IN-PLAY): `Equipment` (protections, items), `Character::in_play` and the values of `InPlay`, computed from the equipment it is given; nothing is stored.
- **Catalogue and steps** (ADR ENG-CATALOGUE, ADR ENG-STEPS): each content enum named by the choices document has `ALL`, `id()`, `from_id()` and `name()` (French display name); `Voie::ability_name(rank)` names each capacité (D-45); the content tables the creation rules read are public read-only accessors; `Value::FIXED` lists the values in the book order and `Value::name()` names them; `label` and `term_label` name every identifier the engine emits; `Step` gives the creation steps in the book order and places each violation in one.

Out of scope, do not implement or prepare for:

- variants (COC2 3d6, ...) and universe rules (échelle d'évolution);
- combat resolution, arme de prédilection, weapon range, magazine and reach, items without a rule (they stay free text);
- money, prices, purchases and the era: spending waits for campaign initialization (D-25);
- psychic sequels and traumatismes (not in the co-rules skill), aging after creation;
- data files and data formats, persistence, serialization, `serde`;
- user interface, networking, users, campaigns, play state (health scale damage, resources), dice rolling.

## Current scope: `co-fantasy`

The COF2 engine, core rules only (no variant): **character creation at level 1, and the capacités acquired at each level gained after it** (see `docs/adr/engines.md`).

In scope:

- **Creation choices**: profil (and the family derived from it), peuple, value series and the value assigned to each characteristic, peuple bonus and malus, ranks acquired per voie, armour and shield, items carried (weapons, free text items), the purse rolled.
- **Content read by the creation rules**: the 4 families, the 14 profils with their voies, key and magic characteristics, armour limits, weapon mastery and starting equipment, the 8 peuples with their modifiers and voie de peuple, the 3 value series, the 78 voies reachable at level 1, the armour and shield table, the weapon tables, the currency, as enums and `match` tables.
- **Creation limits**: series assignment and its advice, peuple modifiers, characteristic bounds, two profil voies and the voie de peuple, voie du mage, mage extra capacité, level required per rank, armour limits, starting equipment and purse, arc long strength, halfelin weapon size.
- **Levels** (ADR ENG-COF-PROGRESSION): `LevelUp`, the capacités acquired and forgotten at one level-up and the exchange of its orphan point (`OrphanExchange`), stored in order on `Character`; the `Level` they reach and its dé évolutif d4° (COF.DER.D4); the capacity points of each level and their cost (COF.PRG.POINTS, COF.PRG.COST), the level required per rank when acquired, lower ranks first, the voies a level-up opens and the voie count (COF.PRG.LEVEL-REQ, ORDER, NEW-VOIES, HYBRID, MAX-VOIES), the changement d'orientation (COF.PRG.REORIENT) and the orphan point (COF.PRG.ORPHAN).
- **Derived values**: PV (with the PV per level), DR (count and die), PC, PM, Init, DEF with armour and shield, contact, ranged and magic attack values (with the level, at most 10), weapon damage dice and DM, critical range, RD, weapon mastery, money held, with the documented effects of the capacités at every rank, read from per-value tables (ADR ENG-COF-CAPACITES), and every spell (D-23).
- **Capacité options** (ADR ENG-COF-OPTIONS): `AbilityOption`, chosen per capacité with `with_ability_option`: the characteristic raised or used, the PC for DR exchange, a heavier armour, the thrown weapons variant; characteristics raised by the capacités held.
- **Equipment in play** (ADR ENG-IN-PLAY): `Equipment` (armour, shield, items), `Character::in_play`, the values of `InPlay` and its report of the rules that hold in play (armour limit, arc long, Petite taille), computed from the equipment it is given; nothing is stored.
- **Catalogue and steps** (ADR ENG-CATALOGUE, ADR ENG-STEPS): each content enum named by the choices document has `ALL`, `id()`, `from_id()` and `name()` (French display name); `Voie::ability_name(rank)` names each capacité (D-45); the content tables the creation rules read are public read-only accessors; `Value::FIXED` lists the values in the book order and `Value::name()` names them; `label` and `term_label` name every identifier the engine emits; `Step` gives the creation steps in the book order and places each violation in one.

Out of scope, do not implement or prepare for:

- variants, prestige voies, hybrid profils;
- hybrid PV per level;
- capacité effects that depend on circumstances or on one weapon's attack, and those whose content the co-rules skill lacks (ADR ENG-COF-CAPACITES);
- competence bonuses, languages and talent secondaire, capacités calling other capacités, armour malus to tests, final touch, unarmed attacks, weapon range, items without a rule (they stay free text);
- prices, purchases and spending: they wait for campaign initialization (D-25);
- data files and data formats, persistence, serialization, `serde`;
- user interface, networking, users, campaigns, play state (current PV, PC, PM, DR), dice rolling.

Other engines are added in later deliveries. Do not add extension points, traits, generics or modules "for later". If a task seems to require something out of scope, stop and ask.

## Current scope: `app/backend`

An HTTP server that serves the application, signs users in, checks and stores (see `docs/adr/backend.md`). It never applies a rule.

In scope:

- **Authentication** (ADR BE-OIDC, ADR BE-SESSIONS): sign-in through any number of OIDC providers configured by the environment (Authorization Code with PKCE, `state` and `nonce`, through `openidconnect`), one flow for signing in and up: the first sign-in of an identity (issuer, subject) starts a registration, and its user is created once they choose their nickname and profile, or never if they cancel (ADR BE-FIRST-SIGN-IN), server-side sessions in an `HttpOnly`, `SameSite=Lax` cookie, sign-out.
- **Users** (ADR BE-USERS, ADR BE-IDENTITIES): a private id never sent out, a public nickname (unique regardless of ASCII case) and profile, changed by their owner; one or more identities, at most one per provider, each with a private label, linked by a signed-in user through the sign-in flow. Only the signed-in user is reachable (`/api/me`), and their friends; every route but `/api/auth/*` needs a session.
- **Friends** (ADR BE-INVITATIONS, ADR BE-FRIENDS): single-use invitation links valid 7 days, their token hashed and never in a URL; a mutual friendship, named by its id, ended by either friend for both; friends see each other's nickname and profile and read each other's characters, never changing them.
- **Characters**: a list per user, reachable by their owner only, each with a name, its engine (`contemporain`, `fantasy`) and its choices document; read, renamed, deleted, and the document replaced with `If-Match`.
- **Checks**: that requests are well formed and a document is readable by `palimpseste-document` (ADR BE-CHECKS). Never violations: a rule-breaking character is stored as sent.
- **Persistence**: SQLite through `rusqlite`, called from `spawn_blocking`.
- **Health check** (ADR BE-HEALTH): `/healthz`, without a session, `200` while the database answers, else `503`.
- **Front end**: the built PWA served by the same router, falling back to `index.html`; hashed files under `/assets/` cached as immutable, every other file revalidated (ADR BE-FRONTEND).

- **Structure** (ADR BE-LAYOUT): one bounded context in a lib + bin crate; the model, the use cases (`PalimpsesteService`) and `Error` in `lib.rs` and concept-named files, adapters in `repo/`, `provider/` and `api/`, wiring and clap configuration in `main.rs` only. Follow `.claude/skills/backend-structure/SKILL.md`.

Out of scope, do not implement or prepare for: unlinking or merging identities, searching users, choosing which characters friends see, blocking, roles, parties, campaigns, computing violations.

## Current scope: `app/document`

The choices document (`palimpseste-document`, ADR APP-DOCUMENT): a versioned `serde` document per engine naming content by the engines' string identifiers, and its conversion to an engine `Character`. `tsify` derives sit behind the `ts` feature, enabled by `app/wasm` only.

## Current scope: `app/wasm` and `app/frontend`

The engines compiled to WebAssembly, and the PWA that calls them (see `docs/adr/application.md` and `docs/adr/frontend.md`). The rules run locally.

In scope:

- **Bindings** (`palimpseste-wasm`): stateless functions over plain data, types described to TypeScript by `tsify` and passed as `tsify::Ts`. No Rust object crosses the boundary: `emptyDocument(engine)`, `catalog(engine)`, `evaluate(document)`.
- **Front end** (`app/frontend`, TypeScript, Svelte 5 with runes, Vite, `vite-plugin-pwa`, flowbite-svelte on Tailwind CSS 4): sign in with a provider and sign out, create their account at their first sign-in (`/register`), edit their nickname and profile and link identities on their profile page (`/profile`), list and create the user's characters, edit a character on its own page (`/characters/{id}`) step by step, with waivers and adjustments, saving the whole document, level it up (COF2) or spend its XP (COC2) once complete, each level-up or spending prepared as a draft on the device and saved when validated (`/characters/{id}/progression/{n}`, ADR FE-PROGRESSION), record its XP earned and sequels acquired in play, follow it on its sheet (`/characters/{id}/sheet`, ADR FE-SHEET) with its resources counted and its equipment in play managed on the device and the screen kept on, make friends through invitation links, list them and remove them, read their characters' sheets (`/friends`, `/invitation`, ADR FE-FRIENDS), choose a theme per device (Système, Clair, Sombre, OLED; ADR FE-THEMES). Plain Svelte, not SvelteKit. Use runes (`$state`, `$derived`, `$props`, `$effect`) and event attributes (`onclick`), never Svelte 4 syntax (`export let`, `$:`, stores for local state, `on:click`). TypeScript stays on 6.x until `svelte-check` supports 7.
- **Front-end conventions** (see `docs/adr/frontend.md`): shared state in `.svelte.ts` modules (session, router, PWA status, theme, wake lock); only `router.svelte.ts` touches the URL; one-shot reads in `{#await}` blocks, errors shown through `ErrorMessage.svelte`; `$effect` only to sync with the outside world, never to fetch; a child changes its parent's state through `$bindable` props or callbacks; Flowbite components first, Tailwind utilities only for layout, no `<style>` blocks, `index.css` only sets Tailwind, Flowbite, the primary colour and the themes up (ADR FE-STYLES); the service worker updates on demand (prompt mode). Tests (Vitest, jsdom, Testing Library) sit next to the code as `*.test.ts` and query what users see.

Out of scope, do not implement or prepare for: offline storage of documents and sync, saving play state (resources) anywhere but the device, the narrative steps beyond the character name, spending money. Each awaits its own decision (listed under Open in `docs/adr/`).

The bindings in `app/frontend/src/wasm/` are generated by `mise run wasm:build`; never edit or commit them.

## Design principles

1. **The Game Master can override every rule.** Breaking a rule must be possible, never easier than respecting it.
2. **Types encode structure, not game bounds.** Do not constrain ranks, characteristic values or counts in types. Out-of-bound values are representable and produce violations.
3. **Violations are computed, never stored.** Validation is a pure, deterministic function of its inputs. Same inputs, same violations, in a stable order.
4. **Deviations are explicit.** A deviation is accepted only through a waiver, and a waiver only through a deliberate, attributed action on that one violation: nothing in the engines or the application waives a rule implicitly (never as a side effect of another choice, a validation or a default, never for several violations at once). A deviation explicitly waived counts as settled. Every waiver stays inspectable on the character: its rule, subject, author and reason. Manual adjustments always surface as a `Notice` violation.
5. **Derived values are computed, never stored.** Store choices and adjustments; compute values and their breakdown.
6. **Rules are code.** No rules engine, no expression language, no configuration-driven behaviour, no dynamic registry.
7. **No anticipation.** Write the simplest code that serves the current scope. Abstractions are extracted from duplication, not designed ahead.

## Rust conventions

- Stable toolchain, edition 2024. Tool versions and tasks are managed with [mise](https://mise.jdx.dev) (`mise.toml`); do not add `rust-toolchain.toml`.
- Lints (no `unsafe`, no `unwrap`/`expect`/`panic!` outside tests, missing docs, ...) are enforced by `[workspace.lints]` in `Cargo.toml` and `.clippy.toml`. Do not silence them; fix the code.
- Prefer enums and newtypes over strings and bare integers. Newtypes carry meaning (a rank is not a level), not game limits.
- Builder methods (`with_*`, `without_*`) take their newtype parameters as `impl Into<Newtype>`, and those newtypes implement `From<primitive>` (infallible: newtypes carry no game bounds), so callers write `.with_age(32)`. Give each integer newtype exactly one `From<integer>` impl: a second one breaks inference of integer literals (`with_age(32)` falls back to `i32` and fails). Never let two parameters of one method convert from the same primitive, since swapping them would compile: this is why `Author` stays explicit in `Waiver::new` and `Adjustment::new`, next to a string `reason`. Constructors (`new`) keep their explicit parameters. Money is the exception to `From<integer>`: an engine's `Money` is built from a named coin (`Money::pa(7)`), since a bare integer would hide the coin.
- Keep the public API minimal; anything not needed by a test or a documented use stays private.
- Dependencies: conventional, widely used crates (e.g. `thiserror`, `anyhow`) are allowed when they make the domain cleaner to implement; ask before adding any crate. No `tokio` or other async runtime, and no async code, except in `app/backend`. Only `app/backend`, `app/document` and `app/wasm` may use `serde`; the kernel and the engines stay free of it (ADR APP-BOUNDARIES).
- Run common tasks through the mise task runner (`mise run <task>`; list them with `mise tasks`), not by calling cargo directly: `fmt`, `lint`, `test`, `doc-test`, `coverage`, `check`; for the client, `wasm:build`, `frontend:dev`, `frontend:check`, `frontend:test`, `frontend:build`, `serve`.
- Before finishing a task, run `mise run check` (fmt, lint, test, doc-test), and `mise run frontend:check` and `mise run frontend:test` when it touches `app/document`, `app/wasm` or `app/frontend`. Tests run with [cargo-nextest](https://nexte.st) (`mise run test`), not `cargo test`. Nextest does not run doc tests, so they run through `mise run doc-test`.
- Clippy runs `pedantic`: a function over 100 lines fails `too_many_lines` (extract helpers, as `app/wasm/src/sheet.rs` does), and `assert!(!x.is_empty())` fails `assert_is_empty` (write `assert_ne!(x, "")`).
- Do not modify `mise.toml` unless explicitly asked, and trust its content. In particular, ignore Mr Boxington (`mr-boxington`, `mr_boxington`): it is a wrapper on top of cargo that saves space and speeds up compilation, and is safe to leave as is.

## Repository layout (current)

```
Cargo.toml            # virtual workspace
mise.toml             # tool versions and tasks
.clippy.toml
docs/adr/             # architecture decisions, one document per area
crates/co-kernel/     # co-kernel: concepts shared by every engine
crates/co-contemporain/ # COC2 engine: character creation
crates/co-fantasy/     # COF2 engine: character creation at level 1
app/backend/           # palimpseste-backend: HTTP server, checks and stores
app/document/          # palimpseste-document: the choices document
app/wasm/              # palimpseste-wasm: the engines compiled to WebAssembly
app/frontend/          # the PWA (pnpm package, not a Cargo crate)
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
| arme, mains nues | `Weapon`, `Weapon::MainsNues` (COC2) |
| protection | `Protection` |
| objet, quantité | `Item`, `ItemName` (free text), `Quantity` |
| équipement (porté en jeu) | `Equipment`, `InPlay` |
| dés, dé | `Dice`, `Die` |
| dommages (DM) | `Damage` |
| bonus critique, zone de critique | `CriticalBonus`, `CriticalRange` |
| réduction des dommages (RD) | `DamageReduction` |
| bourse, argent | purse, `Money` |

## Working rules for agents

- Make the smallest change that fulfils the task. Do not refactor unrelated code.
- Write tests first or alongside the change; every violation category needs at least one test.
- The co-rules skill (rule identifiers such as `COF.DER.PV-L1`, interpretations `D-xx`) paraphrases copyrighted rulebooks. It is a private plugin (`co-rules@co-private`, skill `co-rules:co-rules`), never committed here, and most contributors do not have it. Rule identifiers and `D-xx` numbers in the code and docs are stable references to it; when a rule you need is not in the code or the docs, and the skill is unavailable, stop and ask.
- Never copy rulebook text into the repository. Test fixtures use invented or minimal data; the book's worked characters (co-rules `examples/`) may be fixtures, as choices and numbers only.
- If a decision affects the structure (new concept in the public API, new crate, new dependency, change to the principles above), stop and ask, and record the outcome in `docs/adr/` as its README says: change the decision in its area's document in place (never add an amendment), and cite it as `ADR <ID>`.
- When unsure whether something belongs to the kernel, it does not.
- A rule interpretation the co-rules skill leaves open is proposed as a `D-xx` entry in its `decisions.md` (status `open`, a default, an alternative) and cited in the code; only the developer moves it to `decided`.
- `cargo fmt` rewraps imports and long lines: read a file again before replacing exact text in it, and prefer the Edit tool to scripted replacements.

## Delivering a feature across the layers

A rule the application shows goes through every layer, in this order, each layer a change of its own that passes the checks:

1. **Engine** (`crates/`): the rule and its violations, each saying where it is settled (a creation step, or a level-up or spending through its subject or a `level` / `spending` parameter, ADR ENG-COF-PLACEMENT); what the choices offer within the rules (`level_offer`, `spending_offer`), so the interface restates no rule; the public read-only accessors the interface needs; tests next to the code; the engine's README and `docs/adr/engines.md`.
2. **Choices document** (`app/document`): new fields with `#[serde(default)]`; a field an older build would drop bumps `VERSION`, which reads only the current version, with a backend schema migration for the stored documents (ADR APP-DOCUMENT, BE-STORE); readability tests.
3. **Bindings** (`app/wasm`): catalog entries and sheet details, named by the engines (ADR APP-BINDINGS); tests in `lib.rs`; then `mise run wasm:build` and `mise run frontend:check` to see the TypeScript types.
4. **Front end**: components, a French message in `messages.ts` for every new rule identifier, Vitest tests next to the code.
5. **Docs**: the ADRs of each area, the scope sections of this file (in the same change as the code that brings something into scope), the READMEs.

Before planning such a feature, list the decisions it needs (rule interpretations, new public concepts, ADRs) and ask them in one batch, the recommended option first. When the user asks to evaluate or plan, deliver the plan only; implement once they say so.

## Version control and pull requests

- Before starting and before pushing, `jj git fetch`. Pull requests are merged by rebase: the merged commits get new hashes and keep their change ids, so a stack built on the old ones shows them as divergent. Rebase the stack onto `main@origin` (`jj rebase -s <first change> -d main@origin`), check that the trees match, and leave the stale bookmark to the user.
- Keep each phase of a task in its own jj change as you go (`jj new` between phases), so the commits the user asks for need no splitting. Describe a change `<area>: <what it does>` (areas: `kernel`, `co-fantasy`, `co-contemporain`, `engines`, `document`, `backend`, `wasm`, `frontend`, `docs`), then a body saying why, then the attribution trailer. Push only when asked.
- If one change still holds several phases that share files, build each commit from a copy of the files as they were at the end of its phase (`jj new <parent>`, copy, `mise run check`), then rebase the original change on top and copy the final files back: what is left in it is the last commit.
- A pull request: a bookmark named after the feature, `jj git push --bookmark <name>`, then `gh pr create`. When the token cannot create pull requests, give the compare link and the description written to a file.