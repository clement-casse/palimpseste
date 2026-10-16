//! The content catalogue: every content enum lists all its variants, each
//! with a unique identifier that converts back, and a display name.

use std::collections::BTreeSet;

use co_fantasy::{
    Armour, Character, Family, Peuple, Profil, Series, Shield, Step, Value, Voie, Weapon, WeaponUse,
};

/// Asserts that `ids` are unique, that each converts back with `from_id`,
/// and that every name is set.
fn check<T: Copy + PartialEq + std::fmt::Debug>(
    all: &[T],
    id: impl Fn(T) -> String,
    from_id: impl Fn(&str) -> Option<T>,
    name: impl Fn(T) -> &'static str,
) {
    let ids: BTreeSet<String> = all.iter().map(|item| id(*item)).collect();
    assert_eq!(ids.len(), all.len());
    for item in all {
        assert_eq!(from_id(&id(*item)), Some(*item));
        assert!(!name(*item).is_empty(), "{item:?}");
    }
    assert_eq!(from_id("unknown"), None);
}

#[test]
fn content_ids_round_trip() {
    check(
        &Profil::ALL,
        |p| p.id().as_str().to_owned(),
        Profil::from_id,
        Profil::name,
    );
    check(
        &Peuple::ALL,
        |p| p.id().as_str().to_owned(),
        Peuple::from_id,
        Peuple::name,
    );
    check(
        &Series::ALL,
        |s| s.id().as_str().to_owned(),
        Series::from_id,
        Series::name,
    );
    check(
        &Voie::ALL,
        |v| v.id().as_str().to_owned(),
        Voie::from_id,
        Voie::name,
    );
    check(
        &Armour::ALL,
        |a| a.id().as_str().to_owned(),
        Armour::from_id,
        Armour::name,
    );
    check(
        &Shield::ALL,
        |s| s.id().as_str().to_owned(),
        Shield::from_id,
        Shield::name,
    );
    check(
        &Weapon::ALL,
        |w| w.id().as_str().to_owned(),
        Weapon::from_id,
        Weapon::name,
    );
}

#[test]
fn value_ids_round_trip() {
    let values = [
        Value::VigourPoints,
        Value::Defence,
        Value::Money,
        Value::Damage(Weapon::EpeeLongue, WeaponUse::TwoHanded),
        Value::CriticalRange(Weapon::Rapiere),
    ];
    for value in values {
        assert_eq!(Value::from_id(value.id().as_str()), Some(value));
    }
    assert_eq!(Value::from_id("cof.unknown"), None);
}

#[test]
fn every_profil_voie_belongs_to_its_profil() {
    for profil in Profil::ALL {
        for voie in profil.voies() {
            assert_eq!(voie.profil(), Some(profil));
        }
    }
    assert!(Family::ALL.iter().all(|family| !family.name().is_empty()));
    assert!(Step::ALL.iter().all(|step| !step.name().is_empty()));
}

#[test]
fn peuple_slot_follows_peuple_and_family() {
    let options = |character: Character| character.peuple_slot_options();
    assert_eq!(options(Character::new()), []);
    assert_eq!(
        options(Character::new().with_peuple(Peuple::Nain)),
        [Voie::Nain]
    );
    assert_eq!(
        options(Character::new().with_peuple(Peuple::DemiElfe)),
        [Voie::ElfeHaut, Voie::ElfeSylvain, Voie::Humain]
    );
    assert_eq!(
        options(
            Character::new()
                .with_peuple(Peuple::Gnome)
                .with_profil(Profil::Magicien)
        ),
        [Voie::Gnome, Voie::Mage]
    );
}
