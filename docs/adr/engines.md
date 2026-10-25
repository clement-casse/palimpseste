# Engines

`co-contemporain` (COC2) and `co-fantasy` (COF2): the core rules of
character creation, at level 1 for COF2, then the COF2 levels and the COC2
experience gained after it (scope in AGENTS.md). The acceptance tests are the books' worked
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
`Value::FIXED` lists every value but the weapon values and the test
bonuses, in the book order,
and `Value::name()` names each one; COC2's `AttackMode` has a `name()`, and
an `id()` ending the identifier of its attack value. `label(id)` names every identifier the engine emits as a
violation subject or a breakdown source (content, choices, values,
capacités), and `term_label(rule)` the term a formula sets itself when it
names one (the level, the levels gained, the age bracket). *Why:* the front end offers the
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
capacité name is stored. That identifier is the one key a capacité is
named by everywhere: `Voie::ability(rank)` builds it,
`Voie::from_ability_id` reads it back, and the choices document, the
catalog and the interface use it, so names and effects attach to it
without changing anything that names a capacité.
`Voie::ability_name(rank)` gives a capacité's French name, from the
co-rules catalogue (D-45); `label` names a capacité by its name, then its
voie and rank (`Fin bretteur (Voie des armes de contact, rang 4)`), since
names repeat across voies.
A weapon value builds its identifier from the
weapon (`cof.damage.epee-longue.one-handed`), so `Value::id` is not
`const`.

**ENG-TEST-BONUSES: Test bonuses are values, named by their capacité
(D-50).** A capacité's permanent bonus to the character's own tests is a
`TestBonus`, one per amount the capacité gives, numbered within it, with
the characteristics its catalogue entry names (`characteristics()`, empty
when it names none) and the tests it covers as a scope, a few French
words of our own (`scope()`: `discrétion, déguisement`), never the
catalogue's text. Each bonus is a `Value::TestBonus`
(`coc.test-bonus.<voie>.<rank>.<n>`, `cof.test-bonus...`) with the
capacité as source, adjusted like any value, its notices in the voies
step, and `None` while the capacité is not held or its option not chosen;
`Character::test_bonuses` lists those held, and `Character::bonus_dice`
the capacités giving a dé bonus to tests (`BonusDie`), with the
characteristics named and a scope. Not test bonuses: those used once per session,
scene or combat, given to others or to attacks (a favoured weapon, a
martial art), difficulties changed for others, a characteristic tested in
place of another. In COC2 a bonus is flat or per rank reached, and a "+2
becomes +5" adds 3 to the bonus it upgrades, the upgrading capacité as
source. In COF2 a "rang + 2" is 2 + the rank reached, at most +7, a "+ rank" to
resist spells or a condition (Force d'âme, Résistance légendaire,
Résilient) the rank reached, an opposed test included, a voie
de peuple's bonus +3, kept through the voie du mage like the peuple's
other rank-1 effects (COF.DER.COMPETENCE); Tatouages gives +3 to the
tests of the characteristic chosen, Formation d'élite a dé bonus to them
(ADR ENG-COF-OPTIONS), and Vêtements sacrés' dé bonus holds without
armour, so `InPlay::bonus_dice` follows the equipment. The +15 cap and
the one bonus per source are not checked: they need the fields. *Why
values:* breakdowns and adjustments come with them, as for every computed
value.

**ENG-SPECIALITIES: A capacité's speciality is a free text choice.** Some
capacités ask the player to choose what they apply to: a discipline, an
approach, a corporation, a milieu, a vehicle (COC2, 42 capacités), the
origin of Diversité (COF2). `Voie::speciality(rank)` names what a
capacité asks for, in French; `Character::with_speciality(voie, rank,
text)` writes a `Speciality` (ADR KER-SPECIALITY), `without_speciality`
gives it up. A capacité held that asks for one without it is
`Incomplete` (COC.CNT.VOIES; COF.CNT.VOIE, ADR ENG-COF-OPTIONS) with the
capacité as subject and what it asks for as a `speciality` parameter,
placed where the capacité was acquired. A speciality written for a
capacité not held, or asking for none, is kept and checked by nothing. `TestBonus::on_speciality` and
`BonusDie::on_speciality` say which bonuses apply to it, so the sheet
names them by the player's words. *Why free text:* the lists are open
(any discipline, any vehicle), and no rule reads them.

**ENG-SITUATIONAL: Situational modifiers are listed, never added.** A
capacité's modifier that holds only in a situation (against a larger
creature, outdoors, mounted, with two weapons) is a `Situational`: its
`Situation`, a closed enum named in our own French words (`name()`:
`contre plus grand que lui`), the capacité giving it, and an amount or,
for a weapon's DM, dice (a COF2 dé évolutif resolved at the level).
`Character::situational(target)` lists those of the capacités held for a
`Target`, and `InPlay::situational` with the equipment in play. They are
not values: no total includes them and they take no adjustment; the
Game Master adjusts the total itself. COF2 reads its own table in
`situation.rs` (DEF, Init, every attack, contact or every DM, RD), the
amounts in the shapes of ADR ENG-COF-CAPACITES (D-52, D-53, D-54). COC2's
table gives DEF against a kind of attack or in a moving vehicle, RD
parrying unarmed, and notes without an amount: a manoeuvre or automatic
fire at −2 instead of −5 on its attack, FOR or INT tested in place of
another characteristic, listed on that characteristic. *Why
not toggles:* a situation changes from one attack to the next; the sheet
shows the modifier, the player applies it.

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
added to the dice), `Value::CriticalRange`, in COC2 `Value::CriticalBonus`,
and in COF2 `Value::WeaponAttack` have a breakdown and accept adjustments.
COF2's attack with a weapon in a use starts from the attack value of the
use, its adjustments included, puts AGI (capped by the armour) in place of
a lower FOR when Précision or Attaque en finesse lets a light weapon hit
with it (the vivelame only when mastered, D-46), and adds the capacités'
effects on that weapon's attack (Haches et marteaux). The dice are
`Character::damage_dice` (and `InPlay::damage_dice`): the table's, which
COC2's unarmed rank 2 and Canonnier change, and COF2's Bâton de mage (the
level's dé évolutif for the bâton and the bâton ferré, INT in place of
FOR in their DM, D-47) and Canon double (twice the dice of a powder
weapon, D-48); COF2's `Weapon::dice(use)` stays the table.

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
weapons. The rank-4 capacités of the combat voies are computed: Fin
bretteur, Sniper and Disciple (critical on 18-20, a characteristic added to
the DM and the bonus critique), and Canonnier (1d6 more with heavy weapons,
the bonus critique following the dice as D-30 has it for the unarmed rank
2, D-41). Médecin (médecine rank 2) puts INT in place of FOR in the
contact DM bonus, and of AGI in the DM of Fin bretteur and Disciple,
whenever INT is higher; the bonus critique keeps AGI (D-51).

**ENG-COC-PROTECTIONS: RD and malus, never DEF.** Protections add RD, and
their malus to Init, ATC and AGI tests, never to AGI or DEF (D-04, D-05).
No voie gives a protection (D-30).

**ENG-COC-MONEY: No money at creation.** The book gives none: the social
class describes a lifestyle, not an amount.

**ENG-COC-EXPERIENCE: XP spendings are an ordered history.** The character
stores the XP earned (`Experience`, set by `with_experience`, 1 per
session by default, COC.PRG.XP-GAIN) and each `XpSpending` in order
(`with_xp_spending` appends one, `without_xp_spending` gives up the last).
A spending buys ranks and changes traits: `with_trait` sets a class
trait's points, `with_trait_given_up` gives a trait up. The current ranks
join the creation ranks and every spending's; the current traits apply
each spending's changes to the creation traits. `Value::Experience` is the
XP left: the XP earned (`coc.experience-earned`), less n per rank n bought
(the capacité as source), twice the trait value of each social class
climbed, the class left below the middle class and the class reached
above it (2 x max(|before|, |after|) per class, D-40, the social class as
source), and twice Toxicomane's points to cure it (COC.PRG.COST,
COC.PRG.XP-USE). Spending more than earned is a deviation of COC.PRG.COST
on `Value::Experience`; a rank bought again, one of COC.PRG.COST on the
capacité. A second rank of a voie in one spending, or a rank bought before
the rank below, is a deviation of COC.PRG.ONE-RANK (D-09) with the
capacité as subject; beyond six voies once a spending opens one, of
COC.PRG.LIMITS; any other trait change (a class going down, a trait taken
or set), of COC.PRG.XP-USE with the trait as subject, and it costs
nothing. The Game Master's agreement (COC.PRG.GM-VALIDATION) is not
checkable. The creation budgets and limits (capacity points, maximum voies
and rank, the domaine's voies, the starting equipment, the trait balance
and class, the characteristic limits) read the creation choices only, a
voie opened with XP granting no checked starting equipment (D-44);
`capacity_points_spent`, `voie_count`, `advantages` and `disadvantages`
count them, while `social_class` reads the current traits.
`Character::spending_offer(number)` gives what a spending offers within
the rules (`SpendingOffer`): the XP left, the ranks it may buy (the next
rank of each voie it does not advance yet, rank 1 of a voie not held
while fewer than six are, each within the XP left, `xp_cost`), the trait
change raising the social class one class with its cost, and the cost of
curing Toxicomane.

**ENG-COC-PLACEMENT: A spending's findings belong to it.**
`Character::placement(&violation)` gives a creation `Step`, or a
`SpendingStep` (Rangs, Traits, Caractéristiques) of a spending numbered
from 1. The experience findings carry their spending as a `spending`
parameter: COC.PRG.COST on a capacité, ONE-RANK and LIMITS go to its
ranks, XP-USE to its traits; XP spent beyond the XP earned goes to the
last spending's ranks. A rank 5's characteristic (COC.PRG.CARAC) and a
missing speciality (COC.CNT.VOIES) go where the capacité was bought
(`bought_in`, in its characteristics and ranks steps), and a voie's structure to the
last spending buying a rank of it, unless the creation ranks alone raise
it. Everything else stays in its creation step.

**ENG-COC-RANK-FIVE: Rank 5 raises a characteristic (D-20).** Each voie's
rank 5 raises one characteristic: `Voie::rank_five_raises` gives the fixed
ones (two for the corporations), `Voie::rank_five_choice` the two or three
to choose from, stored by `with_raised_characteristic`. A rank 5 held
without its choice is `Incomplete` of COC.PRG.CARAC, a characteristic not
offered a deviation of it with the capacité as subject, and raised anyway.
The raise counts at creation too, a rank 5 there being a deviation of the
maximum rank.

**ENG-COC-SEQUELS: Sequels acquired in play are apart from the trait.**
`with_acquired_sequel` records a `Sequel` acquired in play (a critical hit
at step 20, COC.CNT.SCALES), never counted in the trait balance and named
by `Sequel::acquired_id`. Diminué (corps) lowers the current CON by 2 and
shortens the échelle de santé by 4 steps, Diminué (esprit) lowers the
current INT by 2; their maximum of +3 is not rechecked, and any other
sequel changes nothing (D-21, D-42).

**ENG-COC-STARTING: The creation limits read the starting values.**
`Character::characteristic` is the current value: the starting value,
with the ranks bought with XP and the sequels acquired in play. The
characteristic limits (CARAC-PHYSICAL, CARAC-LIMITS) read the starting
value, from the creation choices only, so growth after creation never
breaks them (D-43).

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
"Niveau"; PV take 2 family PV and CON at level 1, then (level - 1) x
(family PV + CON) as one term, `Source::Rule(COF.PRG.PV)`, which
`term_label` names "Niveaux gagnés": the breakdown shows what the levels
add in total, and a change of CON changes the PV of every level already
gained (COF.PRG.PV). A rank chosen
at creation above 1 is a deviation of COF.PRG.LEVEL-REQ with the voie as
subject, except the mage family's one rank 2 in a voie of its profil or
the voie du mage (COF.CRE.MAGE-EXTRA), never in another profil's voie,
waived or not.

**ENG-COF-PROGRESSION: Level-ups are an ordered history.** A `LevelUp`
holds the ranks acquired at one level, the ranks forgotten at it
(`with_forgotten`, the changement d'orientation) and the exchange of its
orphan point (`with_orphan`); `Character` keeps its level-ups in order
(`with_level_up` appends one, `without_level_up` gives up the last), the
n-th reaching level n + 1. The current ranks are the creation ranks, then
each level-up in order, forgetting before acquiring; they feed the derived
values and the voie structure (CO.VOIE.*); the creation rules (COF.CRE.VOIES, MAGE-EXTRA,
VOIE-DU-MAGE, the starting equipment) read the creation ranks only. Each
level-up is checked where it happened: its 2 points, plus those the ranks
forgotten refund, spent at once at 1 point up to rank 2 and 2 above, the
orphan point exchanged counting as one (COF.PRG.POINTS: fewer is
`Incomplete`, more a deviation, the level-up `cof.level.<n>` as subject), each rank
against the level it was acquired at (COF.PRG.LEVEL-REQ, the capacité as
subject), a rank bought again (COF.PRG.COST) or before the rank below
(COF.PRG.ORDER, only when that rank comes at a later level: one never
acquired is CO.VOIE.ORDER's). A voie a level-up opens is a voie of the
main profil, or the voie de peuple slot's options: another profil's is a
deviation of COF.PRG.HYBRID, another voie de peuple or the voie du mage
outside the mage family one of COF.PRG.NEW-VOIES. Beyond six voies, or a
second voie in the voie de peuple slot opened by a level-up, is a
deviation of COF.PRG.MAX-VOIES. The changement d'orientation forgets one
capacité per level, two with INT +2 or more (the current INT), each held,
from the top rank of its voie down, never the free rank 1 of the voie de
peuple slot nor a rank-1 capacité of the main profil chosen at creation:
any other is a deviation of COF.PRG.REORIENT, the capacité as subject and
the reason as parameter (the count beyond the limit takes the level-up;
D-36). The
orphan point is exchanged (`OrphanExchange`: 1 PC, 1 DR, 2 PV or 2 PM, the
level-up as source) once no rank 1 or 2 is left in the main profil's voies
or the voie de peuple slot, a voie still to open counting while fewer than
six are held (D-37); earlier, it is a deviation of COF.PRG.ORPHAN. The 2 PM add
to nothing without a spell. `Character::level_offer(level)` gives what a
level-up offers within the rules (`LevelOffer`): its points available
and spent; the capacités it may acquire, the next rank of each voie held,
rank 1 of a main-profil voie not opened while fewer than six are held,
rank 1 of an empty voie de peuple slot, each at its required level
(`required_level`) and within the points left (`rank_cost`); the top
ranks it may forget, the protected ones with their reason, and how many;
and whether the orphan point may be exchanged. The interface offers
these first, anything else as an out-of-rules choice (ADR FE-OVERRIDES). *Why a history, not the current ranks and a level:* the
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
Tour de force to the chemise de mailles (D-49), Briseur d'os to the cotte
de mailles, Armure lourde and Vêtements sacrés,
when their option is chosen, to the armure de plaques and the cotte de
mailles.

**ENG-COF-CAPACITES: Capacité effects are per-value tables (D-22,
D-13).** Every capacité whose catalogue entry states an effect on a value
contributes, at any rank, as `Source::Ability`. Each value reads its own
table in `ability.rs` (PV, DR, PC, PM, Init, DEF, RD, the attack, DM and
critical range of the weapons, mastery, armour limits, characteristic raises) or
`test_bonus.rs` (test bonuses, dé bonus), each row a capacité with an
`Amount` (flat, by the rank reached, per rank, rank plus n, a
characteristic, a characteristic up to the rank, per voie of a profil
holding a rank, a competence bonus), the equipment or level it needs and
the option it depends on. The rank used is the highest held in the voie (CO.VOIE.RANK-SCALING);
a "+n at rank k" in the catalogue is the total reached, a "+1 at rank k"
after a first amount is added to it (D-33). Equipment conditions read the
equipment given, so `InPlay` follows them. The critical range never starts
below 16, a `Source::Rule(COF.DER.CRIT-RANGE)` making up the difference.
`Value::DamageReduction` is `None` while no capacité gives RD. Through the
voie du mage, the peuple's rank-1 effect is kept, with the peuple voie's
rank-1 ability as source; a demi-elfe's upbringing is not recorded, so its
kept effect is left to an adjustment. Effects that depend on
circumstances (outdoors, against large creatures, a companion in sight,
two weapons in hand) are listed apart, never added (ADR
ENG-SITUATIONAL). Not computed: capacités whose content the co-rules skill lacks (the weapon categories of Armes de prédilection,
Science du critique, Spécialisation, D-35; a bone armour for Armure d'os),
unclear ones (Pacte démoniaque's VOL to DR, D-39), unarmed attacks, and
capacités calling other capacités. *Why tables:* over a hundred effects
share a few shapes; a general effect engine stays excluded.

**ENG-COF-OPTIONS: Capacité options are stored choices.** Some capacités
offer an option, an `AbilityOption`: a characteristic (raised by
Supériorité elfique of the elfe haut, Combattant héroïque and the "lowest
characteristic" capacités; used by Peau de pierre, Charisme héroïque,
Endurer and Grosse tête; whose tests Tatouages and Formation d'élite
favour, D-50), the exchange of PC for DR (Éclaireur), a heavier
armour (Armure lourde, Vêtements sacrés) or the thrown weapons variant
(Archer émérite). `with_ability_option(voie, rank, option)` stores it,
keyed by the capacité; `ability_options_offered` lists the options a
capacité offers the character, the lowest characteristics read on the
starting values, ties included (D-34). A capacité with no default (the raises)
held without an option is `Incomplete` of COF.CNT.VOIE; one with a default
keeps it; an option the capacité does not offer is a `standard` deviation
of COF.CNT.VOIE with the capacité as subject, and a characteristic raised
anyway. Grosse tête puts INT in place of CON in the level-1 term of the PV
only (D-38). `AbilityOption` has `ALL`, `id()` (`cof.option.int`), `from_id()`
and `name()` (ADR ENG-CATALOGUE); `ability_option_required` says whether
a capacité has no effect until one is chosen.

**ENG-COF-PLACEMENT: A level-up's findings belong to it.**
`Character::placement(&violation)` gives where a violation is settled: a
creation `Step`, or a `LevelStep` (Capacités, Options, Changement
d'orientation, Point orphelin) of the level-up that reached a `Level`.
The progression rules carry their level, as subject (`cof.level.<n>`) or
as a `level` parameter: COF.PRG.POINTS, COST, LEVEL-REQ, ORDER,
NEW-VOIES, HYBRID and MAX-VOIES go to its capacités, REORIENT to its
orientation, ORPHAN to its orphan point; COF.PRG.LEVEL-REQ raised by the
creation ranks stays in the voies step. A capacité's option (COF.CNT.VOIE)
goes where the capacité was acquired (`acquired_at`), and a voie's
structure (CO.VOIE.*) to the last level-up changing the voie, unless the
creation ranks alone raise it. Everything else stays in its creation
step (`Step::of`). *Why:* the editor shows a level-up's findings in that
level's form, and the creation steps count only their own.

**ENG-COF-PM: Every spell counts (D-23).** `Voie::is_spell` follows the
spell markers of the co-rules catalogue for every voie and rank. PM is
`None` without any spell; a negative total is a `standard` deviation of
COF.DER.PM (D-11).

**ENG-COF-ACTIONS: Each capacité's action and spell cost.**
`Voie::actions(rank)` gives the actions a capacité takes to use, as the
co-rules catalogue marks them (COF.CNT.VOIE): an `Action` (`Limited`,
`Attack`, `Movement`, `Free`, with `name()` and the book's letter
`abbreviation()`), two when the catalogue allows either, none when
passive. `Voie::is_spell` is public, and `Voie::spell_cost(rank)` is the
PM a spell costs, its rank. Armour surcharges and capacités adding PM to a
spell (Tempête de mana) are not computed. *Why letters only:* the action
type is structure; what the capacité does stays out of the repository.

**ENG-COF-FAMILY: No family, no family values.** PV, DR, PC and
`recovery_die` are `None` without a profil, rather than computed without
their family term.

**ENG-COF-WEAPONS: Weapons by use.** Each of the 33 weapons has its uses
(contact in one or two hands, ranged, thrown). The damage adds FOR at
contact, except with the stylet and a mage's staff (INT, D-47). `Profil::masters` gives the profil's
mastery and `Character::masters` adds the capacités', for display (D-28): an unmastered weapon is not a violation. The arc long
needs FOR +1 (COF.EQP.RANGED) and Petite taille forbids large weapons
(D-29), both `standard` deviations.

**ENG-COF-PURSE: The purse is rolled.** `Money` counts pièces de cuivre
and displays with the fewest coins (COF.EQP.CURRENCY). The purse is the
2d6 pa the player rolled (`with_purse`, `purse_dice`): missing is
`Incomplete`, outside 2-12 whole pa a `standard` deviation.

## Open

- Lasting effects the co-rules skill lacks: COF2 mutilations, COC2 psychic
  sequels and traumatismes, aging after creation. A COF2 resurrection's CON
  loss is a Game Master's adjustment until it has a rule entry.
- Spending, at campaign initialization (D-25): prices, purchases, the COC2
  era and the availability it decides, a negative balance; and where the
  campaign concept lives.
- The demi-elfe upbringing as a choice, with a decision entry in the
  co-rules skill.
- Hybrids after level 1 and their PV per level, prestige voies, and the
  capacités calling other capacités (COF.PRG.CALL), all out of scope.
- The weapon categories of Armes de prédilection, from the co-rules skill
  (D-35).
