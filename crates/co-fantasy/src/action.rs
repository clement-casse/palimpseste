//! The action each capacité takes to use, and the PM a spell costs
//! (COF.CNT.VOIE).

use co_kernel::Rank;

use crate::Voie;

/// The action a capacité takes to use, as the catalogue marks it; a
/// passive capacité takes none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    /// Action limitée (L).
    Limited,
    /// Action d'attaque (A).
    Attack,
    /// Action de mouvement (M).
    Movement,
    /// Action gratuite (G).
    Free,
}

impl Action {
    /// Every action, in the book's order.
    pub const ALL: [Self; 4] = [Self::Limited, Self::Attack, Self::Movement, Self::Free];

    /// The French display name, such as `Action limitée`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Limited => "Action limitée",
            Self::Attack => "Action d'attaque",
            Self::Movement => "Action de mouvement",
            Self::Free => "Action gratuite",
        }
    }

    /// The letter the book marks it with, such as `L`.
    #[must_use]
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::Limited => "L",
            Self::Attack => "A",
            Self::Movement => "M",
            Self::Free => "G",
        }
    }
}

/// The actions the capacité at `rank` in `voie` may take, the first
/// usual: empty when passive or outside ranks 1 to 5.
pub(crate) fn of(voie: Voie, rank: Rank) -> &'static [Action] {
    let index = usize::from(rank.get()).wrapping_sub(1);
    ACTIONS
        .iter()
        .find(|(other, _)| *other == voie)
        .and_then(|(_, actions)| actions.get(index).copied())
        .unwrap_or(&[])
}

use Action::{Attack, Free, Limited, Movement};

/// The actions of the capacités, by voie and rank, as the co-rules
/// catalogue marks them; two when the catalogue allows either.
const ACTIONS: [(Voie, [&[Action]; 5]); 78] = [
    (Voie::Artilleur, [&[], &[], &[Limited], &[], &[Limited]]),
    (
        Voie::Explosifs,
        [&[Limited], &[], &[], &[Limited], &[Limited]],
    ),
    (Voie::Mercenaire, [&[], &[Limited], &[], &[], &[]]),
    (Voie::Pistolero, [&[], &[], &[Limited], &[], &[]]),
    (
        Voie::Precision,
        [&[], &[Attack], &[], &[Limited], &[Limited]],
    ),
    (
        Voie::Escrime,
        [&[], &[Limited], &[Movement], &[Limited], &[]],
    ),
    (
        Voie::Musicien,
        [&[Limited], &[Limited], &[Attack], &[Attack], &[Attack]],
    ),
    (Voie::Saltimbanque, [&[], &[], &[Free], &[], &[Free]]),
    (Voie::Seduction, [&[], &[], &[], &[], &[Attack]]),
    (Voie::Vagabond, [&[], &[], &[], &[Attack], &[Attack]]),
    (Voie::Archer, [&[], &[], &[], &[Limited], &[Limited]]),
    (Voie::CompagnonAnimal, [&[], &[], &[Limited], &[], &[]]),
    (Voie::Survie, [&[], &[], &[Free], &[], &[Limited]]),
    (Voie::Traqueur, [&[], &[Limited], &[], &[], &[Limited]]),
    (Voie::CombatADeuxArmes, [&[Free], &[], &[Free], &[], &[]]),
    (
        Voie::Assassin,
        [&[], &[Limited], &[Attack], &[Movement], &[Limited]],
    ),
    (Voie::Aventurier, [&[], &[Limited], &[], &[], &[Limited]]),
    (Voie::Deplacement, [&[], &[], &[Free], &[], &[Free]]),
    (Voie::Roublard, [&[], &[], &[Free], &[], &[]]),
    (Voie::Spadassin, [&[], &[Free], &[], &[Free], &[]]),
    (Voie::Brute, [&[], &[Free], &[Limited], &[], &[]]),
    (Voie::Pagne, [&[], &[], &[], &[], &[]]),
    (
        Voie::Pourfendeur,
        [&[], &[Limited], &[], &[Limited], &[Limited]],
    ),
    (Voie::Primitif, [&[], &[], &[], &[Free], &[]]),
    (Voie::Rage, [&[Free], &[], &[Limited], &[], &[Limited]]),
    (Voie::Cavalier, [&[], &[], &[Limited], &[], &[]]),
    (Voie::Guerre, [&[], &[Movement], &[Limited], &[], &[Free]]),
    (Voie::Preux, [&[Free], &[], &[], &[], &[]]),
    (
        Voie::MeneurDHomme,
        [&[], &[Free], &[Free], &[Free], &[Free]],
    ),
    (Voie::Noblesse, [&[], &[], &[], &[], &[]]),
    (Voie::Bouclier, [&[Free], &[Movement], &[], &[], &[Free]]),
    (Voie::Combat, [&[], &[], &[], &[Limited], &[Limited]]),
    (Voie::MaitreDArmes, [&[], &[], &[], &[Limited], &[Free]]),
    (Voie::Resistance, [&[], &[], &[], &[], &[]]),
    (Voie::Soldat, [&[Free], &[Free], &[Free], &[], &[]]),
    (
        Voie::Air,
        [&[Free], &[Movement], &[Attack], &[Attack], &[Limited]],
    ),
    (
        Voie::Divination,
        [&[Limited], &[Limited], &[Attack], &[], &[]],
    ),
    (
        Voie::Envouteur,
        [&[Attack], &[Limited], &[Attack], &[Limited], &[Attack]],
    ),
    (
        Voie::Illusions,
        [&[Limited], &[Movement], &[Attack], &[Attack], &[Attack]],
    ),
    (
        Voie::Invocation,
        [&[Attack], &[Limited], &[Attack], &[Attack], &[Attack]],
    ),
    (
        Voie::Artefacts,
        [&[], &[Movement], &[], &[Attack], &[Limited]],
    ),
    (
        Voie::Elixirs,
        [&[Limited], &[Limited], &[Limited], &[Limited], &[Limited]],
    ),
    (
        Voie::Metal,
        [&[Movement], &[Movement], &[Attack], &[Attack], &[]],
    ),
    (Voie::Golem, [&[], &[], &[Free], &[Attack], &[]]),
    (Voie::Runes, [&[], &[Limited], &[Attack], &[Attack], &[]]),
    (
        Voie::MagieDesArcanes,
        [&[Attack], &[Movement], &[Attack], &[Attack], &[Attack]],
    ),
    (
        Voie::MagieDestructrice,
        [&[Attack], &[Attack], &[Attack], &[Attack], &[Attack]],
    ),
    (
        Voie::MagieElementaire,
        [
            &[Attack],
            &[Movement],
            &[Attack, Limited],
            &[Attack],
            &[Attack],
        ],
    ),
    (
        Voie::MagieProtectrice,
        [&[Movement], &[Free], &[Attack], &[Attack], &[Attack]],
    ),
    (
        Voie::MagieUniverselle,
        [
            &[Limited],
            &[Attack],
            &[Attack, Limited],
            &[Attack],
            &[Limited],
        ],
    ),
    (
        Voie::Demon,
        [
            &[Movement, Limited],
            &[Limited],
            &[Free],
            &[Attack],
            &[Limited],
        ],
    ),
    (
        Voie::Mort,
        [&[], &[Movement], &[Attack], &[Attack, Limited], &[Attack]],
    ),
    (
        Voie::OutreTombe,
        [&[Attack], &[], &[Limited], &[Limited], &[Limited]],
    ),
    (
        Voie::Sang,
        [&[Attack], &[Movement], &[], &[Attack], &[Attack]],
    ),
    (
        Voie::SombreMagie,
        [&[Limited], &[Movement], &[Attack], &[Limited], &[]],
    ),
    (Voie::Animaux, [&[], &[], &[Attack], &[Attack], &[Attack]]),
    (Voie::Fauve, [&[], &[], &[Limited], &[], &[]]),
    (Voie::Nature, [&[], &[], &[Limited], &[], &[]]),
    (Voie::Protecteur, [&[Limited], &[], &[], &[], &[Attack]]),
    (
        Voie::Vegetaux,
        [&[Movement], &[Limited], &[Attack], &[Limited], &[Attack]],
    ),
    (
        Voie::EnergieVitale,
        [&[Limited], &[Limited], &[], &[Movement], &[]],
    ),
    (Voie::Maitrise, [&[], &[], &[], &[Limited], &[]]),
    (Voie::Meditation, [&[], &[], &[], &[], &[Limited]]),
    (Voie::Poing, [&[], &[], &[Free], &[Limited], &[]]),
    (Voie::Vent, [&[], &[], &[], &[], &[Limited]]),
    (
        Voie::Foi,
        [&[], &[Attack], &[Movement, Limited], &[Attack], &[Attack]],
    ),
    (
        Voie::GuerreSainte,
        [&[Attack], &[], &[Limited], &[Attack], &[Attack]],
    ),
    (
        Voie::Priere,
        [&[Limited], &[Limited], &[Attack], &[], &[Free]],
    ),
    (Voie::Soins, [&[Attack], &[Limited], &[Limited], &[], &[]]),
    (
        Voie::Spiritualite,
        [&[], &[Limited], &[Limited], &[], &[Limited]],
    ),
    (Voie::DemiOrc, [&[], &[], &[], &[Limited], &[]]),
    (Voie::ElfeHaut, [&[], &[], &[], &[], &[]]),
    (Voie::ElfeSylvain, [&[], &[], &[], &[Limited], &[]]),
    (Voie::Gnome, [&[], &[], &[], &[], &[]]),
    (Voie::Halfelin, [&[], &[], &[], &[], &[]]),
    (Voie::Humain, [&[], &[], &[], &[], &[]]),
    (Voie::Nain, [&[], &[], &[], &[], &[]]),
    (Voie::Mage, [&[], &[Limited], &[Free], &[], &[]]),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn actions(voie: Voie, rank: u8) -> &'static [Action] {
        voie.actions(Rank::new(rank))
    }

    #[test]
    fn every_voie_has_its_actions() {
        assert!(
            Voie::ALL
                .iter()
                .all(|voie| ACTIONS.iter().any(|(v, _)| v == voie))
        );
        assert_eq!(actions(Voie::Rage, 3), [Limited]);
        assert_eq!(actions(Voie::Air, 4), [Attack]);
        assert_eq!(actions(Voie::Air, 2), [Movement]);
        assert_eq!(actions(Voie::Air, 1), [Free]);
        assert_eq!(actions(Voie::Brute, 1), []);
        assert_eq!(actions(Voie::MagieElementaire, 3), [Attack, Limited]);
        assert_eq!(actions(Voie::Rage, 6), []);
    }

    #[test]
    fn actions_name_themselves() {
        let names: Vec<_> = Action::ALL
            .iter()
            .map(|action| (action.abbreviation(), action.name()))
            .collect();
        assert_eq!(
            names,
            [
                ("L", "Action limitée"),
                ("A", "Action d'attaque"),
                ("M", "Action de mouvement"),
                ("G", "Action gratuite"),
            ]
        );
    }

    #[test]
    fn a_spell_costs_its_rank_in_pm() {
        assert_eq!(Voie::Air.spell_cost(Rank::new(4)), Some(4));
        assert_eq!(Voie::Brute.spell_cost(Rank::new(4)), None);
        assert!(Voie::Air.is_spell(Rank::new(1)));
        assert!(!Voie::Brute.is_spell(Rank::new(1)));
        let spells = Voie::ALL
            .iter()
            .flat_map(|voie| (1..=5).filter(|rank| voie.is_spell(Rank::new(*rank))))
            .count();
        assert_eq!(spells, 118);
    }
}
