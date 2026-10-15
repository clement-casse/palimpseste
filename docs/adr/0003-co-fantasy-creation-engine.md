# 0003. co-fantasy: COF2 character creation engine

Date: 2026-10-15.
Status: accepted.

## Context

The kernel (ADR 0001) and the COC2 creation engine (ADR 0002) are done. The
second engine covers *Chroniques Oubliées Fantasy*, second edition (COF2),
core rules only. This delivery covers **character creation at level 1**:
the choices a player makes, the limits that frame them, and the derived
values of the creation steps (PV, DR, PC, PM, Init, DEF, attack values).
The acceptance tests are the book's two worked characters at level 1, Lhagva
and Ionas.

## Decisions

- **One crate, `co-fantasy`, depending on `co-kernel` only.** It mirrors
  `co-contemporain` wherever the rules allow: content as closed enums with
  `match` tables beside the mechanic that reads them, French content names
  without accents, choices only in `Character`, values computed on demand
  with their breakdown, adjustments on every computed value through
  `Target`, one notice per adjustment, distinct rule or subject for each
  finding a waiver may accept.
- **Content read by creation.** The 4 families, 14 profils (family, five
  voies, key characteristics, magic characteristic, armour and shield
  limits), 8 peuples (modifiers, voie de peuple), the 3 value series, the
  78 voies reachable at level 1 (70 profil voies, 7 voies de peuple, voie du
  mage) and the armour and shield table. Prestige voies are not included.
- **Characteristics are a series, then peuple modifiers.** The character
  stores the series, the value assigned to each characteristic (unassigned
  until chosen) and the peuple bonus and malus. Bonus and malus are two
  methods taking a `Characteristic`, so they cannot be swapped. A peuple
  offering a single malus applies it without a choice; a chosen malus
  replaces it and is reported when the peuple does not offer it. The humain
  bonus follows D-15, judged once all seven values are assigned.
- **The advice of COF.CRE.CARAC-SERIES is split.** CARAC-SERIES (standard)
  checks the assignment against the series; CARAC-KEY and CARAC-MAGIC
  (recommended) carry the advice on key and magic characteristics, so a
  waiver for the advice never accepts a wrong assignment. CARAC-KEY reads
  the assigned series values, since the advice is about assigning the
  series; CARAC-MAGIC reads the final value, peuple modifier included.
- **Level is fixed at 1.** It is not stored: attack values take it as
  `Source::Rule(COF.CRE.LEVEL1)`. Ranks above 1 are a deviation of
  COF.PRG.LEVEL-REQ, except the mage family's one rank 2 in a voie of its
  profil or the voie du mage (COF.CRE.MAGE-EXTRA); a voie of another profil
  never holds it, waived or not.
- **Voie slots count what is there.** The chosen voies are every voie
  outside the voie de peuple slot, and the slot holds the voies de peuple
  and the voie du mage. A voie of another profil (a level-1 hybrid) or of
  another peuple is a deviation with the voie as subject, so the GM can
  waive it without the counts staying incomplete.
- **Armour and shield are choices.** They add their DEF; the armour caps AGI
  in DEF and in the ranged attack (D-06). Going beyond the profil's limit is
  a `standard` deviation of COF.DER.ARMOUR-LIMIT (D-24); the chevalier's
  Noblesse rank 2 raises the limit to the plaque complète.
- **Capacité effects: documented rank 1-2 ones only (D-22).** Réflexes
  éclair, Murmure dans le vent, Diversité, Petite taille and Petit pote.
  Through the voie du mage, the peuple's rank-1 effect is kept, with the
  peuple voie's rank-1 ability as source. A demi-elfe's upbringing is not
  recorded, so its kept effect (Diversité for a human upbringing) is left to
  an adjustment. Effects of higher ranks are left to adjustments.
- **PM counts the documented spells (D-23).** The engine's spell table
  knows only the spells the co-rules skill marks. PM is `None` without any spell, and
  a negative total is a `standard` deviation of COF.DER.PM (D-11).
- **Values that need the family are `None` without a profil.** PV, DR and
  PC (and `recovery_die`) are not computed rather than computed without
  their family term.
- **No `Uncomputable` finding.** Every COF2 creation finding is either a
  missing choice (`Incomplete`) or a waivable `Deviation`; tests cover
  those and adjustment notices.
- **Voie checks are duplicated.** CO.VOIE.STRUCTURE and CO.VOIE.ORDER are
  copied from `co-contemporain`, where acquisition stayed until a second
  engine showed what to share. Moving the acquired-ranks set and
  CO.VOIE.ORDER into the kernel is proposed as a follow-up decision, not
  made here.

## Deferred

Progression (levels above 1, capacity points, PV per level, orphan point,
changement d'orientation, six voies maximum); prestige voies; hybrid
profils (PV formula, weapon and armour mastery); weapons, DM and critical
ranges; competence bonuses; languages and talent secondaire; capacités
calling other capacités; armour malus to AGI tests and PM surcharge;
starting equipment and the final touch; play-time state.
