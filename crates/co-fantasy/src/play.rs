//! A character in play (ADR ENG-IN-PLAY): what it wears and carries changes
//! during a session, its creation choices do not. The engine stores no play
//! state; it computes from the equipment it is given.

use std::collections::BTreeMap;

use co_kernel::{DerivedValue, Dice, Quantity, ViolationReport};

use crate::{Armour, BonusDie, Character, Item, Shield, Value, Weapon, WeaponUse, armour, weapon};

/// What a character wears and carries: an armour, a shield and items.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Equipment {
    pub(crate) armour: Option<Armour>,
    pub(crate) shield: Option<Shield>,
    pub(crate) items: BTreeMap<Item, Quantity>,
}

impl Equipment {
    /// Creates an equipment with nothing worn or carried.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts on `armour`, replacing any armour worn.
    #[must_use]
    pub const fn with_armour(mut self, armour: Armour) -> Self {
        self.armour = Some(armour);
        self
    }

    /// Takes up `shield`, replacing any shield carried.
    #[must_use]
    pub const fn with_shield(mut self, shield: Shield) -> Self {
        self.shield = Some(shield);
        self
    }

    /// Carries `quantity` of `item`, replacing any quantity already carried;
    /// a quantity of 0 gives the item up.
    #[must_use]
    pub fn with_item(mut self, item: impl Into<Item>, quantity: impl Into<Quantity>) -> Self {
        let item = item.into();
        let quantity = quantity.into();
        if quantity.get() == 0 {
            self.items.remove(&item);
        } else {
            self.items.insert(item, quantity);
        }
        self
    }
}

/// A character in play: its creation choices, adjustments and waivers, with
/// the equipment it wears and carries now in place of the one chosen at
/// creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InPlay {
    character: Character,
}

impl InPlay {
    pub(crate) const fn new(character: Character) -> Self {
        Self { character }
    }

    /// Returns `value` computed with the equipment in play, as
    /// [`Character::value`] does with the creation equipment.
    #[must_use]
    pub fn value(&self, value: Value) -> Option<DerivedValue> {
        self.character.value(value)
    }

    /// Returns the dice rolled for the DM of `weapon` in `weapon_use`, as
    /// [`Character::damage_dice`] does.
    #[must_use]
    pub fn damage_dice(&self, weapon: Weapon, weapon_use: WeaponUse) -> Option<Dice> {
        self.character.damage_dice(weapon, weapon_use)
    }

    /// Returns the dé bonus to tests of the capacités held, with the
    /// equipment in play, as [`Character::bonus_dice`] does.
    #[must_use]
    pub fn bonus_dice(&self) -> Vec<BonusDie> {
        self.character.bonus_dice()
    }

    /// Returns the findings of the rules that hold in play, matched against
    /// the character's waivers: an armour or shield beyond the profil's limit
    /// (COF.DER.ARMOUR-LIMIT, D-24), the arc long's FOR (COF.EQP.RANGED) and
    /// Petite taille (COF.CNT.PEUPLES, D-29). The starting equipment and the
    /// purse are creation rules, never checked in play.
    #[must_use]
    pub fn report(&self) -> ViolationReport {
        let mut violations = Vec::new();
        armour::check(&self.character, &mut violations);
        weapon::check(&self.character, &mut violations);
        ViolationReport::new(violations, self.character.waivers())
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Author, Characteristic, RuleId, Source, Subject, ValueId, Waiver};

    use super::*;
    use crate::{Money, Peuple, Profil, Series, Voie, Weapon};

    /// A barbarian (armour up to cuir renforcé) with FOR `strength`, in cuir
    /// simple with a grand bouclier, an épée longue and two javelots.
    fn barbarian(strength: i32) -> Character {
        Character::new()
            .with_profil(Profil::Barbare)
            .with_peuple(Peuple::Humain)
            .with_series(Series::Expert)
            .with_series_value(Characteristic::For, strength)
            .with_series_value(Characteristic::Agi, 1)
            .with_rank(Voie::Pourfendeur, 1)
            .with_armour(Armour::CuirSimple)
            .with_shield(Shield::GrandBouclier)
            .with_item(Weapon::EpeeLongue, 1)
            .with_item(Weapon::Javelot, 2)
            .with_purse(Money::pa(7))
    }

    fn rules(report: &ViolationReport) -> Vec<(String, Subject, bool)> {
        report
            .entries()
            .iter()
            .map(|entry| {
                let violation = entry.violation();
                let rule = violation.rule().as_str().to_owned();
                (rule, violation.subject().clone(), entry.is_waived())
            })
            .collect()
    }

    #[test]
    fn values_follow_the_equipment_in_play() {
        let character = barbarian(3);
        let creation = character.value(Value::Defence).map(|v| v.total());
        let played = character.in_play(Equipment::new().with_armour(Armour::CuirRenforce));
        let defence = played.value(Value::Defence).unwrap();
        // Cuir renforcé +3 instead of cuir simple +2 and grand bouclier +2.
        assert_eq!(Some(defence.total() + 1), creation);
        assert!(
            defence
                .contributions()
                .iter()
                .any(|c| c.source() == &Source::Value(Armour::CuirRenforce.id()))
        );
    }

    #[test]
    fn dice_and_dice_bonuses_follow_the_equipment_in_play() {
        // A priest in sacred clothes, with Bâton de mage.
        let character = Character::new()
            .with_rank(Voie::Spiritualite, 1)
            .with_rank(Voie::Artefacts, 1);
        let unarmoured = character.in_play(Equipment::new());
        assert_eq!(unarmoured.bonus_dice().len(), 1);
        assert_eq!(
            unarmoured.damage_dice(Weapon::BatonFerre, WeaponUse::TwoHanded),
            character.damage_dice(Weapon::BatonFerre, WeaponUse::TwoHanded)
        );
        let armoured = character.in_play(Equipment::new().with_armour(Armour::CuirSimple));
        assert_eq!(armoured.bonus_dice(), []);
    }

    #[test]
    fn only_the_rules_holding_in_play_are_checked() {
        let character = barbarian(0);
        let played = character.in_play(
            Equipment::new()
                .with_armour(Armour::CotteDeMailles)
                .with_item(Weapon::ArcLong, 1)
                .with_item("corde", 1),
        );
        let found = rules(&played.report());
        assert!(found.contains(&(
            "COF.DER.ARMOUR-LIMIT".to_owned(),
            Subject::Value(ValueId::new("cof.armour")),
            false
        )));
        assert!(found.contains(&(
            "COF.EQP.RANGED".to_owned(),
            Subject::Value(Weapon::ArcLong.id()),
            false
        )));
        // The arc long is no starting equipment of the barbarian, and stays
        // a creation matter.
        assert!(found.iter().all(|(rule, _, _)| rule != "COF.CRE.EQUIPMENT"));
    }

    #[test]
    fn creation_waivers_still_apply_in_play() {
        let waiver = Waiver::new(
            RuleId::new("COF.EQP.RANGED"),
            Subject::Value(Weapon::ArcLong.id()),
            Author::new("Ana"),
            "Arc de famille",
        );
        let played = barbarian(0)
            .with_waiver(waiver)
            .in_play(Equipment::new().with_item(Weapon::ArcLong, 1));
        assert_eq!(
            rules(&played.report()),
            [(
                "COF.EQP.RANGED".to_owned(),
                Subject::Value(Weapon::ArcLong.id()),
                true
            )]
        );
    }
}
