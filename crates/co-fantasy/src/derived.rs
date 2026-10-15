//! Derived values at level 1 (COF.DER.*), with the documented effects of
//! rank 1-2 capacités only (D-22).

use co_kernel::{
    Characteristic, Characteristics, DerivedValue, Rank, Severity, Source, Subject, Violation,
};

use crate::{
    AttackMode, Character, Family, Value, Voie,
    rules::{DEF, DERIVED_MECHANIC, DR, INIT, LEVEL, LEVEL1, PC, PM},
};

/// Adds `amount` from the capacité at `rank` in `voie`, when acquired.
fn with_ability(
    value: DerivedValue,
    character: &Character,
    voie: Voie,
    rank: u8,
    amount: i32,
) -> DerivedValue {
    let rank = Rank::new(rank);
    if character.has_rank(voie, rank) {
        value.with(Source::Ability(voie.ability(rank)), amount)
    } else {
        value
    }
}

/// Adds `amount` from the rank-1 capacité of the voie de peuple `voie`:
/// acquired in that voie, or kept through the voie du mage that replaced it
/// (COF.CRE.VOIE-DU-MAGE).
fn with_peuple_ability(
    value: DerivedValue,
    character: &Character,
    voie: Voie,
    amount: i32,
) -> DerivedValue {
    let first_rank = Rank::new(1);
    let kept = character.peuple().and_then(crate::Peuple::voie) == Some(voie)
        && character.has_rank(Voie::Mage, first_rank);
    if character.has_rank(voie, first_rank) || kept {
        value.with(Source::Ability(voie.ability(first_rank)), amount)
    } else {
        value
    }
}

/// AGI as usable in the armour worn: capped by its AGI max (D-06).
fn effective_agi(character: &Character, values: &Characteristics) -> i32 {
    let agi = values[Characteristic::Agi].get();
    character
        .armour()
        .map_or(agi, |armour| agi.min(armour.agi_max()))
}

/// PV = 2 x family PV + CON (COF.DER.PV-L1).
pub(crate) fn vigour_points(family: Family, values: &Characteristics) -> DerivedValue {
    DerivedValue::new()
        .with(Source::Value(family.id()), 2 * family.vigour_per_level())
        .with_characteristic(values, Characteristic::Con)
}

/// DR = 2 + CON, +1 for the mystiques (COF.DER.DR).
pub(crate) fn recovery_dice(family: Family, values: &Characteristics) -> DerivedValue {
    let value = DerivedValue::new()
        .with(Source::Rule(DR), 2)
        .with_characteristic(values, Characteristic::Con);
    if family == Family::Mystiques {
        value.with(Source::Value(family.id()), 1)
    } else {
        value
    }
}

/// PC = 2 + CHA, +1 for the aventuriers (COF.DER.PC), with Diversité
/// (humain r1) and Petit pote (gnome r2).
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
    let value = with_peuple_ability(value, character, Voie::Humain, 1);
    with_ability(value, character, Voie::Gnome, 2, 1)
}

/// PM = VOL + 1 per spell known (COF.DER.PM); `None` without any spell.
/// Only the documented spells are known (D-23).
pub(crate) fn mana_points(character: &Character, values: &Characteristics) -> Option<DerivedValue> {
    let spells: Vec<_> = character
        .voies()
        .flat_map(|(voie, ranks)| {
            ranks
                .iter()
                .filter(move |rank| voie.is_spell(**rank))
                .map(move |rank| voie.ability(*rank))
        })
        .collect();
    if spells.is_empty() {
        return None;
    }
    Some(spells.into_iter().fold(
        DerivedValue::new().with_characteristic(values, Characteristic::Vol),
        |value, spell| value.with(Source::Ability(spell), 1),
    ))
}

/// Init = 10 + PER (COF.DER.INIT), with Réflexes éclair (pourfendeur r1)
/// and Murmure dans le vent (air r1).
pub(crate) fn initiative(character: &Character, values: &Characteristics) -> DerivedValue {
    let value = DerivedValue::new()
        .with(Source::Rule(INIT), 10)
        .with_characteristic(values, Characteristic::Per);
    let value = with_ability(value, character, Voie::Pourfendeur, 1, 3);
    with_ability(value, character, Voie::Air, 1, 1)
}

/// DEF = 10 + AGI (capped by the armour) + armour + shield (COF.DER.DEF),
/// with Réflexes éclair (+2 from pourfendeur rank 5), Murmure dans le vent
/// and Petite taille (halfelin r1).
pub(crate) fn defence(character: &Character, values: &Characteristics) -> DerivedValue {
    let mut value = DerivedValue::new().with(Source::Rule(DEF), 10).with(
        Source::Characteristic(Characteristic::Agi),
        effective_agi(character, values),
    );
    if let Some(armour) = character.armour() {
        value = value.with(Source::Value(armour.id()), armour.defence());
    }
    if let Some(shield) = character.shield() {
        value = value.with(Source::Value(shield.id()), shield.defence());
    }
    let reflexes = if character.has_rank(Voie::Pourfendeur, Rank::new(5)) {
        2
    } else {
        1
    };
    let value = with_ability(value, character, Voie::Pourfendeur, 1, reflexes);
    let value = with_ability(value, character, Voie::Air, 1, 1);
    with_peuple_ability(value, character, Voie::Halfelin, 1)
}

/// Attack = level + FOR (contact), AGI capped by the armour (ranged) or VOL
/// (magic) (COF.DER.ATTACK).
pub(crate) fn attack(
    character: &Character,
    values: &Characteristics,
    mode: AttackMode,
) -> DerivedValue {
    let value = DerivedValue::new().with(Source::Rule(LEVEL1), LEVEL);
    match mode {
        AttackMode::Contact => value.with_characteristic(values, Characteristic::For),
        AttackMode::Ranged => value.with(
            Source::Characteristic(Characteristic::Agi),
            effective_agi(character, values),
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
