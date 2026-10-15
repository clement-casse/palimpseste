# 0004. Equipment, weapons and money at creation

Date: 2026-10-15.
Status: accepted.

## Context

ADR 0002 and ADR 0003 deferred weapons, protections, inventory and starting
equipment. Character creation must now hold the initial equipment (the
dotation): the profil's in COF2, the voies' in COC2. The rules also give
money: COF2 has a currency and a purse rolled at creation; COC2 has prices
by era but no starting amount. The acceptance tests stay the book's worked
characters: Lhagva and Ionas (COF2), Amelia Steele (COC2).

## Decisions

- **Dice are a kernel concept: a list of dice.** `Die(u8)` is a number of
  faces with no game bound and a single `From<u8>`; `Dice(Vec<Die>)` is the
  dice rolled together (`2d6` is `[d6, d6]`), so mixed dice are
  representable. Dice mean the same thing in both books. They are described,
  never rolled. Flat bonuses ("+1", FOR) are not dice: they stay in the
  breakdown of a `DerivedValue`. `RecoveryDie` (COF2) is left as is.
- **Only items a rule reads are content (D-26).** Weapons (`Weapon`),
  armours and shields (COF2) and protections (COC2) are closed enums with
  `match` tables. Any other item is free text (`Item::Other(ItemName)`) and
  is never checked; items become enums as their rules land. `Item`,
  `ItemName` and `Quantity` are duplicated in both engines, as the voie
  checks were in ADR 0003; moving them to the kernel is a possible
  follow-up, not made here.
- **Carried and worn.** `with_item(item, quantity)` records what the
  character carries; a quantity of 0 gives the item up. Armour and shield
  (COF2) and protections (COC2) stay worn slots, and count as possessed.
- **Starting equipment is checked, never imposed (D-26).** Each line of the
  profil's (COF2) or voies' (COC2) starting equipment covers what it gives:
  an exact weapon, one of a few, a weapon matching a predicate, a worn
  armour or shield. Matching is deterministic: lines in table order, each
  taking the bundle covering the most items, each item line taking the
  first matching items. A rule-backed item left over is a `standard`
  deviation of COF.CRE.EQUIPMENT or COC.CRE.EQUIPMENT with the item as
  subject, which the Game Master waives as a trade. A line left empty is not
  reported: an item may be given up. COF2 checks nothing without a profil.
- **Weapon values are values.** `Value::Damage` (the DM added to the dice),
  `Value::CriticalRange` (both engines) and `Value::CriticalBonus` (COC2)
  get a breakdown and accept adjustments like any value; their identifiers
  are built from the weapon (`cof.damage.epee-longue.one-handed`), so
  `Value::id` is no longer `const`. The dice come from the table:
  `Weapon::dice(use)` in COF2, `Character::damage_dice(weapon)` in COC2,
  where the combat à mains nues rank 2 changes the unarmed dice.
- **COF2 weapons.** 33 weapons, each with its uses (contact in one or two
  hands, ranged or thrown); a weapon of both tables has both. The damage
  adds FOR at contact, except for the stylet. `Profil::masters` gives the
  mastery for display (D-28); carrying an unmastered weapon is not a
  violation. The arc long needs FOR +1 (COF.EQP.RANGED) and Petite taille
  forbids large weapons (D-29), both `standard` deviations.
- **COC2 weapons and protections.** One variant per table row, the unarmed
  attack included (D-12) but never carried. The DM adds the contact DM
  bonus, adjustments included, for unarmed and contact weapons; the bonus
  critique is computed from the dice, `None` for area weapons. The rank-4
  capacités of the armes de contact and armes à distance are computed.
  Protections add RD and their malus to Init, ATC and AGI tests, never to
  AGI or DEF (D-04, D-05). No voie gives a protection (D-30).
- **Money is per engine, as a computed value.** The currencies differ, so
  each engine owns its `Money`, a newtype counted in the smallest coin and
  built from named coins only (`Money::pa(7)`): a conversion from a bare
  integer would hide the coin, an exception to the builder convention.
  The money held is `Value::Money`, a `DerivedValue`, so the Game Master
  adjusts it like any value.
- **COF2 money: the purse (D-27).** `Money` counts pièces de cuivre and
  displays with the fewest coins (COF.EQP.CURRENCY). The purse is the 2d6 pa
  the player rolled (`with_purse`; `purse_dice` gives the dice); a missing
  purse is `Incomplete`, a purse outside 2-12 whole pa a `standard`
  deviation.
- **COC2 has no money at creation.** The book gives none: the social class
  describes a lifestyle, not an amount.

## Deferred

Spending, until campaign initialization, when the character is attached to
a campaign and a Game Master (D-25): purchases and prices (COF2 tables, COC2
prices by era), the COC2 era and the availability it decides, a negative
balance. That delivery needs its own ADR and a decision on where the
campaign concept lives. Also deferred: money in play, selling, vehicles,
ammunition, range, magazines and reach, arme de prédilection, magic items,
weapon effects on Init (two-handed and improvised weapons), COF2 unarmed
attacks, the gear lists of the starting equipment (free text).
