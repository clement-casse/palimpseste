# co-fantasy

The COF2 engine: character creation at level 1 under the core rules of
*Chroniques Oubliées Fantasy*, second edition. It has no variants, no
prestige voies and no progression. It depends on
[`co-kernel`](../co-kernel) only (ADR ENG-CRATES).

A `Character` stores choices, adjustments and waivers. Everything else is
computed on demand: final characteristics, derived values with their
breakdown, and violations. The crate-level doc test in `lib.rs` builds a
complete nain guerrier. Read it first.

## Modules

| File | Holds | Rules |
|---|---|---|
| `character.rs` | `Character` (the choices and their `with_*` / `without_*` builders), `Value` (`FIXED`, `name()`), `Target`, `AttackMode`; `violations()`, `report()`, and options that depend on the choices (`peuple_slot_options`) | |
| `rules.rs` | every `RuleId`, `Mechanic` and engine `ValueId` constant, and the level fixed at creation | |
| `profil.rs` | `Family` (4), `Profil` (14), `RecoveryDie`, weapon mastery | COF.CNT.FAMILIES, COF.CNT.PROFILS |
| `peuple.rs` | `Peuple` (8), its modifiers and voie de peuple | COF.CRE.PEUPLE, COF.CNT.PEUPLES |
| `characteristics.rs` | `Series` (3), the series assignment, peuple modifiers, bounds, `SERIES_VALUE` | COF.CRE.CARAC-* |
| `voie.rs` | `Voie` (the 78 reachable at level 1), the checks on the ranks acquired | CO.VOIE.*, COF.PRG.LEVEL-REQ, COF.CRE.MAGE-EXTRA |
| `derived.rs` | PV, DR, PC, PM, Init, DEF, attacks, with the effects of rank 1-2 capacités (D-22) | COF.DER.* |
| `armour.rs` | `Armour`, `Shield`, the profil's armour limits | COF.EQP.ARMOUR, COF.DER.ARMOUR-LIMIT |
| `weapon.rs` | `Weapon`, `WeaponUse`, dice per use, DM, critical range, the checks on the weapons carried | COF.EQP.MELEE, COF.EQP.RANGED, COF.DER.DM, COF.DER.CRIT-RANGE |
| `equipment.rs` | `Item`, `ItemName`, `Quantity`, the profil's starting equipment (`Gear`, `StartingLine`, `StartingItem`) | COF.CRE.EQUIPMENT |
| `money.rs` | `Money` (built from coins: `Money::pa(9)`), `purse_dice()` | COF.EQP.CURRENCY |
| `step.rs` | `Step` in the book order, and `Step::of(&Violation)` | COF.CRE.STEPS |
| `label.rs` | `label(id)` and `term_label(rule)`: the French names of the identifiers the engine emits | |

Rule identifiers (`COF.CRE.CARAC-SERIES`) and interpretations (`D-15`)
trace to the private co-rules skill. The
design decisions are in [docs/adr/engines.md](../../docs/adr/engines.md)
(`ENG-*`, `ENG-COF-*`).

## How it fits together

- **Content is enums.** Each content enum has `ALL`, `id()` (a stable
  string such as `cof.weapon.epee-longue`), `from_id()` and `name()` (the
  French display name). Its data is a `match` table beside the mechanic
  that reads it. There are no data files (ADR ENG-CONTENT,
  ADR ENG-CATALOGUE).
- **Names.** `Value::FIXED` lists the values in the book order and
  `Value::name()` names them. `label(id)` names every identifier the engine
  emits as a violation subject or a breakdown source, and
  `term_label(rule)` the terms a formula sets (the level), so that the
  front end restates none of them (ADR ENG-CATALOGUE).
- **Violations.** `Character::violations` calls each module's
  `check(&Character, &mut Vec<Violation>)` (profil, peuple,
  characteristics, voies, armour, equipment, money, weapons, derived
  values), then adds one notice per adjustment. `report()` matches the
  result against the waivers.
- **Level 1 is not stored.** Attack values take it as
  `Source::Rule(COF.CRE.LEVEL1)`. A rank above 1 is a deviation, except
  for the mage's extra capacité (ADR ENG-COF-LEVEL).
- **Values.** `Character::value(Value)` returns a `DerivedValue`, or `None`
  when it does not apply. Weapon values are keyed by weapon and use
  (`Value::Damage(Weapon, WeaponUse)`). Every `Value` and every
  characteristic is a `Target` for adjustments (ADR ENG-TARGETS).

## Recipes

**Adding a content item** (a weapon, an armour, a voie):

1. Add the variant (its French name, no accents) to the enum and to `ALL`.
2. Give it a slug in `id()` and a display name in `name()`.
3. Fill in its row in every `match` table the rules read (weapon profile,
   profil voies, starting equipment, ...).
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
   `rules.rs`. Advice and hard limits get distinct identifiers, so that
   waiving the advice never accepts a wrong value (ADR ENG-FINDINGS).
2. Emit it from the `check` of the module that owns the concept. A missing
   choice is `Incomplete`. Anything the Game Master may accept is a
   `Deviation`. COF2 has no `Uncomputable` finding (ADR ENG-WAIVABLE).
3. Place it in a step in `step.rs`.
4. Add its French message in
   [`app/frontend/src/messages.ts`](../../app/frontend/src/messages.ts).

**Adding a capacité effect:** only rank 1-2 capacités with a documented
effect on a value are computed (D-22). The capacité contributes as
`Source::Ability`, with an identifier built from its voie and rank.

## Testing

- Unit tests sit in each module. They include order independence
  (`violations_do_not_depend_on_the_order_of_choices`).
- `tests/lhagva.rs` and `tests/ionas.rs`: the acceptance tests, built from
  the book's worked characters as recorded in the co-rules `examples/`
  (choices and numbers only).
- `tests/catalogue.rs`: every content enum lists all its variants, each
  with a unique identifier that converts back, and a display name.

Fixtures use invented or minimal data. Never copy rulebook text into the
repository.

```sh
mise run check      # fmt, lint, test (nextest), doc-test
```
