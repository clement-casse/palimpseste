//! Items carried and the profil's starting equipment (COF.CRE.EQUIPMENT,
//! COF.CNT.PROFILS, D-26, D-28).

use std::collections::BTreeMap;

use co_kernel::{ItemName, Severity, Subject, ValueId, Violation};

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

/// An item the starting equipment covers: a weapon carried, the armour worn
/// or the shield held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gear {
    /// A weapon carried.
    Weapon(Weapon),
    /// The armour worn.
    Armour(Armour),
    /// The shield held.
    Shield(Shield),
}

impl Gear {
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
    /// The gear the pattern accepts, in gear order.
    fn options(self) -> Vec<Gear> {
        match self {
            Self::Is(weapon) => vec![Gear::Weapon(weapon)],
            Self::OneOf(weapons) => weapons.iter().copied().map(Gear::Weapon).collect(),
            Self::Matching(predicate) => Weapon::ALL
                .into_iter()
                .filter(|weapon| predicate(*weapon))
                .map(Gear::Weapon)
                .collect(),
            Self::Wearing(armour) => vec![Gear::Armour(armour)],
            Self::Holding(shield) => vec![Gear::Shield(shield)],
        }
    }

    fn matches(self, unit: Gear) -> bool {
        match (self, unit) {
            (Self::Is(expected), Gear::Weapon(weapon)) => weapon == expected,
            (Self::OneOf(weapons), Gear::Weapon(weapon)) => weapons.contains(&weapon),
            (Self::Matching(predicate), Gear::Weapon(weapon)) => predicate(weapon),
            (Self::Wearing(expected), Gear::Armour(armour)) => armour == expected,
            (Self::Holding(expected), Gear::Shield(shield)) => shield == expected,
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

/// An item of the starting equipment: one gear among its options, given
/// `quantity` times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingItem {
    options: Vec<Gear>,
    quantity: u32,
}

impl StartingItem {
    /// The gear the item may be: a single one, or a choice.
    #[must_use]
    pub fn options(&self) -> &[Gear] {
        &self.options
    }

    /// How many of the item are given.
    #[must_use]
    pub const fn quantity(&self) -> u32 {
        self.quantity
    }
}

/// A line of the starting equipment: one of its bundles, each bundle giving
/// its items together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingLine {
    bundles: Vec<Vec<StartingItem>>,
}

impl StartingLine {
    /// The bundles to choose from; most lines have a single one.
    #[must_use]
    pub fn bundles(&self) -> &[Vec<StartingItem>] {
        &self.bundles
    }
}

impl Profil {
    /// The profil's starting equipment, line by line (COF.CNT.PROFILS),
    /// besides the adventurer's bag and the purse. Free text items are not
    /// listed: they are never checked (D-26).
    #[must_use]
    pub fn starting_equipment(self) -> Vec<StartingLine> {
        slots(self)
            .iter()
            .map(|slot| StartingLine {
                bundles: slot
                    .iter()
                    .map(|bundle| {
                        bundle
                            .iter()
                            .map(|&(pattern, quantity)| StartingItem {
                                options: pattern.options(),
                                quantity,
                            })
                            .collect()
                    })
                    .collect(),
            })
            .collect()
    }
}

/// The profil's starting equipment, besides the adventurer's bag and the
/// purse. Free text items (instrument, crochets, grimoire...) are not
/// listed: they are never checked (D-26).
const fn slots(profil: Profil) -> &'static [Slot] {
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
fn take(bundle: Bundle, remaining: &BTreeMap<Gear, u32>) -> BTreeMap<Gear, u32> {
    let mut taken: BTreeMap<Gear, u32> = BTreeMap::new();
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

fn units(taken: &BTreeMap<Gear, u32>) -> u64 {
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
    let mut remaining: BTreeMap<Gear, u32> = character
        .weapons_carried()
        .map(|(weapon, quantity)| (Gear::Weapon(weapon), quantity.get()))
        .collect();
    remaining.extend(character.armour().map(|armour| (Gear::Armour(armour), 1)));
    remaining.extend(character.shield().map(|shield| (Gear::Shield(shield), 1)));

    for slot in slots(profil) {
        let mut best: Option<BTreeMap<Gear, u32>> = None;
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
    fn starting_equipment_lists_each_line() {
        let barbare = Profil::Barbare.starting_equipment();
        assert_eq!(barbare.len(), 2);
        let first = &barbare[0].bundles()[0];
        assert_eq!(first[0].options(), [Gear::Weapon(Weapon::Javelot)]);
        assert_eq!(first[0].quantity(), 2);
        let choice = &barbare[1].bundles()[1];
        assert_eq!(
            choice[0].options(),
            [
                Gear::Weapon(Weapon::EpeeBatarde),
                Gear::Weapon(Weapon::EpeeLongue),
                Gear::Weapon(Weapon::Hache),
            ]
        );
        assert_eq!(choice[1].options(), [Gear::Shield(Shield::GrandBouclier)]);
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
            assert!(!slots(profil).is_empty());
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
