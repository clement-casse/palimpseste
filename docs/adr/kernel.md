# Kernel

`co-kernel` holds the concepts that mean the same thing in every engine
(scope in AGENTS.md). It has no I/O, no async code and no `serde`
(ADR APP-BOUNDARIES).

## Values

**KER-CHARACTERISTICS: One signed value.** A characteristic has a single
signed value, which is also its modifier (`CharacteristicValue(i32)`).
Game bounds belong to the engines, which report them as violations.
`Characteristic::abbreviation()` gives the books' abbreviation (`AGI`).

**KER-VOIE: The rank checks of a voie.** `Rank` wraps a `u8` whose width
limits storage, not the game: gaps, rank 0 and ranks 4 to 8 (prestige
voies) are representable. `VoieId` and `AbilityId` name a voie and a
capacité. `voie_rank_violations` checks the ranks acquired in one voie for
the engine's mechanic: a rank outside 1 to 5 is a `standard` deviation of
CO.VOIE.STRUCTURE (`VOIE_STRUCTURE`), the ranks missing below the highest
one `standard` deviation of CO.VOIE.ORDER (`VOIE_ORDER`). The ranks a
character acquired stay in the engines (ENG-VOIE-RANKS). *Why:* both
engines apply these two rules, unchanged, to every voie they hold
(prestige voies, out of scope, would bring their own range); the abilities
of a voie per rank wait for the capacité names (engines.md, Open).

**KER-ITEMS: Free text items and quantities.** `ItemName` names an item
no rule reads, and `Quantity(u32)` counts an item carried. Which items are
content stays in the engines (ADR ENG-ITEMS).

**KER-SOURCES: A closed set of contribution sources.** A `DerivedValue` is
a list of contributions, each with a `Source`: `Characteristic`,
`Ability`, `Rule(RuleId)` (a constant the formula sets itself),
`Value(ValueId)` (a term taken from another value the engine names: the
level, an armour bonus) or `Adjustment`. Every term of a formula is in the
breakdown, so `total()` is always the full value. *Why:* the engines live
in other crates and cannot add variants, so the two they need stay
generic.

**KER-TOTAL: Exact, then clamped.** `DerivedValue::total` sums in `i64`,
then clamps to the `i32` range. *Why:* out-of-bound inputs never panic,
and the total does not depend on the order of the contributions, as
saturating at each step would.

**KER-DICE: Dice are described, never rolled.** `Die(u8)` is a number of
faces; `Dice(Vec<Die>)` the dice rolled together (`2d6` is `[d6, d6]`), so
mixed dice are representable. Flat bonuses are not dice: they stay in the
breakdown of a `DerivedValue`. COF2's `RecoveryDie` stays in its engine.

## Adjustments

**KER-ADJUSTMENTS: Additive, attributed, always noticed.** An `Adjustment`
is a signed amount with an `Author` and a reason, and no target. Applied
to a `DerivedValue`, it is one more contribution: an override is never a
silent replacement of the total. Each adjustment yields a `Notice` of the
project rule `CO.GM.ADJUSTMENT` (`ADJUSTMENT_RULE`, design principle 4),
read from the breakdown for a derived value
(`DerivedValue::adjustment_notices`, so it cannot drift from it) and built
by `adjustment_notice` for a budget.

## Violations and waivers

**KER-IDS: Static identifiers, compared by content.** `RuleId` and
`ValueId` wrap a `Cow<'static, str>`: engines declare constants
(`from_static`), waivers read from storage hold owned strings (`new`).
`ValueId` is the only place an engine names its own concepts (values,
budgets, the level). `Mechanic` wraps a `&'static str`; it is for tracing
only, never persisted nor matched.

**KER-SUBJECT: A closed set of subjects.** `Subject` is `Character`,
`Characteristic`, `Voie`, `Ability` or `Value`.

**KER-SEVERITY: A deviation carries a severity, for display.**
`Category::Deviation(Severity)`, with `Recommended`, `Standard` and
`Absolute`. A waiver matches whatever the severity: the Game Master can
waive an `Absolute` rule like any other. An engine therefore gives each
soft limit of a distinct severity its own rule identifier
(ENG-FINDINGS).

**KER-MATCHING: Rule and subject, deviations only.** A waiver matches a
violation on its rule and subject, and only when it is a `Deviation`:
`Incomplete`, `Uncomputable` and `Notice` cannot be waived. Parameters are
for display and never matched, so a waived deviation stays waived when the
offending value changes.

**KER-REPORT: A pure, ordered report.** `ViolationReport::new` sorts the
violations (rule, mechanic, subject, category, parameters), so the result
does not depend on the order the mechanics ran in. Each deviation takes
the first matching waiver; waivers that accepted nothing, duplicates
included, are reported as unused, so the Game Master sees stale ones.
Identical violations are kept, not merged. `open()` and `is_clean()`
ignore notices, which `notices()` lists. *Why:* a notice cannot be waived,
so counting it as open would leave every adjusted character unclean for
good.
