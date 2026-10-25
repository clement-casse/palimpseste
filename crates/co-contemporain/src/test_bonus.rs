//! The capacités' permanent bonuses to the character's own tests, and their
//! dé bonus (COC.CNT.VOIES, D-50).
//!
//! A capacité gives one bonus per amount, with the characteristics its
//! catalogue entry names and the tests it covers in a few words of our own
//! (its scope). Bonuses
//! used once per session or scene, given to others, to attacks, or testing
//! one characteristic in place of another are not test bonuses.

use co_kernel::{AbilityId, Characteristic, DerivedValue, Rank, Source, ValueId};

use crate::{Character, Voie};

/// One permanent bonus a capacité gives to some tests: the n-th of the
/// capacité at its rank in its voie. [`Character::test_bonuses`] lists those
/// held; each is a [`Value`](crate::Value), adjustable like any other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TestBonus {
    voie: Voie,
    rank: Rank,
    index: u8,
}

impl TestBonus {
    /// The voie of the capacité giving the bonus.
    #[must_use]
    pub const fn voie(self) -> Voie {
        self.voie
    }

    /// The rank of the capacité giving the bonus.
    #[must_use]
    pub const fn rank(self) -> Rank {
        self.rank
    }

    /// The identifier of the capacité giving the bonus.
    #[must_use]
    pub fn ability(self) -> AbilityId {
        self.voie.ability(self.rank)
    }

    /// The characteristics of the tests the bonus applies to, when the
    /// catalogue names them; empty otherwise.
    #[must_use]
    pub fn characteristics(self) -> &'static [Characteristic] {
        self.row().map_or(&[], |row| row.characteristics)
    }

    /// The tests the bonus applies to, in French and in a few words
    /// (`premiers soins, stabiliser un mourant`), besides the
    /// characteristics named.
    #[must_use]
    pub fn scope(self) -> &'static str {
        self.row().map_or("", |row| row.scope)
    }

    /// Whether the bonus applies to the speciality chosen for its capacité
    /// ([`Voie::speciality`]).
    #[must_use]
    pub fn on_speciality(self) -> bool {
        self.row().is_some_and(|row| row.on_speciality)
    }

    /// The identifier of the bonus as a value,
    /// `coc.test-bonus.<voie>.<rank>.<n>`.
    pub(crate) fn id(self) -> ValueId {
        let voie = self.voie.id();
        let voie = voie.as_str();
        ValueId::new(format!(
            "coc.test-bonus.{}.{}.{}",
            voie.strip_prefix("coc.").unwrap_or(voie),
            self.rank.get(),
            self.index
        ))
    }

    /// The French display name of the bonus as a value.
    pub(crate) fn name(self) -> String {
        let ability = self.voie.ability_name(self.rank).unwrap_or_default();
        if self.index == 1 {
            format!("Bonus aux tests : {ability}")
        } else {
            format!("Bonus aux tests : {ability} ({})", self.index)
        }
    }

    /// Every bonus of the catalogue, in voie and rank order.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        BONUSES.iter().enumerate().map(|(position, row)| Self {
            voie: row.voie,
            rank: Rank::new(row.rank),
            index: index_of(position),
        })
    }

    fn row(self) -> Option<&'static Row> {
        Self::all()
            .zip(BONUSES)
            .find(|(bonus, _)| *bonus == self)
            .map(|(_, row)| row)
    }
}

/// The index, from 1, of the bonus at `position` among those of its
/// capacité.
fn index_of(position: usize) -> u8 {
    let row = &BONUSES[position];
    let earlier = BONUSES[..position]
        .iter()
        .filter(|other| other.voie == row.voie && other.rank == row.rank)
        .count();
    u8::try_from(earlier + 1).unwrap_or(u8::MAX)
}

/// A capacité's dé bonus to some tests: [`Character::bonus_dice`] lists those
/// held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BonusDie {
    voie: Voie,
    rank: Rank,
    characteristics: &'static [Characteristic],
    scope: &'static str,
    on_speciality: bool,
}

impl BonusDie {
    /// The voie of the capacité giving the dé bonus.
    #[must_use]
    pub const fn voie(self) -> Voie {
        self.voie
    }

    /// The rank of the capacité giving the dé bonus.
    #[must_use]
    pub const fn rank(self) -> Rank {
        self.rank
    }

    /// The identifier of the capacité giving the dé bonus.
    #[must_use]
    pub fn ability(self) -> AbilityId {
        self.voie.ability(self.rank)
    }

    /// The characteristics of the tests the dé bonus applies to, when the
    /// catalogue names them; empty otherwise.
    #[must_use]
    pub const fn characteristics(self) -> &'static [Characteristic] {
        self.characteristics
    }

    /// The tests the dé bonus applies to, in French and in a few words.
    #[must_use]
    pub const fn scope(self) -> &'static str {
        self.scope
    }

    /// Whether the dé bonus applies to the speciality chosen for its
    /// capacité ([`Voie::speciality`]).
    #[must_use]
    pub const fn on_speciality(self) -> bool {
        self.on_speciality
    }
}

/// How much a bonus gives.
#[derive(Debug, Clone, Copy)]
enum Amount {
    /// A flat amount.
    Flat(i32),
    /// This much per rank reached in the voie (CO.VOIE.RANK-SCALING).
    PerRank(i32),
}

/// A bonus of the catalogue.
#[derive(Debug)]
struct Row {
    voie: Voie,
    rank: u8,
    characteristics: &'static [Characteristic],
    amount: Amount,
    scope: &'static str,
    on_speciality: bool,
}

impl Row {
    /// The same bonus, to the tests of the speciality chosen.
    const fn on_speciality(mut self) -> Self {
        self.on_speciality = true;
        self
    }
}

const fn row(
    voie: Voie,
    rank: u8,
    characteristics: &'static [Characteristic],
    amount: Amount,
    scope: &'static str,
) -> Row {
    Row {
        voie,
        rank,
        characteristics,
        amount,
        scope,
        on_speciality: false,
    }
}

use Amount::{Flat, PerRank};
use Characteristic::{Agi, Cha, Con, For, Int, Per, Vol};

/// The usual "+5 to some tests".
const FIVE: Amount = Flat(5);

/// The bonuses of the capacités, in voie and rank order, a capacité giving
/// two amounts in two rows.
const BONUSES: &[Row] = &[
    // Amateur, Artilleur: against the shock of violence.
    row(
        Voie::ArmesDeContact,
        1,
        &[Vol],
        PerRank(1),
        "résister au choc d’un combat au contact",
    ),
    row(
        Voie::ArmesADistance,
        1,
        &[Vol],
        PerRank(1),
        "résister au choc d’une fusillade",
    ),
    row(
        Voie::ArmesLourdes,
        1,
        &[Vol],
        PerRank(1),
        "résister au choc des armes lourdes",
    ),
    // Artiste/artisan, Copiste, Inspiré, Célébrité.
    row(
        Voie::ArtsEtArtisanats,
        1,
        &[Int, Cha, Agi],
        FIVE,
        "culture et pratique de la discipline",
    )
    .on_speciality(),
    row(
        Voie::ArtsEtArtisanats,
        2,
        &[],
        FIVE,
        "discipline, copie d’une œuvre",
    )
    .on_speciality(),
    row(Voie::ArtsEtArtisanats, 3, &[], FIVE, "discipline").on_speciality(),
    row(Voie::ArtsEtArtisanats, 4, &[], FIVE, "discipline").on_speciality(),
    // Sportif amateur, Sportif confirmé, Déterminé, Athlète.
    row(
        Voie::Athletisme,
        1,
        &[For, Agi, Con],
        PerRank(1),
        "athlétisme, hors combat",
    ),
    row(Voie::Athletisme, 2, &[], Flat(2), "discipline sportive").on_speciality(),
    row(Voie::Athletisme, 3, &[], Flat(2), "discipline sportive").on_speciality(),
    row(Voie::Athletisme, 4, &[], Flat(2), "discipline sportive").on_speciality(),
    // Amateur.
    row(
        Voie::CombatAMainsNues,
        1,
        &[Vol],
        PerRank(1),
        "résister au choc d’une bagarre",
    ),
    // Membre, Connecté, Cadre, Influenceur.
    row(
        Voie::Corporations,
        1,
        &[Cha],
        PerRank(1),
        "relations avec les membres de la corporation",
    )
    .on_speciality(),
    row(
        Voie::Corporations,
        2,
        &[Int],
        FIVE,
        "informations accessibles à l’organisation",
    ),
    row(
        Voie::Corporations,
        3,
        &[Cha],
        FIVE,
        "obtenir aide ou moyens par l’autorité",
    ),
    row(
        Voie::Corporations,
        4,
        &[Cha],
        FIVE,
        "pousser une direction à l’illégalité",
    ),
    // Pickpocket, Cambrioleur, Imposteur.
    row(
        Voie::Crime,
        1,
        &[Agi, Cha],
        FIVE,
        "vol, triche, diversion, bluff",
    ),
    row(
        Voie::Crime,
        3,
        &[],
        FIVE,
        "serrures, coffres, systèmes de sécurité",
    ),
    row(Voie::Crime, 4, &[], FIVE, "se faire passer pour un autre"),
    // Convaincant, Éloquent, Rhétoricien, Meneur.
    row(Voie::Discours, 1, &[], FIVE, "approche sociale").on_speciality(),
    row(Voie::Discours, 2, &[], FIVE, "approche sociale").on_speciality(),
    row(Voie::Discours, 3, &[], FIVE, "approche sociale").on_speciality(),
    row(Voie::Discours, 4, &[], FIVE, "approche sociale").on_speciality(),
    // Furtif, Vigilant, Infiltré.
    row(
        Voie::Furtivite,
        1,
        &[Agi],
        FIVE,
        "se déplacer en silence, agir discrètement",
    ),
    row(
        Voie::Furtivite,
        2,
        &[Per],
        FIVE,
        "perception par les cinq sens",
    ),
    row(
        Voie::Furtivite,
        4,
        &[Per],
        FIVE,
        "trouver une cachette, une faille, un accès",
    ),
    // Fin limier.
    row(
        Voie::Investigation,
        1,
        &[Per, Int],
        PerRank(1),
        "fouilles : indices et détails",
    ),
    // Philologue, Linguiste, Cryptographe.
    row(
        Voie::Langues,
        2,
        &[Int],
        FIVE,
        "pays dont il parle la langue",
    ),
    row(
        Voie::Langues,
        3,
        &[Int],
        FIVE,
        "langue inconnue d’une famille parlée",
    ),
    row(
        Voie::Langues,
        4,
        &[Int],
        FIVE,
        "écritures et codes inconnus, mathématiques",
    ),
    // Secouriste: first aid, and against the shock of medical scenes.
    row(
        Voie::Medecine,
        1,
        &[Int],
        FIVE,
        "premiers soins, stabiliser un mourant",
    ),
    row(
        Voie::Medecine,
        1,
        &[Vol],
        PerRank(1),
        "résister au choc des scènes médicales",
    ),
    // Médecin, Chirurgien, Homme-médecine, Professeur.
    row(
        Voie::Medecine,
        2,
        &[Int],
        FIVE,
        "spécialité médicale, culture médicale",
    )
    .on_speciality(),
    row(Voie::Medecine, 3, &[], FIVE, "spécialité médicale").on_speciality(),
    row(Voie::Medecine, 4, &[], FIVE, "médecine traditionnelle").on_speciality(),
    row(
        Voie::Medecine,
        5,
        &[],
        FIVE,
        "relations sociales sur ses spécialités",
    ),
    // Observateur.
    row(
        Voie::Psychologie,
        1,
        &[Per],
        FIVE,
        "lire les émotions et les attitudes",
    ),
    // Introspectif: against trauma, and to calm someone.
    row(
        Voie::Psychologie,
        3,
        &[Vol],
        Flat(2),
        "résister aux événements traumatisants",
    ),
    row(
        Voie::Psychologie,
        3,
        &[Vol],
        FIVE,
        "calmer une personne choquée ou violente",
    ),
    // Geek, Ingénieur, Connecté, Optimisateur.
    row(
        Voie::Technologie,
        1,
        &[Int],
        PerRank(1),
        "comprendre, entretenir, réparer, saboter un système",
    ),
    row(Voie::Technologie, 2, &[], FIVE, "spécialité technique").on_speciality(),
    row(Voie::Technologie, 3, &[], FIVE, "spécialité technique").on_speciality(),
    row(Voie::Technologie, 4, &[], FIVE, "spécialité technique").on_speciality(),
    // Alumni: a discipline, and with peers.
    row(Voie::Sciences, 1, &[Int], FIVE, "discipline scientifique").on_speciality(),
    row(
        Voie::Sciences,
        1,
        &[Cha],
        PerRank(1),
        "relations avec ses pairs",
    ),
    // Professionnel, Esprit pratique, Expert reconnu, Personnalité publique.
    row(Voie::Sciences, 2, &[], FIVE, "discipline scientifique").on_speciality(),
    row(Voie::Sciences, 3, &[], FIVE, "discipline scientifique").on_speciality(),
    row(Voie::Sciences, 4, &[], FIVE, "discipline scientifique").on_speciality(),
    row(
        Voie::Sciences,
        5,
        &[],
        FIVE,
        "relations sociales sur ses spécialités",
    ),
    // Voyageur du monde, Explorateur.
    row(
        Voie::Survie,
        1,
        &[Int],
        PerRank(1),
        "coutumes locales, voyages, passer pour un local",
    ),
    row(Voie::Survie, 2, &[], FIVE, "survie dans le milieu").on_speciality(),
    // Baroudeur: against fatigue, and a second milieu.
    row(
        Voie::Survie,
        3,
        &[Con],
        PerRank(1),
        "résister à la fatigue, la faim, la maladie, le poison",
    ),
    row(Voie::Survie, 3, &[], FIVE, "survie dans le milieu").on_speciality(),
    // Guide.
    row(Voie::Survie, 4, &[], FIVE, "survie dans le milieu").on_speciality(),
    // Chauffeur: a vehicle, and its maintenance.
    row(Voie::Vehicules, 1, &[], Flat(2), "conduite du véhicule").on_speciality(),
    row(Voie::Vehicules, 1, &[], PerRank(1), "entretien courant"),
    // Conducteur expérimenté, Casse-cou, Pilote émérite, As des as: one
    // more vehicle each.
    row(Voie::Vehicules, 2, &[], Flat(2), "conduite du véhicule").on_speciality(),
    row(Voie::Vehicules, 3, &[], Flat(2), "conduite du véhicule").on_speciality(),
    row(Voie::Vehicules, 4, &[], Flat(2), "conduite du véhicule").on_speciality(),
    row(Voie::Vehicules, 5, &[], Flat(2), "conduite du véhicule").on_speciality(),
];

/// What a "+2 becomes +5" adds to the bonus it upgrades.
const UPGRADE: i32 = 3;

/// The capacités upgrading an earlier bonus of their voie (D-50): the voie,
/// the upgrading rank, then the rank and index of the bonus upgraded.
const UPGRADES: [(Voie, u8, u8, u8); 4] = [
    // Champion: Déterminé's second discipline.
    (Voie::Athletisme, 5, 3, 1),
    // Conducteur expérimenté: Chauffeur's vehicle.
    (Voie::Vehicules, 2, 1, 1),
    // Pilote émérite: Conducteur expérimenté's second vehicle.
    (Voie::Vehicules, 4, 2, 1),
    // As des as: Casse-cou's third vehicle.
    (Voie::Vehicules, 5, 3, 1),
];

/// The dé bonus of the capacités, in voie and rank order: the voie, the
/// rank, and the characteristics named.
const BONUS_DICE: &[(Voie, u8, &[Characteristic], &str, bool)] = &[
    // Génie, Champion, Boss, Cerveau, Marionnettiste, Fantôme.
    (Voie::ArtsEtArtisanats, 5, &[], "discipline choisie", true),
    (
        Voie::Athletisme,
        5,
        &[],
        "discipline sportive choisie",
        true,
    ),
    (Voie::Corporations, 5, &[], "un test des rangs 2 à 4", false),
    (Voie::Crime, 5, &[], "un test des rangs 1, 3 ou 4", false),
    (Voie::Discours, 5, &[], "approche choisie", true),
    (Voie::Furtivite, 5, &[], "un test des rangs 1 à 4", false),
    // Spécialiste, Mémoire vivante.
    (
        Voie::Investigation,
        2,
        &[Int],
        "spécialité d’investigation",
        true,
    ),
    (
        Voie::Investigation,
        4,
        &[],
        "deuxième spécialité d’investigation",
        true,
    ),
    // Hyperpolyglotte, Professeur, Clairvoyant, Génie, Personnalité
    // publique, Aventurier, As des as.
    (Voie::Langues, 5, &[], "tests des rangs 2 à 4", false),
    (
        Voie::Medecine,
        5,
        &[Int],
        "spécialité médicale choisie",
        true,
    ),
    (Voie::Psychologie, 5, &[Per], "psychologie", false),
    (
        Voie::Technologie,
        5,
        &[],
        "spécialité technique choisie",
        true,
    ),
    (
        Voie::Sciences,
        5,
        &[],
        "discipline scientifique choisie",
        true,
    ),
    (
        Voie::Survie,
        5,
        &[],
        "connaissance ou survie dans le milieu choisi",
        true,
    ),
    (Voie::Vehicules, 5, &[], "véhicule choisi", false),
];

/// The bonuses of the capacités the character holds now, in voie and rank
/// order.
pub(crate) fn held(character: &Character) -> Vec<TestBonus> {
    TestBonus::all()
        .filter(|bonus| character.has_rank(bonus.voie, bonus.rank))
        .collect()
}

/// The dé bonus of the capacités the character holds now, in voie and rank
/// order.
pub(crate) fn bonus_dice(character: &Character) -> Vec<BonusDie> {
    BONUS_DICE
        .iter()
        .map(
            |(voie, rank, characteristics, scope, on_speciality)| BonusDie {
                voie: *voie,
                rank: Rank::new(*rank),
                characteristics,
                scope,
                on_speciality: *on_speciality,
            },
        )
        .filter(|die| character.has_rank(die.voie, die.rank))
        .collect()
}

/// The value of `bonus`, the capacité as source, with the upgrades of the
/// capacités held; `None` when the capacité is not held.
pub(crate) fn value(character: &Character, bonus: TestBonus) -> Option<DerivedValue> {
    if !character.has_rank(bonus.voie, bonus.rank) {
        return None;
    }
    let amount = match bonus.row()?.amount {
        Flat(amount) => amount,
        PerRank(amount) => {
            let rank = character.highest_rank(bonus.voie).map_or(0, Rank::get);
            amount * i32::from(rank)
        }
    };
    let value = DerivedValue::new().with(Source::Ability(bonus.ability()), amount);
    Some(
        UPGRADES
            .iter()
            .filter(|(voie, by, rank, index)| {
                *voie == bonus.voie
                    && Rank::new(*rank) == bonus.rank
                    && *index == bonus.index
                    && character.has_rank(*voie, Rank::new(*by))
            })
            .fold(value, |value, (voie, by, _, _)| {
                value.with(Source::Ability(voie.ability(Rank::new(*by))), UPGRADE)
            }),
    )
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Category, Subject};

    use super::*;
    use crate::{Step, Target, Value, XpSpending};

    fn ranks(character: Character, voie: Voie, highest: u8) -> Character {
        (1..=highest).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    fn bonus(voie: Voie, rank: u8, index: u8) -> TestBonus {
        TestBonus {
            voie,
            rank: Rank::new(rank),
            index,
        }
    }

    fn breakdown(character: &Character, bonus: TestBonus) -> Vec<(Source, i32)> {
        character
            .value(Value::TestBonus(bonus))
            .map(|value| {
                value
                    .contributions()
                    .iter()
                    .map(|c| (c.source().clone(), c.amount()))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn the_row_builder_makes_the_row_it_names() {
        let built = row(Voie::Crime, 1, &[Agi, Cha], FIVE, "vol");
        assert_eq!((built.voie, built.rank), (Voie::Crime, 1));
        assert_eq!(built.characteristics, [Agi, Cha]);
        assert!(matches!(built.amount, Amount::Flat(5)));
        assert_eq!(built.scope, "vol");
    }

    #[test]
    fn bonuses_to_a_speciality_say_so() {
        assert!(bonus(Voie::ArtsEtArtisanats, 1, 1).on_speciality());
        assert!(bonus(Voie::Sciences, 1, 1).on_speciality());
        assert!(!bonus(Voie::Sciences, 1, 2).on_speciality());
        assert!(!bonus(Voie::Crime, 1, 1).on_speciality());
        let artist = ranks(Character::new(), Voie::ArtsEtArtisanats, 5);
        assert!(artist.bonus_dice().iter().all(|die| die.on_speciality()));
        let boss = ranks(Character::new(), Voie::Corporations, 5);
        assert!(boss.bonus_dice().iter().all(|die| !die.on_speciality()));
        assert!(
            TestBonus::all()
                .filter(|bonus| bonus.on_speciality())
                .all(|bonus| bonus.voie().speciality(bonus.rank()).is_some())
        );
        assert!(
            BONUS_DICE
                .iter()
                .filter(|(.., on_speciality)| *on_speciality)
                .all(|(voie, rank, ..)| voie.speciality(Rank::new(*rank)).is_some())
        );
    }

    #[test]
    fn every_bonus_names_its_tests() {
        assert!(TestBonus::all().all(|bonus| !bonus.scope().is_empty()));
        assert!(
            BONUS_DICE
                .iter()
                .all(|(_, _, _, scope, _)| !scope.is_empty())
        );
        assert_eq!(
            bonus(Voie::Medecine, 1, 1).scope(),
            "premiers soins, stabiliser un mourant"
        );
        assert_eq!(
            bonus(Voie::Medecine, 1, 2).scope(),
            "résister au choc des scènes médicales"
        );
        let character = ranks(Character::new(), Voie::Psychologie, 5);
        let dice = character.bonus_dice();
        assert_eq!(
            dice.iter().map(|die| die.scope()).collect::<Vec<_>>(),
            ["psychologie"]
        );
    }

    #[test]
    fn the_catalogue_follows_voie_and_rank_order() {
        let all: Vec<_> = TestBonus::all().collect();
        let mut sorted = all.clone();
        sorted.sort();
        assert_eq!(all, sorted);
        assert!(all.iter().all(|b| b.voie.ability_name(b.rank).is_some()));
        assert!(
            BONUS_DICE
                .windows(2)
                .all(|pair| (pair[0].0, pair[0].1) < (pair[1].0, pair[1].1))
        );
    }

    #[test]
    fn flat_bonuses_name_their_capacite() {
        let character = ranks(Character::new(), Voie::Crime, 1);
        let pickpocket = bonus(Voie::Crime, 1, 1);
        assert_eq!(character.test_bonuses(), [pickpocket]);
        assert_eq!(pickpocket.characteristics(), [Agi, Cha]);
        assert_eq!(pickpocket.ability().as_str(), "coc.crime.1");
        assert_eq!(
            breakdown(&character, pickpocket),
            [(Source::Ability(Voie::Crime.ability(Rank::new(1))), 5)]
        );
    }

    #[test]
    fn per_rank_bonuses_read_the_rank_reached() {
        let character = ranks(Character::new(), Voie::Investigation, 3);
        assert_eq!(
            breakdown(&character, bonus(Voie::Investigation, 1, 1)),
            [(
                Source::Ability(Voie::Investigation.ability(Rank::new(1))),
                3
            )]
        );
    }

    #[test]
    fn a_capacite_may_give_two_bonuses() {
        let character = ranks(Character::new(), Voie::Medecine, 2);
        assert_eq!(
            character.test_bonuses(),
            [
                bonus(Voie::Medecine, 1, 1),
                bonus(Voie::Medecine, 1, 2),
                bonus(Voie::Medecine, 2, 1),
            ]
        );
        let total = |b| character.value(Value::TestBonus(b)).map(|v| v.total());
        assert_eq!(total(bonus(Voie::Medecine, 1, 1)), Some(5));
        assert_eq!(total(bonus(Voie::Medecine, 1, 2)), Some(2));
        assert_eq!(bonus(Voie::Medecine, 1, 2).characteristics(), [Vol]);
    }

    #[test]
    fn upgrades_raise_an_earlier_bonus() {
        let ability = |rank| Source::Ability(Voie::Vehicules.ability(Rank::new(rank)));
        let character = ranks(Character::new(), Voie::Vehicules, 5);
        assert_eq!(
            breakdown(&character, bonus(Voie::Vehicules, 1, 1)),
            [(ability(1), 2), (ability(2), 3)]
        );
        assert_eq!(
            breakdown(&character, bonus(Voie::Vehicules, 2, 1)),
            [(ability(2), 2), (ability(4), 3)]
        );
        assert_eq!(
            breakdown(&character, bonus(Voie::Vehicules, 3, 1)),
            [(ability(3), 2), (ability(5), 3)]
        );
        assert_eq!(
            breakdown(&character, bonus(Voie::Vehicules, 4, 1)),
            [(ability(4), 2)]
        );
    }

    #[test]
    fn bonuses_follow_the_ranks_bought_with_xp() {
        let character = Character::new()
            .with_rank(Voie::Furtivite, 1)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Furtivite, 2));
        assert_eq!(
            character.test_bonuses(),
            [bonus(Voie::Furtivite, 1, 1), bonus(Voie::Furtivite, 2, 1)]
        );
        assert_eq!(
            character.value(Value::TestBonus(bonus(Voie::Crime, 1, 1))),
            None
        );
    }

    #[test]
    fn bonus_dice_list_the_capacites_held() {
        let character = ranks(Character::new(), Voie::Investigation, 4);
        let dice = character.bonus_dice();
        assert_eq!(
            dice.iter()
                .map(|die| (die.ability(), die.characteristics()))
                .collect::<Vec<_>>(),
            [
                (Voie::Investigation.ability(Rank::new(2)), [Int].as_slice()),
                (Voie::Investigation.ability(Rank::new(4)), [].as_slice()),
            ]
        );
        assert_eq!(dice[0].voie(), Voie::Investigation);
        assert_eq!(dice[0].rank(), Rank::new(2));
    }

    #[test]
    fn bonuses_are_values_with_their_own_names() {
        let second = bonus(Voie::Sciences, 1, 2);
        let value = Value::TestBonus(second);
        assert_eq!(value.id().as_str(), "coc.test-bonus.sciences.1.2");
        assert_eq!(Value::from_id("coc.test-bonus.sciences.1.2"), Some(value));
        assert_eq!(value.name(), "Bonus aux tests : Alumni (2)");
        assert_eq!(
            Value::TestBonus(bonus(Voie::Crime, 4, 1)).name(),
            "Bonus aux tests : Imposteur"
        );
    }

    #[test]
    fn adjusted_bonuses_surface_in_the_voies_step() {
        let pickpocket = bonus(Voie::Crime, 1, 1);
        let character = ranks(Character::new(), Voie::Crime, 1).with_adjustment(
            Target::Value(Value::TestBonus(pickpocket)),
            Adjustment::new(2, Author::new("gm"), "a gifted thief"),
        );
        assert_eq!(
            character
                .value(Value::TestBonus(pickpocket))
                .map(|v| v.total()),
            Some(7)
        );
        let notice = character
            .violations()
            .into_iter()
            .find(|v| v.subject() == &Subject::Value(Value::TestBonus(pickpocket).id()))
            .expect("the adjustment surfaces");
        assert_eq!(notice.category(), Category::Notice);
        assert_eq!(Step::of(&notice), Step::Voies);
    }
}
