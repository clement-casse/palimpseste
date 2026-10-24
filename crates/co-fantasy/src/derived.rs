//! Derived values (COF.DER.*, COF.PRG.PV), with the documented effects of
//! the capacités (D-22) read from their tables in [`ability`].

use co_kernel::{
    Characteristic, Characteristics, DerivedValue, Rank, Severity, Source, Subject, Violation,
};

use crate::{
    AttackMode, Character, Family, Value, Voie,
    ability::{self, AbilityOption},
    rules::{ATTACK, DEF, DERIVED_MECHANIC, DR, INIT, PC, PM},
};

/// Whether the option `characteristic` is chosen for the capacité at `rank`
/// in `voie`, the capacité being held.
fn uses(character: &Character, voie: Voie, rank: u8, characteristic: Characteristic) -> bool {
    ability::holds(character, voie, Rank::new(rank))
        && ability::chose(
            character,
            voie,
            rank,
            AbilityOption::Characteristic(characteristic),
        )
}

/// Adds the orphan points exchanged for `target` (COF.PRG.ORPHAN), each
/// with the level-up that exchanged it as source.
fn with_orphans(value: DerivedValue, character: &Character, target: Value) -> DerivedValue {
    character
        .orphan_exchanges()
        .map(|(level, exchange)| (level, exchange.gain()))
        .filter(|(_, (gained, _))| *gained == target)
        .fold(value, |value, (level, (_, amount))| {
            value.with(Source::Value(level.id()), amount)
        })
}

/// `c` as usable in the armour worn: capped by its AGI max (D-06).
fn capped(character: &Character, values: &Characteristics, c: Characteristic) -> i32 {
    let value = values[c].get();
    character
        .armour()
        .map_or(value, |armour| value.min(armour.agi_max()))
}

/// PV = 2 x family PV + CON at level 1 (COF.DER.PV-L1), then family PV +
/// CON per level-up (COF.PRG.PV), with the capacités' effects. The current
/// CON counts for every level, so a change of CON changes the PV of the
/// levels already gained. Grosse tête (golem r1) may put INT in place of CON
/// at level 1.
pub(crate) fn vigour_points(
    character: &Character,
    family: Family,
    values: &Characteristics,
) -> DerivedValue {
    let level = i32::from(character.level().get());
    let value = DerivedValue::new().with(
        Source::Value(family.id()),
        (level + 1) * family.vigour_per_level(),
    );
    let con = values[Characteristic::Con].get();
    let value = if uses(character, Voie::Golem, 1, Characteristic::Int) {
        value
            .with(
                Source::Characteristic(Characteristic::Int),
                values[Characteristic::Int].get(),
            )
            .with(
                Source::Characteristic(Characteristic::Con),
                (level - 1) * con,
            )
    } else {
        value.with(Source::Characteristic(Characteristic::Con), level * con)
    };
    let value = ability::apply(value, character, values, ability::VIGOUR);
    with_orphans(value, character, Value::VigourPoints)
}

/// DR = 2 + CON, +1 for the mystiques (COF.DER.DR), with the capacités'
/// effects.
pub(crate) fn recovery_dice(
    character: &Character,
    family: Family,
    values: &Characteristics,
) -> DerivedValue {
    let mut value = DerivedValue::new()
        .with(Source::Rule(DR), 2)
        .with_characteristic(values, Characteristic::Con);
    if family == Family::Mystiques {
        value = value.with(Source::Value(family.id()), 1);
    }
    let value = ability::apply(value, character, values, ability::RECOVERY);
    with_orphans(value, character, Value::RecoveryDice)
}

/// PC = 2 + CHA, +1 for the aventuriers (COF.DER.PC), with the capacités'
/// effects.
pub(crate) fn luck_points(
    character: &Character,
    family: Family,
    values: &Characteristics,
) -> DerivedValue {
    let mut value = DerivedValue::new()
        .with(Source::Rule(PC), 2)
        .with_characteristic(values, Characteristic::Cha);
    if family == Family::Aventuriers {
        value = value.with(Source::Value(family.id()), 1);
    }
    let value = ability::apply(value, character, values, ability::LUCK);
    with_orphans(value, character, Value::LuckPoints)
}

/// PM = VOL + 1 per spell known (COF.DER.PM), with the capacités' effects;
/// `None` without any spell. Charisme héroïque (séduction r4) may put CHA in
/// place of VOL.
pub(crate) fn mana_points(character: &Character, values: &Characteristics) -> Option<DerivedValue> {
    let spells: Vec<_> = character
        .voies()
        .into_iter()
        .flat_map(|(voie, ranks)| {
            ranks
                .into_iter()
                .filter(move |rank| voie.is_spell(*rank))
                .map(move |rank| voie.ability(rank))
        })
        .collect();
    if spells.is_empty() {
        return None;
    }
    let characteristic = if uses(character, Voie::Seduction, 4, Characteristic::Cha) {
        Characteristic::Cha
    } else {
        Characteristic::Vol
    };
    let value = spells.into_iter().fold(
        DerivedValue::new().with_characteristic(values, characteristic),
        |value, spell| value.with(Source::Ability(spell), 1),
    );
    let value = ability::apply(value, character, values, ability::MANA);
    Some(with_orphans(value, character, Value::ManaPoints))
}

/// Init = 10 + PER (COF.DER.INIT), with the capacités' effects.
pub(crate) fn initiative(character: &Character, values: &Characteristics) -> DerivedValue {
    let value = DerivedValue::new()
        .with(Source::Rule(INIT), 10)
        .with_characteristic(values, Characteristic::Per);
    ability::apply(value, character, values, ability::INITIATIVE)
}

/// DEF = 10 + AGI (capped by the armour) + armour + shield (COF.DER.DEF),
/// with the capacités' effects. Peau de pierre (pagne r2) may put CON, capped
/// alike, in place of AGI.
pub(crate) fn defence(character: &Character, values: &Characteristics) -> DerivedValue {
    let characteristic = if uses(character, Voie::Pagne, 2, Characteristic::Con) {
        Characteristic::Con
    } else {
        Characteristic::Agi
    };
    let mut value = DerivedValue::new().with(Source::Rule(DEF), 10).with(
        Source::Characteristic(characteristic),
        capped(character, values, characteristic),
    );
    if let Some(armour) = character.armour() {
        value = value.with(Source::Value(armour.id()), armour.defence());
    }
    if let Some(shield) = character.shield() {
        value = value.with(Source::Value(shield.id()), shield.defence());
    }
    ability::apply(value, character, values, ability::DEFENCE)
}

/// RD from the capacités; `None` when none gives any.
pub(crate) fn damage_reduction(
    character: &Character,
    values: &Characteristics,
) -> Option<DerivedValue> {
    ability::any(character, values, ability::REDUCTION)
        .then(|| ability::apply(DerivedValue::new(), character, values, ability::REDUCTION))
}

/// Attack = level (at most 10) + FOR (contact), AGI capped by the armour
/// (ranged) or VOL (magic) (COF.DER.ATTACK).
pub(crate) fn attack(
    character: &Character,
    values: &Characteristics,
    mode: AttackMode,
) -> DerivedValue {
    let value = DerivedValue::new().with(Source::Rule(ATTACK), character.level().attack_term());
    match mode {
        AttackMode::Contact => value.with_characteristic(values, Characteristic::For),
        AttackMode::Ranged => value.with(
            Source::Characteristic(Characteristic::Agi),
            capped(character, values, Characteristic::Agi),
        ),
        AttackMode::Magic => value.with_characteristic(values, Characteristic::Vol),
    }
}

/// Checks the derived values: PM below 0 (D-11).
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    if let Some(total) = character.value(Value::ManaPoints).map(|v| v.total())
        && total < 0
    {
        violations.push(
            Violation::deviation(
                PM,
                DERIVED_MECHANIC,
                Subject::Value(Value::ManaPoints.id()),
                Severity::Standard,
            )
            .with_parameter("value", total),
        );
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;
    use crate::{Armour, Peuple, Profil, Shield};

    fn total(character: &Character, value: Value) -> Option<i32> {
        character.value(value).map(|v| v.total())
    }

    fn with_values(character: Character, values: [(Characteristic, i32); 7]) -> Character {
        values
            .into_iter()
            .fold(character, |c, (characteristic, value)| {
                c.with_series_value(characteristic, value)
            })
    }

    /// AGI 2, CON 1, FOR 2, PER -1, CHA 0, INT 1, VOL 2 (polyvalent).
    fn values(character: Character) -> Character {
        with_values(
            character.with_series(crate::Series::Polyvalent),
            [
                (Characteristic::Agi, 2),
                (Characteristic::Con, 1),
                (Characteristic::For, 2),
                (Characteristic::Per, -1),
                (Characteristic::Cha, 0),
                (Characteristic::Int, 1),
                (Characteristic::Vol, 2),
            ],
        )
    }

    #[test]
    fn family_values_need_a_profil() {
        let character = values(Character::new());
        for value in [Value::VigourPoints, Value::RecoveryDice, Value::LuckPoints] {
            assert_eq!(total(&character, value), None);
        }
        assert_eq!(total(&character, Value::Initiative), Some(9));
        assert_eq!(total(&character, Value::Defence), Some(12));
        assert_eq!(character.recovery_die(), None);
    }

    #[test]
    fn family_sets_vigour_recovery_and_luck() {
        let rodeur = values(Character::new().with_profil(Profil::Rodeur));
        assert_eq!(total(&rodeur, Value::VigourPoints), Some(9));
        assert_eq!(total(&rodeur, Value::RecoveryDice), Some(3));
        assert_eq!(total(&rodeur, Value::LuckPoints), Some(3));
        let pretre = values(Character::new().with_profil(Profil::Pretre));
        assert_eq!(total(&pretre, Value::RecoveryDice), Some(4));
        assert_eq!(total(&pretre, Value::LuckPoints), Some(2));
        let sorcier = values(Character::new().with_profil(Profil::Sorcier));
        assert_eq!(total(&sorcier, Value::VigourPoints), Some(7));
    }

    #[test]
    fn attacks_add_the_level() {
        let character = values(Character::new());
        let attack = |mode| total(&character, Value::Attack(mode));
        assert_eq!(attack(AttackMode::Contact), Some(3));
        assert_eq!(attack(AttackMode::Ranged), Some(3));
        assert_eq!(attack(AttackMode::Magic), Some(3));
    }

    #[test]
    fn armour_caps_agi_in_defence_and_ranged_attack() {
        let character = values(Character::new())
            .with_armour(Armour::ArmureDePlaques)
            .with_shield(Shield::PetitBouclier);
        // 10 + AGI 2 capped at 2 + 6 + 1.
        assert_eq!(total(&character, Value::Defence), Some(19));
        let heavy = character.with_armour(Armour::PlaqueComplete);
        // 10 + AGI capped at 1 + 7 + 1.
        assert_eq!(total(&heavy, Value::Defence), Some(19));
        assert_eq!(total(&heavy, Value::Attack(AttackMode::Ranged)), Some(2));
    }

    #[test]
    fn capacites_modify_initiative_defence_and_luck() {
        let character = values(Character::new().with_profil(Profil::Barbare))
            .with_rank(Voie::Pourfendeur, 1)
            .with_rank(Voie::Air, 1)
            .with_rank(Voie::Humain, 1)
            .with_rank(Voie::Gnome, 1)
            .with_rank(Voie::Gnome, 2)
            .with_rank(Voie::Halfelin, 1);
        assert_eq!(total(&character, Value::Initiative), Some(9 + 3 + 1));
        assert_eq!(total(&character, Value::Defence), Some(12 + 1 + 1 + 1));
        assert_eq!(total(&character, Value::LuckPoints), Some(2 + 1 + 1));
        let master = (2..=5).fold(character, |c, rank| c.with_rank(Voie::Pourfendeur, rank));
        assert_eq!(total(&master, Value::Defence), Some(12 + 2 + 1 + 1));
    }

    #[test]
    fn voie_du_mage_keeps_the_peuple_first_capacite() {
        let character =
            values(Character::new().with_peuple(Peuple::Halfelin)).with_rank(Voie::Mage, 1);
        assert_eq!(total(&character, Value::Defence), Some(13));
        let character = character.with_peuple(Peuple::DemiElfe);
        assert_eq!(total(&character, Value::Defence), Some(12));
    }

    #[test]
    fn mana_points_count_the_known_spells() {
        let character = values(Character::new());
        assert_eq!(total(&character, Value::ManaPoints), None);
        let caster = character
            .with_rank(Voie::Air, 1)
            .with_rank(Voie::Invocation, 1)
            .with_rank(Voie::Invocation, 2)
            .with_rank(Voie::Mage, 1);
        assert_eq!(total(&caster, Value::ManaPoints), Some(2 + 3));
    }

    #[test]
    fn negative_mana_points_deviate() {
        let character = values(Character::new())
            .with_series_value(Characteristic::Vol, -2)
            .with_rank(Voie::Air, 1);
        assert_eq!(total(&character, Value::ManaPoints), Some(-1));
        let mut violations = Vec::new();
        check(&character, &mut violations);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule(), &PM);
        assert_eq!(
            violations[0].category(),
            Category::Deviation(Severity::Standard)
        );
    }
}
