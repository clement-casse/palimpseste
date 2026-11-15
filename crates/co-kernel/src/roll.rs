//! Rolls resolved from the faces their dice showed (ADR KER-ROLLS): what
//! both rulesets share of a test (CO.RES.TEST, CO.RES.CRIT-NATURAL). The
//! kernel draws nothing: whoever rolls gives the faces.

use crate::{Dice, Die};

/// The die of a test.
const D20: Die = Die::new(20);

/// A die and the face it showed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rolled {
    die: Die,
    face: u8,
}

impl Rolled {
    /// Creates the result of `die` showing `face`.
    #[must_use]
    pub const fn new(die: Die, face: u8) -> Self {
        Self { die, face }
    }

    /// Returns the die rolled.
    #[must_use]
    pub const fn die(self) -> Die {
        self.die
    }

    /// Returns the face shown.
    #[must_use]
    pub const fn face(self) -> u8 {
        self.face
    }

    /// Returns the sum of the faces `rolled` showed.
    #[must_use]
    pub fn sum(rolled: &[Self]) -> i32 {
        rolled.iter().map(|r| i32::from(r.face)).sum()
    }

    /// Whether `rolled` are the results of `dice`, die for die and in
    /// order.
    #[must_use]
    pub fn are_of(rolled: &[Self], dice: &Dice) -> bool {
        rolled.iter().map(|r| r.die).eq(dice.dice().iter().copied())
    }
}

/// How the die of a test is rolled: alone, or with a second die, keeping
/// the higher (dé bonus) or the lower (dé malus).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollMode {
    /// One d20.
    #[default]
    Plain,
    /// Two d20, the higher kept.
    BonusDie,
    /// Two d20, the lower kept.
    MalusDie,
}

impl RollMode {
    /// Returns the dice to roll for a test in this mode.
    #[must_use]
    pub fn dice(self) -> Dice {
        match self {
            Self::Plain => Dice::new([D20]),
            Self::BonusDie | Self::MalusDie => Dice::new([D20, D20]),
        }
    }
}

/// A natural result of the die kept (CO.RES.CRIT-NATURAL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Natural {
    /// The die showed 20.
    Twenty,
    /// The die showed 1.
    One,
}

/// The dice of a test once rolled: the die kept, and the one set aside by a
/// dé bonus or malus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestRoll {
    kept: Rolled,
    discarded: Option<Rolled>,
}

impl TestRoll {
    /// Reads the dice of a test rolled in `mode`; `None` when `rolled` are
    /// not the dice of [`RollMode::dice`]. Of two equal dice, the first is
    /// kept.
    #[must_use]
    pub fn new(mode: RollMode, rolled: &[Rolled]) -> Option<Self> {
        if !Rolled::are_of(rolled, &mode.dice()) {
            return None;
        }
        let (&first, rest) = rolled.split_first()?;
        let Some(&second) = rest.first() else {
            return Some(Self {
                kept: first,
                discarded: None,
            });
        };
        let first_kept = match mode {
            RollMode::MalusDie => first.face <= second.face,
            RollMode::Plain | RollMode::BonusDie => first.face >= second.face,
        };
        let (kept, discarded) = if first_kept {
            (first, second)
        } else {
            (second, first)
        };
        Some(Self {
            kept,
            discarded: Some(discarded),
        })
    }

    /// Returns the die kept.
    #[must_use]
    pub const fn kept(self) -> Rolled {
        self.kept
    }

    /// Returns the die set aside, when two were rolled.
    #[must_use]
    pub const fn discarded(self) -> Option<Rolled> {
        self.discarded
    }

    /// Returns the natural 20 or 1 of the die kept, if any.
    #[must_use]
    pub const fn natural(self) -> Option<Natural> {
        match self.kept.face {
            20 => Some(Natural::Twenty),
            1 => Some(Natural::One),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d20(face: u8) -> Rolled {
        Rolled::new(D20, face)
    }

    #[test]
    fn a_plain_test_keeps_its_die() {
        let roll = TestRoll::new(RollMode::Plain, &[d20(12)]).unwrap();
        assert_eq!(roll.kept(), d20(12));
        assert_eq!(roll.discarded(), None);
        assert_eq!(roll.natural(), None);
    }

    #[test]
    fn a_bonus_die_keeps_the_higher_and_a_malus_die_the_lower() {
        let rolled = [d20(7), d20(15)];
        let bonus = TestRoll::new(RollMode::BonusDie, &rolled).unwrap();
        assert_eq!((bonus.kept(), bonus.discarded()), (d20(15), Some(d20(7))));
        let malus = TestRoll::new(RollMode::MalusDie, &rolled).unwrap();
        assert_eq!((malus.kept(), malus.discarded()), (d20(7), Some(d20(15))));
    }

    #[test]
    fn naturals_are_read_from_the_die_kept() {
        let twenty = TestRoll::new(RollMode::BonusDie, &[d20(20), d20(1)]).unwrap();
        assert_eq!(twenty.natural(), Some(Natural::Twenty));
        let one = TestRoll::new(RollMode::MalusDie, &[d20(20), d20(1)]).unwrap();
        assert_eq!(one.natural(), Some(Natural::One));
    }

    #[test]
    fn dice_of_another_roll_are_refused() {
        assert_eq!(TestRoll::new(RollMode::Plain, &[d20(3), d20(4)]), None);
        assert_eq!(TestRoll::new(RollMode::BonusDie, &[d20(3)]), None);
        let d6 = Rolled::new(Die::new(6), 3);
        assert_eq!(TestRoll::new(RollMode::Plain, &[d6]), None);
    }

    #[test]
    fn rolled_dice_sum_and_match_their_dice() {
        let rolled = [Rolled::new(Die::new(6), 4), Rolled::new(Die::new(4), 2)];
        assert_eq!(Rolled::sum(&rolled), 6);
        assert!(Rolled::are_of(&rolled, &Dice::new([6, 4])));
        assert!(!Rolled::are_of(&rolled, &Dice::new([4, 6])));
        assert!(!Rolled::are_of(&rolled, &Dice::new([6])));
    }
}
