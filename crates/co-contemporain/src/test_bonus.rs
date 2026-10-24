//! The capacités' permanent bonuses to the character's own tests, and their
//! dé bonus (COC.CNT.VOIES, D-50).
//!
//! A capacité gives one bonus per amount, with the characteristics its
//! catalogue entry names; the fields of the tests are not recorded. Bonuses
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
}

const fn row(
    voie: Voie,
    rank: u8,
    characteristics: &'static [Characteristic],
    amount: Amount,
) -> Row {
    Row {
        voie,
        rank,
        characteristics,
        amount,
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
    row(Voie::ArmesDeContact, 1, &[Vol], PerRank(1)),
    row(Voie::ArmesADistance, 1, &[Vol], PerRank(1)),
    row(Voie::ArmesLourdes, 1, &[Vol], PerRank(1)),
    // Artiste/artisan, Copiste, Inspiré, Célébrité.
    row(Voie::ArtsEtArtisanats, 1, &[Int, Cha, Agi], FIVE),
    row(Voie::ArtsEtArtisanats, 2, &[], FIVE),
    row(Voie::ArtsEtArtisanats, 3, &[], FIVE),
    row(Voie::ArtsEtArtisanats, 4, &[], FIVE),
    // Sportif amateur, Sportif confirmé, Déterminé, Athlète.
    row(Voie::Athletisme, 1, &[For, Agi, Con], PerRank(1)),
    row(Voie::Athletisme, 2, &[], Flat(2)),
    row(Voie::Athletisme, 3, &[], Flat(2)),
    row(Voie::Athletisme, 4, &[], Flat(2)),
    // Amateur.
    row(Voie::CombatAMainsNues, 1, &[Vol], PerRank(1)),
    // Membre, Connecté, Cadre, Influenceur.
    row(Voie::Corporations, 1, &[Cha], PerRank(1)),
    row(Voie::Corporations, 2, &[Int], FIVE),
    row(Voie::Corporations, 3, &[Cha], FIVE),
    row(Voie::Corporations, 4, &[Cha], FIVE),
    // Pickpocket, Cambrioleur, Imposteur.
    row(Voie::Crime, 1, &[Agi, Cha], FIVE),
    row(Voie::Crime, 3, &[], FIVE),
    row(Voie::Crime, 4, &[], FIVE),
    // Convaincant, Éloquent, Rhétoricien, Meneur.
    row(Voie::Discours, 1, &[], FIVE),
    row(Voie::Discours, 2, &[], FIVE),
    row(Voie::Discours, 3, &[], FIVE),
    row(Voie::Discours, 4, &[], FIVE),
    // Furtif, Vigilant, Infiltré.
    row(Voie::Furtivite, 1, &[Agi], FIVE),
    row(Voie::Furtivite, 2, &[Per], FIVE),
    row(Voie::Furtivite, 4, &[Per], FIVE),
    // Fin limier.
    row(Voie::Investigation, 1, &[Per, Int], PerRank(1)),
    // Philologue, Linguiste, Cryptographe.
    row(Voie::Langues, 2, &[Int], FIVE),
    row(Voie::Langues, 3, &[Int], FIVE),
    row(Voie::Langues, 4, &[Int], FIVE),
    // Secouriste: first aid, and against the shock of medical scenes.
    row(Voie::Medecine, 1, &[Int], FIVE),
    row(Voie::Medecine, 1, &[Vol], PerRank(1)),
    // Médecin, Chirurgien, Homme-médecine, Professeur.
    row(Voie::Medecine, 2, &[Int], FIVE),
    row(Voie::Medecine, 3, &[], FIVE),
    row(Voie::Medecine, 4, &[], FIVE),
    row(Voie::Medecine, 5, &[], FIVE),
    // Observateur.
    row(Voie::Psychologie, 1, &[Per], FIVE),
    // Introspectif: against trauma, and to calm someone.
    row(Voie::Psychologie, 3, &[Vol], Flat(2)),
    row(Voie::Psychologie, 3, &[Vol], FIVE),
    // Geek, Ingénieur, Connecté, Optimisateur.
    row(Voie::Technologie, 1, &[Int], PerRank(1)),
    row(Voie::Technologie, 2, &[], FIVE),
    row(Voie::Technologie, 3, &[], FIVE),
    row(Voie::Technologie, 4, &[], FIVE),
    // Alumni: a discipline, and with peers.
    row(Voie::Sciences, 1, &[Int], FIVE),
    row(Voie::Sciences, 1, &[Cha], PerRank(1)),
    // Professionnel, Esprit pratique, Expert reconnu, Personnalité publique.
    row(Voie::Sciences, 2, &[], FIVE),
    row(Voie::Sciences, 3, &[], FIVE),
    row(Voie::Sciences, 4, &[], FIVE),
    row(Voie::Sciences, 5, &[], FIVE),
    // Voyageur du monde, Explorateur.
    row(Voie::Survie, 1, &[Int], PerRank(1)),
    row(Voie::Survie, 2, &[], FIVE),
    // Baroudeur: against fatigue, and a second milieu.
    row(Voie::Survie, 3, &[Con], PerRank(1)),
    row(Voie::Survie, 3, &[], FIVE),
    // Guide.
    row(Voie::Survie, 4, &[], FIVE),
    // Chauffeur: a vehicle, and its maintenance.
    row(Voie::Vehicules, 1, &[], Flat(2)),
    row(Voie::Vehicules, 1, &[], PerRank(1)),
    // Conducteur expérimenté, Casse-cou, Pilote émérite, As des as: one
    // more vehicle each.
    row(Voie::Vehicules, 2, &[], Flat(2)),
    row(Voie::Vehicules, 3, &[], Flat(2)),
    row(Voie::Vehicules, 4, &[], Flat(2)),
    row(Voie::Vehicules, 5, &[], Flat(2)),
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
const BONUS_DICE: &[(Voie, u8, &[Characteristic])] = &[
    // Génie, Champion, Boss, Cerveau, Marionnettiste, Fantôme.
    (Voie::ArtsEtArtisanats, 5, &[]),
    (Voie::Athletisme, 5, &[]),
    (Voie::Corporations, 5, &[]),
    (Voie::Crime, 5, &[]),
    (Voie::Discours, 5, &[]),
    (Voie::Furtivite, 5, &[]),
    // Spécialiste, Mémoire vivante.
    (Voie::Investigation, 2, &[Int]),
    (Voie::Investigation, 4, &[]),
    // Hyperpolyglotte, Professeur, Clairvoyant, Génie, Personnalité
    // publique, Aventurier, As des as.
    (Voie::Langues, 5, &[]),
    (Voie::Medecine, 5, &[Int]),
    (Voie::Psychologie, 5, &[Per]),
    (Voie::Technologie, 5, &[]),
    (Voie::Sciences, 5, &[]),
    (Voie::Survie, 5, &[]),
    (Voie::Vehicules, 5, &[]),
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
        .map(|(voie, rank, characteristics)| BonusDie {
            voie: *voie,
            rank: Rank::new(*rank),
            characteristics,
        })
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
