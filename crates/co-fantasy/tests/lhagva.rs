//! Acceptance test: Lhagva, human barbarian of the COF2 book (COF p. 20-40),
//! at levels 1 and 2, from the co-rules fixture `cof2-lhagva`.

use co_fantasy::{
    Armour, AttackMode, Character, Family, Level, LevelUp, Money, Peuple, Profil, RecoveryDie,
    Series, Shield, Value, Voie, Weapon, WeaponUse,
};
use co_kernel::{AbilityId, Characteristic, Characteristics, Dice, Source};

fn lhagva() -> Character {
    let values = [
        (Characteristic::For, 3),
        (Characteristic::Con, 2),
        (Characteristic::Agi, 1),
        (Characteristic::Vol, 1),
        (Characteristic::Cha, -1),
        (Characteristic::Per, 0),
        (Characteristic::Int, 0),
    ];
    let character = Character::new()
        .with_profil(Profil::Barbare)
        .with_peuple(Peuple::Humain)
        .with_series(Series::Expert);
    values
        .into_iter()
        .fold(character, |c, (characteristic, value)| {
            c.with_series_value(characteristic, value)
        })
        // Humain: +1 to one of the two lowest (D-15).
        .with_peuple_bonus(Characteristic::Per)
        .with_rank(Voie::Pourfendeur, 1)
        .with_rank(Voie::Rage, 1)
        .with_rank(Voie::Humain, 1)
        // Starting equipment: the "one-handed d8 weapon and shield" option,
        // her cimeterre treated as an épée longue.
        .with_armour(Armour::CuirSimple)
        .with_shield(Shield::GrandBouclier)
        .with_item(Weapon::EpeeLongue, 1)
        .with_item(Weapon::Javelot, 2)
        .with_item(Weapon::Dague, 1)
        .with_item("adventurer's bag", 1)
        // The pre-generated purse: the average of 2d6 pa.
        .with_purse(Money::pa(7))
}

fn total(character: &Character, value: Value) -> Option<i32> {
    character.value(value).map(|v| v.total())
}

#[test]
fn family_is_combattants() {
    let lhagva = lhagva();
    assert_eq!(lhagva.family(), Some(Family::Combattants));
    assert_eq!(lhagva.recovery_die(), Some(RecoveryDie::D10));
}

#[test]
fn characteristics() {
    let expected = Characteristics::from_fn(|c| {
        match c {
            Characteristic::Agi | Characteristic::Per | Characteristic::Vol => 1,
            Characteristic::Con => 2,
            Characteristic::For => 3,
            Characteristic::Cha => -1,
            Characteristic::Int => 0,
        }
        .into()
    });
    assert_eq!(lhagva().characteristics(), expected);
}

#[test]
fn resources() {
    let lhagva = lhagva();
    // The book's sheets show 15; the rule text gives 12 (D-01, D-14).
    assert_eq!(total(&lhagva, Value::VigourPoints), Some(12));
    assert_eq!(total(&lhagva, Value::RecoveryDice), Some(4));
    assert_eq!(total(&lhagva, Value::LuckPoints), Some(2));
    assert_eq!(total(&lhagva, Value::ManaPoints), None);
}

#[test]
fn test_bonuses_come_from_her_three_capacites() {
    let lhagva = lhagva();
    let bonuses: Vec<_> = lhagva
        .test_bonuses()
        .into_iter()
        .map(|bonus| {
            (
                bonus.ability(),
                bonus.characteristics(),
                total(&lhagva, Value::TestBonus(bonus)),
            )
        })
        .collect();
    // Réflexes éclair: rank + 2 to AGI dodge tests; Cri de guerre: rank + 2
    // to resist fear; Diversité: +3 on two origin domains.
    assert_eq!(
        bonuses,
        [
            (
                Voie::Pourfendeur.ability(1.into()),
                [Characteristic::Agi].as_slice(),
                Some(3)
            ),
            (Voie::Rage.ability(1.into()), [].as_slice(), Some(3)),
            (Voie::Humain.ability(1.into()), [].as_slice(), Some(3)),
        ]
    );
}

#[test]
fn luck_comes_from_diversite() {
    let luck = lhagva().value(Value::LuckPoints);
    let breakdown: Vec<_> = luck
        .iter()
        .flat_map(co_kernel::DerivedValue::contributions)
        .map(|c| (c.source().clone(), c.amount()))
        .collect();
    assert_eq!(
        breakdown[1..],
        [
            (Source::Characteristic(Characteristic::Cha), -1),
            (Source::Ability(AbilityId::from("cof.humain.1")), 1),
        ]
    );
}

#[test]
fn initiative_and_defence() {
    let lhagva = lhagva();
    assert_eq!(total(&lhagva, Value::Initiative), Some(14));
    assert_eq!(total(&lhagva, Value::Defence), Some(16));
    let defence = lhagva.value(Value::Defence);
    let breakdown: Vec<_> = defence
        .iter()
        .flat_map(co_kernel::DerivedValue::contributions)
        .map(|c| (c.source().clone(), c.amount()))
        .collect();
    assert_eq!(
        breakdown[1..],
        [
            (Source::Characteristic(Characteristic::Agi), 1),
            (Source::Value(Armour::CuirSimple.id()), 2),
            (Source::Value(Shield::GrandBouclier.id()), 2),
            (Source::Ability(AbilityId::from("cof.pourfendeur.1")), 1),
        ]
    );
}

#[test]
fn attacks() {
    let lhagva = lhagva();
    let attack = |mode| total(&lhagva, Value::Attack(mode));
    assert_eq!(attack(AttackMode::Contact), Some(4));
    assert_eq!(attack(AttackMode::Ranged), Some(2));
    assert_eq!(attack(AttackMode::Magic), Some(2));
}

#[test]
fn damage() {
    let lhagva = lhagva();
    // Cimeterre: 1d8 + FOR 3.
    let cimeterre = WeaponUse::OneHanded;
    assert_eq!(Weapon::EpeeLongue.dice(cimeterre), Some(Dice::new([8])));
    assert_eq!(
        total(&lhagva, Value::Damage(Weapon::EpeeLongue, cimeterre)),
        Some(3)
    );
    assert_eq!(cimeterre.attack_mode(), AttackMode::Contact);
    // Javelot: 1d6, no characteristic.
    let javelot = WeaponUse::Ranged;
    assert_eq!(Weapon::Javelot.dice(javelot), Some(Dice::new([6])));
    assert_eq!(
        total(&lhagva, Value::Damage(Weapon::Javelot, javelot)),
        Some(0)
    );
    assert_eq!(
        total(&lhagva, Value::CriticalRange(Weapon::EpeeLongue)),
        Some(20)
    );
}

#[test]
fn money() {
    assert_eq!(total(&lhagva(), Value::Money), Some(Money::pa(7).in_pc()));
}

#[test]
fn no_violation() {
    let lhagva = lhagva();
    assert_eq!(lhagva.violations(), []);
    assert!(lhagva.report().is_clean());
}

/// Level 2: Charge (pourfendeur r2) and Défier la mort (rage r2), one point
/// each.
fn lhagva_level_two() -> Character {
    lhagva().with_level_up(
        LevelUp::new()
            .with_rank(Voie::Pourfendeur, 2)
            .with_rank(Voie::Rage, 2),
    )
}

#[test]
fn level_two_adds_vigour_and_attack() {
    let lhagva = lhagva_level_two();
    assert_eq!(lhagva.level(), Level::from(2));
    // 12, then 5 (combattants) + CON 2.
    assert_eq!(total(&lhagva, Value::VigourPoints), Some(19));
    let attack = |mode| total(&lhagva, Value::Attack(mode));
    assert_eq!(attack(AttackMode::Contact), Some(5));
    assert_eq!(attack(AttackMode::Ranged), Some(3));
    assert_eq!(attack(AttackMode::Magic), Some(3));
}

#[test]
fn level_two_has_no_violation() {
    let lhagva = lhagva_level_two();
    assert_eq!(lhagva.violations(), []);
    assert!(lhagva.report().is_clean());
}
