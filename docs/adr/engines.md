# Engines

`co-contemporain` (COC2) and `co-fantasy` (COF2): the core rules of
character creation, at level 1 for COF2 (scope in AGENTS.md). The
acceptance tests are the books' worked characters: Amelia Steele (COC2),
Lhagva and Ionas (COF2). Rule identifiers (`COF.CRE.CARAC-SERIES`) and
rule interpretations (D-xx) come from the co-rules skill.

## Both engines

**ENG-CRATES: One crate per ruleset, on the kernel only.** Each engine
depends on `co-kernel` alone and holds its rules as code, with the content
they read.

**ENG-CONTENT: Content as enums.** Content a rule reads is a closed enum,
its data a `match` table beside the mechanic that reads it; there is no
data file. Variants keep their French name without accents
(`Voie::ArmesDeContact`), concept types are English. *Why:* rules are code
(design principle 6), and the co-rules skill forbids invented
translations.

**ENG-CATALOGUE: Content is listed and named by identifiers.** Each
content enum the choices document names has `ALL`, `id()` (a stable
string: `cof.epee-longue`, `coc.crime`), `from_id()` and `name()` (the
French display name). The content tables the rules read are public
read-only accessors; options depending on the current choices (the voies a
peuple slot accepts) are `Character` methods. *Why:* the front end offers
the options the rules allow without restating them.

**ENG-HEALTH-STATES: The COC2 health-scale states are content.**
`HealthState` (Contusionné, Affaibli, Blessé, Mourant) gives each state's
step and malus (COC.CNT.SCALES), and `HEALTH_SCALE_STEPS` the steps of a
full scale; the catalogue carries both, for the sheet to label its track
(ADR FE-SHEET). The engine holds no play state: the damage taken, and so
the state reached, are counted by the sheet.

**ENG-CHARACTER: A character stores choices only.** `Character` holds the
choices, the adjustments and the waivers; every other value is computed on
demand, with its breakdown. Choices are made and given up in any order
(`with_*`, `without_*`).

**ENG-TARGETS: Every computed value accepts adjustments.** A `Target` is a
characteristic or a `Value`, budgets included, so the Game Master can
raise a budget (an adjustment, reported as a notice) or waive the
deviation. Limits are checked against adjusted values. Each stored
adjustment yields one notice, even when its value is uncomputable.

**ENG-VALUE-IDS: Engine concepts are `ValueId`s.** Computed values
(`Value`), budgets, traits, domaine, age and social class are named by a
`ValueId`, as subjects and as sources. A capacité contributes as
`Source::Ability` with an identifier per voie and rank (`coc.crime.2`); no
capacité name is stored. A weapon value builds its identifier from the
weapon (`cof.damage.epee-longue.one-handed`), so `Value::id` is not
`const`.

**ENG-FINDINGS: One finding per waiver.** Waivers match rule and subject
and ignore severity (ADR KER-MATCHING), so two findings a waiver could
confuse get distinct rules or subjects. COC.CRE.CARAC is split into
CARAC-POINTS, CARAC-LOWERED, CARAC-PHYSICAL (recommended) and CARAC-LIMITS
(absolute); COF.CRE.CARAC-SERIES (standard) stands apart from the advice
of CARAC-KEY and CARAC-MAGIC (recommended), so waiving the advice never
accepts a wrong assignment.

**ENG-WAIVABLE: What the Game Master may accept is a deviation.** A
missing choice is `Incomplete`; any other state the Game Master may accept
is a `Deviation`, never `Uncomputable`, which cannot be waived. COC2's only
`Uncomputable` finding is an age below every bracket; COF2 has none.
Prerequisites needing the Game Master's agreement are not checkable and
emit nothing.

**ENG-STEPS: Steps guide, never gate.** Each engine has a `Step` enum in
the book order (COC.CRE.STEPS, COF.CRE.STEPS) and places each violation in
one. Every step can be opened at any time.

**ENG-VOIE-RANKS: Acquired ranks are a set.** The ranks acquired per voie
are a `BTreeSet<Rank>`, so a gap is representable and reported
(CO.VOIE.ORDER). The set, the CO.VOIE.STRUCTURE and CO.VOIE.ORDER checks,
and `Item`, `ItemName` and `Quantity` are duplicated in both engines (see
Open).

## Equipment and money

**ENG-ITEMS: Only items a rule reads are content (D-26).** Weapons,
armours and shields (COF2) and protections (COC2) are enums with their
tables. Any other item is free text (`Item::Other(ItemName)`), never
checked; an item becomes an enum when a rule reading it lands.
`with_item(item, quantity)` records what is carried, and a quantity of 0
gives it up. Armour and shield (COF2) and protections (COC2) are worn
slots, and count as possessed.

**ENG-EQUIPMENT: Starting equipment is checked, never imposed (D-26).**
Each line of the profil's (COF2) or the voies' (COC2) starting equipment
covers what it gives: an exact weapon, one of a few, a weapon matching a
predicate, a worn armour or shield. Matching is deterministic: lines in
table order, each taking the bundle covering the most items, each item
line the first matching items. A rule-backed item left over is a
`standard` deviation of COF.CRE.EQUIPMENT or COC.CRE.EQUIPMENT with the
item as subject, which the Game Master waives as a trade. An empty line is
not reported: an item may be given up. COF2 checks nothing without a
profil.

**ENG-WEAPON-VALUES: Weapon values are values.** `Value::Damage` (the DM
added to the dice), `Value::CriticalRange` and, in COC2,
`Value::CriticalBonus` have a breakdown and accept adjustments. The dice
come from the table: `Weapon::dice(use)` in COF2,
`Character::damage_dice(weapon)` in COC2.

**ENG-MONEY: Money is per engine, and a value.** Each engine owns its
`Money`, counted in its smallest coin and built from named coins only
(`Money::pa(7)`, the exception to `From<integer>`: a bare integer would
hide the coin). The money held is `Value::Money`, adjusted like any value.

## COC2

**ENG-COC-AGE: The bracket follows the age.** The age is stored in years
and the bracket derived from it: below 10 there is none (the budgets are
uncomputable); 10-15 and 66+ use the 16-30 and 51-65 rows. D'un autre
âge's budget changes appear in the breakdown only when the trait is taken
(D-16, D-18); an age without the trait takes the age as subject, the trait
without the age takes the trait.

**ENG-COC-TRAITS: Trait points are a magnitude.** `TraitPoints(u8)`; the
trait's side gives the sign. Points the catalogue does not allow are
reported (COC.CNT.TRAITS). Contradictory or invalid class traits are a
deviation; the domaine's class requirement takes the missing class trait
as subject.

**ENG-COC-POINTS: Allocated points have their own source.** The points
allocated to a characteristic contribute as `ALLOCATED_POINTS`, distinct
from the `CaracPoints` budget.

**ENG-COC-ATTACKS: Attacks by weapon category.** ATC and ATD are computed
per `AttackMode` (`Unarmed`, `ContactWeapon`, `RangedWeapon`,
`HeavyWeapon`) from the rank in the matching voie, without any weapon. The
contact DM bonus (FOR, Force de la nature) is computed alone, without
dice.

**ENG-COC-WEAPONS: One variant per table row.** The unarmed attack is a
weapon (D-12), never carried; the combat à mains nues rank 2 changes its
dice. Unarmed and contact weapons add the contact DM bonus, adjustments
included. The bonus critique is computed from the dice, `None` for area
weapons. The rank-4 capacités of the armes de contact and armes à distance
are computed.

**ENG-COC-PROTECTIONS: RD and malus, never DEF.** Protections add RD, and
their malus to Init, ATC and AGI tests, never to AGI or DEF (D-04, D-05).
No voie gives a protection (D-30).

**ENG-COC-MONEY: No money at creation.** The book gives none: the social
class describes a lifestyle, not an amount.

## COF2

**ENG-COF-CHARACTERISTICS: A series, then peuple modifiers.** The
character stores the series, the value assigned to each characteristic
(unassigned until chosen), and the peuple bonus and malus, set by two
methods taking a `Characteristic` so they cannot be swapped. A peuple with
a single malus applies it without a choice; a chosen malus replaces it and
is reported when the peuple does not offer it. The humain bonus follows
D-15, judged once all seven values are assigned. CARAC-KEY reads the
assigned series values, CARAC-MAGIC the final value.

**ENG-COF-LEVEL: Level 1, not stored.** Attack values take it as
`Source::Rule(COF.CRE.LEVEL1)`. A rank above 1 is a deviation of
COF.PRG.LEVEL-REQ, except the mage family's one rank 2 in a voie of its
profil or the voie du mage (COF.CRE.MAGE-EXTRA), never in another profil's
voie, waived or not.

**ENG-COF-VOIES: Slots count what is there.** The chosen voies are every
voie outside the voie de peuple slot, which holds the voies de peuple and
the voie du mage. Another profil's voie (a level-1 hybrid) or another
peuple's is a deviation with the voie as subject, so once waived the
counts do not stay incomplete.

**ENG-COF-ARMOUR: Armour and shield are choices.** They add their DEF;
the armour caps AGI in DEF and in the ranged attack (D-06). Beyond the
profil's limit is a `standard` deviation of COF.DER.ARMOUR-LIMIT (D-24);
the chevalier's Noblesse rank 2 raises the limit to the plaque complète.

**ENG-COF-CAPACITES: Documented rank 1-2 effects only (D-22).** Réflexes
éclair, Murmure dans le vent, Diversité, Petite taille and Petit pote.
Through the voie du mage, the peuple's rank-1 effect is kept, with the
peuple voie's rank-1 ability as source. A demi-elfe's upbringing is not
recorded, so its kept effect is left to an adjustment, as are the effects
of higher ranks.

**ENG-COF-PM: Documented spells only (D-23).** PM is `None` without any
spell; a negative total is a `standard` deviation of COF.DER.PM (D-11).

**ENG-COF-FAMILY: No family, no family values.** PV, DR, PC and
`recovery_die` are `None` without a profil, rather than computed without
their family term.

**ENG-COF-WEAPONS: Weapons by use.** Each of the 33 weapons has its uses
(contact in one or two hands, ranged, thrown). The damage adds FOR at
contact, except with the stylet. `Profil::masters` gives the mastery for
display (D-28): an unmastered weapon is not a violation. The arc long
needs FOR +1 (COF.EQP.RANGED) and Petite taille forbids large weapons
(D-29), both `standard` deviations.

**ENG-COF-PURSE: The purse is rolled.** `Money` counts pièces de cuivre
and displays with the fewest coins (COF.EQP.CURRENCY). The purse is the
2d6 pa the player rolled (`with_purse`, `purse_dice`): missing is
`Incomplete`, outside 2-12 whole pa a `standard` deviation.

## Open

- Moving the acquired ranks and the voie checks, and `Item`, `ItemName`
  and `Quantity`, into the kernel.
- Spending, at campaign initialization (D-25): prices, purchases, the COC2
  era and the availability it decides, a negative balance; and where the
  campaign concept lives.
- The demi-elfe upbringing as a choice, and capacité names per voie and
  rank, each with a decision entry in the co-rules skill.
