//! A character in play (ADR ENG-IN-PLAY): what it wears and carries changes
//! during a session, its creation choices do not. The engine stores no play
//! state; it computes from the equipment it is given. No COC2 rule about
//! equipment holds in play: the starting equipment is a creation matter.

use std::collections::{BTreeMap, BTreeSet};

use co_kernel::{DerivedValue, Dice, Quantity};

use crate::{Character, Item, Protection, Situational, Target, Value, Weapon};

/// What a character wears and carries: protections and items.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Equipment {
    pub(crate) protections: BTreeSet<Protection>,
    pub(crate) items: BTreeMap<Item, Quantity>,
}

impl Equipment {
    /// Creates an equipment with nothing worn or carried.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts on `protection`; a helmet is worn with any other protection.
    #[must_use]
    pub fn with_protection(mut self, protection: Protection) -> Self {
        self.protections.insert(protection);
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

    /// Returns the capacités' modifiers to `target` that hold only in a
    /// situation (ADR ENG-SITUATIONAL).
    #[must_use]
    pub fn situational(&self, target: Target) -> Vec<Situational> {
        self.character.situational(target)
    }

    /// Returns the dice rolled for the DM of `weapon`, as
    /// [`Character::damage_dice`] does.
    #[must_use]
    pub fn damage_dice(&self, weapon: Weapon) -> Dice {
        self.character.damage_dice(weapon)
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Characteristic, Source};

    use super::*;

    fn total(value: Option<DerivedValue>) -> Option<i32> {
        value.map(|v| v.total())
    }

    #[test]
    fn values_follow_the_equipment_in_play() {
        let character = Character::new()
            .with_age(32)
            .with_carac_points(Characteristic::Per, 2)
            .with_item(Weapon::Couteau, 1);
        let played = character.in_play(
            Equipment::new()
                .with_protection(Protection::GiletSouple)
                .with_item(Weapon::Fleuret, 1),
        );

        let reduction = played.value(Value::DamageReduction).unwrap();
        assert_eq!(reduction.total(), 4);
        assert!(
            reduction
                .contributions()
                .iter()
                .any(|c| c.source() == &Source::Value(Protection::GiletSouple.id()))
        );
        assert_eq!(
            total(played.value(Value::Initiative)),
            total(character.value(Value::Initiative)).map(|init| init - 2)
        );
        assert_eq!(played.damage_dice(Weapon::Fleuret), Dice::new([8]));
        // The creation character is unchanged.
        assert_eq!(total(character.value(Value::DamageReduction)), Some(0));
    }
}
