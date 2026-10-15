//! Families and profils (COF.CNT.FAMILIES, COF.CNT.PROFILS).

use co_kernel::{Characteristic, Subject, ValueId, Violation};

use crate::{
    Armour, Character, Shield, Voie,
    rules::{PROFIL, PROFIL_MECHANIC, PROFIL_VALUE},
};

/// One of the four families of profils.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    /// Aventuriers.
    Aventuriers,
    /// Combattants.
    Combattants,
    /// Mages.
    Mages,
    /// Mystiques.
    Mystiques,
}

impl Family {
    /// Returns the kernel identifier of the family.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Aventuriers => "cof.family.aventuriers",
            Self::Combattants => "cof.family.combattants",
            Self::Mages => "cof.family.mages",
            Self::Mystiques => "cof.family.mystiques",
        })
    }

    /// PV gained per level.
    pub(crate) const fn vigour_per_level(self) -> i32 {
        match self {
            Self::Aventuriers | Self::Mystiques => 4,
            Self::Combattants => 5,
            Self::Mages => 3,
        }
    }

    /// The die of the dés de récupération (COF.DER.DR).
    #[must_use]
    pub const fn recovery_die(self) -> RecoveryDie {
        match self {
            Self::Aventuriers | Self::Mystiques => RecoveryDie::D8,
            Self::Combattants => RecoveryDie::D10,
            Self::Mages => RecoveryDie::D6,
        }
    }
}

/// The die rolled for a dé de récupération.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecoveryDie {
    /// Six-sided die.
    D6,
    /// Eight-sided die.
    D8,
    /// Ten-sided die.
    D10,
}

/// One of the 14 profils of COF2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Profil {
    /// Arquebusier (aventuriers).
    Arquebusier,
    /// Barde (aventuriers).
    Barde,
    /// Rôdeur (aventuriers).
    Rodeur,
    /// Voleur (aventuriers).
    Voleur,
    /// Barbare (combattants).
    Barbare,
    /// Chevalier (combattants).
    Chevalier,
    /// Guerrier (combattants).
    Guerrier,
    /// Ensorceleur (mages).
    Ensorceleur,
    /// Forgesort (mages).
    Forgesort,
    /// Magicien (mages).
    Magicien,
    /// Sorcier (mages).
    Sorcier,
    /// Druide (mystiques).
    Druide,
    /// Moine (mystiques).
    Moine,
    /// Prêtre (mystiques).
    Pretre,
}

impl Profil {
    /// Every profil, in declaration order.
    pub(crate) const ALL: [Self; 14] = [
        Self::Arquebusier,
        Self::Barde,
        Self::Rodeur,
        Self::Voleur,
        Self::Barbare,
        Self::Chevalier,
        Self::Guerrier,
        Self::Ensorceleur,
        Self::Forgesort,
        Self::Magicien,
        Self::Sorcier,
        Self::Druide,
        Self::Moine,
        Self::Pretre,
    ];

    /// The family of the profil.
    #[must_use]
    pub const fn family(self) -> Family {
        match self {
            Self::Arquebusier | Self::Barde | Self::Rodeur | Self::Voleur => Family::Aventuriers,
            Self::Barbare | Self::Chevalier | Self::Guerrier => Family::Combattants,
            Self::Ensorceleur | Self::Forgesort | Self::Magicien | Self::Sorcier => Family::Mages,
            Self::Druide | Self::Moine | Self::Pretre => Family::Mystiques,
        }
    }

    /// The five voies of the profil.
    pub(crate) const fn voies(self) -> [Voie; 5] {
        use Voie::{
            Air, Animaux, Archer, Artefacts, Artilleur, Assassin, Aventurier, Bouclier, Brute,
            Cavalier, Combat, CombatADeuxArmes, CompagnonAnimal, Demon, Deplacement, Divination,
            Elixirs, EnergieVitale, Envouteur, Escrime, Explosifs, Fauve, Foi, Golem, Guerre,
            GuerreSainte, Illusions, Invocation, MagieDesArcanes, MagieDestructrice,
            MagieElementaire, MagieProtectrice, MagieUniverselle, MaitreDArmes, Maitrise,
            Meditation, MeneurDHomme, Mercenaire, Metal, Mort, Musicien, Nature, Noblesse,
            OutreTombe, Pagne, Pistolero, Poing, Pourfendeur, Precision, Preux, Priere, Primitif,
            Protecteur, Rage, Resistance, Roublard, Runes, Saltimbanque, Sang, Seduction, Soins,
            Soldat, SombreMagie, Spadassin, Spiritualite, Survie, Traqueur, Vagabond, Vegetaux,
            Vent,
        };
        match self {
            Self::Arquebusier => [Artilleur, Explosifs, Mercenaire, Pistolero, Precision],
            Self::Barde => [Escrime, Musicien, Saltimbanque, Seduction, Vagabond],
            Self::Rodeur => [Archer, CompagnonAnimal, Survie, Traqueur, CombatADeuxArmes],
            Self::Voleur => [Assassin, Aventurier, Deplacement, Roublard, Spadassin],
            Self::Barbare => [Brute, Pagne, Pourfendeur, Primitif, Rage],
            Self::Chevalier => [Cavalier, Guerre, Preux, MeneurDHomme, Noblesse],
            Self::Guerrier => [Bouclier, Combat, MaitreDArmes, Resistance, Soldat],
            Self::Ensorceleur => [Air, Divination, Envouteur, Illusions, Invocation],
            Self::Forgesort => [Artefacts, Elixirs, Metal, Golem, Runes],
            Self::Magicien => [
                MagieDesArcanes,
                MagieDestructrice,
                MagieElementaire,
                MagieProtectrice,
                MagieUniverselle,
            ],
            Self::Sorcier => [Demon, Mort, OutreTombe, Sang, SombreMagie],
            Self::Druide => [Animaux, Fauve, Nature, Protecteur, Vegetaux],
            Self::Moine => [EnergieVitale, Maitrise, Meditation, Poing, Vent],
            Self::Pretre => [Foi, GuerreSainte, Priere, Soins, Spiritualite],
        }
    }

    /// The three key characteristics; a slot may offer alternatives (the
    /// druide's third key is CON or AGI).
    pub(crate) const fn key_characteristics(self) -> [&'static [Characteristic]; 3] {
        use Characteristic::{Agi, Cha, Con, For, Int, Per, Vol};
        match self {
            Self::Arquebusier => [&[Agi], &[Int], &[Con]],
            Self::Barde => [&[Cha], &[Agi], &[Vol]],
            Self::Rodeur => [&[Agi], &[Per], &[Con]],
            Self::Voleur => [&[Agi], &[Int], &[Cha]],
            Self::Barbare | Self::Guerrier => [&[For], &[Con], &[Agi]],
            Self::Chevalier => [&[For], &[Cha], &[Con]],
            Self::Ensorceleur => [&[Cha], &[Vol], &[Agi]],
            Self::Forgesort | Self::Sorcier => [&[Int], &[Vol], &[Con]],
            Self::Magicien => [&[Int], &[Vol], &[Agi]],
            Self::Druide => [&[Per], &[Vol], &[Con, Agi]],
            Self::Moine => [&[Vol], &[Per], &[Agi]],
            Self::Pretre => [&[Cha], &[Vol], &[For]],
        }
    }

    /// The characteristic of the profil's spells, for spellcasters.
    pub(crate) const fn magic_characteristic(self) -> Option<Characteristic> {
        match self {
            Self::Barde | Self::Ensorceleur | Self::Pretre => Some(Characteristic::Cha),
            Self::Forgesort | Self::Magicien | Self::Sorcier => Some(Characteristic::Int),
            Self::Druide => Some(Characteristic::Per),
            Self::Arquebusier
            | Self::Rodeur
            | Self::Voleur
            | Self::Barbare
            | Self::Chevalier
            | Self::Guerrier
            | Self::Moine => None,
        }
    }

    /// The heaviest armour the profil's capacités allow, `None` for no armour.
    pub(crate) const fn armour_limit(self) -> Option<Armour> {
        match self {
            Self::Arquebusier | Self::Pretre => Some(Armour::ChemiseDeMailles),
            Self::Barde | Self::Rodeur | Self::Barbare => Some(Armour::CuirRenforce),
            Self::Voleur | Self::Forgesort | Self::Druide => Some(Armour::CuirSimple),
            Self::Chevalier => Some(Armour::ArmureDePlaques),
            Self::Guerrier => Some(Armour::CotteDeMailles),
            Self::Ensorceleur | Self::Magicien | Self::Sorcier | Self::Moine => None,
        }
    }

    /// The largest shield the profil's capacités allow, `None` for no shield.
    pub(crate) const fn shield_limit(self) -> Option<Shield> {
        match self {
            Self::Barbare | Self::Chevalier | Self::Guerrier => Some(Shield::GrandBouclier),
            Self::Druide | Self::Pretre => Some(Shield::PetitBouclier),
            Self::Arquebusier
            | Self::Barde
            | Self::Rodeur
            | Self::Voleur
            | Self::Ensorceleur
            | Self::Forgesort
            | Self::Magicien
            | Self::Sorcier
            | Self::Moine => None,
        }
    }
}

/// Checks that a profil is chosen.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    if character.profil().is_none() {
        violations.push(Violation::incomplete(
            PROFIL,
            PROFIL_MECHANIC,
            Subject::Value(PROFIL_VALUE),
        ));
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    #[test]
    fn missing_profil_is_incomplete() {
        let mut violations = Vec::new();
        check(&Character::new(), &mut violations);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule(), &PROFIL);
        assert_eq!(violations[0].category(), Category::Incomplete);

        let mut violations = Vec::new();
        check(
            &Character::new().with_profil(Profil::Moine),
            &mut violations,
        );
        assert_eq!(violations, []);
    }

    #[test]
    fn family_gives_vigour_and_recovery_die() {
        assert_eq!(Profil::Barbare.family(), Family::Combattants);
        assert_eq!(Family::Combattants.vigour_per_level(), 5);
        assert_eq!(Family::Mages.recovery_die(), RecoveryDie::D6);
        assert_eq!(Family::Mystiques.id().as_str(), "cof.family.mystiques");
    }

    #[test]
    fn every_voie_belongs_to_at_most_one_profil() {
        let mut voies: Vec<_> = Profil::ALL.iter().flat_map(|p| p.voies()).collect();
        voies.sort();
        voies.dedup();
        assert_eq!(voies.len(), 70);
        for profil in Profil::ALL {
            for voie in profil.voies() {
                assert_eq!(voie.profil(), Some(profil));
            }
        }
    }
}
