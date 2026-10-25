//! The capacités' permanent bonuses to the character's own tests, and their
//! dé bonus (COF.DER.COMPETENCE, D-50).
//!
//! A capacité gives one bonus per amount, with the characteristics its
//! catalogue entry names and the tests it covers in a few words of our own
//! (its scope); the +15 cap and the one bonus per source are not checked. Bonuses used once
//! per combat, given to others, to attacks, or testing one characteristic in
//! place of another are not test bonuses.

use co_kernel::{AbilityId, Characteristic, Characteristics, DerivedValue, Rank, ValueId};

use crate::{
    AbilityOption, Character, Profil, Voie,
    ability::{self, Amount, Condition, Effect, When},
};

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
    /// catalogue names them, or the one chosen (Tatouages); empty otherwise.
    #[must_use]
    pub fn characteristics(self) -> &'static [Characteristic] {
        rows(self)
            .next()
            .map_or(&[], |(_, characteristics, _, _)| characteristics)
    }

    /// The tests the bonus applies to, in French and in a few words
    /// (`discrétion, déguisement`), besides the characteristics named.
    #[must_use]
    pub fn scope(self) -> &'static str {
        rows(self).next().map_or("", |(_, _, scope, _)| scope)
    }

    /// Whether the bonus applies to the speciality chosen for its capacité
    /// ([`Voie::speciality`]): a capacité asking for one gives a single
    /// bonus, to it.
    #[must_use]
    pub const fn on_speciality(self) -> bool {
        self.voie.speciality(self.rank).is_some()
    }

    /// The identifier of the bonus as a value,
    /// `cof.test-bonus.<voie>.<rank>.<n>`.
    pub(crate) fn id(self) -> ValueId {
        let voie = self.voie.id();
        let voie = voie.as_str();
        ValueId::new(format!(
            "cof.test-bonus.{}.{}.{}",
            voie.strip_prefix("cof.").unwrap_or(voie),
            self.rank.get(),
            self.index
        ))
    }

    /// The French display name of the bonus as a value, with its
    /// characteristics when its capacité gives one per characteristic.
    pub(crate) fn name(self) -> String {
        let ability = self.voie.ability_name(self.rank).unwrap_or_default();
        let siblings = Self::all()
            .filter(|other| other.voie == self.voie && other.rank == self.rank)
            .count();
        if siblings > 1 {
            let characteristics: Vec<_> = self
                .characteristics()
                .iter()
                .map(|c| c.abbreviation())
                .collect();
            format!(
                "Bonus aux tests : {ability} ({})",
                characteristics.join(", ")
            )
        } else {
            format!("Bonus aux tests : {ability}")
        }
    }

    /// Every bonus of the catalogue, in voie and rank order.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        let mut keys: Vec<Self> = BONUSES
            .iter()
            .map(|(index, _, _, effect)| key(*index, effect))
            .collect();
        keys.dedup();
        keys.into_iter()
    }
}

/// A row of the catalogue: the bonus's index within its capacité, the
/// characteristics named, the tests in a few words, and its effect.
type Bonus = (u8, &'static [Characteristic], &'static str, Effect);

/// A dé bonus of the catalogue: the characteristics named, the tests in a
/// few words, and the capacité with the equipment or option it needs.
type Die = (&'static [Characteristic], &'static str, Effect);

/// The scope of a bonus to every test of its characteristics.
const EVERY_TEST: &str = "tous les tests";

/// The scope of Éclectique.
const ECLECTIQUE: &str = "tous les tests de compétence";

/// The bonus a row of the catalogue gives.
const fn key(index: u8, effect: &Effect) -> TestBonus {
    TestBonus {
        voie: effect.voie(),
        rank: Rank::new(effect.rank()),
        index,
    }
}

/// The rows of the catalogue giving `bonus`.
fn rows(bonus: TestBonus) -> impl Iterator<Item = &'static Bonus> {
    BONUSES
        .iter()
        .filter(move |(index, _, _, effect)| key(*index, effect) == bonus)
}

/// A capacité's dé bonus to some tests: [`Character::bonus_dice`] lists those
/// held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BonusDie {
    voie: Voie,
    rank: Rank,
    characteristics: &'static [Characteristic],
    scope: &'static str,
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
    /// catalogue names them, or the one chosen (Formation d'élite); empty
    /// otherwise.
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
        self.voie.speciality(self.rank).is_some()
    }
}

use Characteristic::{Agi, Cha, Con, For, Int, Per, Vol};

/// A competence bonus of a voie de profil, "rang + 2" in the catalogue.
const fn competence(voie: Voie, rank: u8, scope: &'static str) -> Bonus {
    (1, &[], scope, Effect::new(voie, rank, Amount::Competence))
}

/// "+ rank" to resist something, an opposed test included (D-50).
const fn resisting(voie: Voie, rank: u8, scope: &'static str) -> Bonus {
    (1, &[], scope, Effect::new(voie, rank, Amount::RankPlus(0)))
}

/// A voie de peuple's +3 (COF.DER.COMPETENCE).
const fn peuple(
    voie: Voie,
    rank: u8,
    characteristics: &'static [Characteristic],
    scope: &'static str,
) -> Bonus {
    (
        1,
        characteristics,
        scope,
        Effect::flat(voie, rank, PEUPLE_BONUS),
    )
}

/// The competence bonus of a voie de peuple.
const PEUPLE_BONUS: i32 = 3;

/// `c` alone, as the characteristics a bonus names.
const fn alone(c: Characteristic) -> &'static [Characteristic] {
    match c {
        Agi => &[Agi],
        Con => &[Con],
        For => &[For],
        Per => &[Per],
        Cha => &[Cha],
        Int => &[Int],
        Vol => &[Vol],
    }
}

/// Tatouages' +3 to the tests of the characteristic chosen, as the bonus
/// numbered `index`.
const fn tattoo(index: u8, c: Characteristic) -> Bonus {
    (
        index,
        alone(c),
        EVERY_TEST,
        Effect::flat(Voie::Pagne, 3, 3).when(When::Chosen(AbilityOption::Characteristic(c))),
    )
}

/// The bonuses of the capacités, in voie and rank order, a bonus summing
/// the rows sharing its index.
const BONUSES: &[Bonus] = &[
    // Mécanismes, Tir de grenaille, Pilier de bar.
    competence(Voie::Artilleur, 1, "mécanismes, engins de siège"),
    competence(Voie::Explosifs, 1, "explosifs, artifices"),
    competence(Voie::Mercenaire, 1, "vie de taverne, résister à l’alcool"),
    // Chant des héros (besides the spell), Acrobate, Grâce féline, Charmant,
    // Rumeurs et légendes.
    competence(Voie::Musicien, 1, "musique, chant"),
    competence(
        Voie::Saltimbanque,
        1,
        "acrobaties, équilibre, saut, escalade",
    ),
    competence(Voie::Saltimbanque, 2, "danse, mime, jonglerie"),
    competence(Voie::Seduction, 1, "séduire, convaincre, mentir"),
    competence(
        Voie::Vagabond,
        1,
        "légendes, identifier les objets magiques",
    ),
    // Éclectique: +1, and +1 per rank 4 in a barde voie.
    (1, &[], ECLECTIQUE, Effect::flat(Voie::Vagabond, 2, 1)),
    (
        1,
        &[],
        ECLECTIQUE,
        Effect::new(Voie::Vagabond, 2, Amount::PerVoieAt(Profil::Barde, 4, 1)),
    ),
    // Survie, Éclaireur (outdoors).
    competence(Voie::Survie, 1, "escalade, survie en milieu sauvage"),
    competence(
        Voie::Traqueur,
        1,
        "discrétion, vigilance, pistage en extérieur",
    ),
    // Discrétion, Baratin, Agile, Doigts agiles, Aux aguets, Attaque en
    // finesse.
    competence(
        Voie::Assassin,
        1,
        "discrétion, déguisement, dissimuler une arme",
    ),
    competence(
        Voie::Aventurier,
        1,
        "bluff, séduction, négociation, marché noir",
    ),
    competence(Voie::Deplacement, 1, "déplacement"),
    competence(
        Voie::Roublard,
        1,
        "crochetage, pièges, vol à la tire, estimation",
    ),
    competence(
        Voie::Roublard,
        2,
        "fouille, pièges, passages secrets, embuscades",
    ),
    competence(Voie::Spadassin, 1, "intimidation"),
    // Vigueur, Tatouages (a magic bonus).
    competence(Voie::Pagne, 1, "course, saut, escalade"),
    tattoo(1, Agi),
    tattoo(2, Con),
    tattoo(3, For),
    tattoo(4, Per),
    tattoo(5, Cha),
    tattoo(6, Int),
    tattoo(7, Vol),
    // Réflexes éclair: AGI to dodge.
    (
        1,
        &[Agi],
        "esquiver",
        Effect::new(Voie::Pourfendeur, 1, Amount::Competence),
    ),
    // Proche de la nature, Vigilance, Cri de guerre, Fidèle monture.
    competence(Voie::Primitif, 1, "survie et discrétion en extérieur"),
    competence(Voie::Primitif, 3, "repérer pièges et embuscades"),
    competence(Voie::Rage, 1, "résister à la peur"),
    competence(Voie::Cavalier, 1, "équitation, dressage"),
    // Ignorer la douleur, Sans peur, Éduqué, Autorité naturelle.
    competence(Voie::Preux, 1, "haranguer une foule"),
    competence(Voie::MeneurDHomme, 1, "stratégie, commandement"),
    competence(
        Voie::Noblesse,
        1,
        "histoire, héraldique, géographie, étiquette",
    ),
    competence(Voie::Noblesse, 3, "commander, intimider"),
    // Protéger un allié, Armes de prédilection, Robustesse, Teigneux.
    competence(Voie::Bouclier, 1, "éviter la surprise"),
    competence(
        Voie::MaitreDArmes,
        1,
        "expertise des armes, réputation martiale",
    ),
    competence(
        Voie::Resistance,
        1,
        "résister à l’effort, à la chaleur, au froid",
    ),
    // Résilient.
    resisting(Voie::Resistance, 2, "résister à étourdi et affaibli"),
    competence(Voie::Soldat, 1, "résister à l’alcool et aux privations"),
    // Injonction, Mirage (besides the spells), Fortifiant, Morsure de la
    // forge (besides the spell), Grosse tête.
    competence(Voie::Envouteur, 1, "persuasion, séduction"),
    competence(Voie::Illusions, 1, "tromper"),
    competence(Voie::Elixirs, 1, "alchimie, identifier les potions"),
    competence(Voie::Metal, 1, "forge"),
    competence(Voie::Golem, 1, "bricolage, sciences"),
    // Ténèbres (besides the spell), Langage des animaux, Vitesse du félin,
    // Maître de la survie, Baies magiques and Peau d'écorce (besides the
    // spells).
    competence(Voie::SombreMagie, 1, "savoirs des ténèbres"),
    competence(Voie::Animaux, 1, "influencer les animaux"),
    competence(Voie::Fauve, 1, "course, escalade, saut"),
    competence(Voie::Nature, 1, "survie en extérieur"),
    competence(Voie::Protecteur, 1, "vigilance et discrétion en extérieur"),
    competence(Voie::Vegetaux, 1, "connaissance des plantes"),
    // Agilité du singe, Pacifisme, Pas du vent.
    competence(Voie::Maitrise, 1, "acrobaties, esquive"),
    competence(Voie::Meditation, 1, "empathie, apaiser"),
    competence(Voie::Vent, 1, "saut, course, escalade"),
    // Prédicateur, Bénédiction and Récupération mineure (besides the
    // spells), Vigueur divine.
    competence(Voie::Foi, 1, "convaincre, convertir"),
    competence(Voie::Priere, 1, "théologie"),
    competence(Voie::Soins, 1, "médecine"),
    competence(Voie::Soins, 2, "résister aux poisons et aux maladies"),
    // Impressionnant, Lumière intérieure, Lumière des étoiles, Don étrange,
    // Petit pote, Petite taille, Diversité, Habitant des tunnels.
    peuple(Voie::DemiOrc, 1, &[], "intimidation"),
    peuple(Voie::ElfeHaut, 1, &[Int, Cha], "érudition, arts"),
    // Force d'âme.
    resisting(Voie::ElfeHaut, 2, "résister aux sorts"),
    peuple(Voie::ElfeSylvain, 1, &[], "survie en forêt"),
    peuple(Voie::Gnome, 1, &[Int], "sciences"),
    peuple(Voie::Gnome, 2, &[], "relations sociales, sauf intimidation"),
    peuple(Voie::Halfelin, 1, &[], "discrétion, vol"),
    // Résistance légendaire.
    resisting(Voie::Halfelin, 2, "résister aux sorts"),
    peuple(Voie::Humain, 1, &[], "domaines de l’origine"),
    peuple(
        Voie::Nain,
        1,
        &[],
        "pierre, architecture, mines, passages secrets",
    ),
    // Occultisme: the voie du mage's rank 1.
    competence(Voie::Mage, 1, "connaissances magiques"),
];

/// A héroïque capacité's dé bonus to the tests of its characteristic.
const fn heroic(voie: Voie, rank: u8, c: Characteristic) -> Die {
    (alone(c), EVERY_TEST, Effect::flat(voie, rank, 1))
}

/// Formation d'élite's dé bonus to the tests of the characteristic chosen.
const fn elite(c: Characteristic) -> Die {
    (
        alone(c),
        EVERY_TEST,
        Effect::flat(Voie::Noblesse, 5, 1).when(When::Chosen(AbilityOption::Characteristic(c))),
    )
}

/// The dé bonus of the capacités, in voie and rank order.
const BONUS_DICE: &[Die] = &[
    // Constitution héroïque, Agilité héroïque, Charisme héroïque.
    heroic(Voie::Mercenaire, 4, Con),
    heroic(Voie::Pistolero, 4, Agi),
    heroic(Voie::Seduction, 4, Cha),
    // Constitution héroïque, Perception héroïque.
    heroic(Voie::Survie, 4, Con),
    heroic(Voie::Traqueur, 4, Per),
    // Combattant héroïque, with AGI raised.
    (
        &[Agi],
        EVERY_TEST,
        Effect::flat(Voie::CombatADeuxArmes, 4, 1)
            .when(When::Chosen(AbilityOption::Characteristic(Agi))),
    ),
    // Charisme héroïque, Agilité héroïque, Expert en criminalité.
    heroic(Voie::Aventurier, 4, Cha),
    heroic(Voie::Deplacement, 4, Agi),
    (
        &[],
        "investigation, contrefaçon",
        Effect::flat(Voie::Roublard, 4, 1),
    ),
    // Force héroïque, Constitution héroïque.
    heroic(Voie::Brute, 4, For),
    heroic(Voie::Pagne, 4, Con),
    // Force héroïque, Charisme héroïque.
    heroic(Voie::Guerre, 4, For),
    heroic(Voie::Preux, 4, Cha),
    // Formation d'élite.
    elite(Agi),
    elite(Con),
    elite(For),
    elite(Per),
    elite(Cha),
    elite(Int),
    elite(Vol),
    // Constitution héroïque, Force héroïque.
    heroic(Voie::Resistance, 4, Con),
    heroic(Voie::Soldat, 4, For),
    // Perception héroïque, Endurer, Pacte ténébreux.
    heroic(Voie::Divination, 4, Per),
    heroic(Voie::Metal, 5, Con),
    heroic(Voie::SombreMagie, 5, Con),
    // Constitution héroïque, Perception héroïque.
    heroic(Voie::Nature, 4, Con),
    heroic(Voie::Protecteur, 4, Per),
    // Ascétisme, Volonté héroïque, Agilité héroïque.
    heroic(Voie::EnergieVitale, 5, Con),
    heroic(Voie::Meditation, 4, Vol),
    heroic(Voie::Vent, 4, Agi),
    // Volonté héroïque.
    heroic(Voie::Priere, 4, Vol),
    // Vêtements sacrés, without armour, against mind control.
    (
        &[],
        "résister au contrôle mental",
        Effect::flat(Voie::Spiritualite, 1, 1)
            .only(Condition::NoArmour)
            .when(When::NotChosen(AbilityOption::Armour)),
    ),
    // Charisme héroïque.
    heroic(Voie::Spiritualite, 4, Cha),
    // Esprit supérieur.
    heroic(Voie::Mage, 4, Int),
];

/// Whether the effects of `bonus` apply to the character.
fn applies(character: &Character, values: &Characteristics, bonus: TestBonus) -> bool {
    let effects: Vec<Effect> = rows(bonus).map(|(_, _, _, effect)| *effect).collect();
    ability::any(character, values, &effects)
}

/// The bonuses of the capacités the character holds now, those depending on
/// an option once it is chosen, in voie and rank order.
pub(crate) fn held(character: &Character) -> Vec<TestBonus> {
    let values = character.characteristics();
    TestBonus::all()
        .filter(|bonus| applies(character, &values, *bonus))
        .collect()
}

/// The dé bonus of the capacités the character holds now, with the equipment
/// it wears, in voie and rank order.
pub(crate) fn bonus_dice(character: &Character) -> Vec<BonusDie> {
    let values = character.characteristics();
    BONUS_DICE
        .iter()
        .filter(|(_, _, effect)| ability::any(character, &values, &[*effect]))
        .map(|(characteristics, scope, effect)| BonusDie {
            voie: effect.voie(),
            rank: Rank::new(effect.rank()),
            characteristics,
            scope,
        })
        .collect()
}

/// The value of `bonus`, the capacité as source; `None` while it does not
/// apply.
pub(crate) fn value(
    character: &Character,
    values: &Characteristics,
    bonus: TestBonus,
) -> Option<DerivedValue> {
    let effects: Vec<Effect> = rows(bonus).map(|(_, _, _, effect)| *effect).collect();
    ability::any(character, values, &effects)
        .then(|| ability::apply(DerivedValue::new(), character, values, &effects))
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Category, Source, Subject};

    use super::*;
    use crate::{Armour, LevelUp, Peuple, Step, Target, Value};

    fn bonus(voie: Voie, rank: u8, index: u8) -> TestBonus {
        TestBonus {
            voie,
            rank: Rank::new(rank),
            index,
        }
    }

    fn total(character: &Character, bonus: TestBonus) -> Option<i32> {
        character.value(Value::TestBonus(bonus)).map(|v| v.total())
    }

    fn ranks(character: Character, voie: Voie, highest: u8) -> Character {
        (1..=highest).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    #[test]
    fn the_row_builders_make_the_rows_they_name() {
        for c in Characteristic::ALL {
            assert_eq!(alone(c), [c]);
        }
        let place = |effect: Effect| (effect.voie(), effect.rank());
        let (index, named, scope, effect) = competence(Voie::Assassin, 1, "discrétion");
        assert_eq!((index, named, scope), (1, [].as_slice(), "discrétion"));
        assert_eq!(place(effect), (Voie::Assassin, 1));
        let (index, named, scope, effect) = peuple(Voie::ElfeHaut, 1, &[Int, Cha], "arts");
        assert_eq!((index, named, scope), (1, [Int, Cha].as_slice(), "arts"));
        assert_eq!(place(effect), (Voie::ElfeHaut, 1));
        let (index, named, scope, effect) = tattoo(3, For);
        assert_eq!((index, named, scope), (3, [For].as_slice(), EVERY_TEST));
        assert_eq!(place(effect), (Voie::Pagne, 3));
        let (named, scope, effect) = heroic(Voie::Brute, 4, For);
        assert_eq!(
            (named, scope, place(effect)),
            ([For].as_slice(), EVERY_TEST, (Voie::Brute, 4))
        );
        let (named, scope, effect) = elite(Cha);
        assert_eq!(
            (named, scope, place(effect)),
            ([Cha].as_slice(), EVERY_TEST, (Voie::Noblesse, 5))
        );
    }

    #[test]
    fn diversite_gives_its_bonus_to_the_origin() {
        assert!(bonus(Voie::Humain, 1, 1).on_speciality());
        assert!(!bonus(Voie::Assassin, 1, 1).on_speciality());
        assert!(
            TestBonus::all()
                .filter(|bonus| bonus.voie().speciality(bonus.rank()).is_some())
                .all(|bonus| bonus.index == 1)
        );
        let rogue = ranks(Character::new(), Voie::Roublard, 4);
        assert!(rogue.bonus_dice().iter().all(|die| !die.on_speciality()));
    }

    #[test]
    fn every_bonus_names_its_tests() {
        assert!(TestBonus::all().all(|bonus| !bonus.scope().is_empty()));
        assert!(BONUS_DICE.iter().all(|(_, scope, _)| !scope.is_empty()));
        assert_eq!(
            bonus(Voie::Assassin, 1, 1).scope(),
            "discrétion, déguisement, dissimuler une arme"
        );
        let rogue = ranks(Character::new(), Voie::Roublard, 4);
        let dice = rogue.bonus_dice();
        assert_eq!(
            dice.iter().map(|die| die.scope()).collect::<Vec<_>>(),
            ["investigation, contrefaçon"]
        );
    }

    #[test]
    fn resisting_gives_the_rank_reached() {
        let halfling = ranks(Character::new(), Voie::Halfelin, 2);
        let resisting = bonus(Voie::Halfelin, 2, 1);
        assert!(halfling.test_bonuses().contains(&resisting));
        assert_eq!(total(&halfling, resisting), Some(2));
        assert_eq!(resisting.scope(), "résister aux sorts");
        let halfling = ranks(Character::new(), Voie::Halfelin, 5);
        assert_eq!(total(&halfling, resisting), Some(5));
        let tough = ranks(Character::new(), Voie::Resistance, 2);
        assert_eq!(total(&tough, bonus(Voie::Resistance, 2, 1)), Some(2));
    }

    #[test]
    fn the_catalogue_follows_voie_and_rank_order() {
        let all: Vec<_> = TestBonus::all().collect();
        let mut sorted = all.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(all, sorted);
        assert!(all.iter().all(|b| b.voie.ability_name(b.rank).is_some()));
        let dice: Vec<_> = BONUS_DICE
            .iter()
            .map(|(_, _, effect)| (effect.voie(), effect.rank()))
            .collect();
        let mut sorted = dice.clone();
        sorted.sort();
        assert_eq!(dice, sorted);
    }

    #[test]
    fn competence_bonuses_grow_with_the_rank_up_to_seven() {
        let dodge = bonus(Voie::Pourfendeur, 1, 1);
        let character = ranks(Character::new(), Voie::Pourfendeur, 1);
        assert_eq!(character.test_bonuses(), [dodge]);
        assert_eq!(dodge.characteristics(), [Agi]);
        assert_eq!(total(&character, dodge), Some(3));
        let character = ranks(character, Voie::Pourfendeur, 5).with_rank(Voie::Pourfendeur, 6);
        assert_eq!(total(&character, dodge), Some(7));
        assert_eq!(
            character
                .value(Value::TestBonus(dodge))
                .map(|v| v.contributions()[0].source().clone()),
            Some(Source::Ability(Voie::Pourfendeur.ability(Rank::new(1))))
        );
    }

    #[test]
    fn a_voie_de_peuple_gives_three() {
        let character = Character::new()
            .with_peuple(Peuple::Gnome)
            .with_rank(Voie::Gnome, 1)
            .with_rank(Voie::Gnome, 2);
        assert_eq!(
            character.test_bonuses(),
            [bonus(Voie::Gnome, 1, 1), bonus(Voie::Gnome, 2, 1)]
        );
        assert_eq!(total(&character, bonus(Voie::Gnome, 2, 1)), Some(3));
    }

    #[test]
    fn eclectique_counts_the_barde_ranks_four() {
        let eclectic = bonus(Voie::Vagabond, 2, 1);
        let character = ranks(Character::new(), Voie::Vagabond, 2);
        assert_eq!(total(&character, eclectic), Some(1));
        let character = ranks(character, Voie::Musicien, 4);
        let character = ranks(character, Voie::Vagabond, 4);
        assert_eq!(total(&character, eclectic), Some(3));
    }

    #[test]
    fn tatouages_gives_its_bonus_to_the_characteristic_chosen() {
        let character = ranks(Character::new(), Voie::Pagne, 3);
        let tattoos: Vec<_> = character.test_bonuses();
        assert_eq!(tattoos, [bonus(Voie::Pagne, 1, 1)]);
        assert!(character.violations().iter().any(|v| {
            v.category() == Category::Incomplete
                && v.subject() == &Subject::Ability(Voie::Pagne.ability(Rank::new(3)))
        }));
        let character =
            character.with_ability_option(Voie::Pagne, 3, AbilityOption::Characteristic(For));
        let strength = bonus(Voie::Pagne, 3, 3);
        assert_eq!(
            character.test_bonuses(),
            [bonus(Voie::Pagne, 1, 1), strength]
        );
        assert_eq!(strength.characteristics(), [For]);
        assert_eq!(total(&character, strength), Some(3));
        assert_eq!(total(&character, bonus(Voie::Pagne, 3, 1)), None);
        assert_eq!(
            Value::TestBonus(strength).name(),
            "Bonus aux tests : Tatouages (FOR)"
        );
    }

    #[test]
    fn bonus_dice_list_the_capacites_held() {
        let character = Character::new()
            .with_rank(Voie::Brute, 4)
            .with_rank(Voie::Spiritualite, 1);
        let dice: Vec<_> = character
            .bonus_dice()
            .into_iter()
            .map(|die| (die.ability(), die.characteristics()))
            .collect();
        assert_eq!(
            dice,
            [
                (Voie::Brute.ability(Rank::new(4)), [For].as_slice()),
                (Voie::Spiritualite.ability(Rank::new(1)), [].as_slice()),
            ]
        );
        // Vêtements sacrés works without armour only.
        let armoured = character.with_armour(Armour::CuirSimple);
        assert_eq!(armoured.bonus_dice().len(), 1);
        let elite = Character::new()
            .with_rank(Voie::Noblesse, 5)
            .with_ability_option(Voie::Noblesse, 5, AbilityOption::Characteristic(Cha));
        let chosen = elite.bonus_dice()[0];
        assert_eq!(
            (chosen.voie(), chosen.rank()),
            (Voie::Noblesse, Rank::new(5))
        );
        assert_eq!(chosen.characteristics(), [Cha]);
    }

    #[test]
    fn bonuses_follow_the_level_ups() {
        let character = ranks(Character::new(), Voie::Roublard, 1)
            .with_level_up(LevelUp::new().with_rank(Voie::Roublard, 2));
        assert_eq!(
            character.test_bonuses(),
            [bonus(Voie::Roublard, 1, 1), bonus(Voie::Roublard, 2, 1)]
        );
        assert_eq!(total(&character, bonus(Voie::Roublard, 1, 1)), Some(4));
    }

    #[test]
    fn bonuses_are_values_with_their_own_names() {
        let value = Value::TestBonus(bonus(Voie::Golem, 1, 1));
        assert_eq!(value.id().as_str(), "cof.test-bonus.golem.1.1");
        assert_eq!(Value::from_id("cof.test-bonus.golem.1.1"), Some(value));
        assert_eq!(value.name(), "Bonus aux tests : Grosse tête");
    }

    #[test]
    fn adjusted_bonuses_surface_in_the_voies_step() {
        let lore = bonus(Voie::Vagabond, 1, 1);
        let character = ranks(Character::new(), Voie::Vagabond, 1).with_adjustment(
            Target::Value(Value::TestBonus(lore)),
            Adjustment::new(1, Author::new("gm"), "a scholar"),
        );
        assert_eq!(total(&character, lore), Some(4));
        let notice = character
            .violations()
            .into_iter()
            .find(|v| v.subject() == &Subject::Value(Value::TestBonus(lore).id()))
            .expect("the adjustment surfaces");
        assert_eq!(notice.category(), Category::Notice);
        assert_eq!(Step::of(&notice), Step::Voies);
    }
}
