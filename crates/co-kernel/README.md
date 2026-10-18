# co-kernel

The concepts that mean exactly the same thing in every Chroniques Oubliées
engine: characteristics, the voie skeleton, derived values with their
breakdown, adjustments, violations, waivers, the violation report, and
dice. Nothing engine-specific belongs here. No formula, budget, table or
game content goes in this crate.

It has no dependencies. Like the engines, it has no I/O, no async code and
no `serde` (ADR APP-BOUNDARIES). Every engine and every application crate
depends on it.

## Modules

| File | Holds | ADR |
|---|---|---|
| `characteristic.rs` | `Characteristic` (the seven, with their abbreviations), `CharacteristicValue(i32)`, `Characteristics` (an array indexed by characteristic) | KER-CHARACTERISTICS |
| `voie.rs` | `Rank`, `VoieId`, `AbilityId`, `Voie` (rank → ability) | KER-VOIE |
| `derived.rs` | `DerivedValue`, `Contribution`, `Source` | KER-SOURCES, KER-TOTAL |
| `adjustment.rs` | `Adjustment`, `Author` | KER-ADJUSTMENTS |
| `violation.rs` | `Violation`, `RuleId`, `Mechanic`, `ValueId`, `Subject`, `Category`, `Severity`, `Parameter`, `ADJUSTMENT_RULE`, `adjustment_notice` | KER-IDS, KER-SUBJECT, KER-SEVERITY |
| `waiver.rs` | `Waiver` | KER-MATCHING |
| `report.rs` | `ViolationReport`, `ReportEntry` | KER-REPORT |
| `dice.rs` | `Die`, `Dice` (described, never rolled) | KER-DICE |

The crate-level doc test in `lib.rs` walks through the whole flow with
invented identifiers. Read it first.

## How an engine uses the kernel

1. **Compute values.** Build a `DerivedValue` from contributions:
   `with(Source::Rule(..), n)` for a constant of the formula,
   `with_characteristic` for a characteristic, `Source::Value(..)` for a
   term taken from another value, and `with_adjustment` for a Game Master
   adjustment. `total()` is always the sum of the breakdown. It is summed
   in `i64` and then clamped, so it never panics and does not depend on the
   order of the contributions.
2. **Emit violations.** Build each one with `Violation::incomplete`,
   `deviation(.., Severity)`, `uncomputable` or `notice`, giving a
   `RuleId`, the emitting `Mechanic` and a `Subject`. Add display
   parameters with `with_parameter`.
3. **Notice adjustments.** Every adjustment becomes a `Notice` of
   `CO.GM.ADJUSTMENT`. Use `DerivedValue::adjustment_notices` for a derived
   value, and `adjustment_notice` for a budget.
4. **Report.** `ViolationReport::new(violations, &waivers)` sorts the
   violations and matches deviations against waivers on rule and subject.
   It lists the open entries, the notices and the unused waivers.

## Invariants to keep

- **No game bounds in types.** `Rank(u8)`, `CharacteristicValue(i32)` and
  `Die(u8)` limit storage, not the game. Out-of-bound values must stay
  representable, because the engines report them as violations.
- **Pure and ordered.** The report depends only on its inputs, never on
  the order the mechanics ran in.
- **Only deviations can be waived.** `Incomplete`, `Uncomputable` and
  `Notice` entries never match a waiver. Matching ignores the severity
  and the parameters.
- **Closed sets.** `Source` and `Subject` are closed enums. Engines name
  their own concepts through `ValueId` only, and never through a new
  variant.
- **Newtypes.** Builder parameters take `impl Into<Newtype>`, and each
  integer newtype has exactly one `From<integer>`. `Author` stays explicit
  in `Waiver::new` and `Adjustment::new`.

Before adding anything here, ask whether it means *exactly* the same thing
in COF2 and COC2. If you are unsure, it belongs in an engine. A new public
concept is a structural decision: record it in
[docs/adr/kernel.md](../../docs/adr/kernel.md).

## Testing

Unit tests sit in each module (`#[cfg(test)]`), and the crate example is a
doc test. Every violation category has at least one test. Identifiers in
the tests are invented (`TEST.*`).

```sh
mise run test       # nextest, whole workspace
mise run doc-test   # doc tests (nextest does not run them)
mise run check      # fmt, lint, test, doc-test: run before finishing
```
