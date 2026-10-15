//! Items carried and the starting equipment of the voies acquired
//! (COC.CRE.EQUIPMENT, COC.CNT.VOIES, D-26, D-30).

use std::collections::BTreeMap;

use co_kernel::{Severity, Subject, ValueId, Violation};

use crate::{
    AttackMode, Character, Protection, Voie, Weapon,
    rules::{EQUIPMENT, EQUIPMENT_MECHANIC},
};

/// An item carried by the character.
///
/// Only items an engine rule reads are content; anything else is a free
/// text item, never checked (D-26).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item {
    /// A weapon of the weapon tables.
    Weapon(Weapon),
    /// Any other item, named freely.
    Other(ItemName),
}

impl From<Weapon> for Item {
    fn from(weapon: Weapon) -> Self {
        Self::Weapon(weapon)
    }
}

impl From<ItemName> for Item {
    fn from(name: ItemName) -> Self {
        Self::Other(name)
    }
}

impl From<&str> for Item {
    fn from(name: &str) -> Self {
        Self::Other(name.into())
    }
}

impl From<String> for Item {
    fn from(name: String) -> Self {
        Self::Other(name.into())
    }
}

/// The name of a free text item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemName(String);

impl ItemName {
    /// Returns the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ItemName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for ItemName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

/// How many of an item the character carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(u32);

impl Quantity {
    /// Creates a quantity.
    #[must_use]
    pub const fn new(quantity: u32) -> Self {
        Self(quantity)
    }

    /// Returns the quantity.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for Quantity {
    fn from(quantity: u32) -> Self {
        Self(quantity)
    }
}

/// An item the starting equipment covers: a weapon carried or a protection
/// worn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Unit {
    Weapon(Weapon),
    Protection(Protection),
}

impl Unit {
    fn id(self) -> ValueId {
        match self {
            Self::Weapon(weapon) => weapon.id(),
            Self::Protection(protection) => protection.id(),
        }
    }
}

/// The weapon category each weapon voie gives one weapon of. The other
/// voies give free text items only (COC.CNT.VOIES).
const fn starting_weapon(voie: Voie) -> Option<AttackMode> {
    match voie {
        Voie::ArmesDeContact => Some(AttackMode::ContactWeapon),
        Voie::ArmesADistance => Some(AttackMode::RangedWeapon),
        Voie::ArmesLourdes => Some(AttackMode::HeavyWeapon),
        _ => None,
    }
}

/// Checks the weapons and protections against the starting equipment of the
/// voies acquired (COC.CRE.EQUIPMENT, D-26). Each weapon voie, in voie order,
/// takes the first weapon of its category; units left over were not given,
/// and each one is a `standard` deviation the Game Master may waive. No voie
/// gives a protection (D-30), and the mains nues are never carried.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let mut remaining: BTreeMap<Unit, u32> = character
        .weapons_carried()
        .filter(|(weapon, _)| *weapon != Weapon::MainsNues)
        .map(|(weapon, quantity)| (Unit::Weapon(weapon), quantity.get()))
        .collect();
    remaining.extend(
        character
            .protections()
            .map(|protection| (Unit::Protection(protection), 1)),
    );

    for category in character
        .voies()
        .filter_map(|(voie, _)| starting_weapon(voie))
    {
        let given = remaining.iter_mut().find(|(unit, count)| {
            **count > 0 && matches!(unit, Unit::Weapon(weapon) if weapon.attack_mode() == category)
        });
        if let Some((_, count)) = given {
            *count -= 1;
        }
    }

    for (unit, count) in remaining.into_iter().filter(|(_, count)| *count > 0) {
        violations.push(
            Violation::deviation(
                EQUIPMENT,
                EQUIPMENT_MECHANIC,
                Subject::Value(unit.id()),
                Severity::Standard,
            )
            .with_parameter("quantity", i64::from(count)),
        );
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Category, Parameter, ParameterValue};

    use super::*;

    fn leftovers(character: &Character) -> Vec<(Subject, i64)> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
            .iter()
            .map(|v| {
                assert_eq!(v.category(), Category::Deviation(Severity::Standard));
                let quantity = match v.parameters().first().map(Parameter::value) {
                    Some(ParameterValue::Integer(quantity)) => *quantity,
                    _ => 0,
                };
                (v.subject().clone(), quantity)
            })
            .collect()
    }

    #[test]
    fn each_weapon_voie_gives_a_weapon_of_its_category() {
        let character = Character::new()
            .with_rank(Voie::ArmesDeContact, 1)
            .with_rank(Voie::ArmesADistance, 1)
            .with_item(Weapon::Machette, 1)
            .with_item(Weapon::PistoletLourd, 1);
        assert_eq!(leftovers(&character), []);
        let second = character.with_item(Weapon::Machette, 2);
        assert_eq!(
            leftovers(&second),
            [(Subject::Value(Weapon::Machette.id()), 1)]
        );
    }

    #[test]
    fn weapon_of_another_category_deviates() {
        let character = Character::new()
            .with_rank(Voie::ArmesADistance, 1)
            .with_item(Weapon::Mitrailleuse, 1);
        assert_eq!(
            leftovers(&character),
            [(Subject::Value(Weapon::Mitrailleuse.id()), 1)]
        );
    }

    #[test]
    fn protections_are_never_given() {
        let character = Character::new()
            .with_rank(Voie::ArmesLourdes, 1)
            .with_protection(Protection::GiletSouple);
        assert_eq!(
            leftovers(&character),
            [(Subject::Value(Protection::GiletSouple.id()), 1)]
        );
    }

    #[test]
    fn free_text_items_and_bare_hands_are_never_checked() {
        let character = Character::new()
            .with_item("hiking gear", 1)
            .with_item(Weapon::MainsNues, 2);
        assert_eq!(leftovers(&character), []);
    }

    #[test]
    fn the_game_master_waives_a_protection() {
        use co_kernel::{Author, Waiver};

        let character = Character::new().with_protection(Protection::GiletSouple);
        let subject = Subject::Value(Protection::GiletSouple.id());
        let is_open = |character: &Character| {
            character
                .report()
                .open()
                .any(|entry| entry.violation().rule() == &EQUIPMENT)
        };
        assert!(is_open(&character));
        let waiver = Waiver::new(EQUIPMENT, subject, Author::new("gm"), "police officer");
        assert!(!is_open(&character.with_waiver(waiver)));
    }

    #[test]
    fn a_weapon_may_be_given_up() {
        let character = Character::new().with_rank(Voie::ArmesLourdes, 1);
        assert_eq!(leftovers(&character), []);
    }
}
