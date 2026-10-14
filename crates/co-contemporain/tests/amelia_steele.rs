//! Acceptance test: Amelia Steele, the worked character of the COC2 book
//! (COC p. 14-28), from the co-rules fixture `examples/coc2-amelia-steele.md`.
//!
//! Deferred with weapons: the damage dice (1d4 unarmed, 1d6 couteau) and the
//! bonus critique (4, 6). Their flat part is the contact DM bonus below.

use co_contemporain::{
    ALLOCATED_POINTS, AgeBracket, AttackMode, Character, Domaine, SocialClass, Trait, Value, Voie,
};
use co_kernel::{Characteristic, Characteristics, Source};

fn amelia_steele() -> Character {
    let ranks = [
        (Voie::Corporations, 1),
        (Voie::ArmesDeContact, 1),
        (Voie::Sciences, 1),
        (Voie::Survie, 2),
        (Voie::Vehicules, 2),
    ];
    let traits = [
        (Trait::Endurant, 1),
        (Trait::ForceDeLaNature, 2),
        (Trait::Inebranlable, 1),
        (Trait::Remarquable, 1),
        (Trait::CodeDeConduite, 2),
        (Trait::Nemesis, 2),
        (Trait::Toxicomane, 1),
    ];
    let points = [
        (Characteristic::Agi, 2),
        (Characteristic::Con, 0),
        (Characteristic::For, 1),
        (Characteristic::Per, 2),
        (Characteristic::Cha, 2),
        (Characteristic::Int, 0),
        (Characteristic::Vol, 1),
    ];

    let character = Character::new()
        .with_age(32)
        .with_domaine(Domaine::Aventure);
    let character = ranks.into_iter().fold(character, |c, (voie, highest)| {
        (1..=highest).fold(c, |c, rank| c.with_rank(voie, rank))
    });
    let character = traits
        .into_iter()
        .fold(character, |c, (t, points)| c.with_trait(t, points));
    points
        .into_iter()
        .fold(character, |c, (characteristic, points)| {
            c.with_carac_points(characteristic, points)
        })
}

fn total(character: &Character, value: Value) -> Option<i32> {
    character.value(value).map(|v| v.total())
}

#[test]
fn age_bracket_is_31_to_50() {
    assert_eq!(amelia_steele().age_bracket(), Some(AgeBracket::From31To50));
}

#[test]
fn characteristics() {
    let expected = Characteristics::from_fn(|c| {
        match c {
            Characteristic::Agi | Characteristic::Per | Characteristic::Cha => 2,
            Characteristic::Con | Characteristic::For | Characteristic::Vol => 1,
            Characteristic::Int => 0,
        }
        .into()
    });
    assert_eq!(amelia_steele().characteristics(), expected);
}

#[test]
fn constitution_comes_from_force_de_la_nature() {
    let con = amelia_steele().characteristic(Characteristic::Con);
    let breakdown: Vec<_> = con
        .contributions()
        .iter()
        .map(|c| (c.source().clone(), c.amount()))
        .collect();
    assert_eq!(
        breakdown,
        [
            (Source::Value(ALLOCATED_POINTS), 0),
            (Source::Value(Trait::ForceDeLaNature.id()), 1),
        ]
    );
}

#[test]
fn creation_budgets_are_spent() {
    let amelia = amelia_steele();
    assert_eq!(amelia.carac_points_spent(), 8);
    assert_eq!(total(&amelia, Value::CaracPoints), Some(8));
    assert_eq!(amelia.capacity_points_spent(), 7);
    assert_eq!(total(&amelia, Value::CapacityPoints), Some(7));
    assert_eq!(amelia.voie_count(), 5);
    assert_eq!(total(&amelia, Value::MaxVoies), Some(5));
    assert_eq!(total(&amelia, Value::MaxRank), Some(3));
}

#[test]
fn traits_balance() {
    let amelia = amelia_steele();
    assert_eq!(amelia.advantages(), 5);
    assert_eq!(amelia.disadvantages(), -5);
    assert_eq!(amelia.social_class(), Some(SocialClass::Moyenne));
}

#[test]
fn secondary_characteristics() {
    let amelia = amelia_steele();
    assert_eq!(total(&amelia, Value::Initiative), Some(12));
    assert_eq!(total(&amelia, Value::Defence), Some(14));
    assert_eq!(total(&amelia, Value::HealingCapacity), Some(4));
    assert_eq!(total(&amelia, Value::HealthScale), Some(20));
}

#[test]
fn attacks() {
    let amelia = amelia_steele();
    let attack = |mode| total(&amelia, Value::Attack(mode));
    assert_eq!(attack(AttackMode::Unarmed), Some(1));
    // Couteau: FOR 1 + armes de contact rank 1.
    assert_eq!(attack(AttackMode::ContactWeapon), Some(2));
    assert_eq!(attack(AttackMode::RangedWeapon), Some(2));
    assert_eq!(attack(AttackMode::HeavyWeapon), Some(2));
    // FOR 1 + Force de la nature 1: the "+2" of 1d4+2 and 1d6+2.
    assert_eq!(total(&amelia, Value::ContactDamageBonus), Some(2));
}

#[test]
fn no_violation() {
    let amelia = amelia_steele();
    assert_eq!(amelia.violations(), []);
    assert!(amelia.report().is_clean());
}
