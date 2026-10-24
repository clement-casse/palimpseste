# co-contemporain

The COC2 engine: character creation, and the experience spent after it,
under the core rules of *Chroniques Oubliées Contemporain*, second edition. It has no variants and no universe
rules. It depends on [`co-kernel`](../co-kernel) only (ADR ENG-CRATES).

A `Character` stores choices, adjustments and waivers. Everything else is
computed on demand: characteristics, secondary characteristics with their
breakdown, budgets, and violations. The crate-level doc test in `lib.rs`
builds a complete character. Read it first.

## Modules

| File | Holds | Rules |
|---|---|---|
| `character.rs` | `Character` (the choices and their `with_*` / `without_*` builders), `Value` (`FIXED`, `name()`), `Target`, `AttackMode` (`id()`, `name()`); the `violations()` and `report()` entry points | |
| `rules.rs` | every `RuleId`, `Mechanic` and engine `ValueId` constant | |
| `age.rs` | `Age`, `AgeBracket`, the limits of each bracket | COC.CRE.AGE-BRACKET |
| `domaine.rs` | `Domaine` (18), its voie slots and social class requirements | COC.CRE.DOMAINE, COC.CNT.DOMAINES |
| `voie.rs` | `Voie` (18), the names of its capacités, the characteristic each rank 5 raises, the checks on the ranks acquired | CO.VOIE.* (the kernel's `voie_rank_violations`), COC.CRE.CAPACITY-POINTS, COC.CRE.MAX-VOIES |
| `experience.rs` | `Experience`, `XpSpending`, the XP left and the checks on each spending | COC.PRG.COST, ONE-RANK, LIMITS, XP-USE |
| `traits.rs` | `Trait`, `Sequel` (and its identifier once acquired in play), `TraitPoints`, `SocialClass` | COC.CRE.TRAITS, COC.CNT.TRAITS, COC.CRE.SOCIAL-CLASS |
| `characteristics.rs` | the characteristic points allocated, `ALLOCATED_POINTS`, starting and current values, the rank-5 raises | COC.CRE.CARAC-*, COC.PRG.CARAC |
| `secondary.rs` | Init, DEF, ATC/ATD, CG, health scale | COC.DER.* |
| `weapon.rs` | `Weapon` (one variant per table row, `MainsNues` included), damage dice and DM, bonus critique, critical range | COC.EQP.MELEE, COC.EQP.RANGED, COC.DER.DM-BONUS, COC.DER.CRIT-* |
| `protection.rs` | `Protection`: RD and malus, never DEF | COC.EQP.PROTECTIONS |
| `equipment.rs` | `Item` (a weapon, or free text named by the kernel's `ItemName`), the starting equipment of the voies | COC.CRE.EQUIPMENT |
| `step.rs` | `Step` in the book order, and `Step::of(&Violation)` | COC.CRE.STEPS |
| `label.rs` | `label(id)` and `term_label(rule)`: the French names of the identifiers the engine emits | |

Rule identifiers (`COC.CRE.TRAITS`) and interpretations (`D-07`) trace to
the private co-rules skill. The design
decisions are in [docs/adr/engines.md](../../docs/adr/engines.md) (`ENG-*`,
`ENG-COC-*`).

## How it fits together

- **Content is enums.** Each content enum has `ALL`, `id()` (a stable
  string such as `coc.crime`), `from_id()` and `name()` (the French
  display name). Its data is a `match` table beside the mechanic that
  reads it. There are no data files (ADR ENG-CONTENT, ADR ENG-CATALOGUE).
- **Names.** `Value::FIXED` lists the values in the book order and
  `Value::name()` names them; `Voie::ability_name(rank)` names each
  capacité. `label(id)` names every identifier the engine
  emits as a violation subject or a breakdown source, and
  `term_label(rule)` the terms a formula sets (the age bracket), so that
  the front end restates none of them (ADR ENG-CATALOGUE).
- **Violations.** `Character::violations` calls each module's
  `check(&Character, &mut Vec<Violation>)` (age, domaine, voies, traits,
  characteristics, equipment), then adds one notice per adjustment.
  `report()` matches the result against the waivers.
- **Values.** `Character::value(Value)` returns a `DerivedValue`, or `None`
  when it does not apply. Every `Value` and every characteristic is a
  `Target` for adjustments, budgets included (ADR ENG-TARGETS).

## Recipes

**Adding a content item** (a weapon, a protection, a trait):

1. Add the variant (its French name, no accents) to the enum and to `ALL`.
2. Give it a slug in `id()` and a display name in `name()`.
3. Fill in its row in every `match` table the rules read.
4. `tests/catalogue.rs` checks that identifiers are unique and round-trip.
   `palimpseste-document` and `palimpseste-wasm` pick it up through
   `from_id` and `ALL`, and `label` names it through `from_id`.

**Adding a value:** add the variant to `Value`, its identifier to `id()`,
its name to `name()`, and the variant to `Value::FIXED` unless it is a
weapon value: `from_id`, and so the adjustments of the choices document,
find it there, and the sheet lists it in that order. A new choice the violations name
(a `ValueId` in `rules.rs`) gets its name in `CHOICES` in `label.rs`.

**Adding a rule:**

1. Find the rule identifier in the co-rules skill, and declare it in
   `rules.rs`. Soft limits of different severities get distinct
   identifiers, because a waiver matches on rule and subject only
   (ADR ENG-FINDINGS).
2. Emit it from the `check` of the module that owns the concept. A missing
   choice is `Incomplete`. Anything the Game Master may accept is a
   `Deviation`, never `Uncomputable` (ADR ENG-WAIVABLE).
3. Place it in a step in `step.rs` (`RULE_STEPS`).
4. Add its French message in
   [`app/frontend/src/messages.ts`](../../app/frontend/src/messages.ts).
   Without one, the editor shows the identifier.

## Testing

- Unit tests sit in each module. They include order independence
  (`violations_do_not_depend_on_the_order_of_choices`) and giving choices
  up.
- `tests/amelia_steele.rs`: the acceptance test, built from the book's
  worked character as recorded in the co-rules `examples/` (choices and
  numbers only).
- `tests/catalogue.rs`: every content enum lists all its variants, each
  with a unique identifier that converts back, and a display name.

Fixtures use invented or minimal data. Never copy rulebook text into the
repository.

```sh
mise run check      # fmt, lint, test (nextest), doc-test
```
