# 0001. Initial design of co-kernel

Date: 2026-10-13, amended 2026-10-14 (dependencies; violations and waivers;
review: contribution sources, static identifiers, notices; ergonomics;
code review: clamped totals, duplicate waivers).
Status: accepted.

## Context

The first delivery of `co-kernel` covers characteristics, the voie skeleton,
derived values with their breakdown, and manual adjustments. The second
delivery adds violations, waivers and the violation report, which completes
the kernel scope.

## Decisions

- **Characteristic values.** A characteristic has a single signed value,
  which is also its modifier (`CharacteristicValue(i32)`). Game bounds belong
  to the engines and are checked by validation.
- **Voie skeleton.** A `Voie` maps each `Rank` to one `AbilityId`
  (`BTreeMap`), following CO.VOIE.STRUCTURE. `Rank` wraps a `u8`; its width
  limits storage only and is not a game bound. Gaps, rank 0 and
  ranks 4 to 8 (prestige voies) can be represented.
- **Voie acquisition is deferred.** The ranks a character has acquired in a
  voie arrive with the first engine, together with CO.VOIE.ORDER and
  CO.VOIE.RANK-SCALING, which use them.
- **Contribution sources.** `Source` is a closed enum: `Characteristic`,
  `Ability`, `Rule`, `Value`, `Adjustment`. Engines live in other crates and
  cannot add variants, so the two variants they rely on stay generic:
  `Rule(RuleId)` for a term the formula sets itself (a constant),
  `Value(ValueId)` for a term taken from another value the engine names (the
  level, an armour bonus). Every term of a formula fits the breakdown, so
  `total()` is always the full value.
- **Adjustments are additive.** An `Adjustment` is a signed amount with an
  author and a reason, and carries no target. Applied to a `DerivedValue`, it
  appears in the breakdown as a contribution. A manual override of a formula
  is expressed this way, never as a silent replacement of the total. Each
  adjustment surfaces as a `Notice`. For a derived value, the notices are
  read from the breakdown (`DerivedValue::adjustment_notices`), so they
  cannot drift from it; `adjustment_notice` remains for budgets. Both build
  the notice the same way.
- **Totals are clamped.** `DerivedValue::total` computes the exact sum in
  `i64`, then clamps it to the `i32` range, so out-of-bound inputs cannot
  panic and the total does not depend on the order of the contributions
  (saturating at each step would).
- **Dependencies.** The initial assumption of a standard-library-only kernel
  is relaxed. `co-kernel` may depend on conventional, widely used crates
  (such as `thiserror` or `anyhow`) when they make the domain cleaner to
  implement. It must not depend on `tokio` or any async runtime, and it
  contains no async code. Adding a crate is still proposed to and approved by
  the developer before it lands.
- **No serde.** Serialization stays out of scope, so the unused `serde`
  workspace dependency is removed.
- **Doc tests.** The crate documentation carries a runnable example of the
  whole flow. Nextest does not run doc tests, so a `doc-test` mise task
  (`cargo test --doc`) runs them and `check` depends on it.

## Violations and waivers

- **Identifiers.** `RuleId` and `ValueId` wrap a `Cow<'static, str>`.
  Rules are code, so engines declare their ids as constants
  (`from_static`); waivers loaded from storage use owned strings (`new`).
  Both forms compare by content. `Mechanic` wraps a `&'static str`: mechanics
  are code and never persisted (violations are not stored, waivers carry no
  mechanic). It is for tracing only and never takes part in matching.
- **Subject.** `Subject` is a closed enum: `Character`, `Characteristic`,
  `Voie`, `Ability`, `Value`. `ValueId` is the only place where an engine
  names its own concepts (its derived values, budgets and other values such
  as the level).
- **Parameters are for display.** A violation's parameters are excluded from
  matching, so a waived deviation stays waived when the offending value
  changes.
- **Matching.** A waiver matches a violation on rule and subject, and only
  when the violation is a `Deviation`. The other categories cannot be waived.
- **Report.** `ViolationReport::new` is a pure function. It sorts the
  violations (derived order: rule, mechanic, subject, category, parameters),
  so the result does not depend on the order mechanics ran in. Each deviation
  takes the first matching waiver. Waivers that accepted no violation are
  reported as unused, so the GM can see stale waivers; a duplicate of an
  earlier waiver is one of them. Identical violations are kept, not merged.
- **Notices are information.** `open()` and `is_clean()` ignore notices;
  `notices()` lists them. A notice cannot be waived, so counting it as open
  would leave every adjusted character unclean for good.
- **`CO.GM.ADJUSTMENT`.** The rule behind the adjustment `Notice` is a
  project rule (principle 4), not a book rule. It is exposed as
  `ADJUSTMENT_RULE`, so callers recognise adjustment notices without
  comparing strings.
- **Severity is deferred.** The co-rules skill gives soft limits a severity
  (recommended, standard, absolute). It is added when an engine first emits
  soft-limit deviations.
