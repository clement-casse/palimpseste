//! The content catalogue: every content enum lists all its variants, each
//! with a unique identifier that converts back, and a display name.

use std::collections::BTreeSet;

use co_contemporain::{Domaine, Protection, Step, Trait, Value, Voie, Weapon};

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
        &Domaine::ALL,
        |d| d.id().as_str().to_owned(),
        Domaine::from_id,
        Domaine::name,
    );
    check(
        &Voie::ALL,
        |v| v.id().as_str().to_owned(),
        Voie::from_id,
        Voie::name,
    );
    check(
        &Trait::ALL,
        |t| t.id().as_str().to_owned(),
        Trait::from_id,
        Trait::name,
    );
    check(
        &Weapon::ALL,
        |w| w.id().as_str().to_owned(),
        Weapon::from_id,
        Weapon::name,
    );
    check(
        &Protection::ALL,
        |p| p.id().as_str().to_owned(),
        Protection::from_id,
        Protection::name,
    );
    assert!(Step::ALL.iter().all(|step| !step.name().is_empty()));
}

#[test]
fn value_ids_round_trip() {
    let values = [
        Value::Initiative,
        Value::MaxRank,
        Value::Damage(Weapon::Epee),
        Value::CriticalBonus(Weapon::PistoletLourd),
        Value::CriticalRange(Weapon::FusilDePrecision),
    ];
    for value in values {
        assert_eq!(Value::from_id(value.id().as_str()), Some(value));
    }
    assert_eq!(Value::from_id("coc.unknown"), None);
}

#[test]
fn domaine_slots_and_weapon_voies() {
    assert_eq!(Domaine::Sante.minimum_retained(), 2);
    assert!(Domaine::Criminalite.is_extra_professional());
    assert_eq!(Domaine::Criminalite.minimum_retained(), 3);
    assert_eq!(Domaine::Sante.slots()[1], [Voie::Medecine]);
    assert!(Voie::ArmesLourdes.starting_weapon().is_some());
    assert_eq!(Voie::Crime.starting_weapon(), None);
}
