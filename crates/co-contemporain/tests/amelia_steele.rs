//! Acceptance test: Amelia Steele, the worked character of the COC2 book
//! (COC p. 14-28), from the co-rules fixture `coc2-amelia-steele`.

use co_contemporain::{
    ALLOCATED_POINTS, AgeBracket, AttackMode, Character, Domaine, SocialClass, Trait, Value, Voie,
    Weapon,
};
use co_kernel::{Characteristic, Characteristics, Dice, Rank, Source};

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
    // Her sheet names no speciality: each capacité asking for one gets an
    // invented one.
    let character = ranks.into_iter().fold(character, |c, (voie, highest)| {
        (1..=highest).fold(c, |c, rank| {
            let c = c.with_rank(voie, rank);
            if voie.speciality(Rank::new(rank)).is_some() {
                c.with_speciality(voie, rank, "invented")
            } else {
                c
            }
        })
    });
    let character = traits
        .into_iter()
        .fold(character, |c, (t, points)| c.with_trait(t, points));
    let character = points
        .into_iter()
        .fold(character, |c, (characteristic, points)| {
            c.with_carac_points(characteristic, points)
        });
    // Her lucky Bowie knife comes with the voie des armes de contact; the
    // rest of her sheet is free text.
    [
        "motorbike",
        "hiking gear",
        "cigar case",
        "metal whisky flask",
    ]
    .into_iter()
    .fold(character.with_item(Weapon::Couteau, 1), |c, item| {
        c.with_item(item, 1)
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
fn weapons() {
    let amelia = amelia_steele();
    // Unarmed: ATC +1, 1d4+2, bonus critique 4.
    let unarmed = Weapon::MainsNues;
    assert_eq!(
        total(&amelia, Value::Attack(unarmed.attack_mode())),
        Some(1)
    );
    assert_eq!(amelia.damage_dice(unarmed), Dice::new([4]));
    assert_eq!(total(&amelia, Value::Damage(unarmed)), Some(2));
    assert_eq!(total(&amelia, Value::CriticalBonus(unarmed)), Some(4));
    // Couteau: ATC +2, 1d6+2, bonus critique 6.
    let couteau = Weapon::Couteau;
    assert_eq!(
        total(&amelia, Value::Attack(couteau.attack_mode())),
        Some(2)
    );
    assert_eq!(amelia.damage_dice(couteau), Dice::new([6]));
    assert_eq!(total(&amelia, Value::Damage(couteau)), Some(2));
    assert_eq!(total(&amelia, Value::CriticalBonus(couteau)), Some(6));
    assert_eq!(total(&amelia, Value::CriticalRange(couteau)), Some(20));
    assert_eq!(total(&amelia, Value::DamageReduction), Some(0));
}

#[test]
fn no_violation() {
    let amelia = amelia_steele();
    assert_eq!(amelia.violations(), []);
    assert!(amelia.report().is_clean());
}
