//! The seven characteristics and their values.

use std::ops::Index;

/// One of the seven characteristics shared by every engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Characteristic {
    /// Agilité (AGI).
    Agi,
    /// Constitution (CON).
    Con,
    /// Force (FOR).
    For,
    /// Perception (PER).
    Per,
    /// Charisme (CHA).
    Cha,
    /// Intelligence (INT).
    Int,
    /// Volonté (VOL).
    Vol,
}

impl Characteristic {
    /// Every characteristic, in the stable order used for iteration.
    pub const ALL: [Self; 7] = [
        Self::Agi,
        Self::Con,
        Self::For,
        Self::Per,
        Self::Cha,
        Self::Int,
        Self::Vol,
    ];

    /// The abbreviation the books use: `AGI`, `CON`, `FOR`, `PER`, `CHA`,
    /// `INT`, `VOL`.
    #[must_use]
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::Agi => "AGI",
            Self::Con => "CON",
            Self::For => "FOR",
            Self::Per => "PER",
            Self::Cha => "CHA",
            Self::Int => "INT",
            Self::Vol => "VOL",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Agi => 0,
            Self::Con => 1,
            Self::For => 2,
            Self::Per => 3,
            Self::Cha => 4,
            Self::Int => 5,
            Self::Vol => 6,
        }
    }
}

/// The value of a characteristic, which is also its modifier.
///
/// Signed and unbounded: game bounds are engine rules checked by validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CharacteristicValue(i32);

impl CharacteristicValue {
    /// Creates a value.
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    /// Returns the value.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for CharacteristicValue {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<CharacteristicValue> for i32 {
    fn from(value: CharacteristicValue) -> Self {
        value.0
    }
}

/// A value for each of the seven characteristics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Characteristics([CharacteristicValue; 7]);

impl Characteristics {
    /// Builds the values by calling `f` on each characteristic, in
    /// [`Characteristic::ALL`] order.
    #[must_use]
    pub fn from_fn(f: impl FnMut(Characteristic) -> CharacteristicValue) -> Self {
        Self(Characteristic::ALL.map(f))
    }

    /// Returns the value of `characteristic`.
    #[must_use]
    pub fn get(&self, characteristic: Characteristic) -> CharacteristicValue {
        self[characteristic]
    }

    /// Sets the value of `characteristic`.
    pub fn set(&mut self, characteristic: Characteristic, value: CharacteristicValue) {
        self.0[characteristic.index()] = value;
    }

    /// Iterates over every characteristic and its value, in [`Characteristic::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (Characteristic, CharacteristicValue)> + '_ {
        Characteristic::ALL.into_iter().map(|c| (c, self.get(c)))
    }
}

impl Index<Characteristic> for Characteristics {
    type Output = CharacteristicValue;

    fn index(&self, characteristic: Characteristic) -> &CharacteristicValue {
        &self.0[characteristic.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lists_the_seven_characteristics_in_order() {
        assert_eq!(
            Characteristic::ALL,
            [
                Characteristic::Agi,
                Characteristic::Con,
                Characteristic::For,
                Characteristic::Per,
                Characteristic::Cha,
                Characteristic::Int,
                Characteristic::Vol,
            ]
        );
    }

    #[test]
    fn abbreviations_are_the_books() {
        let abbreviations = Characteristic::ALL.map(Characteristic::abbreviation);
        assert_eq!(
            abbreviations,
            ["AGI", "CON", "FOR", "PER", "CHA", "INT", "VOL"]
        );
    }

    #[test]
    fn default_values_are_zero() {
        let characteristics = Characteristics::default();
        for c in Characteristic::ALL {
            assert_eq!(characteristics.get(c), CharacteristicValue::new(0));
        }
    }

    #[test]
    fn set_then_get_touches_only_that_characteristic() {
        let mut characteristics = Characteristics::default();
        characteristics.set(Characteristic::Per, CharacteristicValue::new(2));
        assert_eq!(characteristics.get(Characteristic::Per).get(), 2);
        assert_eq!(characteristics.get(Characteristic::Cha).get(), 0);
    }

    #[test]
    fn out_of_bound_values_are_representable() {
        let mut characteristics = Characteristics::default();
        characteristics.set(Characteristic::For, CharacteristicValue::new(-50));
        characteristics.set(Characteristic::Int, CharacteristicValue::new(99));
        assert_eq!(characteristics.get(Characteristic::For).get(), -50);
        assert_eq!(characteristics.get(Characteristic::Int).get(), 99);
    }

    #[test]
    fn iter_follows_all_order() {
        let mut characteristics = Characteristics::default();
        characteristics.set(Characteristic::Vol, CharacteristicValue::new(1));
        let pairs: Vec<_> = characteristics.iter().collect();
        assert_eq!(pairs.len(), 7);
        assert_eq!(pairs[0], (Characteristic::Agi, CharacteristicValue::new(0)));
        assert_eq!(pairs[6], (Characteristic::Vol, CharacteristicValue::new(1)));
    }

    #[test]
    fn from_fn_fills_each_characteristic_with_its_own_value() {
        let distinct = |c: Characteristic| match c {
            Characteristic::Agi => 1,
            Characteristic::Con => 2,
            Characteristic::For => 3,
            Characteristic::Per => 4,
            Characteristic::Cha => 5,
            Characteristic::Int => 6,
            Characteristic::Vol => 7,
        };
        let characteristics = Characteristics::from_fn(|c| distinct(c).into());
        for c in Characteristic::ALL {
            assert_eq!(i32::from(characteristics.get(c)), distinct(c));
        }
    }

    #[test]
    fn index_and_get_agree() {
        let mut characteristics = Characteristics::default();
        characteristics.set(Characteristic::Cha, 3.into());
        for c in Characteristic::ALL {
            assert_eq!(characteristics[c], characteristics.get(c));
        }
        assert_eq!(characteristics[Characteristic::Cha].get(), 3);
    }

    #[test]
    fn value_converts_to_and_from_i32() {
        let value = CharacteristicValue::from(-2);
        assert_eq!(value, CharacteristicValue::new(-2));
        assert_eq!(i32::from(value), -2);
    }
}
