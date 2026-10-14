# 0002. co-contemporain: COC2 character creation engine

Date: 2026-10-14.
Status: accepted.

## Context

The kernel is complete (ADR 0001). The first engine covers *Chroniques
Oubliées Contemporain*, second edition (COC2), core rules only, no variant and
no universe. This delivery covers **character creation**: the choices a player
makes, the budgets and limits that frame them, and the secondary
characteristics computed from them. Play time, combat resolution,
progression (XP) and inventory come later. The acceptance test is the book's
worked character, Amelia Steele.

## Decisions

- **One crate, `co-contemporain`, depending on `co-kernel` only.** It holds
  the rules as code and the COC2 content the creation rules read.
- **Content as enums.** Domaines (18), voies (18), traits distinctifs and
  sequels are closed Rust enums; their data (domaine voie slots, points each
  trait allows, age-bracket limits) are `match` tables beside the mechanic
  that reads them. No data file, no serialization (still out of scope).
- **French content names.** Concept types are English (`Trait`, `Sequel`,
  `AgeBracket`, `SocialClass`; `Domaine` and `Voie` kept). Content variants
  keep their French name without accents (`Voie::ArmesDeContact`,
  `Trait::ForceDeLaNature`, `Domaine::Aventure`), as the co-rules skill asks
  not to invent translations.
- **A character stores choices only.** `Character` holds the age, the
  domaine, the ranks acquired per voie, the traits with their points, the
  characteristic points allocated, the adjustments and the waivers. Every
  other value is computed on demand, with its breakdown. Choices are made and
  given up in any order (`with_*`, `without_rank`, `without_trait`).
- **The bracket follows the age.** The age is stored in years and the
  bracket derived from it: below 10 there is none (budgets uncomputable),
  10-15 and 66+ are the adolescent and elderly brackets, which use the 16-30
  and 51-65 rows; D'un autre âge's budget changes appear in the breakdown
  only when the trait is taken (D-16, D-18).
- **Ranks are a set.** Acquired ranks per voie are a `BTreeSet<Rank>`, so a
  gap is representable and reported (CO.VOIE.ORDER); acquisition stays in
  the engine until a second engine shows what to share.
- **Trait points are a magnitude.** `TraitPoints(u8)`; the trait's side gives
  the sign. Points the catalogue does not allow are representable and
  reported (COC.CNT.TRAITS).
- **Engine concepts use `ValueId`.** Traits, domaine, age, social class and
  the computed values (`Value`) are named with `ValueId`: as violation
  subjects, and as breakdown sources for trait contributions. Capacité
  contributions use `Source::Ability` with an identifier per voie and rank
  (`coc.crime.2`); no capacité name is stored. The points the player
  allocates to a characteristic have their own source, `ALLOCATED_POINTS`,
  distinct from the `CaracPoints` budget.
- **Every computed value accepts adjustments.** A `Target` is a
  characteristic or a `Value`, budgets included: the Game Master can either
  raise a budget (adjustment, reported as a notice) or waive the deviation.
  Limits are checked against adjusted values. Each stored adjustment yields
  one notice, even when the adjusted value is uncomputable.
- **No two findings share a waiver.** Waivers match rule and subject and
  ignore severity (ADR 0001), so distinct findings get a distinct rule or
  subject: COC.CRE.CARAC is split into CARAC-POINTS, CARAC-LOWERED,
  CARAC-PHYSICAL (recommended) and CARAC-LIMITS (absolute); the domaine class
  requirement takes the missing class trait as subject; an age without D'un
  autre âge and the trait without the age take the age and the trait.
- **Waivable over uncomputable.** A state the GM may accept is a deviation,
  never an `Uncomputable` (which cannot be waived): contradictory or invalid
  class traits are a deviation. Only an age below every bracket is
  uncomputable.
- **Attacks by weapon category.** ATC and ATD are computed per
  `AttackMode` (unarmed, contact weapon, ranged weapon, heavy weapon) with
  the rank reached in the matching voie, without any weapon item. The contact
  DM bonus (FOR, Force de la nature) is computed alone, without dice.
- **Narrative prerequisites emit nothing.** Traits needing the Game Master's
  agreement are not checkable and produce no violation.

## Deferred

Weapon dice, bonus critique and critical range; arme de prédilection;
conditional DEF bonuses; rank 3 transfers; protections; starting equipment;
voie parameters; the rank 5 characteristic increase (D-20); the échelle
d'évolution (universe-specific); movement; XP and progression; play-time
state.
