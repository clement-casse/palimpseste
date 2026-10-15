//! Dice to roll, described but never rolled.

use std::fmt;

/// A die, by its number of faces.
///
/// The width limits storage only: any number of faces is representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Die(u8);

impl Die {
    /// Creates a die with `faces` faces.
    #[must_use]
    pub const fn new(faces: u8) -> Self {
        Self(faces)
    }

    /// Returns the number of faces.
    #[must_use]
    pub const fn faces(self) -> u8 {
        self.0
    }
}

impl From<u8> for Die {
    fn from(faces: u8) -> Self {
        Self(faces)
    }
}

/// The dice rolled together for one roll, such as `2d6`: a list of dice.
///
/// Dice of different sizes can be mixed. Flat bonuses are not dice: they
/// belong to the breakdown of a derived value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Dice(Vec<Die>);

impl Dice {
    /// Creates the list of `dice`, in the given order.
    #[must_use]
    pub fn new(dice: impl IntoIterator<Item = impl Into<Die>>) -> Self {
        Self(dice.into_iter().map(Into::into).collect())
    }

    /// Returns the dice, in the order they were given.
    #[must_use]
    pub fn dice(&self) -> &[Die] {
        &self.0
    }

    /// Returns the highest result the dice can give together.
    #[must_use]
    pub fn highest(&self) -> i32 {
        self.0.iter().map(|die| i32::from(die.faces())).sum()
    }
}

impl FromIterator<Die> for Dice {
    fn from_iter<I: IntoIterator<Item = Die>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Writes runs of identical dice as `NdF`, joined by `+`: `2d6`,
/// `1d6+1d4`. No dice is written `0`.
impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("0");
        }
        let mut runs = self.0.chunk_by(|a, b| a == b).peekable();
        while let Some(run) = runs.next() {
            write!(f, "{}d{}", run.len(), run[0].faces())?;
            if runs.peek().is_some() {
                f.write_str("+")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_keep_their_order() {
        let dice = Dice::new([6, 4, 6]);
        assert_eq!(dice.dice(), [Die::new(6), Die::new(4), Die::new(6)]);
        let collected: Dice = [Die::new(6), Die::new(4), Die::new(6)]
            .into_iter()
            .collect();
        assert_eq!(collected, dice);
    }

    #[test]
    fn highest_adds_every_face() {
        assert_eq!(Dice::new([6, 6]).highest(), 12);
        assert_eq!(Dice::new([8]).highest(), 8);
        assert_eq!(Dice::default().highest(), 0);
    }

    #[test]
    fn display_groups_identical_dice() {
        assert_eq!(Dice::new([8]).to_string(), "1d8");
        assert_eq!(Dice::new([6, 6]).to_string(), "2d6");
        assert_eq!(Dice::new([6, 4]).to_string(), "1d6+1d4");
        assert_eq!(Dice::new([6, 4, 6]).to_string(), "1d6+1d4+1d6");
        assert_eq!(Dice::default().to_string(), "0");
    }

    #[test]
    fn any_number_of_faces_is_representable() {
        assert_eq!(Dice::new([0, 255]).highest(), 255);
    }
}
