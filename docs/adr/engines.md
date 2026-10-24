# Engines

`co-contemporain` (COC2) and `co-fantasy` (COF2): the core rules of
character creation, at level 1 for COF2, and the COF2 levels gained after
it (scope in AGENTS.md). The acceptance tests are the books' worked
characters: Amelia Steele (COC2), Lhagva and Ionas (COF2, at levels 1
and 2). Rule identifiers (`COF.CRE.CARAC-SERIES`) and
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
peuple slot accepts) are `Character` methods. Values are named alike:
`Value::FIXED` lists every value but the weapon values, in the book order,
and `Value::name()` names each one; COC2's `AttackMode` has a `name()`, and
an `id()` ending the identifier of its attack value. `label(id)` names every identifier the engine emits as a
violation subject or a breakdown source (content, choices, values,
capacités), and `term_label(rule)` the term a formula sets itself when it
names one (the level, the age bracket). *Why:* the front end offers the
options the rules allow, and names what the engine reports, without
restating any identifier.

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
(CO.VOIE.ORDER). Each engine keeps the set, keyed by its own `Voie`; the
CO.VOIE.STRUCTURE and CO.VOIE.ORDER checks are the kernel's
(ADR KER-VOIE). *Why not a generic set in the kernel:* a few short methods
per engine do not pay for a type parameter.

## Equipment and money

**ENG-ITEMS: Only items a rule reads are content (D-26).** Weapons,
armours and shields (COF2) and protections (COC2) are enums with their
tables. Any other item is free text (`Item::Other(ItemName)`), never
checked; an item becomes an enum when a rule reading it lands.
`with_item(item, quantity)` records what is carried, and a quantity of 0
gives it up. Armour and shield (COF2) and protections (COC2) are worn
slots, and count as possessed. Each engine keeps its `Item`, naming its
own `Weapon`; `ItemName` and `Quantity` are the kernel's (ADR KER-ITEMS).

**ENG-IN-PLAY: A character in play wears another equipment.** `Equipment`
groups what a character wears and carries (COF2 armour, shield and items;
COC2 protections and items); `Character` holds the one chosen at creation,
which `with_equipment` sets whole. `Character::in_play(equipment)` gives an
`InPlay`: the creation choices, adjustments and waivers with `equipment` in
place of the creation one. Its values follow that equipment, every
contribution attributed to its item as at creation. COF2's
`InPlay::report` checks only the rules that hold whatever was chosen at
creation: an armour or shield beyond the profil's limit
(COF.DER.ARMOUR-LIMIT, D-24), the arc long's FOR (COF.EQP.RANGED) and
Petite taille (COF.CNT.PEUPLES, D-29); the starting equipment and the
purse are creation rules, never checked in play. COC2 has no rule about
equipment in play, so its `InPlay` has no report. Nothing is stored: the
engine computes from the equipment it is given. *Why not waivers or
adjustments:* an equipment's effects are formulas (the AGI cap, several
values at once, new damage lines), not amounts, and a waiver accepts a
deviation without changing a value.

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
assigned series values, CARAC-BOUNDS and CARAC-MAGIC the adjusted starting
value. `Character::characteristic` is the starting value raised by the
capacités held (ENG-COF-CAPACITES): a raise is no creation choice, so it
never breaks the creation bounds.

**ENG-COF-LEVEL: The level counts the level-ups.** `Character::level()`
is 1 plus the number of level-ups, never stored. Attack values take
min(level, 10) as `Source::Rule(COF.DER.ATTACK)`, which `term_label` names
"Niveau"; PV take (level + 1) family PV and level x CON, so a change of CON
changes the PV of every level already gained (COF.PRG.PV). A rank chosen
at creation above 1 is a deviation of COF.PRG.LEVEL-REQ with the voie as
subject, except the mage family's one rank 2 in a voie of its profil or
the voie du mage (COF.CRE.MAGE-EXTRA), never in another profil's voie,
waived or not.

**ENG-COF-PROGRESSION: Level-ups are an ordered history.** A `LevelUp`
holds the ranks acquired at one level; `Character` keeps its level-ups in
order (`with_level_up` appends one, `without_level_up` gives up the last),
the n-th reaching level n + 1. The current ranks join the creation ranks
and those of every level-up, and feed the derived values and the voie
structure (CO.VOIE.*); the creation rules (COF.CRE.VOIES, MAGE-EXTRA,
VOIE-DU-MAGE, the starting equipment) read the creation ranks only. Each
level-up is checked where it happened: its 2 points spent at once at 1
point up to rank 2 and 2 above (COF.PRG.POINTS: fewer is `Incomplete`,
more a deviation, the level-up `cof.level.<n>` as subject), each rank
against the level it was acquired at (COF.PRG.LEVEL-REQ, the capacité as
subject), a rank bought again (COF.PRG.COST) or before the rank below
(COF.PRG.ORDER, only when that rank comes at a later level: one never
acquired is CO.VOIE.ORDER's). A voie a level-up opens is a voie of the
main profil, or the voie de peuple slot's options: another profil's is a
deviation of COF.PRG.HYBRID, another voie de peuple or the voie du mage
outside the mage family one of COF.PRG.NEW-VOIES. Beyond six voies, or a
second voie in the voie de peuple slot opened by a level-up, is a
deviation of COF.PRG.MAX-VOIES. The progression findings sit in
`Step::Voies`. *Why a history, not the current ranks and a level:* the
points of each level, the level a rank was acquired at, and the changement
d'orientation (the rank-1 capacités of level 1 are never forgotten) are
read per level.

**ENG-COF-VOIES: Slots count what is there.** The chosen voies are every
voie chosen at creation outside the voie de peuple slot, which holds the voies de peuple and
the voie du mage. Another profil's voie (a level-1 hybrid) or another
peuple's is a deviation with the voie as subject, so once waived the
counts do not stay incomplete.

**ENG-COF-ARMOUR: Armour and shield are choices.** They add their DEF;
the armour caps AGI in DEF and in the ranged attack (D-06). Beyond the
profil's limit is a `standard` deviation of COF.DER.ARMOUR-LIMIT (D-24);
the chevalier's Noblesse rank 2 raises the limit to the plaque complète,
Briseur d'os to the cotte de mailles, Armure lourde and Vêtements sacrés,
when their option is chosen, to the armure de plaques and the cotte de
mailles.

**ENG-COF-CAPACITES: Capacité effects are per-value tables (D-22,
D-13).** Every capacité whose catalogue entry states an effect on a value
contributes, at any rank, as `Source::Ability`. Each value reads its own
table in `ability.rs` (PV, DR, PC, PM, Init, DEF, RD, the DM and critical
range of the weapons, mastery, armour limits, characteristic raises), each
row a capacité with an `Amount` (flat, by the rank reached, per rank, rank
plus n, a characteristic, a characteristic up to the rank, per rank 5 in a
profil's voies), the equipment or level it needs and the option it depends
on. The rank used is the highest held in the voie (CO.VOIE.RANK-SCALING);
a "+n at rank k" in the catalogue is the total reached, a "+1 at rank k"
after a first amount is added to it. Equipment conditions read the
equipment given, so `InPlay` follows them. The critical range never starts
below 16, a `Source::Rule(COF.DER.CRIT-RANGE)` making up the difference.
`Value::DamageReduction` is `None` while no capacité gives RD. Through the
voie du mage, the peuple's rank-1 effect is kept, with the peuple voie's
rank-1 ability as source; a demi-elfe's upbringing is not recorded, so its
kept effect is left to an adjustment. Not computed: effects that depend on
circumstances (outdoors, against large creatures, a companion in sight,
two weapons in hand), effects on one weapon's attack (Haches et marteaux's
attack, Précision, Attaque en finesse), capacités whose content the
co-rules skill lacks (the weapon categories of Armes de prédilection,
Science du critique, Spécialisation; a bone armour for Armure d'os),
unclear ones (Pacte démoniaque's VOL to DR), unarmed attacks, capacités
calling other capacités, and competence bonuses. *Why tables:* about sixty
effects share a few shapes; a general effect engine stays excluded.

**ENG-COF-OPTIONS: Capacité options are stored choices.** Some capacités
offer an option, an `AbilityOption`: a characteristic (raised by
Supériorité elfique of the elfe haut, Combattant héroïque and the "lowest
characteristic" capacités; used by Peau de pierre, Charisme héroïque,
Endurer and Grosse tête), the exchange of PC for DR (Éclaireur), a heavier
armour (Armure lourde, Vêtements sacrés) or the thrown weapons variant
(Archer émérite). `with_ability_option(voie, rank, option)` stores it,
keyed by the capacité; `ability_options_offered` lists the options a
capacité offers the character, the lowest characteristics read on the
starting values, ties included. A capacité with no default (the raises)
held without an option is `Incomplete` of COF.CNT.VOIE; one with a default
keeps it; an option the capacité does not offer is a `standard` deviation
of COF.CNT.VOIE with the capacité as subject, and a characteristic raised
anyway. Grosse tête puts INT in place of CON in the level-1 term of the PV
only.

**ENG-COF-PM: Every spell counts (D-23).** `Voie::is_spell` follows the
spell markers of the co-rules catalogue for every voie and rank. PM is
`None` without any spell; a negative total is a `standard` deviation of
COF.DER.PM (D-11).

**ENG-COF-FAMILY: No family, no family values.** PV, DR, PC and
`recovery_die` are `None` without a profil, rather than computed without
their family term.

**ENG-COF-WEAPONS: Weapons by use.** Each of the 33 weapons has its uses
(contact in one or two hands, ranged, thrown). The damage adds FOR at
contact, except with the stylet. `Profil::masters` gives the profil's
mastery and `Character::masters` adds the capacités', for display (D-28): an unmastered weapon is not a violation. The arc long
needs FOR +1 (COF.EQP.RANGED) and Petite taille forbids large weapons
(D-29), both `standard` deviations.

**ENG-COF-PURSE: The purse is rolled.** `Money` counts pièces de cuivre
and displays with the fewest coins (COF.EQP.CURRENCY). The purse is the
2d6 pa the player rolled (`with_purse`, `purse_dice`): missing is
`Incomplete`, outside 2-12 whole pa a `standard` deviation.

## Open

- Spending, at campaign initialization (D-25): prices, purchases, the COC2
  era and the availability it decides, a negative balance; and where the
  campaign concept lives.
- The demi-elfe upbringing as a choice, and capacité names per voie and
  rank, each with a decision entry in the co-rules skill.
- The rest of COF2 progression: the orphan point (COF.PRG.ORPHAN), the
  changement d'orientation (COF.PRG.REORIENT), hybrids after level 1 and
  their PV per level; then COC2 experience (COC.PRG.*, D-09, D-20).
- Per-weapon attack values, for the capacités changing one weapon's attack;
  the weapon categories of Armes de prédilection, from the co-rules skill.
