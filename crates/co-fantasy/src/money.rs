//! Money (COF.EQP.CURRENCY) and the purse of the starting equipment
//! (COF.CRE.EQUIPMENT, D-27).
//!
//! At creation the character holds its purse and spends nothing: spending
//! waits for the campaign (D-25).

use std::fmt;

use co_kernel::{DerivedValue, Dice, Severity, Source, Subject, Violation};

use crate::{
    Character,
    rules::{EQUIPMENT, EQUIPMENT_MECHANIC, PURSE},
};

/// An amount of money, counted in pièces de cuivre (pc).
///
/// Built from a coin with [`Money::pc`], [`Money::pa`], [`Money::po`] or
/// [`Money::pp`]; there is no conversion from a bare integer, which would
/// hide the coin. Negative amounts are representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(i32);

/// Pièces de cuivre in a pièce d'argent.
const PC_PER_PA: i32 = 10;
/// Pièces de cuivre in a pièce d'or.
const PC_PER_PO: i32 = 100;
/// Pièces de cuivre in a pièce de platine.
const PC_PER_PP: i32 = 1000;

impl Money {
    /// `amount` pièces de cuivre.
    #[must_use]
    pub const fn pc(amount: i32) -> Self {
        Self(amount)
    }

    /// `amount` pièces d'argent: 10 pc each.
    #[must_use]
    pub const fn pa(amount: i32) -> Self {
        Self(amount.saturating_mul(PC_PER_PA))
    }

    /// `amount` pièces d'or: 10 pa each.
    #[must_use]
    pub const fn po(amount: i32) -> Self {
        Self(amount.saturating_mul(PC_PER_PO))
    }

    /// `amount` pièces de platine: 10 po each.
    #[must_use]
    pub const fn pp(amount: i32) -> Self {
        Self(amount.saturating_mul(PC_PER_PP))
    }

    /// Returns the amount in pièces de cuivre.
    #[must_use]
    pub const fn in_pc(self) -> i32 {
        self.0
    }
}

/// Writes the amount with the fewest coins, largest first: `1 po 2 pa 5 pc`.
/// Zero is written `0 pc`.
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return f.write_str("0 pc");
        }
        if self.0 < 0 {
            f.write_str("-")?;
        }
        let mut rest = self.0.unsigned_abs();
        let mut first = true;
        for (coin, value) in [
            ("pp", PC_PER_PP),
            ("po", PC_PER_PO),
            ("pa", PC_PER_PA),
            ("pc", 1),
        ] {
            let value = value.unsigned_abs();
            let count = rest / value;
            rest %= value;
            if count > 0 {
                if !first {
                    f.write_str(" ")?;
                }
                write!(f, "{count} {coin}")?;
                first = false;
            }
        }
        Ok(())
    }
}

/// The lowest purse: 2 on 2d6, in pa.
const PURSE_MIN: Money = Money::pa(2);
/// The highest purse: 12 on 2d6, in pa.
const PURSE_MAX: Money = Money::pa(12);

/// The dice rolled for the purse of the starting equipment, counted in
/// pièces d'argent (COF.CRE.EQUIPMENT).
#[must_use]
pub fn purse_dice() -> Dice {
    Dice::new([6, 6])
}

/// The money held: the purse rolled at creation; `None` until it is rolled.
pub(crate) fn money(character: &Character) -> Option<DerivedValue> {
    character
        .purse()
        .map(|purse| DerivedValue::new().with(Source::Value(PURSE), purse.in_pc()))
}

/// Checks the purse: rolled (D-27), and a result of 2d6 pa.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let subject = || Subject::Value(PURSE);
    let Some(purse) = character.purse() else {
        violations.push(Violation::incomplete(
            EQUIPMENT,
            EQUIPMENT_MECHANIC,
            subject(),
        ));
        return;
    };
    let pc = purse.in_pc();
    if !(PURSE_MIN..=PURSE_MAX).contains(&purse) || pc % PC_PER_PA != 0 {
        violations.push(
            Violation::deviation(EQUIPMENT, EQUIPMENT_MECHANIC, subject(), Severity::Standard)
                .with_parameter("value", pc)
                .with_parameter("min", PURSE_MIN.in_pc())
                .with_parameter("max", PURSE_MAX.in_pc()),
        );
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    fn purse_violations(character: &Character) -> Vec<Category> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations.iter().map(Violation::category).collect()
    }

    #[test]
    fn coins_convert_to_copper() {
        assert_eq!(Money::pa(7).in_pc(), 70);
        assert_eq!(Money::po(1), Money::pa(10));
        assert_eq!(Money::pp(1), Money::po(10));
        assert_eq!(Money::pc(5).in_pc(), 5);
        assert_eq!(Money::pp(i32::MAX).in_pc(), i32::MAX);
    }

    #[test]
    fn display_uses_the_fewest_coins() {
        assert_eq!(Money::pa(7).to_string(), "7 pa");
        assert_eq!(Money::pc(125).to_string(), "1 po 2 pa 5 pc");
        assert_eq!(Money::pa(120).to_string(), "1 pp 2 po");
        assert_eq!(Money::pc(-15).to_string(), "-1 pa 5 pc");
        assert_eq!(Money::default().to_string(), "0 pc");
        assert_eq!(
            Money::pc(i32::MIN).to_string(),
            "-2147483 pp 6 po 4 pa 8 pc"
        );
    }

    #[test]
    fn the_purse_is_two_d6_pa() {
        assert_eq!(purse_dice(), Dice::new([6, 6]));
        for pa in [2, 7, 12] {
            let character = Character::new().with_purse(Money::pa(pa));
            assert_eq!(purse_violations(&character), []);
        }
    }

    #[test]
    fn missing_purse_is_incomplete() {
        assert_eq!(purse_violations(&Character::new()), [Category::Incomplete]);
        assert_eq!(money(&Character::new()), None);
    }

    #[test]
    fn purse_outside_the_dice_deviates() {
        for purse in [Money::pa(1), Money::pa(13), Money::pc(75), Money::pa(-2)] {
            let character = Character::new().with_purse(purse);
            assert_eq!(
                purse_violations(&character),
                [Category::Deviation(Severity::Standard)]
            );
        }
    }

    #[test]
    fn money_held_is_the_purse() {
        let character = Character::new().with_purse(Money::pa(7));
        let value = money(&character).map(|v| v.total());
        assert_eq!(value, Some(70));
    }
}
