//! Armours and shields (COF.EQP.ARMOUR) and the profil's limits
//! (COF.DER.ARMOUR-LIMIT, D-24).

use co_kernel::{Rank, Severity, Subject, ValueId, Violation};

use crate::{
    Character, Profil, Voie, ability,
    rules::{ARMOUR, ARMOUR_LIMIT, ARMOUR_MECHANIC, SHIELD},
};

/// An armour, from the lightest to the heaviest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Armour {
    /// Tissus matelassés, fourrures.
    TissusMatelasses,
    /// Cuir simple.
    CuirSimple,
    /// Cuir renforcé, broigne.
    CuirRenforce,
    /// Chemise de mailles.
    ChemiseDeMailles,
    /// Cotte de mailles.
    CotteDeMailles,
    /// Armure de plaques.
    ArmureDePlaques,
    /// Plaque complète.
    PlaqueComplete,
}

impl Armour {
    /// Every armour, from the lightest to the heaviest.
    pub const ALL: [Self; 7] = [
        Self::TissusMatelasses,
        Self::CuirSimple,
        Self::CuirRenforce,
        Self::ChemiseDeMailles,
        Self::CotteDeMailles,
        Self::ArmureDePlaques,
        Self::PlaqueComplete,
    ];

    /// Returns the armour whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|armour| armour.id().as_str() == id)
    }

    /// The French display name of the armour.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TissusMatelasses => "Tissus matelassés, fourrures",
            Self::CuirSimple => "Cuir simple",
            Self::CuirRenforce => "Cuir renforcé, broigne",
            Self::ChemiseDeMailles => "Chemise de mailles",
            Self::CotteDeMailles => "Cotte de mailles",
            Self::ArmureDePlaques => "Armure de plaques",
            Self::PlaqueComplete => "Plaque complète",
        }
    }

    /// Returns the kernel identifier of the armour.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::TissusMatelasses => "cof.armour.tissus-matelasses",
            Self::CuirSimple => "cof.armour.cuir-simple",
            Self::CuirRenforce => "cof.armour.cuir-renforce",
            Self::ChemiseDeMailles => "cof.armour.chemise-de-mailles",
            Self::CotteDeMailles => "cof.armour.cotte-de-mailles",
            Self::ArmureDePlaques => "cof.armour.armure-de-plaques",
            Self::PlaqueComplete => "cof.armour.plaque-complete",
        })
    }

    /// The DEF bonus of the armour.
    #[must_use]
    pub const fn defence(self) -> i32 {
        match self {
            Self::TissusMatelasses => 1,
            Self::CuirSimple => 2,
            Self::CuirRenforce => 3,
            Self::ChemiseDeMailles => 4,
            Self::CotteDeMailles => 5,
            Self::ArmureDePlaques => 6,
            Self::PlaqueComplete => 7,
        }
    }

    /// The highest AGI value usable while wearing the armour (D-06). The
    /// table is calibrated so that AGI and armour never exceed +8 together.
    pub(crate) const fn agi_max(self) -> i32 {
        8 - self.defence()
    }
}

/// A shield, from the smallest to the largest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shield {
    /// Petit bouclier.
    PetitBouclier,
    /// Grand bouclier.
    GrandBouclier,
}

impl Shield {
    /// Every shield, from the smallest to the largest.
    pub const ALL: [Self; 2] = [Self::PetitBouclier, Self::GrandBouclier];

    /// Returns the shield whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|shield| shield.id().as_str() == id)
    }

    /// The French display name of the shield.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PetitBouclier => "Petit bouclier",
            Self::GrandBouclier => "Grand bouclier",
        }
    }

    /// Returns the kernel identifier of the shield.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::PetitBouclier => "cof.shield.petit-bouclier",
            Self::GrandBouclier => "cof.shield.grand-bouclier",
        })
    }

    /// The DEF bonus of the shield.
    #[must_use]
    pub const fn defence(self) -> i32 {
        match self {
            Self::PetitBouclier => 1,
            Self::GrandBouclier => 2,
        }
    }
}

/// The heaviest armour the character's capacités allow: the profil's limit,
/// raised to the plaque complète by the chevalier's Noblesse rank 2, and by
/// the capacités raising it (Briseur d'os, Armure lourde, Vêtements sacrés).
fn armour_limit(character: &Character, profil: Profil) -> Option<Armour> {
    let profil_limit =
        if profil == Profil::Chevalier && character.has_rank(Voie::Noblesse, Rank::new(2)) {
            Some(Armour::PlaqueComplete)
        } else {
            profil.armour_limit()
        };
    profil_limit.max(ability::armour_limit(character))
}

/// Checks the armour and shield against the profil's limits. Without a
/// profil, there is no limit to check.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let Some(profil) = character.profil() else {
        return;
    };
    let limit = armour_limit(character, profil);
    if let Some(armour) = character.armour()
        && limit.is_none_or(|limit| armour > limit)
    {
        violations.push(Violation::deviation(
            ARMOUR_LIMIT,
            ARMOUR_MECHANIC,
            Subject::Value(ARMOUR),
            Severity::Standard,
        ));
    }
    if let Some(shield) = character.shield()
        && profil.shield_limit().is_none_or(|limit| shield > limit)
    {
        violations.push(Violation::deviation(
            ARMOUR_LIMIT,
            ARMOUR_MECHANIC,
            Subject::Value(SHIELD),
            Severity::Standard,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn armour_violations(character: &Character) -> Vec<Subject> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
            .into_iter()
            .map(|v| v.subject().clone())
            .collect()
    }

    #[test]
    fn agi_cap_keeps_agi_and_armour_at_eight() {
        assert_eq!(Armour::TissusMatelasses.agi_max(), 7);
        assert_eq!(Armour::ChemiseDeMailles.agi_max(), 4);
        assert_eq!(Armour::PlaqueComplete.agi_max(), 1);
    }

    #[test]
    fn armour_within_the_limit_is_clean() {
        let character = Character::new()
            .with_profil(Profil::Barbare)
            .with_armour(Armour::CuirRenforce)
            .with_shield(Shield::GrandBouclier);
        assert_eq!(armour_violations(&character), []);
    }

    #[test]
    fn heavier_armour_and_forbidden_shield_deviate() {
        let character = Character::new()
            .with_profil(Profil::Voleur)
            .with_armour(Armour::CuirRenforce)
            .with_shield(Shield::PetitBouclier);
        assert_eq!(
            armour_violations(&character),
            [Subject::Value(ARMOUR), Subject::Value(SHIELD)]
        );
        let mage = Character::new()
            .with_profil(Profil::Magicien)
            .with_armour(Armour::TissusMatelasses);
        assert_eq!(armour_violations(&mage), [Subject::Value(ARMOUR)]);
        let druide = Character::new()
            .with_profil(Profil::Druide)
            .with_shield(Shield::GrandBouclier);
        assert_eq!(armour_violations(&druide), [Subject::Value(SHIELD)]);
    }

    #[test]
    fn plaque_complete_needs_noblesse_rank_2() {
        let chevalier = Character::new()
            .with_profil(Profil::Chevalier)
            .with_armour(Armour::PlaqueComplete);
        assert_eq!(armour_violations(&chevalier), [Subject::Value(ARMOUR)]);
        let noble = chevalier
            .with_rank(Voie::Noblesse, 1)
            .with_rank(Voie::Noblesse, 2);
        assert_eq!(armour_violations(&noble), []);
    }

    #[test]
    fn no_limit_without_a_profil() {
        let character = Character::new().with_armour(Armour::PlaqueComplete);
        assert_eq!(armour_violations(&character), []);
    }
}
