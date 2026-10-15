//! Voies (COF.CNT.PROFILS, COF.CNT.PEUPLES) and the checks on the ranks
//! acquired at level 1.

use std::collections::BTreeSet;

use co_kernel::{AbilityId, Rank, Severity, Subject, Violation, VoieId};

use crate::{
    Character, Family, Profil,
    rules::{
        LEVEL_REQ, MAGE_EXTRA, PEUPLE_VOIE, PROFIL_VOIES, VOIE_DU_MAGE, VOIE_MECHANIC, VOIE_ORDER,
        VOIE_STRUCTURE, VOIES,
    },
};

/// One of the 78 voies a level-1 COF2 character can acquire: the five voies
/// of each profil, the voies de peuple and the voie du mage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Voie {
    /// Voie de l'artilleur (arquebusier).
    Artilleur,
    /// Voie des explosifs (arquebusier).
    Explosifs,
    /// Voie du mercenaire (arquebusier).
    Mercenaire,
    /// Voie du pistolero (arquebusier).
    Pistolero,
    /// Voie de la précision (arquebusier).
    Precision,
    /// Voie de l'escrime (barde).
    Escrime,
    /// Voie du musicien (barde).
    Musicien,
    /// Voie du saltimbanque (barde).
    Saltimbanque,
    /// Voie de la séduction (barde).
    Seduction,
    /// Voie du vagabond (barde).
    Vagabond,
    /// Voie de l'archer (rôdeur).
    Archer,
    /// Voie du compagnon animal (rôdeur).
    CompagnonAnimal,
    /// Voie de la survie (rôdeur).
    Survie,
    /// Voie du traqueur (rôdeur).
    Traqueur,
    /// Voie du combat à deux armes (rôdeur).
    CombatADeuxArmes,
    /// Voie de l'assassin (voleur).
    Assassin,
    /// Voie de l'aventurier (voleur).
    Aventurier,
    /// Voie du déplacement (voleur).
    Deplacement,
    /// Voie du roublard (voleur).
    Roublard,
    /// Voie du spadassin (voleur).
    Spadassin,
    /// Voie de la brute (barbare).
    Brute,
    /// Voie du pagne (barbare).
    Pagne,
    /// Voie du pourfendeur (barbare).
    Pourfendeur,
    /// Voie du primitif (barbare).
    Primitif,
    /// Voie de la rage (barbare).
    Rage,
    /// Voie du cavalier (chevalier).
    Cavalier,
    /// Voie de la guerre (chevalier).
    Guerre,
    /// Voie du preux (chevalier).
    Preux,
    /// Voie du meneur d'homme (chevalier).
    MeneurDHomme,
    /// Voie de la noblesse (chevalier).
    Noblesse,
    /// Voie du bouclier (guerrier).
    Bouclier,
    /// Voie du combat (guerrier).
    Combat,
    /// Voie du maître d'armes (guerrier).
    MaitreDArmes,
    /// Voie de la résistance (guerrier).
    Resistance,
    /// Voie du soldat (guerrier).
    Soldat,
    /// Voie de l'air (ensorceleur).
    Air,
    /// Voie de la divination (ensorceleur).
    Divination,
    /// Voie de l'envoûteur (ensorceleur).
    Envouteur,
    /// Voie des illusions (ensorceleur).
    Illusions,
    /// Voie de l'invocation (ensorceleur).
    Invocation,
    /// Voie des artefacts (forgesort).
    Artefacts,
    /// Voie des élixirs (forgesort).
    Elixirs,
    /// Voie du métal (forgesort).
    Metal,
    /// Voie du golem (forgesort).
    Golem,
    /// Voie des runes (forgesort).
    Runes,
    /// Voie de la magie des arcanes (magicien).
    MagieDesArcanes,
    /// Voie de la magie destructrice (magicien).
    MagieDestructrice,
    /// Voie de la magie élémentaire (magicien).
    MagieElementaire,
    /// Voie de la magie protectrice (magicien).
    MagieProtectrice,
    /// Voie de la magie universelle (magicien).
    MagieUniverselle,
    /// Voie du démon (sorcier).
    Demon,
    /// Voie de la mort (sorcier).
    Mort,
    /// Voie de l'outre-tombe (sorcier).
    OutreTombe,
    /// Voie du sang (sorcier).
    Sang,
    /// Voie de la sombre magie (sorcier).
    SombreMagie,
    /// Voie des animaux (druide).
    Animaux,
    /// Voie du fauve (druide).
    Fauve,
    /// Voie de la nature (druide).
    Nature,
    /// Voie du protecteur (druide).
    Protecteur,
    /// Voie des végétaux (druide).
    Vegetaux,
    /// Voie de l'énergie vitale (moine).
    EnergieVitale,
    /// Voie de la maîtrise (moine).
    Maitrise,
    /// Voie de la méditation (moine).
    Meditation,
    /// Voie du poing (moine).
    Poing,
    /// Voie du vent (moine).
    Vent,
    /// Voie de la foi (prêtre).
    Foi,
    /// Voie de la guerre sainte (prêtre).
    GuerreSainte,
    /// Voie de la prière (prêtre).
    Priere,
    /// Voie des soins (prêtre).
    Soins,
    /// Voie de la spiritualité (prêtre).
    Spiritualite,
    /// Voie du demi-orc (voie de peuple).
    DemiOrc,
    /// Voie de l'elfe haut (voie de peuple).
    ElfeHaut,
    /// Voie de l'elfe sylvain (voie de peuple).
    ElfeSylvain,
    /// Voie du gnome (voie de peuple).
    Gnome,
    /// Voie du halfelin (voie de peuple).
    Halfelin,
    /// Voie de l'humain (voie de peuple).
    Humain,
    /// Voie du nain (voie de peuple).
    Nain,
    /// Voie du mage (mage family, replaces the voie de peuple).
    Mage,
}

impl Voie {
    /// Returns the kernel identifier of the voie.
    #[must_use]
    pub fn id(self) -> VoieId {
        VoieId::new(self.slug())
    }

    /// Returns the identifier of the capacité at `rank` in this voie.
    pub(crate) fn ability(self, rank: Rank) -> AbilityId {
        AbilityId::new(format!("{}.{}", self.slug(), rank.get()))
    }

    /// The profil the voie belongs to, if any.
    pub(crate) fn profil(self) -> Option<Profil> {
        Profil::ALL
            .into_iter()
            .find(|profil| profil.voies().contains(&self))
    }

    /// Whether the voie fills the voie de peuple slot: a voie de peuple, or
    /// the voie du mage replacing it.
    pub(crate) const fn is_peuple_slot(self) -> bool {
        matches!(
            self,
            Self::DemiOrc
                | Self::ElfeHaut
                | Self::ElfeSylvain
                | Self::Gnome
                | Self::Halfelin
                | Self::Humain
                | Self::Nain
                | Self::Mage
        )
    }

    /// Whether the capacité at `rank` is a spell. Only the spells the co-rules
    /// skill documents are known (D-23).
    pub(crate) fn is_spell(self, rank: Rank) -> bool {
        matches!(
            (self, rank.get()),
            (Self::Air | Self::Invocation, 1 | 2) | (Self::Mage, 2 | 3)
        )
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Artilleur => "cof.artilleur",
            Self::Explosifs => "cof.explosifs",
            Self::Mercenaire => "cof.mercenaire",
            Self::Pistolero => "cof.pistolero",
            Self::Precision => "cof.precision",
            Self::Escrime => "cof.escrime",
            Self::Musicien => "cof.musicien",
            Self::Saltimbanque => "cof.saltimbanque",
            Self::Seduction => "cof.seduction",
            Self::Vagabond => "cof.vagabond",
            Self::Archer => "cof.archer",
            Self::CompagnonAnimal => "cof.compagnon-animal",
            Self::Survie => "cof.survie",
            Self::Traqueur => "cof.traqueur",
            Self::CombatADeuxArmes => "cof.combat-a-deux-armes",
            Self::Assassin => "cof.assassin",
            Self::Aventurier => "cof.aventurier",
            Self::Deplacement => "cof.deplacement",
            Self::Roublard => "cof.roublard",
            Self::Spadassin => "cof.spadassin",
            Self::Brute => "cof.brute",
            Self::Pagne => "cof.pagne",
            Self::Pourfendeur => "cof.pourfendeur",
            Self::Primitif => "cof.primitif",
            Self::Rage => "cof.rage",
            Self::Cavalier => "cof.cavalier",
            Self::Guerre => "cof.guerre",
            Self::Preux => "cof.preux",
            Self::MeneurDHomme => "cof.meneur-d-homme",
            Self::Noblesse => "cof.noblesse",
            Self::Bouclier => "cof.bouclier",
            Self::Combat => "cof.combat",
            Self::MaitreDArmes => "cof.maitre-d-armes",
            Self::Resistance => "cof.resistance",
            Self::Soldat => "cof.soldat",
            Self::Air => "cof.air",
            Self::Divination => "cof.divination",
            Self::Envouteur => "cof.envouteur",
            Self::Illusions => "cof.illusions",
            Self::Invocation => "cof.invocation",
            Self::Artefacts => "cof.artefacts",
            Self::Elixirs => "cof.elixirs",
            Self::Metal => "cof.metal",
            Self::Golem => "cof.golem",
            Self::Runes => "cof.runes",
            Self::MagieDesArcanes => "cof.magie-des-arcanes",
            Self::MagieDestructrice => "cof.magie-destructrice",
            Self::MagieElementaire => "cof.magie-elementaire",
            Self::MagieProtectrice => "cof.magie-protectrice",
            Self::MagieUniverselle => "cof.magie-universelle",
            Self::Demon => "cof.demon",
            Self::Mort => "cof.mort",
            Self::OutreTombe => "cof.outre-tombe",
            Self::Sang => "cof.sang",
            Self::SombreMagie => "cof.sombre-magie",
            Self::Animaux => "cof.animaux",
            Self::Fauve => "cof.fauve",
            Self::Nature => "cof.nature",
            Self::Protecteur => "cof.protecteur",
            Self::Vegetaux => "cof.vegetaux",
            Self::EnergieVitale => "cof.energie-vitale",
            Self::Maitrise => "cof.maitrise",
            Self::Meditation => "cof.meditation",
            Self::Poing => "cof.poing",
            Self::Vent => "cof.vent",
            Self::Foi => "cof.foi",
            Self::GuerreSainte => "cof.guerre-sainte",
            Self::Priere => "cof.priere",
            Self::Soins => "cof.soins",
            Self::Spiritualite => "cof.spiritualite",
            Self::DemiOrc => "cof.demi-orc",
            Self::ElfeHaut => "cof.elfe-haut",
            Self::ElfeSylvain => "cof.elfe-sylvain",
            Self::Gnome => "cof.gnome",
            Self::Halfelin => "cof.halfelin",
            Self::Humain => "cof.humain",
            Self::Nain => "cof.nain",
            Self::Mage => "cof.mage",
        }
    }
}

/// Lowest and highest rank of a voie (CO.VOIE.STRUCTURE).
const RANKS: std::ops::RangeInclusive<u8> = 1..=5;

/// Voies chosen at level 1 besides the voie de peuple (COF.CRE.VOIES).
const PROFIL_VOIE_COUNT: i32 = 2;

/// The rank of the mage family's extra capacité (COF.CRE.MAGE-EXTRA).
const MAGE_EXTRA_RANK: Rank = Rank::new(2);

/// The level required to acquire `rank` (COF.PRG.LEVEL-REQ), if the table
/// gives one.
const fn required_level(rank: u8) -> Option<i32> {
    match rank {
        1 => Some(1),
        2 => Some(2),
        3 => Some(3),
        4 => Some(5),
        5 => Some(7),
        6 => Some(9),
        7 => Some(11),
        8 => Some(13),
        _ => None,
    }
}

/// Checks the acquired ranks against the voie structure and the level-1
/// choices.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        check_structure(voie, ranks, violations);
    }
    check_slots(character, violations);
    check_ranks(character, violations);
}

/// CO.VOIE.STRUCTURE and CO.VOIE.ORDER for one voie.
fn check_structure(voie: Voie, ranks: &BTreeSet<Rank>, violations: &mut Vec<Violation>) {
    let subject = || Subject::Voie(voie.id());
    for rank in ranks.iter().filter(|rank| !RANKS.contains(&rank.get())) {
        violations.push(
            Violation::deviation(VOIE_STRUCTURE, VOIE_MECHANIC, subject(), Severity::Standard)
                .with_parameter("rank", i32::from(rank.get())),
        );
    }
    let highest = ranks.last().map_or(0, |rank| rank.get());
    let mut missing = (1..highest)
        .filter(|r| !ranks.contains(&Rank::new(*r)))
        .peekable();
    if let Some(&first) = missing.peek() {
        let count = missing.fold(0_i32, |count, _| count.saturating_add(1));
        violations.push(
            Violation::deviation(VOIE_ORDER, VOIE_MECHANIC, subject(), Severity::Standard)
                .with_parameter("first_missing", i32::from(first))
                .with_parameter("missing", count),
        );
    }
}

/// COF.CRE.VOIES and COF.CRE.VOIE-DU-MAGE: two chosen voies, one voie de
/// peuple, each from where it may come.
fn check_slots(character: &Character, violations: &mut Vec<Violation>) {
    let profil = character.profil();
    let peuple = character.peuple();
    let is_mage = character.family().map(|family| family == Family::Mages);
    let mut chosen = 0_i32;
    let mut peuple_slot = 0_i32;
    for (voie, _) in character.voies() {
        let foreign = if voie == Voie::Mage {
            peuple_slot = peuple_slot.saturating_add(1);
            if is_mage == Some(false) {
                violations.push(Violation::deviation(
                    VOIE_DU_MAGE,
                    VOIE_MECHANIC,
                    Subject::Voie(voie.id()),
                    Severity::Standard,
                ));
            }
            false
        } else if voie.is_peuple_slot() {
            peuple_slot = peuple_slot.saturating_add(1);
            peuple.is_some_and(|peuple| !peuple.allows_voie(voie))
        } else {
            chosen = chosen.saturating_add(1);
            profil.is_some_and(|profil| voie.profil() != Some(profil))
        };
        if foreign {
            violations.push(Violation::deviation(
                VOIES,
                VOIE_MECHANIC,
                Subject::Voie(voie.id()),
                Severity::Standard,
            ));
        }
    }

    let count_violation = |count: i32, expected: i32, subject| {
        let violation = match count.cmp(&expected) {
            std::cmp::Ordering::Less => Violation::incomplete(VOIES, VOIE_MECHANIC, subject),
            std::cmp::Ordering::Greater => {
                Violation::deviation(VOIES, VOIE_MECHANIC, subject, Severity::Standard)
            }
            std::cmp::Ordering::Equal => return None,
        };
        Some(
            violation
                .with_parameter("voies", count)
                .with_parameter("expected", expected),
        )
    };
    violations.extend(count_violation(
        chosen,
        PROFIL_VOIE_COUNT,
        Subject::Value(PROFIL_VOIES),
    ));
    violations.extend(count_violation(peuple_slot, 1, Subject::Value(PEUPLE_VOIE)));
}

/// COF.PRG.LEVEL-REQ and COF.CRE.MAGE-EXTRA: rank 1 only at level 1, except
/// one rank 2 for the mage family, in a voie of its profil or the voie du
/// mage.
fn check_ranks(character: &Character, violations: &mut Vec<Violation>) {
    let profil = character.profil();
    let is_mage = character.family() == Some(Family::Mages);
    let mut extras = 0_i32;
    for (voie, ranks) in character.voies() {
        let covers_extra = is_mage && (voie == Voie::Mage || voie.profil() == profil);
        if covers_extra && ranks.contains(&MAGE_EXTRA_RANK) {
            extras = extras.saturating_add(1);
        }
        let allowed = if covers_extra { 2 } else { 1 };
        if let Some(highest) = ranks.last().map(|rank| rank.get())
            && highest > allowed
        {
            let mut violation = Violation::deviation(
                LEVEL_REQ,
                VOIE_MECHANIC,
                Subject::Voie(voie.id()),
                Severity::Standard,
            )
            .with_parameter("rank", i32::from(highest))
            .with_parameter("level", 1);
            if let Some(required) = required_level(highest) {
                violation = violation.with_parameter("required", required);
            }
            violations.push(violation);
        }
    }
    if is_mage {
        let violation = match extras {
            0 => Violation::incomplete(MAGE_EXTRA, VOIE_MECHANIC, Subject::Character),
            1 => return,
            _ => Violation::deviation(
                MAGE_EXTRA,
                VOIE_MECHANIC,
                Subject::Character,
                Severity::Standard,
            ),
        };
        violations.push(violation.with_parameter("extras", extras));
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;
    use crate::Peuple;

    fn voie_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
    }

    fn rules(violations: &[Violation]) -> Vec<(&str, Category)> {
        violations
            .iter()
            .map(|v| (v.rule().as_str(), v.category()))
            .collect()
    }

    const DEVIATION: Category = Category::Deviation(Severity::Standard);

    /// A humain guerrier with its three rank-1 capacités.
    fn guerrier() -> Character {
        Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(Peuple::Humain)
            .with_rank(Voie::Bouclier, 1)
            .with_rank(Voie::Soldat, 1)
            .with_rank(Voie::Humain, 1)
    }

    /// A gnome magicien with its mage extra capacité.
    fn magicien() -> Character {
        Character::new()
            .with_profil(Profil::Magicien)
            .with_peuple(Peuple::Gnome)
            .with_rank(Voie::MagieDesArcanes, 1)
            .with_rank(Voie::MagieDesArcanes, 2)
            .with_rank(Voie::MagieElementaire, 1)
            .with_rank(Voie::Gnome, 1)
    }

    #[test]
    fn level_one_choices_are_clean() {
        assert_eq!(voie_violations(&guerrier()), []);
        assert_eq!(voie_violations(&magicien()), []);
    }

    #[test]
    fn ability_ids_name_voie_and_rank() {
        assert_eq!(Voie::Rage.ability(Rank::new(1)).as_str(), "cof.rage.1");
        assert_eq!(Voie::MeneurDHomme.id().as_str(), "cof.meneur-d-homme");
    }

    #[test]
    fn missing_voies_are_incomplete() {
        let subjects: Vec<_> = voie_violations(&Character::new())
            .into_iter()
            .map(|v| (v.subject().clone(), v.category()))
            .collect();
        assert_eq!(
            subjects,
            [
                (Subject::Value(PROFIL_VOIES), Category::Incomplete),
                (Subject::Value(PEUPLE_VOIE), Category::Incomplete),
            ]
        );
    }

    #[test]
    fn extra_voies_deviate() {
        let character = guerrier()
            .with_rank(Voie::Combat, 1)
            .with_rank(Voie::Mage, 1);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [
                ("COF.CRE.VOIE-DU-MAGE", DEVIATION),
                ("COF.CRE.VOIES", DEVIATION),
                ("COF.CRE.VOIES", DEVIATION),
            ]
        );
        assert_eq!(violations[1].subject(), &Subject::Value(PROFIL_VOIES));
        assert_eq!(violations[2].subject(), &Subject::Value(PEUPLE_VOIE));
    }

    #[test]
    fn voies_from_elsewhere_deviate() {
        let character = Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(Peuple::Nain)
            .with_rank(Voie::Bouclier, 1)
            .with_rank(Voie::Rage, 1)
            .with_rank(Voie::Humain, 1);
        let subjects: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| v.subject().clone())
            .collect();
        assert_eq!(
            subjects,
            [
                Subject::Voie(Voie::Rage.id()),
                Subject::Voie(Voie::Humain.id())
            ]
        );
    }

    #[test]
    fn demi_elfe_takes_a_voie_of_its_upbringing() {
        let character = Character::new()
            .with_profil(Profil::Barde)
            .with_peuple(Peuple::DemiElfe)
            .with_rank(Voie::Escrime, 1)
            .with_rank(Voie::Musicien, 1)
            .with_rank(Voie::ElfeSylvain, 1);
        assert_eq!(voie_violations(&character), []);
    }

    #[test]
    fn mage_replaces_its_voie_de_peuple() {
        let character = magicien()
            .without_rank(Voie::Gnome, 1)
            .with_rank(Voie::Mage, 1);
        assert_eq!(voie_violations(&character), []);
        let both = character.with_rank(Voie::Gnome, 1);
        assert_eq!(
            rules(&voie_violations(&both)),
            [("COF.CRE.VOIES", DEVIATION)]
        );
    }

    #[test]
    fn mage_extra_may_be_in_the_voie_du_mage() {
        let character = magicien()
            .without_rank(Voie::MagieDesArcanes, 2)
            .without_rank(Voie::Gnome, 1)
            .with_rank(Voie::Mage, 1)
            .with_rank(Voie::Mage, 2);
        assert_eq!(voie_violations(&character), []);
    }

    #[test]
    fn mage_extra_stays_in_the_profil_voies() {
        let character = magicien()
            .without_rank(Voie::MagieDesArcanes, 2)
            .without_rank(Voie::MagieElementaire, 1)
            .with_rank(Voie::Rage, 1)
            .with_rank(Voie::Rage, 2);
        let found: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.category()))
            .collect();
        assert_eq!(
            found,
            [
                ("COF.CRE.VOIES".to_owned(), DEVIATION),
                ("COF.PRG.LEVEL-REQ".to_owned(), DEVIATION),
                ("COF.CRE.MAGE-EXTRA".to_owned(), Category::Incomplete),
            ]
        );
    }

    #[test]
    fn missing_mage_extra_is_incomplete() {
        let character = magicien().without_rank(Voie::MagieDesArcanes, 2);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("COF.CRE.MAGE-EXTRA", Category::Incomplete)]
        );
    }

    #[test]
    fn second_mage_extra_deviates() {
        let character = magicien().with_rank(Voie::MagieElementaire, 2);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("COF.CRE.MAGE-EXTRA", DEVIATION)]
        );
    }

    #[test]
    fn rank_above_level_one_deviates() {
        let character = guerrier()
            .with_rank(Voie::Soldat, 2)
            .with_rank(Voie::Humain, 2);
        let subjects: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.subject().clone()))
            .collect();
        assert_eq!(
            subjects,
            [
                (
                    "COF.PRG.LEVEL-REQ".to_owned(),
                    Subject::Voie(Voie::Soldat.id())
                ),
                (
                    "COF.PRG.LEVEL-REQ".to_owned(),
                    Subject::Voie(Voie::Humain.id())
                ),
            ]
        );
        // A mage's voie de peuple does not take the extra rank.
        let mage = magicien().with_rank(Voie::Gnome, 2);
        assert_eq!(
            rules(&voie_violations(&mage)),
            [("COF.PRG.LEVEL-REQ", DEVIATION)]
        );
        let rank_three = magicien().with_rank(Voie::MagieDesArcanes, 3);
        let violations = voie_violations(&rank_three);
        assert_eq!(rules(&violations), [("COF.PRG.LEVEL-REQ", DEVIATION)]);
        assert_eq!(
            violations[0].parameters()[2].value(),
            &co_kernel::ParameterValue::Integer(3)
        );
    }

    #[test]
    fn gap_and_structure_deviate() {
        let character = guerrier().with_rank(Voie::Soldat, 9);
        assert_eq!(
            rules(&voie_violations(&character)),
            [
                ("CO.VOIE.STRUCTURE", DEVIATION),
                ("CO.VOIE.ORDER", DEVIATION),
                ("COF.PRG.LEVEL-REQ", DEVIATION),
            ]
        );
    }

    #[test]
    fn documented_spells_are_known() {
        assert!(Voie::Air.is_spell(Rank::new(1)));
        assert!(Voie::Invocation.is_spell(Rank::new(2)));
        assert!(!Voie::Mage.is_spell(Rank::new(1)));
        assert!(Voie::Mage.is_spell(Rank::new(2)));
        assert!(!Voie::Rage.is_spell(Rank::new(1)));
    }
}
