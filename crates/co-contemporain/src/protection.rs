//! Protections (COC.EQP.PROTECTIONS): RD, and a malus to Init, AGI tests and
//! ATC. Protections never add DEF.

use co_kernel::{DerivedValue, Source, ValueId};

use crate::Character;

/// A protection of the protections table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Protection {
    /// Gilet rembourré.
    GiletRembourre,
    /// Gilet souple.
    GiletSouple,
    /// Gilet rigide.
    GiletRigide,
    /// Gilet tactique multi-menaces.
    GiletTactique,
    /// Gilet tactique multi-menaces with its ceramic plates, which double
    /// its RD and malus.
    GiletTactiqueAvecPlaques,
    /// Cotte de mailles.
    CotteDeMailles,
    /// Armure de plaques.
    ArmureDePlaques,
    /// Casque, modern or medieval: its RD adds to the armour's (D-04).
    Casque,
}

impl Protection {
    /// Returns the kernel identifier of the protection.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::GiletRembourre => "coc.protection.gilet-rembourre",
            Self::GiletSouple => "coc.protection.gilet-souple",
            Self::GiletRigide => "coc.protection.gilet-rigide",
            Self::GiletTactique => "coc.protection.gilet-tactique",
            Self::GiletTactiqueAvecPlaques => "coc.protection.gilet-tactique-avec-plaques",
            Self::CotteDeMailles => "coc.protection.cotte-de-mailles",
            Self::ArmureDePlaques => "coc.protection.armure-de-plaques",
            Self::Casque => "coc.protection.casque",
        })
    }

    /// The RD (réduction des dommages) of the protection.
    const fn damage_reduction(self) -> i32 {
        match self {
            Self::GiletRembourre => 2,
            Self::GiletSouple | Self::GiletTactique | Self::CotteDeMailles | Self::Casque => 4,
            Self::GiletRigide | Self::GiletTactiqueAvecPlaques | Self::ArmureDePlaques => 8,
        }
    }

    /// The malus to Init, AGI tests and ATC, as a negative amount.
    const fn malus(self) -> i32 {
        match self {
            Self::GiletRembourre => -1,
            Self::GiletSouple | Self::GiletTactique | Self::CotteDeMailles | Self::Casque => -2,
            Self::GiletRigide | Self::GiletTactiqueAvecPlaques | Self::ArmureDePlaques => -4,
        }
    }
}

/// RD = the sum of the protections worn.
pub(crate) fn damage_reduction(character: &Character) -> DerivedValue {
    character
        .protections()
        .fold(DerivedValue::new(), |value, protection| {
            value.with(
                Source::Value(protection.id()),
                protection.damage_reduction(),
            )
        })
}

/// Adds the malus of every protection worn to Init, ATC or AGI tests.
pub(crate) fn with_malus(value: DerivedValue, character: &Character) -> DerivedValue {
    character.protections().fold(value, |value, protection| {
        value.with(Source::Value(protection.id()), protection.malus())
    })
}

#[cfg(test)]
mod tests {
    use co_kernel::Characteristic;

    use super::*;
    use crate::{AttackMode, Value};

    fn total(character: &Character, value: Value) -> Option<i32> {
        character.value(value).map(|v| v.total())
    }

    #[test]
    fn helmet_adds_to_the_armour() {
        let character = Character::new()
            .with_protection(Protection::GiletRigide)
            .with_protection(Protection::Casque);
        assert_eq!(total(&character, Value::DamageReduction), Some(12));
        assert_eq!(total(&character, Value::AgiTestModifier), Some(-6));
        assert_eq!(total(&Character::new(), Value::DamageReduction), Some(0));
    }

    #[test]
    fn ceramic_plates_double_rd_and_malus() {
        let soft = Character::new().with_protection(Protection::GiletTactique);
        let plates = Character::new().with_protection(Protection::GiletTactiqueAvecPlaques);
        assert_eq!(total(&soft, Value::DamageReduction), Some(4));
        assert_eq!(total(&plates, Value::DamageReduction), Some(8));
        assert_eq!(total(&plates, Value::AgiTestModifier), Some(-4));
    }

    #[test]
    fn malus_lowers_initiative_and_contact_attacks_only() {
        let character = Character::new()
            .with_carac_points(Characteristic::Agi, 2)
            .with_protection(Protection::CotteDeMailles);
        assert_eq!(total(&character, Value::Initiative), Some(8));
        assert_eq!(
            total(&character, Value::Attack(AttackMode::Unarmed)),
            Some(-2)
        );
        assert_eq!(
            total(&character, Value::Attack(AttackMode::ContactWeapon)),
            Some(-2)
        );
        assert_eq!(
            total(&character, Value::Attack(AttackMode::RangedWeapon)),
            Some(2)
        );
        // D-05: AGI itself, and so DEF, are unchanged.
        assert_eq!(total(&character, Value::Defence), Some(12));
        assert_eq!(character.characteristics()[Characteristic::Agi].get(), 2);
    }
}
