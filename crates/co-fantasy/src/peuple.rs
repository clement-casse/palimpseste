//! Peuples (COF.CRE.PEUPLE, COF.CNT.PEUPLES).

use co_kernel::{Characteristic, Subject, ValueId, Violation};

use crate::{
    Character, Voie,
    rules::{PEUPLE, PEUPLE_MECHANIC, PEUPLE_VALUE},
};

/// One of the 8 peuples of COF2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Peuple {
    /// Demi-elfe.
    DemiElfe,
    /// Demi-orc.
    DemiOrc,
    /// Elfe haut.
    ElfeHaut,
    /// Elfe sylvain.
    ElfeSylvain,
    /// Gnome.
    Gnome,
    /// Halfelin.
    Halfelin,
    /// Humain.
    Humain,
    /// Nain.
    Nain,
}

impl Peuple {
    /// Every peuple, in declaration order.
    pub const ALL: [Self; 8] = [
        Self::DemiElfe,
        Self::DemiOrc,
        Self::ElfeHaut,
        Self::ElfeSylvain,
        Self::Gnome,
        Self::Halfelin,
        Self::Humain,
        Self::Nain,
    ];

    /// Returns the peuple whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|peuple| peuple.id().as_str() == id)
    }

    /// The French display name of the peuple.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DemiElfe => "Demi-elfe",
            Self::DemiOrc => "Demi-orc",
            Self::ElfeHaut => "Elfe haut",
            Self::ElfeSylvain => "Elfe sylvain",
            Self::Gnome => "Gnome",
            Self::Halfelin => "Halfelin",
            Self::Humain => "Humain",
            Self::Nain => "Nain",
        }
    }

    /// Returns the kernel identifier of the peuple.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::DemiElfe => "cof.peuple.demi-elfe",
            Self::DemiOrc => "cof.peuple.demi-orc",
            Self::ElfeHaut => "cof.peuple.elfe-haut",
            Self::ElfeSylvain => "cof.peuple.elfe-sylvain",
            Self::Gnome => "cof.peuple.gnome",
            Self::Halfelin => "cof.peuple.halfelin",
            Self::Humain => "cof.peuple.humain",
            Self::Nain => "cof.peuple.nain",
        })
    }

    /// Whether the peuple is human, for the bound of the characteristics.
    pub(crate) const fn is_human(self) -> bool {
        matches!(self, Self::Humain)
    }

    /// The characteristics the peuple may raise by 1. A humain may raise any
    /// of its two lowest (D-15), checked on the assigned values.
    #[must_use]
    pub const fn bonus_options(self) -> &'static [Characteristic] {
        use Characteristic::{Agi, Cha, Con, For, Int, Per, Vol};
        match self {
            Self::DemiElfe => &[Per, Cha],
            Self::DemiOrc => &[For, Con],
            Self::ElfeHaut => &[Int, Cha],
            Self::ElfeSylvain => &[Agi, Per],
            Self::Gnome => &[Int, Per],
            Self::Halfelin => &[Agi, Vol],
            Self::Humain => &Characteristic::ALL,
            Self::Nain => &[Con, Vol],
        }
    }

    /// The characteristics the peuple may lower by 1: none, one (applied
    /// without a choice), or a choice of two.
    #[must_use]
    pub const fn malus_options(self) -> &'static [Characteristic] {
        use Characteristic::{Agi, Cha, Con, For, Int};
        match self {
            Self::DemiElfe => &[For, Con],
            Self::DemiOrc => &[Cha, Int],
            Self::ElfeHaut | Self::ElfeSylvain | Self::Gnome | Self::Halfelin => &[For],
            Self::Humain => &[],
            Self::Nain => &[Agi],
        }
    }

    /// The voie de peuple; a demi-elfe has none of its own.
    #[must_use]
    pub const fn voie(self) -> Option<Voie> {
        match self {
            Self::DemiElfe => None,
            Self::DemiOrc => Some(Voie::DemiOrc),
            Self::ElfeHaut => Some(Voie::ElfeHaut),
            Self::ElfeSylvain => Some(Voie::ElfeSylvain),
            Self::Gnome => Some(Voie::Gnome),
            Self::Halfelin => Some(Voie::Halfelin),
            Self::Humain => Some(Voie::Humain),
            Self::Nain => Some(Voie::Nain),
        }
    }

    /// Whether `voie` may be the character's voie de peuple.
    pub(crate) fn allows_voie(self, voie: Voie) -> bool {
        match self.voie() {
            Some(own) => own == voie,
            None => matches!(voie, Voie::Humain | Voie::ElfeHaut | Voie::ElfeSylvain),
        }
    }
}

/// Checks that a peuple is chosen.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    if character.peuple().is_none() {
        violations.push(Violation::incomplete(
            PEUPLE,
            PEUPLE_MECHANIC,
            Subject::Value(PEUPLE_VALUE),
        ));
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    #[test]
    fn missing_peuple_is_incomplete() {
        let mut violations = Vec::new();
        check(&Character::new(), &mut violations);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule(), &PEUPLE);
        assert_eq!(violations[0].category(), Category::Incomplete);

        let mut violations = Vec::new();
        check(&Character::new().with_peuple(Peuple::Nain), &mut violations);
        assert_eq!(violations, []);
    }

    #[test]
    fn modifiers_follow_the_table() {
        use Characteristic::{Agi, Con, For, Int, Per, Vol};
        assert_eq!(Peuple::ElfeSylvain.bonus_options(), [Agi, Per]);
        assert_eq!(Peuple::Gnome.bonus_options(), [Int, Per]);
        assert_eq!(Peuple::Halfelin.bonus_options(), [Agi, Vol]);
        assert_eq!(Peuple::Nain.malus_options(), [Agi]);
        assert_eq!(Peuple::DemiElfe.malus_options(), [For, Con]);
        assert_eq!(Peuple::Humain.malus_options(), []);
    }

    #[test]
    fn demi_elfe_chooses_its_voie_de_peuple() {
        assert!(Peuple::DemiElfe.allows_voie(Voie::Humain));
        assert!(Peuple::DemiElfe.allows_voie(Voie::ElfeSylvain));
        assert!(!Peuple::DemiElfe.allows_voie(Voie::Nain));
        assert!(Peuple::Nain.allows_voie(Voie::Nain));
        assert!(!Peuple::Nain.allows_voie(Voie::Humain));
    }
}
