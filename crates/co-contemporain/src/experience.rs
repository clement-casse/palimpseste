//! Experience after creation (COC.PRG.*): the XP earned, what each spending
//! buys, and the checks on them.
//!
//! A character stores its XP spendings in order (ADR ENG-COC-EXPERIENCE);
//! its current ranks, traits and the XP left are computed from them.

use std::collections::{BTreeMap, BTreeSet};

use co_kernel::{DerivedValue, Rank, Severity, Source, Subject, Violation};

use crate::{
    Character, Trait, TraitPoints, Value, Voie,
    rules::{
        EXPERIENCE_EARNED, EXPERIENCE_MECHANIC, ONE_RANK, PRG_COST, PRG_LIMITS, SOCIAL_CLASS_VALUE,
        XP_USE,
    },
};

/// Voies allowed in total after creation (COC.PRG.LIMITS).
const MAX_VOIES: usize = 6;

/// XP earned by a character: 1 per game session by default
/// (COC.PRG.XP-GAIN).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Experience(u32);

impl Experience {
    /// Returns the XP as a number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for Experience {
    fn from(xp: u32) -> Self {
        Self(xp)
    }
}

/// What one spending of XP buys: ranks, a social class or a cure
/// (COC.PRG.XP-USE).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XpSpending {
    ranks: BTreeSet<(Voie, Rank)>,
    traits: BTreeMap<Trait, Option<TraitPoints>>,
}

impl XpSpending {
    /// Creates a spending buying nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Buys `rank` in `voie`.
    #[must_use]
    pub fn with_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.ranks.insert((voie, rank.into()));
        self
    }

    /// Gives up buying `rank` in `voie`.
    #[must_use]
    pub fn without_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.ranks.remove(&(voie, rank.into()));
        self
    }

    /// Sets the trait `t` to `points`: a class trait raised a social class.
    #[must_use]
    pub fn with_trait(mut self, t: Trait, points: impl Into<TraitPoints>) -> Self {
        self.traits.insert(t, Some(points.into()));
        self
    }

    /// Gives the trait `t` up: Toxicomane cured, or the Classe inférieure
    /// left for the middle class.
    #[must_use]
    pub fn with_trait_given_up(mut self, t: Trait) -> Self {
        self.traits.insert(t, None);
        self
    }

    /// Gives up changing the trait `t`.
    #[must_use]
    pub fn without_trait_change(mut self, t: Trait) -> Self {
        self.traits.remove(&t);
        self
    }

    /// The ranks bought, in voie and rank order.
    pub(crate) fn ranks(&self) -> impl Iterator<Item = (Voie, Rank)> + '_ {
        self.ranks.iter().copied()
    }

    /// Applies the spending to the traits held before it.
    pub(crate) fn apply_traits(&self, traits: &mut BTreeMap<Trait, TraitPoints>) {
        for (t, points) in &self.traits {
            match points {
                Some(points) => traits.insert(*t, *points),
                None => traits.remove(t),
            };
        }
    }

    /// Applies the spending to the ranks held before it.
    pub(crate) fn apply_ranks(&self, voies: &mut BTreeMap<Voie, BTreeSet<Rank>>) {
        for (voie, rank) in self.ranks() {
            voies.entry(voie).or_default().insert(rank);
        }
    }
}

/// The XP a rank costs: its rank (COC.PRG.COST).
#[must_use]
pub fn xp_cost(rank: Rank) -> i32 {
    i32::from(rank.get())
}

/// The social class as a level from the middle class: + the Classe
/// supérieure's points, - the Classe inférieure's; `None` when the class
/// traits contradict each other or leave the catalogue.
fn class_level(traits: &BTreeMap<Trait, TraitPoints>) -> Option<i32> {
    let points = |t| traits.get(&t).map(|points| i32::from(points.get()));
    match (
        points(Trait::ClasseSuperieure),
        points(Trait::ClasseInferieure),
    ) {
        (None, None) => Some(0),
        (Some(points @ 1..=3), None) => Some(points),
        (None, Some(points @ 1..=2)) => Some(-points),
        _ => None,
    }
}

/// The XP a rise from class `from` to class `to` costs: twice the trait
/// value of each class climbed, the class left below the middle class, the
/// class reached above it (COC.PRG.XP-USE).
fn class_cost(from: i32, to: i32) -> i32 {
    (from..to)
        .map(|level| 2 * level.abs().max((level + 1).abs()))
        .sum()
}

/// A trait change of a spending, as the XP spending rules see it.
enum TraitChange {
    /// The social class rises: its cost.
    ClassRise(i32),
    /// Toxicomane is cured: its cost.
    Cure(i32),
    /// Not something XP buys.
    NotBought(Trait),
}

/// The trait changes of `spending` on the traits held before it.
fn trait_changes(spending: &XpSpending, before: &BTreeMap<Trait, TraitPoints>) -> Vec<TraitChange> {
    let mut after = before.clone();
    spending.apply_traits(&mut after);
    let mut changes = Vec::new();
    let class_traits = [Trait::ClasseSuperieure, Trait::ClasseInferieure];
    if class_traits.iter().any(|t| before.get(t) != after.get(t)) {
        match (class_level(before), class_level(&after)) {
            (Some(from), Some(to)) if to > from => {
                changes.push(TraitChange::ClassRise(class_cost(from, to)));
            }
            _ => changes.extend(
                class_traits
                    .into_iter()
                    .filter(|t| spending.traits.contains_key(t))
                    .map(TraitChange::NotBought),
            ),
        }
    }
    for (t, points) in &spending.traits {
        if class_traits.contains(t) {
            continue;
        }
        let cured = *t == Trait::Toxicomane && points.is_none();
        match before.get(t) {
            Some(held) if cured => changes.push(TraitChange::Cure(2 * i32::from(held.get()))),
            _ => changes.push(TraitChange::NotBought(*t)),
        }
    }
    changes
}

/// The XP left: earned, less every rank and trait change bought
/// (COC.PRG.COST, COC.PRG.XP-USE), each cost with what it bought as source.
pub(crate) fn experience(character: &Character) -> DerivedValue {
    let earned = i32::try_from(character.experience().get()).unwrap_or(i32::MAX);
    let mut value = DerivedValue::new().with(Source::Value(EXPERIENCE_EARNED), earned);
    let mut traits = character.creation_traits();
    for spending in character.spendings() {
        for (voie, rank) in spending.ranks() {
            value = value.with(Source::Ability(voie.ability(rank)), -xp_cost(rank));
        }
        for change in trait_changes(spending, &traits) {
            value = match change {
                TraitChange::ClassRise(cost) => {
                    value.with(Source::Value(SOCIAL_CLASS_VALUE), -cost)
                }
                TraitChange::Cure(cost) => value.with(Source::Value(Trait::Toxicomane.id()), -cost),
                TraitChange::NotBought(_) => value,
            };
        }
        spending.apply_traits(&mut traits);
    }
    value
}

/// Checks the spendings: XP spent beyond the XP earned, ranks one at a time,
/// ranks bought again, the voie count and the trait changes XP buys.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    if let Some(left) = character.value(Value::Experience).map(|v| v.total())
        && left < 0
    {
        violations.push(
            Violation::deviation(
                PRG_COST,
                EXPERIENCE_MECHANIC,
                Subject::Value(Value::Experience.id()),
                Severity::Standard,
            )
            .with_parameter("value", left),
        );
    }
    let spendings: Vec<&XpSpending> = character.spendings().collect();
    let mut held: BTreeMap<Voie, BTreeSet<Rank>> = character
        .creation_voies()
        .map(|(voie, ranks)| (voie, ranks.clone()))
        .collect();
    let mut traits = character.creation_traits();
    // The last spending opening a voie.
    let mut opened: Option<i32> = None;
    for (index, spending) in spendings.iter().enumerate() {
        let number = spending_number(index);
        check_ranks(spending, number, &held, &spendings[index + 1..], violations);
        if spending.ranks().any(|(voie, _)| !held.contains_key(&voie)) {
            opened = Some(number);
        }
        for change in trait_changes(spending, &traits) {
            if let TraitChange::NotBought(t) = change {
                violations.push(
                    Violation::deviation(
                        XP_USE,
                        EXPERIENCE_MECHANIC,
                        Subject::Value(t.id()),
                        Severity::Standard,
                    )
                    .with_parameter("spending", number),
                );
            }
        }
        spending.apply_ranks(&mut held);
        spending.apply_traits(&mut traits);
    }
    let count = held.len();
    if let Some(number) = opened
        && count > MAX_VOIES
    {
        violations.push(
            Violation::deviation(
                PRG_LIMITS,
                EXPERIENCE_MECHANIC,
                Subject::Character,
                Severity::Standard,
            )
            .with_parameter("voies", as_parameter(count))
            .with_parameter("max", as_parameter(MAX_VOIES))
            .with_parameter("spending", number),
        );
    }
}

/// `count` as a violation parameter.
fn as_parameter(count: usize) -> i32 {
    i32::try_from(count).unwrap_or(i32::MAX)
}

/// The number of the spending at `index`, from 1.
fn spending_number(index: usize) -> i32 {
    as_parameter(index).saturating_add(1)
}

/// COC.PRG.ONE-RANK (D-09: per voie per spending) and COC.PRG.COST: one rank
/// of a voie per spending, never before the rank below, never bought again.
fn check_ranks(
    spending: &XpSpending,
    number: i32,
    held: &BTreeMap<Voie, BTreeSet<Rank>>,
    later: &[&XpSpending],
    violations: &mut Vec<Violation>,
) {
    let mut voies_seen = BTreeSet::new();
    for (voie, rank) in spending.ranks() {
        let ability = || Subject::Ability(voie.ability(rank));
        if held.get(&voie).is_some_and(|ranks| ranks.contains(&rank)) {
            violations.push(
                Violation::deviation(PRG_COST, EXPERIENCE_MECHANIC, ability(), Severity::Standard)
                    .with_parameter("spending", number),
            );
        }
        // Ranks come in order, so the first of a voie is its lowest.
        let second = !voies_seen.insert(voie);
        let below = rank
            .get()
            .checked_sub(1)
            .filter(|below| *below >= 1)
            .map(Rank::new);
        let bought_later = below.is_some_and(|below| {
            !held.get(&voie).is_some_and(|ranks| ranks.contains(&below))
                && !spending.ranks.contains(&(voie, below))
                && later
                    .iter()
                    .any(|spending| spending.ranks.contains(&(voie, below)))
        });
        if second || bought_later {
            violations.push(
                Violation::deviation(ONE_RANK, EXPERIENCE_MECHANIC, ability(), Severity::Standard)
                    .with_parameter("spending", number),
            );
        }
    }
}

/// What a spending of XP offers within the rules, given what was held
/// before it and what it already buys: the XP left, the ranks to buy, the
/// next social class and the cure of Toxicomane, each with its cost. The
/// capacités are named by voie and rank; their identifier is
/// [`Voie::ability`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingOffer {
    xp_left: i32,
    acquirable: Vec<(Voie, Rank)>,
    class_rise: Option<(Trait, Option<TraitPoints>, i32)>,
    cure: Option<i32>,
}

impl SpendingOffer {
    /// The XP left once every spending is paid (`Value::Experience`).
    #[must_use]
    pub const fn xp_left(&self) -> i32 {
        self.xp_left
    }

    /// The ranks the spending may buy within the rules, in voie order: the
    /// next rank of each voie it does not advance yet, rank 1 of a voie not
    /// held while fewer than six are, each within the XP left
    /// ([`xp_cost`], COC.PRG.ONE-RANK, LIMITS).
    #[must_use]
    pub fn acquirable(&self) -> &[(Voie, Rank)] {
        &self.acquirable
    }

    /// The trait change raising the social class one class, its points
    /// (`None` to give the Classe inférieure up) and its XP cost; `None` at
    /// the highest class or when the class traits contradict each other.
    #[must_use]
    pub const fn class_rise(&self) -> Option<(Trait, Option<TraitPoints>, i32)> {
        self.class_rise
    }

    /// The XP the cure of Toxicomane costs, when the trait is held.
    #[must_use]
    pub const fn cure(&self) -> Option<i32> {
        self.cure
    }
}

/// The offer of the spending numbered `number`, from 1, if there is one.
pub(crate) fn offer(character: &Character, number: usize) -> Option<SpendingOffer> {
    let index = number.checked_sub(1)?;
    let spendings: Vec<&XpSpending> = character.spendings().collect();
    let spending = *spendings.get(index)?;
    let mut held: BTreeMap<Voie, BTreeSet<Rank>> = character
        .creation_voies()
        .map(|(voie, ranks)| (voie, ranks.clone()))
        .collect();
    let mut traits = character.creation_traits();
    for earlier in spendings.get(..index)? {
        earlier.apply_ranks(&mut held);
        earlier.apply_traits(&mut traits);
    }
    let xp_left = character
        .value(Value::Experience)
        .map_or(0, |value| value.total());
    let advanced: BTreeSet<Voie> = spending.ranks().map(|(voie, _)| voie).collect();
    let mut after = held.clone();
    spending.apply_ranks(&mut after);
    let acquirable = Voie::ALL
        .into_iter()
        .filter(|voie| !advanced.contains(voie))
        .filter_map(|voie| {
            let ranks = held.get(&voie);
            let next = (1..=5)
                .map(Rank::new)
                .find(|rank| !ranks.is_some_and(|ranks| ranks.contains(rank)))?;
            let may_open = ranks.is_some() || after.len() < MAX_VOIES;
            (may_open && xp_cost(next) <= xp_left).then_some((voie, next))
        })
        .collect();
    let class_rise = class_level(&traits)
        .filter(|level| *level < 3)
        .map(|level| {
            let next = level + 1;
            let cost = class_cost(level, next);
            match next {
                ..0 => (
                    Trait::ClasseInferieure,
                    Some(TraitPoints::new(u8::try_from(-next).unwrap_or(u8::MAX))),
                    cost,
                ),
                0 => (Trait::ClasseInferieure, None, cost),
                _ => (
                    Trait::ClasseSuperieure,
                    Some(TraitPoints::new(u8::try_from(next).unwrap_or(u8::MAX))),
                    cost,
                ),
            }
        });
    let cure = traits
        .get(&Trait::Toxicomane)
        .map(|points| 2 * i32::from(points.get()));
    Some(SpendingOffer {
        xp_left,
        acquirable,
        class_rise,
        cure,
    })
}

#[cfg(test)]
mod tests {
    use co_kernel::{Category, Characteristic};

    use super::*;
    use crate::{Sequel, SocialClass};

    const DEVIATION: Category = Category::Deviation(Severity::Standard);

    fn total(character: &Character, value: Value) -> Option<i32> {
        character.value(value).map(|v| v.total())
    }

    fn experience_violations(character: &Character) -> Vec<(String, Subject, Category)> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
            .into_iter()
            .map(|v| {
                (
                    v.rule().as_str().to_owned(),
                    v.subject().clone(),
                    v.category(),
                )
            })
            .collect()
    }

    fn ability(voie: Voie, rank: u8) -> Subject {
        Subject::Ability(voie.ability(Rank::new(rank)))
    }

    /// A character with Crime 1 and 2 and Survie 1 at creation.
    fn criminal() -> Character {
        Character::new()
            .with_rank(Voie::Crime, 1)
            .with_rank(Voie::Crime, 2)
            .with_rank(Voie::Survie, 1)
    }

    #[test]
    fn ranks_cost_their_rank_in_xp() {
        let character = criminal()
            .with_experience(10)
            .with_xp_spending(
                XpSpending::new()
                    .with_rank(Voie::Crime, 3)
                    .with_rank(Voie::Survie, 2),
            )
            .with_xp_spending(XpSpending::new().with_rank(Voie::Medecine, 1));
        // 10 - 3 - 2 - 1.
        assert_eq!(total(&character, Value::Experience), Some(4));
        assert_eq!(experience_violations(&character), []);
        assert!(character.has_rank(Voie::Crime, Rank::new(3)));
        assert_eq!(character.highest_rank(Voie::Survie), Some(Rank::new(2)));
        // The creation budgets count the creation ranks only.
        assert_eq!(character.capacity_points_spent(), 3);
        assert_eq!(character.voie_count(), 2);
        let undone = character.without_xp_spending().without_xp_spending();
        assert_eq!(undone, criminal().with_experience(10));
    }

    #[test]
    fn xp_spent_beyond_the_xp_earned_deviates() {
        let character = criminal()
            .with_experience(2)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Crime, 3));
        assert_eq!(total(&character, Value::Experience), Some(-1));
        assert_eq!(
            experience_violations(&character),
            [(
                "COC.PRG.COST".to_owned(),
                Subject::Value(Value::Experience.id()),
                DEVIATION
            )]
        );
    }

    #[test]
    fn one_rank_of_a_voie_at_a_time() {
        let character = criminal().with_experience(20).with_xp_spending(
            XpSpending::new()
                .with_rank(Voie::Crime, 3)
                .with_rank(Voie::Crime, 4),
        );
        assert_eq!(
            experience_violations(&character),
            [(
                "COC.PRG.ONE-RANK".to_owned(),
                ability(Voie::Crime, 4),
                DEVIATION
            )]
        );
        // Rank 4 before rank 3, bought at a later spending.
        let reversed = criminal()
            .with_experience(20)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Crime, 4))
            .with_xp_spending(XpSpending::new().with_rank(Voie::Crime, 3));
        assert_eq!(
            experience_violations(&reversed),
            [(
                "COC.PRG.ONE-RANK".to_owned(),
                ability(Voie::Crime, 4),
                DEVIATION
            )]
        );
        // A rank bought again costs XP for nothing.
        let again = criminal()
            .with_experience(20)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Crime, 2));
        assert_eq!(
            experience_violations(&again),
            [(
                "COC.PRG.COST".to_owned(),
                ability(Voie::Crime, 2),
                DEVIATION
            )]
        );
    }

    #[test]
    fn a_seventh_voie_deviates() {
        let character = [
            Voie::Crime,
            Voie::Discours,
            Voie::Furtivite,
            Voie::Langues,
            Voie::Medecine,
            Voie::Sciences,
        ]
        .into_iter()
        .fold(Character::new(), |c, voie| c.with_rank(voie, 1))
        .with_experience(1)
        .with_xp_spending(XpSpending::new().with_rank(Voie::Survie, 1));
        assert_eq!(
            experience_violations(&character),
            [("COC.PRG.LIMITS".to_owned(), Subject::Character, DEVIATION)]
        );
    }

    #[test]
    fn xp_raises_the_social_class() {
        // Middle class to classe supérieure +1: 2 XP; then to +2: 4 XP.
        let character = Character::new()
            .with_experience(6)
            .with_xp_spending(XpSpending::new().with_trait(Trait::ClasseSuperieure, 1))
            .with_xp_spending(XpSpending::new().with_trait(Trait::ClasseSuperieure, 2));
        assert_eq!(total(&character, Value::Experience), Some(0));
        assert_eq!(
            character.social_class(),
            Some(SocialClass::GrandeBourgeoisie)
        );
        assert_eq!(experience_violations(&character), []);
        // Leaving the marginal class: twice the trait left, 4 XP.
        let marginal = Character::new()
            .with_trait(Trait::ClasseInferieure, 2)
            .with_experience(4)
            .with_xp_spending(XpSpending::new().with_trait(Trait::ClasseInferieure, 1));
        assert_eq!(total(&marginal, Value::Experience), Some(0));
        assert_eq!(marginal.social_class(), Some(SocialClass::Populaire));
        // The creation checks still read the trait taken at creation.
        assert!(
            marginal
                .violations()
                .iter()
                .all(|v| v.rule().as_str() != "COC.PRG.XP-USE")
        );
    }

    #[test]
    fn xp_cures_toxicomane() {
        let character = Character::new()
            .with_trait(Trait::Toxicomane, 3)
            .with_experience(6)
            .with_xp_spending(XpSpending::new().with_trait_given_up(Trait::Toxicomane));
        assert_eq!(total(&character, Value::Experience), Some(0));
        assert_eq!(experience_violations(&character), []);
        let change = XpSpending::new()
            .with_trait_given_up(Trait::Toxicomane)
            .without_trait_change(Trait::Toxicomane);
        assert_eq!(change, XpSpending::new());
    }

    #[test]
    fn xp_buys_no_other_trait_change() {
        let character = Character::new()
            .with_trait(Trait::ClasseSuperieure, 2)
            .with_experience(10)
            .with_xp_spending(
                XpSpending::new()
                    .with_trait(Trait::Rapide, 2)
                    .with_trait(Trait::ClasseSuperieure, 1)
                    .with_trait(Trait::Sequelle(Sequel::Autre), 2),
            );
        let found: Vec<_> = experience_violations(&character)
            .into_iter()
            .map(|(rule, subject, _)| (rule, subject))
            .collect();
        assert_eq!(
            found,
            [
                (
                    "COC.PRG.XP-USE".to_owned(),
                    Subject::Value(Trait::ClasseSuperieure.id())
                ),
                (
                    "COC.PRG.XP-USE".to_owned(),
                    Subject::Value(Trait::Rapide.id())
                ),
                (
                    "COC.PRG.XP-USE".to_owned(),
                    Subject::Value(Trait::Sequelle(Sequel::Autre).id())
                ),
            ]
        );
        // Nothing bought, nothing paid.
        assert_eq!(total(&character, Value::Experience), Some(10));
    }

    #[test]
    fn creation_limits_read_the_creation_ranks() {
        // Rank 3 bought with XP by a 16-30 character, whose creation
        // maximum is rank 2.
        let character = Character::new()
            .with_age(20)
            .with_rank(Voie::Crime, 1)
            .with_rank(Voie::Crime, 2)
            .with_experience(3)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Crime, 3));
        let rules: Vec<_> = character
            .violations()
            .into_iter()
            .filter(|v| v.subject() == &Subject::Voie(Voie::Crime.id()))
            .map(|v| v.rule().as_str().to_owned())
            .collect();
        assert_eq!(rules, Vec::<String>::new());
        assert_eq!(character.characteristics()[Characteristic::Per].get(), 0);
    }

    #[test]
    fn a_rank_given_up_is_not_bought() {
        let spending = XpSpending::new()
            .with_rank(Voie::Crime, 3)
            .without_rank(Voie::Crime, 3);
        assert_eq!(spending, XpSpending::new());
        let character = criminal().with_experience(3).with_xp_spending(spending);
        assert_eq!(total(&character, Value::Experience), Some(3));
    }

    fn pairs(list: &[(Voie, Rank)]) -> Vec<(Voie, u8)> {
        list.iter()
            .map(|(voie, rank)| (*voie, rank.get()))
            .collect()
    }

    #[test]
    fn a_spending_offers_the_next_ranks_within_the_xp_left() {
        let character = criminal()
            .with_experience(3)
            .with_xp_spending(XpSpending::new());
        assert_eq!(character.spending_offer(0), None);
        assert_eq!(character.spending_offer(2), None);
        let Some(offer) = character.spending_offer(1) else {
            panic!("spending 1 has an offer");
        };
        assert_eq!(offer.xp_left(), 3);
        let acquirable = pairs(offer.acquirable());
        // Crime 3 and Survie 2 cost 3 and 2; any other voie opens at rank 1.
        assert!(acquirable.contains(&(Voie::Crime, 3)));
        assert!(acquirable.contains(&(Voie::Survie, 2)));
        assert!(acquirable.contains(&(Voie::Medecine, 1)));
        assert!(!acquirable.contains(&(Voie::Crime, 4)));
        // A voie advanced by the spending offers nothing more; the XP left
        // drops with what is bought.
        let bought = criminal()
            .with_experience(3)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Survie, 2));
        let Some(offer) = bought.spending_offer(1) else {
            panic!("spending 1 has an offer");
        };
        assert_eq!(offer.xp_left(), 1);
        let acquirable = pairs(offer.acquirable());
        assert!(!acquirable.iter().any(|(voie, _)| *voie == Voie::Survie));
        assert!(!acquirable.contains(&(Voie::Crime, 3)));
        assert!(acquirable.contains(&(Voie::Medecine, 1)));
    }

    #[test]
    fn a_spending_offers_the_next_class_and_the_cure() {
        let middle = Character::new()
            .with_experience(10)
            .with_xp_spending(XpSpending::new());
        assert_eq!(
            middle
                .spending_offer(1)
                .and_then(|offer| offer.class_rise()),
            Some((Trait::ClasseSuperieure, Some(TraitPoints::new(1)), 2))
        );
        let populaire = middle.clone().with_trait(Trait::ClasseInferieure, 1);
        assert_eq!(
            populaire
                .spending_offer(1)
                .and_then(|offer| offer.class_rise()),
            Some((Trait::ClasseInferieure, None, 2))
        );
        let fortune = middle.clone().with_trait(Trait::ClasseSuperieure, 3);
        assert_eq!(
            fortune
                .spending_offer(1)
                .and_then(|offer| offer.class_rise()),
            None
        );
        let addict = middle.with_trait(Trait::Toxicomane, 3);
        assert_eq!(
            addict.spending_offer(1).and_then(|offer| offer.cure()),
            Some(6)
        );
    }

    #[test]
    fn a_seventh_voie_is_not_offered() {
        let six = [
            Voie::Crime,
            Voie::Discours,
            Voie::Furtivite,
            Voie::Langues,
            Voie::Medecine,
            Voie::Sciences,
        ]
        .into_iter()
        .fold(Character::new(), |c, voie| c.with_rank(voie, 1))
        .with_experience(9)
        .with_xp_spending(XpSpending::new());
        let Some(offer) = six.spending_offer(1) else {
            panic!("spending 1 has an offer");
        };
        assert!(
            offer
                .acquirable()
                .iter()
                .all(|(voie, _)| *voie != Voie::Survie)
        );
        assert!(offer.acquirable().contains(&(Voie::Crime, Rank::new(2))));
    }
}
