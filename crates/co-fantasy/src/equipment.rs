//! Items carried and the profil's starting equipment (COF.CRE.EQUIPMENT,
//! COF.CNT.PROFILS, D-26, D-28).

use std::collections::BTreeMap;

use co_kernel::{Severity, Subject, ValueId, Violation};

use self::Pattern::{Holding, Is, Matching, OneOf, Wearing};
use crate::{
    Armour, Armour as A, Character, Profil, Shield, Shield as S, Weapon, Weapon as W, WeaponUse,
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

/// An item the starting equipment covers: a weapon carried, the armour worn
/// or the shield held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Unit {
    Weapon(Weapon),
    Armour(Armour),
    Shield(Shield),
}

impl Unit {
    fn id(self) -> ValueId {
        match self {
            Self::Weapon(weapon) => weapon.id(),
            Self::Armour(armour) => armour.id(),
            Self::Shield(shield) => shield.id(),
        }
    }
}

/// What a line of the starting equipment accepts.
#[derive(Clone, Copy)]
enum Pattern {
    /// That weapon.
    Is(Weapon),
    /// One of these weapons.
    OneOf(&'static [Weapon]),
    /// A weapon satisfying the predicate.
    Matching(fn(Weapon) -> bool),
    /// That armour, worn.
    Wearing(Armour),
    /// That shield, held.
    Holding(Shield),
}

impl Pattern {
    fn matches(self, unit: Unit) -> bool {
        match (self, unit) {
            (Self::Is(expected), Unit::Weapon(weapon)) => weapon == expected,
            (Self::OneOf(weapons), Unit::Weapon(weapon)) => weapons.contains(&weapon),
            (Self::Matching(predicate), Unit::Weapon(weapon)) => predicate(weapon),
            (Self::Wearing(expected), Unit::Armour(armour)) => armour == expected,
            (Self::Holding(expected), Unit::Shield(shield)) => shield == expected,
            _ => false,
        }
    }
}

/// Items given together, each with its quantity.
type Bundle = &'static [(Pattern, u32)];

/// A line of the starting equipment: one of its bundles.
type Slot = &'static [Bundle];

const ARQUEBUSIER: &[Slot] = &[&[&[
    (Is(W::Petoire), 1),
    (Is(W::EpeeLongue), 1),
    (Is(W::Dague), 1),
    (Wearing(A::CuirRenforce), 1),
]]];

const BARDE: &[Slot] = &[&[&[
    (Is(W::Rapiere), 1),
    (Is(W::Dague), 1),
    (Wearing(A::CuirSimple), 1),
]]];

const RODEUR: &[Slot] = &[&[&[
    (Is(W::EpeeLongue), 1),
    (Is(W::Dague), 1),
    (Wearing(A::CuirRenforce), 1),
    (
        OneOf(&[W::ArcCourt, W::EpeeCourte, W::Hachette, W::Lance]),
        1,
    ),
]]];

const VOLEUR: &[Slot] = &[&[&[
    (Is(W::Rapiere), 1),
    (Is(W::Dague), 5),
    (Wearing(A::CuirSimple), 1),
]]];

const BARBARE: &[Slot] = &[
    &[&[
        (Is(W::Javelot), 2),
        (Is(W::Dague), 1),
        (Wearing(A::CuirSimple), 1),
    ]],
    &[
        &[(Is(W::HacheADeuxMains), 1)],
        &[(Matching(one_handed_d8), 1), (Holding(S::GrandBouclier), 1)],
    ],
];

const CHEVALIER: &[Slot] = &[&[&[
    (Is(W::EpeeLongue), 1),
    (Is(W::LanceDeCavalerie), 1),
    (Is(W::Dague), 1),
    (Holding(S::GrandBouclier), 1),
    (Wearing(A::CotteDeMailles), 1),
]]];

const GUERRIER: &[Slot] = &[&[&[
    (Is(W::EpeeLongue), 1),
    (OneOf(&[W::EpeeADeuxMains, W::HacheADeuxMains]), 1),
    (OneOf(&[W::Dague, W::Hachette]), 1),
    (Holding(S::GrandBouclier), 1),
    (Wearing(A::ChemiseDeMailles), 1),
]]];

const ENSORCELEUR: &[Slot] = &[&[&[
    (Is(W::BatonFerre), 1),
    (Is(W::Dague), 1),
    (Matching(ensorceleur_weapon), 1),
]]];

const FORGESORT: &[Slot] = &[&[&[
    (Is(W::Dague), 1),
    (Is(W::BatonFerre), 1),
    (Is(W::Marteau), 1),
]]];

const MAGICIEN_SORCIER: &[Slot] = &[&[&[(Is(W::BatonFerre), 1), (Is(W::Dague), 1)]]];

const DRUIDE: &[Slot] = &[&[&[
    (Is(W::Dague), 1),
    (Is(W::ArcCourt), 1),
    (Wearing(A::CuirSimple), 1),
    (OneOf(&[W::Baton, W::BatonFerre, W::Epieu]), 1),
]]];

const MOINE: &[Slot] = &[&[&[(OneOf(&[W::Baton, W::BatonFerre]), 1)]]];

const PRETRE: &[Slot] = &[&[&[
    (Holding(S::PetitBouclier), 1),
    (Wearing(A::ChemiseDeMailles), 1),
    (OneOf(&[W::Masse, W::Marteau, W::BatonFerre]), 1),
]]];

/// The barbare's one-handed weapon dealing 1d8.
fn one_handed_d8(weapon: Weapon) -> bool {
    weapon
        .dice(WeaponUse::OneHanded)
        .is_some_and(|dice| dice.highest() == 8 && dice.dice().len() == 1)
}

/// The ensorceleur's extra weapon: any weapon it masters (D-28).
fn ensorceleur_weapon(weapon: Weapon) -> bool {
    Profil::Ensorceleur.masters(weapon)
}

/// The profil's starting equipment, besides the adventurer's bag and the
/// purse. Free text items (instrument, crochets, grimoire...) are not
/// listed: they are never checked (D-26).
const fn starting_equipment(profil: Profil) -> &'static [Slot] {
    match profil {
        Profil::Arquebusier => ARQUEBUSIER,
        Profil::Barde => BARDE,
        Profil::Rodeur => RODEUR,
        Profil::Voleur => VOLEUR,
        Profil::Barbare => BARBARE,
        Profil::Chevalier => CHEVALIER,
        Profil::Guerrier => GUERRIER,
        Profil::Ensorceleur => ENSORCELEUR,
        Profil::Forgesort => FORGESORT,
        Profil::Magicien | Profil::Sorcier => MAGICIEN_SORCIER,
        Profil::Druide => DRUIDE,
        Profil::Moine => MOINE,
        Profil::Pretre => PRETRE,
    }
}

/// The units `bundle` takes from `remaining`: each line takes the first
/// matching units, in unit order.
fn take(bundle: Bundle, remaining: &BTreeMap<Unit, u32>) -> BTreeMap<Unit, u32> {
    let mut taken: BTreeMap<Unit, u32> = BTreeMap::new();
    for &(pattern, quantity) in bundle {
        let mut left = quantity;
        for (&unit, &count) in remaining {
            if left == 0 {
                break;
            }
            if !pattern.matches(unit) {
                continue;
            }
            let already = taken.get(&unit).copied().unwrap_or(0);
            let count = count.saturating_sub(already).min(left);
            if count > 0 {
                *taken.entry(unit).or_default() += count;
                left -= count;
            }
        }
    }
    taken
}

fn units(taken: &BTreeMap<Unit, u32>) -> u64 {
    taken.values().map(|count| u64::from(*count)).sum()
}

/// Checks the weapons, armour and shield against the profil's starting
/// equipment (COF.CRE.EQUIPMENT, D-26). Each line, in table order, takes the
/// bundle covering the most units; units left over were not given, and each
/// one is a `standard` deviation the Game Master may waive (a trade). Lines
/// left empty are not reported: an item may be given up. Without a profil,
/// there is no starting equipment to check.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let Some(profil) = character.profil() else {
        return;
    };
    let mut remaining: BTreeMap<Unit, u32> = character
        .weapons_carried()
        .map(|(weapon, quantity)| (Unit::Weapon(weapon), quantity.get()))
        .collect();
    remaining.extend(character.armour().map(|armour| (Unit::Armour(armour), 1)));
    remaining.extend(character.shield().map(|shield| (Unit::Shield(shield), 1)));

    for slot in starting_equipment(profil) {
        let mut best: Option<BTreeMap<Unit, u32>> = None;
        for bundle in *slot {
            let taken = take(bundle, &remaining);
            if best.as_ref().is_none_or(|best| units(&taken) > units(best)) {
                best = Some(taken);
            }
        }
        for (unit, count) in best.unwrap_or_default() {
            if let Some(left) = remaining.get_mut(&unit) {
                *left = left.saturating_sub(count);
            }
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

    fn weapon(weapon: Weapon, quantity: i64) -> (Subject, i64) {
        (Subject::Value(weapon.id()), quantity)
    }

    #[test]
    fn the_whole_starting_equipment_is_clean() {
        let guerrier = Character::new()
            .with_profil(Profil::Guerrier)
            .with_item(Weapon::EpeeLongue, 1)
            .with_item(Weapon::HacheADeuxMains, 1)
            .with_item(Weapon::Hachette, 1)
            .with_shield(Shield::GrandBouclier)
            .with_armour(Armour::ChemiseDeMailles);
        assert_eq!(leftovers(&guerrier), []);
        let voleur = Character::new()
            .with_profil(Profil::Voleur)
            .with_item(Weapon::Rapiere, 1)
            .with_item(Weapon::Dague, 5)
            .with_armour(Armour::CuirSimple);
        assert_eq!(leftovers(&voleur), []);
    }

    #[test]
    fn items_may_be_given_up() {
        let character = Character::new()
            .with_profil(Profil::Chevalier)
            .with_item(Weapon::Dague, 1);
        assert_eq!(leftovers(&character), []);
    }

    #[test]
    fn free_text_items_are_never_checked() {
        let character = Character::new()
            .with_profil(Profil::Magicien)
            .with_item("grimoire", 1)
            .with_item("luxurious clothes", 3);
        assert_eq!(leftovers(&character), []);
    }

    #[test]
    fn items_beyond_the_starting_equipment_deviate() {
        let character = Character::new()
            .with_profil(Profil::Voleur)
            .with_item(Weapon::Dague, 7)
            .with_item(Weapon::EpeeLongue, 1)
            .with_armour(Armour::CotteDeMailles)
            .with_shield(Shield::PetitBouclier);
        assert_eq!(
            leftovers(&character),
            [
                weapon(Weapon::Dague, 2),
                weapon(Weapon::EpeeLongue, 1),
                (Subject::Value(Armour::CotteDeMailles.id()), 1),
                (Subject::Value(Shield::PetitBouclier.id()), 1),
            ]
        );
    }

    #[test]
    fn a_line_takes_one_of_its_bundles() {
        let shield = Character::new()
            .with_profil(Profil::Barbare)
            .with_item(Weapon::Hache, 1)
            .with_shield(Shield::GrandBouclier);
        assert_eq!(leftovers(&shield), []);
        let both = shield.clone().with_item(Weapon::HacheADeuxMains, 1);
        assert_eq!(leftovers(&both), [weapon(Weapon::HacheADeuxMains, 1)]);
        let two_handed = Character::new()
            .with_profil(Profil::Barbare)
            .with_item(Weapon::HacheADeuxMains, 1);
        assert_eq!(leftovers(&two_handed), []);
        let not_d8 = Character::new()
            .with_profil(Profil::Barbare)
            .with_item(Weapon::EpeeCourte, 1);
        assert_eq!(leftovers(&not_d8), [weapon(Weapon::EpeeCourte, 1)]);
    }

    #[test]
    fn ensorceleur_may_take_an_extra_mastered_weapon() {
        let character = Character::new()
            .with_profil(Profil::Ensorceleur)
            .with_item(Weapon::BatonFerre, 1)
            .with_item(Weapon::Dague, 1)
            .with_item(Weapon::ArbaleteLegere, 1);
        assert_eq!(leftovers(&character), []);
        let unmastered = character
            .with_item(Weapon::ArbaleteLegere, 0)
            .with_item(Weapon::EpeeLongue, 1);
        assert_eq!(leftovers(&unmastered), [weapon(Weapon::EpeeLongue, 1)]);
    }

    #[test]
    fn no_starting_equipment_without_a_profil() {
        let character = Character::new()
            .with_item(Weapon::Mousquet, 3)
            .with_armour(Armour::PlaqueComplete);
        assert_eq!(leftovers(&character), []);
    }

    #[test]
    fn every_profil_has_a_starting_equipment() {
        for profil in Profil::ALL {
            assert!(!starting_equipment(profil).is_empty());
        }
    }

    #[test]
    fn the_game_master_waives_a_trade_and_the_purse() {
        use co_kernel::{Author, Waiver};

        use crate::{
            Money,
            rules::{EQUIPMENT, PURSE},
        };

        let character = Character::new()
            .with_profil(Profil::Magicien)
            .with_item(Weapon::BatonFerre, 1)
            .with_item(Weapon::EpeeLongue, 1)
            .with_purse(Money::pa(15));
        let open = |character: &Character| -> Vec<Subject> {
            character
                .report()
                .open()
                .map(co_kernel::ReportEntry::violation)
                .filter(|v| v.rule() == &EQUIPMENT)
                .map(|v| v.subject().clone())
                .collect()
        };
        assert_eq!(
            open(&character),
            [
                Subject::Value(PURSE),
                Subject::Value(Weapon::EpeeLongue.id())
            ]
        );
        let gm = || Author::new("gm");
        let waived = character
            .with_waiver(Waiver::new(
                EQUIPMENT,
                Subject::Value(Weapon::EpeeLongue.id()),
                gm(),
                "traded for the grimoire",
            ))
            .with_waiver(Waiver::new(
                EQUIPMENT,
                Subject::Value(PURSE),
                gm(),
                "inheritance",
            ));
        assert_eq!(open(&waived), []);
        assert_eq!(waived.report().unused_waivers(), []);
    }

    #[test]
    fn huge_quantities_do_not_overflow() {
        let character = Character::new()
            .with_profil(Profil::Voleur)
            .with_item(Weapon::Dague, u32::MAX);
        assert_eq!(
            leftovers(&character),
            [weapon(Weapon::Dague, i64::from(u32::MAX) - 5)]
        );
    }
}
