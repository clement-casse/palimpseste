//! Families and profils (COF.CNT.FAMILIES, COF.CNT.PROFILS).

use co_kernel::{Characteristic, Subject, ValueId, Violation};

use crate::{
    Armour, Character, Shield, Voie, Weapon,
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
    /// Every family, in declaration order.
    pub const ALL: [Self; 4] = [
        Self::Aventuriers,
        Self::Combattants,
        Self::Mages,
        Self::Mystiques,
    ];

    /// The French display name of the family.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aventuriers => "Aventuriers",
            Self::Combattants => "Combattants",
            Self::Mages => "Mages",
            Self::Mystiques => "Mystiques",
        }
    }

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
    pub const ALL: [Self; 14] = [
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

    /// Returns the identifier of the profil, used by the choices document.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Arquebusier => "cof.profil.arquebusier",
            Self::Barde => "cof.profil.barde",
            Self::Rodeur => "cof.profil.rodeur",
            Self::Voleur => "cof.profil.voleur",
            Self::Barbare => "cof.profil.barbare",
            Self::Chevalier => "cof.profil.chevalier",
            Self::Guerrier => "cof.profil.guerrier",
            Self::Ensorceleur => "cof.profil.ensorceleur",
            Self::Forgesort => "cof.profil.forgesort",
            Self::Magicien => "cof.profil.magicien",
            Self::Sorcier => "cof.profil.sorcier",
            Self::Druide => "cof.profil.druide",
            Self::Moine => "cof.profil.moine",
            Self::Pretre => "cof.profil.pretre",
        })
    }

    /// Returns the profil whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|profil| profil.id().as_str() == id)
    }

    /// The French display name of the profil.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Arquebusier => "Arquebusier",
            Self::Barde => "Barde",
            Self::Rodeur => "Rôdeur",
            Self::Voleur => "Voleur",
            Self::Barbare => "Barbare",
            Self::Chevalier => "Chevalier",
            Self::Guerrier => "Guerrier",
            Self::Ensorceleur => "Ensorceleur",
            Self::Forgesort => "Forgesort",
            Self::Magicien => "Magicien",
            Self::Sorcier => "Sorcier",
            Self::Druide => "Druide",
            Self::Moine => "Moine",
            Self::Pretre => "Prêtre",
        }
    }

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
    #[must_use]
    pub const fn voies(self) -> [Voie; 5] {
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
    #[must_use]
    pub const fn key_characteristics(self) -> [&'static [Characteristic]; 3] {
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
    #[must_use]
    pub const fn magic_characteristic(self) -> Option<Characteristic> {
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
    #[must_use]
    pub const fn armour_limit(self) -> Option<Armour> {
        match self {
            Self::Arquebusier | Self::Pretre => Some(Armour::ChemiseDeMailles),
            Self::Barde | Self::Rodeur | Self::Barbare => Some(Armour::CuirRenforce),
            Self::Voleur | Self::Forgesort | Self::Druide => Some(Armour::CuirSimple),
            Self::Chevalier => Some(Armour::ArmureDePlaques),
            Self::Guerrier => Some(Armour::CotteDeMailles),
            Self::Ensorceleur | Self::Magicien | Self::Sorcier | Self::Moine => None,
        }
    }

    /// Whether the profil masters `weapon` (COF.CNT.PROFILS,
    /// COF.DER.WEAPON-PROFICIENCY). A weapon not mastered gives a dé malus
    /// to the attack test; carrying it is not a violation. Only the
    /// arquebusier masters powder weapons.
    ///
    /// "Bâton" covers the bâton and the bâton ferré, the barbare's "complex
    /// technology" means powder weapons, and every weapon eligible as the
    /// ensorceleur's weapon of choice counts as mastered, since the choice is
    /// not recorded (D-28).
    #[must_use]
    pub fn masters(self, weapon: Weapon) -> bool {
        use Weapon::{ArbaleteLegere, ArcCourt, Baton, BatonFerre, Dague, Epieu, Javelot, Marteau};
        let powder = weapon.is_powder();
        match self {
            Self::Arquebusier => weapon.is_one_handed_contact() || weapon.is_ranged(),
            Self::Barde => weapon.is_one_handed() && !powder,
            Self::Rodeur | Self::Voleur => {
                (weapon.is_one_handed_contact() || weapon.is_ranged()) && !powder
            }
            Self::Barbare => !weapon.is_crossbow() && !powder,
            Self::Chevalier => weapon.is_contact(),
            Self::Guerrier | Self::Moine => !powder,
            Self::Ensorceleur => {
                matches!(weapon, Dague | BatonFerre | ArbaleteLegere)
                    || (weapon.highest_damage() <= ENSORCELEUR_CHOICE_MAX && !powder)
            }
            Self::Forgesort => matches!(
                weapon,
                Dague | Baton | BatonFerre | Marteau | ArbaleteLegere
            ),
            Self::Magicien | Self::Sorcier => matches!(weapon, Dague | Baton | BatonFerre),
            Self::Druide => matches!(
                weapon,
                Dague | Baton | BatonFerre | Epieu | Javelot | ArcCourt
            ),
            Self::Pretre => {
                (weapon.is_blunt() && weapon.is_one_handed()) || matches!(weapon, BatonFerre)
            }
        }
    }

    /// The largest shield the profil's capacités allow, `None` for no shield.
    #[must_use]
    pub const fn shield_limit(self) -> Option<Shield> {
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

/// The highest damage of the ensorceleur's weapon of choice: 1d6.
const ENSORCELEUR_CHOICE_MAX: i32 = 6;

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
    fn mastery_follows_the_profil() {
        assert!(Profil::Arquebusier.masters(Weapon::Petoire));
        assert!(!Profil::Guerrier.masters(Weapon::Petoire));
        assert!(Profil::Guerrier.masters(Weapon::ArcLong));
        assert!(!Profil::Barbare.masters(Weapon::ArbaleteLegere));
        assert!(Profil::Barbare.masters(Weapon::HacheADeuxMains));
        assert!(!Profil::Chevalier.masters(Weapon::ArcCourt));
        assert!(Profil::Chevalier.masters(Weapon::Dague));
        assert!(Profil::Barde.masters(Weapon::Rapiere));
        assert!(!Profil::Barde.masters(Weapon::EpeeADeuxMains));
        assert!(Profil::Voleur.masters(Weapon::ArcCourt));
        assert!(!Profil::Voleur.masters(Weapon::EpeeADeuxMains));
        assert!(Profil::Ensorceleur.masters(Weapon::EpeeCourte));
        assert!(!Profil::Ensorceleur.masters(Weapon::EpeeLongue));
        assert!(Profil::Magicien.masters(Weapon::BatonFerre));
        assert!(!Profil::Magicien.masters(Weapon::EpeeCourte));
        assert!(Profil::Pretre.masters(Weapon::Masse));
        assert!(Profil::Pretre.masters(Weapon::BatonFerre));
        assert!(!Profil::Pretre.masters(Weapon::EpeeCourte));
        assert!(Profil::Druide.masters(Weapon::Epieu));
        assert!(Profil::Moine.masters(Weapon::Vivelame));
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
