//! Secondary characteristics computed at creation (COC.DER.*).

use co_kernel::{Characteristic, Characteristics, DerivedValue, Rank, Source};

use crate::{
    AgeBracket, AttackMode, Character, Sequel, Trait, Voie,
    rules::{CG, DEF, HEALTH_SCALE, INIT},
};

/// Bonus of D'un autre âge to Init and DEF for an adolescent.
const ADOLESCENT_BONUS: i32 = 2;

/// Adds the adolescent bonus of D'un autre âge, when it applies.
fn with_adolescent_bonus(value: DerivedValue, character: &Character) -> DerivedValue {
    if character.has_trait(Trait::DUnAutreAge)
        && character.age_bracket() == Some(AgeBracket::Adolescent)
    {
        value.with(Source::Value(Trait::DUnAutreAge.id()), ADOLESCENT_BONUS)
    } else {
        value
    }
}

/// Init = 10 + INT + PER (COC.DER.INIT), with Rapide +3 and Crime rank 2 +2.
pub(crate) fn initiative(character: &Character, values: &Characteristics) -> DerivedValue {
    let mut value = DerivedValue::new()
        .with(Source::Rule(INIT), 10)
        .with_characteristic(values, Characteristic::Int)
        .with_characteristic(values, Characteristic::Per);
    if character.has_trait(Trait::Rapide) {
        value = value.with(Source::Value(Trait::Rapide.id()), 3);
    }
    let crime_rank = Rank::new(2);
    if character.has_rank(Voie::Crime, crime_rank) {
        value = value.with(Source::Ability(Voie::Crime.ability(crime_rank)), 2);
    }
    with_adolescent_bonus(value, character)
}

/// DEF = 10 + AGI + PER (COC.DER.DEF).
pub(crate) fn defence(character: &Character, values: &Characteristics) -> DerivedValue {
    let value = DerivedValue::new()
        .with(Source::Rule(DEF), 10)
        .with_characteristic(values, Characteristic::Agi)
        .with_characteristic(values, Characteristic::Per);
    with_adolescent_bonus(value, character)
}

/// ATC = FOR (COC.DER.ATC) or ATD = AGI (COC.DER.ATD), plus the rank reached
/// in the voie matching the attack mode (CO.VOIE.RANK-SCALING).
pub(crate) fn attack(
    character: &Character,
    values: &Characteristics,
    mode: AttackMode,
) -> DerivedValue {
    let (characteristic, voie) = match mode {
        AttackMode::Unarmed => (Characteristic::For, Voie::CombatAMainsNues),
        AttackMode::ContactWeapon => (Characteristic::For, Voie::ArmesDeContact),
        AttackMode::RangedWeapon => (Characteristic::Agi, Voie::ArmesADistance),
        AttackMode::HeavyWeapon => (Characteristic::Agi, Voie::ArmesLourdes),
    };
    let value = DerivedValue::new().with_characteristic(values, characteristic);
    let first_rank = Rank::new(1);
    if character.has_rank(voie, first_rank) {
        let rank = character.highest_rank(voie).map_or(0, Rank::get);
        value.with(Source::Ability(voie.ability(first_rank)), i32::from(rank))
    } else {
        value
    }
}

/// Contact DM bonus = FOR (COC.DER.DM-BONUS), with Force de la nature +1.
pub(crate) fn contact_damage_bonus(
    character: &Character,
    values: &Characteristics,
) -> DerivedValue {
    let value = DerivedValue::new().with_characteristic(values, Characteristic::For);
    if character.has_trait(Trait::ForceDeLaNature) {
        value.with(Source::Value(Trait::ForceDeLaNature.id()), 1)
    } else {
        value
    }
}

/// CG = CON + 3 (COC.DER.CG).
pub(crate) fn healing_capacity(values: &Characteristics) -> DerivedValue {
    DerivedValue::new()
        .with_characteristic(values, Characteristic::Con)
        .with(Source::Rule(CG), 3)
}

/// 20 steps, 4 fewer with D'un autre âge and with Diminué (corps)
/// (COC.DER.HEALTH-SCALE).
pub(crate) fn health_scale(character: &Character) -> DerivedValue {
    [Trait::DUnAutreAge, Trait::Sequelle(Sequel::DiminueCorps)]
        .into_iter()
        .filter(|t| character.has_trait(*t))
        .fold(
            DerivedValue::new().with(Source::Rule(HEALTH_SCALE), 20),
            |value, t| value.with(Source::Value(t.id()), -4),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    fn total(character: &Character, value: Value) -> Option<i32> {
        character.value(value).map(|v| v.total())
    }

    #[test]
    fn base_formulas_read_the_characteristics() {
        let character = Character::new()
            .with_carac_points(Characteristic::Agi, 1)
            .with_carac_points(Characteristic::Con, -1)
            .with_carac_points(Characteristic::For, 2)
            .with_carac_points(Characteristic::Per, 3)
            .with_carac_points(Characteristic::Int, 4);
        assert_eq!(total(&character, Value::Initiative), Some(17));
        assert_eq!(total(&character, Value::Defence), Some(14));
        assert_eq!(total(&character, Value::HealingCapacity), Some(2));
        assert_eq!(total(&character, Value::ContactDamageBonus), Some(2));
        assert_eq!(total(&character, Value::HealthScale), Some(20));
        for (mode, expected) in [
            (AttackMode::Unarmed, 2),
            (AttackMode::ContactWeapon, 2),
            (AttackMode::RangedWeapon, 1),
            (AttackMode::HeavyWeapon, 1),
        ] {
            assert_eq!(total(&character, Value::Attack(mode)), Some(expected));
        }
    }

    #[test]
    fn attack_adds_the_highest_rank_of_the_matching_voie() {
        let character = Character::new()
            .with_rank(Voie::ArmesADistance, 1)
            .with_rank(Voie::ArmesADistance, 2)
            .with_rank(Voie::ArmesADistance, 3)
            .with_rank(Voie::CombatAMainsNues, 1);
        assert_eq!(
            total(&character, Value::Attack(AttackMode::RangedWeapon)),
            Some(3)
        );
        assert_eq!(
            total(&character, Value::Attack(AttackMode::Unarmed)),
            Some(1)
        );
        assert_eq!(
            total(&character, Value::Attack(AttackMode::HeavyWeapon)),
            Some(0)
        );
        assert_eq!(
            total(&character, Value::Attack(AttackMode::ContactWeapon)),
            Some(0)
        );
    }

    #[test]
    fn traits_and_capacites_modify_initiative() {
        let character = Character::new()
            .with_trait(Trait::Rapide, 2)
            .with_rank(Voie::Crime, 1)
            .with_rank(Voie::Crime, 2);
        assert_eq!(total(&character, Value::Initiative), Some(15));
    }

    #[test]
    fn d_un_autre_age_helps_adolescents_and_shortens_the_health_scale() {
        let adolescent = Character::new()
            .with_age(14)
            .with_trait(Trait::DUnAutreAge, 2);
        assert_eq!(total(&adolescent, Value::Initiative), Some(12));
        assert_eq!(total(&adolescent, Value::Defence), Some(12));
        assert_eq!(total(&adolescent, Value::HealthScale), Some(16));
        let elderly = adolescent.with_age(70);
        assert_eq!(total(&elderly, Value::Initiative), Some(10));
        assert_eq!(total(&elderly, Value::Defence), Some(10));
    }

    #[test]
    fn diminue_corps_shortens_the_health_scale() {
        let character = Character::new()
            .with_trait(Trait::DUnAutreAge, 2)
            .with_trait(Trait::Sequelle(Sequel::DiminueCorps), 2);
        assert_eq!(total(&character, Value::HealthScale), Some(12));
    }

    #[test]
    fn force_de_la_nature_adds_to_contact_damage() {
        let character = Character::new()
            .with_carac_points(Characteristic::For, 1)
            .with_trait(Trait::ForceDeLaNature, 2);
        assert_eq!(total(&character, Value::ContactDamageBonus), Some(2));
    }
}
