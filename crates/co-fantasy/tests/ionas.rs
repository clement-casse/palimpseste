//! Acceptance test: Ionas Melenwë, high-elf ensorceleur of the COF2 book
//! (COF p. 20-40), at levels 1 and 2, from the co-rules fixture `cof2-ionas`.
//!
//! Deferred with spell effects: the DM of Choc (1d4+4).

use co_fantasy::{
    AttackMode, Character, Family, LevelUp, Money, Peuple, Profil, RecoveryDie, SERIES_VALUE,
    Series, Value, Voie, Weapon, WeaponUse,
};
use co_kernel::{AbilityId, Characteristic, Characteristics, Dice, Source};

fn ionas() -> Character {
    let values = [
        (Characteristic::Cha, 3),
        (Characteristic::Vol, 2),
        (Characteristic::Agi, 1),
        (Characteristic::Con, 1),
        (Characteristic::For, -1),
        (Characteristic::Per, 0),
        (Characteristic::Int, 0),
    ];
    let character = Character::new()
        .with_profil(Profil::Ensorceleur)
        .with_peuple(Peuple::ElfeHaut)
        .with_series(Series::Expert);
    values
        .into_iter()
        .fold(character, |c, (characteristic, value)| {
            c.with_series_value(characteristic, value)
        })
        .with_peuple_bonus(Characteristic::Cha)
        // The voie du mage replaces the voie de peuple.
        .with_rank(Voie::Mage, 1)
        .with_rank(Voie::Air, 1)
        .with_rank(Voie::Invocation, 1)
        // Mage extra capacité: Serviteur invisible.
        .with_rank(Voie::Invocation, 2)
        // Starting equipment; the optional extra weapon was traded with the
        // GM for luxurious clothes, a free text item.
        .with_item(Weapon::BatonFerre, 1)
        .with_item(Weapon::Dague, 1)
        .with_item("luxurious clothes", 1)
        .with_purse(Money::pa(7))
}

fn total(character: &Character, value: Value) -> Option<i32> {
    character.value(value).map(|v| v.total())
}

#[test]
fn family_is_mages() {
    let ionas = ionas();
    assert_eq!(ionas.family(), Some(Family::Mages));
    assert_eq!(ionas.recovery_die(), Some(RecoveryDie::D6));
}

#[test]
fn characteristics() {
    let expected = Characteristics::from_fn(|c| {
        match c {
            Characteristic::Agi | Characteristic::Con => 1,
            Characteristic::For => -2,
            Characteristic::Per | Characteristic::Int => 0,
            Characteristic::Cha => 4,
            Characteristic::Vol => 2,
        }
        .into()
    });
    assert_eq!(ionas().characteristics(), expected);
}

#[test]
fn strength_takes_the_elfe_haut_malus() {
    let strength = ionas().characteristic(Characteristic::For);
    let breakdown: Vec<_> = strength
        .contributions()
        .iter()
        .map(|c| (c.source().clone(), c.amount()))
        .collect();
    assert_eq!(
        breakdown,
        [
            (Source::Value(SERIES_VALUE), -1),
            (Source::Value(Peuple::ElfeHaut.id()), -1),
        ]
    );
}

#[test]
fn resources() {
    let ionas = ionas();
    assert_eq!(total(&ionas, Value::VigourPoints), Some(7));
    assert_eq!(total(&ionas, Value::RecoveryDice), Some(3));
    assert_eq!(total(&ionas, Value::LuckPoints), Some(6));
}

#[test]
fn mana_counts_three_spells() {
    let mana = ionas().value(Value::ManaPoints);
    let breakdown: Vec<_> = mana
        .iter()
        .flat_map(co_kernel::DerivedValue::contributions)
        .map(|c| (c.source().clone(), c.amount()))
        .collect();
    assert_eq!(
        breakdown,
        [
            (Source::Characteristic(Characteristic::Vol), 2),
            (Source::Ability(AbilityId::from("cof.air.1")), 1),
            (Source::Ability(AbilityId::from("cof.invocation.1")), 1),
            (Source::Ability(AbilityId::from("cof.invocation.2")), 1),
        ]
    );
    assert_eq!(total(&ionas(), Value::ManaPoints), Some(5));
}

#[test]
fn test_bonuses_keep_lumiere_interieure_and_add_occultisme() {
    let ionas = ionas();
    let bonuses: Vec<_> = ionas
        .test_bonuses()
        .into_iter()
        .map(|bonus| {
            (
                bonus.ability(),
                bonus.characteristics(),
                total(&ionas, Value::TestBonus(bonus)),
            )
        })
        .collect();
    // The voie du mage keeps the elfe haut's +3 to INT and CHA tests, and
    // gives rank + 2 to magic knowledge.
    assert_eq!(
        bonuses,
        [
            (
                Voie::ElfeHaut.ability(1.into()),
                [Characteristic::Int, Characteristic::Cha].as_slice(),
                Some(3)
            ),
            (Voie::Mage.ability(1.into()), [].as_slice(), Some(3)),
        ]
    );
}

#[test]
fn initiative_and_defence() {
    let ionas = ionas();
    // Murmure dans le vent adds 1 to both.
    assert_eq!(total(&ionas, Value::Initiative), Some(11));
    assert_eq!(total(&ionas, Value::Defence), Some(12));
}

#[test]
fn attacks() {
    let ionas = ionas();
    let attack = |mode| total(&ionas, Value::Attack(mode));
    assert_eq!(attack(AttackMode::Contact), Some(-1));
    assert_eq!(attack(AttackMode::Ranged), Some(2));
    assert_eq!(attack(AttackMode::Magic), Some(3));
}

#[test]
fn damage() {
    // Bâton ferré, two-handed: 1d6 + FOR (-2), negative FOR applies.
    let use_ = WeaponUse::TwoHanded;
    assert_eq!(Weapon::BatonFerre.dice(use_), Some(Dice::new([6])));
    assert_eq!(
        total(&ionas(), Value::Damage(Weapon::BatonFerre, use_)),
        Some(-2)
    );
    assert!(Profil::Ensorceleur.masters(Weapon::BatonFerre));
}

#[test]
fn no_violation() {
    let ionas = ionas();
    assert_eq!(ionas.violations(), []);
    assert!(ionas.report().is_clean());
}

/// Level 2: Sous tension* (air r2) and Maîtrise de la magie* (mage r2).
fn ionas_level_two() -> Character {
    ionas().with_level_up(
        LevelUp::new()
            .with_rank(Voie::Air, 2)
            .with_rank(Voie::Mage, 2),
    )
}

#[test]
fn level_two_adds_vigour_mana_and_attack() {
    let ionas = ionas_level_two();
    // 7, then 3 (mages) + CON 1.
    assert_eq!(total(&ionas, Value::VigourPoints), Some(11));
    // Two more spells.
    assert_eq!(total(&ionas, Value::ManaPoints), Some(7));
    let attack = |mode| total(&ionas, Value::Attack(mode));
    assert_eq!(attack(AttackMode::Contact), Some(0));
    assert_eq!(attack(AttackMode::Ranged), Some(3));
    assert_eq!(attack(AttackMode::Magic), Some(4));
}

#[test]
fn level_two_has_no_violation() {
    assert_eq!(ionas_level_two().violations(), []);
}
