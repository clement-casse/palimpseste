//! A COC2 character at creation: choices only, everything else computed.

use std::collections::{BTreeMap, BTreeSet};

use co_kernel::{
    Adjustment, Characteristic, CharacteristicValue, Characteristics, DerivedValue, Rank, Subject,
    ValueId, Violation, ViolationReport, Waiver, adjustment_notice,
};

use crate::{
    Age, AgeBracket, Domaine, SocialClass, Trait, TraitPoints, Voie, age, characteristics, domaine,
    secondary, traits, voie,
};

/// The weapon category an attack value applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttackMode {
    /// Unarmed attack (ATC, voie du combat à mains nues).
    Unarmed,
    /// Contact weapon attack (ATC, voie des armes de contact).
    ContactWeapon,
    /// Light ranged weapon attack (ATD, voie des armes à distance).
    RangedWeapon,
    /// Heavy weapon attack (ATD, voie des armes lourdes).
    HeavyWeapon,
}

/// A value the engine computes, other than a characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// Init (COC.DER.INIT).
    Initiative,
    /// DEF (COC.DER.DEF).
    Defence,
    /// ATC or ATD for a weapon category (COC.DER.ATC, COC.DER.ATD).
    Attack(AttackMode),
    /// Bonus added to unarmed and contact weapon DM (COC.DER.DM-BONUS).
    ContactDamageBonus,
    /// CG, DM healed per week (COC.DER.CG).
    HealingCapacity,
    /// Number of steps of the échelle de santé (COC.DER.HEALTH-SCALE).
    HealthScale,
    /// Characteristic points to distribute (COC.CRE.AGE-BRACKET).
    CaracPoints,
    /// Capacity points to spend on ranks (COC.CRE.AGE-BRACKET).
    CapacityPoints,
    /// Voies allowed at creation (COC.CRE.AGE-BRACKET).
    MaxVoies,
    /// Highest rank allowed at creation (COC.CRE.AGE-BRACKET).
    MaxRank,
}

impl Value {
    /// Returns the kernel identifier of the value.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Initiative => "coc.initiative",
            Self::Defence => "coc.defence",
            Self::Attack(AttackMode::Unarmed) => "coc.attack.unarmed",
            Self::Attack(AttackMode::ContactWeapon) => "coc.attack.contact-weapon",
            Self::Attack(AttackMode::RangedWeapon) => "coc.attack.ranged-weapon",
            Self::Attack(AttackMode::HeavyWeapon) => "coc.attack.heavy-weapon",
            Self::ContactDamageBonus => "coc.contact-damage-bonus",
            Self::HealingCapacity => "coc.healing-capacity",
            Self::HealthScale => "coc.health-scale",
            Self::CaracPoints => "coc.carac-points",
            Self::CapacityPoints => "coc.capacity-points",
            Self::MaxVoies => "coc.max-voies",
            Self::MaxRank => "coc.max-rank",
        })
    }
}

/// What a manual adjustment applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    /// A starting characteristic.
    Characteristic(Characteristic),
    /// Another computed value.
    Value(Value),
}

impl Target {
    fn subject(self) -> Subject {
        match self {
            Self::Characteristic(c) => Subject::Characteristic(c),
            Self::Value(value) => Subject::Value(value.id()),
        }
    }
}

/// The choices made for a character at creation, with the adjustments and
/// waivers granted on them.
///
/// Every value is computed from these choices, never stored. Any choice can
/// be made in any order, and out-of-rule choices are kept: they show up as
/// violations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Character {
    age: Option<Age>,
    domaine: Option<Domaine>,
    voies: BTreeMap<Voie, BTreeSet<Rank>>,
    traits: BTreeMap<Trait, TraitPoints>,
    allocated: Characteristics,
    adjustments: Vec<(Target, Adjustment)>,
    waivers: Vec<Waiver>,
}

impl Character {
    /// Creates a character with no choice made.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the age.
    #[must_use]
    pub fn with_age(mut self, age: impl Into<Age>) -> Self {
        self.age = Some(age.into());
        self
    }

    /// Sets the domaine.
    #[must_use]
    pub const fn with_domaine(mut self, domaine: Domaine) -> Self {
        self.domaine = Some(domaine);
        self
    }

    /// Acquires `rank` in `voie`.
    #[must_use]
    pub fn with_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.voies.entry(voie).or_default().insert(rank.into());
        self
    }

    /// Gives up `rank` in `voie`; the voie is dropped once it has no rank left.
    #[must_use]
    pub fn without_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        if let Some(ranks) = self.voies.get_mut(&voie) {
            ranks.remove(&rank.into());
            if ranks.is_empty() {
                self.voies.remove(&voie);
            }
        }
        self
    }

    /// Takes the trait `t` with `points`, replacing any points already set.
    #[must_use]
    pub fn with_trait(mut self, t: Trait, points: impl Into<TraitPoints>) -> Self {
        self.traits.insert(t, points.into());
        self
    }

    /// Gives up the trait `t`.
    #[must_use]
    pub fn without_trait(mut self, t: Trait) -> Self {
        self.traits.remove(&t);
        self
    }

    /// Allocates `points` to `characteristic`, replacing any earlier
    /// allocation; negative points lower it.
    #[must_use]
    pub fn with_carac_points(mut self, characteristic: Characteristic, points: i32) -> Self {
        self.allocated
            .set(characteristic, CharacteristicValue::new(points));
        self
    }

    /// Adds a manual adjustment to `target`.
    #[must_use]
    pub fn with_adjustment(mut self, target: Target, adjustment: Adjustment) -> Self {
        self.adjustments.push((target, adjustment));
        self
    }

    /// Adds a waiver granted by the Game Master.
    #[must_use]
    pub fn with_waiver(mut self, waiver: Waiver) -> Self {
        self.waivers.push(waiver);
        self
    }

    /// Returns the age bracket, if the age is set and within a bracket.
    #[must_use]
    pub fn age_bracket(&self) -> Option<AgeBracket> {
        self.age.and_then(AgeBracket::of)
    }

    /// Returns the starting value of `characteristic` and its breakdown.
    #[must_use]
    pub fn characteristic(&self, characteristic: Characteristic) -> DerivedValue {
        self.adjusted(
            Target::Characteristic(characteristic),
            characteristics::characteristic(self, characteristic),
        )
    }

    /// Returns the seven starting values.
    #[must_use]
    pub fn characteristics(&self) -> Characteristics {
        Characteristics::from_fn(|c| self.characteristic(c).total().into())
    }

    /// Returns `value` and its breakdown.
    ///
    /// Only the budgets (`CaracPoints`, `CapacityPoints`, `MaxVoies`,
    /// `MaxRank`) can be `None`: while the age bracket is unknown. Their
    /// adjustments then still surface as notices.
    #[must_use]
    pub fn value(&self, value: Value) -> Option<DerivedValue> {
        let values = self.characteristics();
        let base = match value {
            Value::Initiative => secondary::initiative(self, &values),
            Value::Defence => secondary::defence(self, &values),
            Value::Attack(mode) => secondary::attack(self, &values, mode),
            Value::ContactDamageBonus => secondary::contact_damage_bonus(self, &values),
            Value::HealingCapacity => secondary::healing_capacity(&values),
            Value::HealthScale => secondary::health_scale(self),
            Value::CaracPoints | Value::CapacityPoints | Value::MaxVoies | Value::MaxRank => {
                age::budget(self, value)?
            }
        };
        Some(self.adjusted(Target::Value(value), base))
    }

    /// Returns the characteristic points allocated, lowered characteristics
    /// counting negatively.
    ///
    /// The sum is exact: seven `i32` values cannot overflow an `i64`.
    #[must_use]
    pub fn carac_points_spent(&self) -> i64 {
        self.allocated
            .iter()
            .map(|(_, points)| i64::from(points.get()))
            .sum()
    }

    /// Returns the capacity points spent: one per rank acquired.
    #[must_use]
    pub fn capacity_points_spent(&self) -> i32 {
        self.voies.values().flatten().fold(0, |spent, _| spent + 1)
    }

    /// Returns the number of voies with at least one rank.
    #[must_use]
    pub fn voie_count(&self) -> i32 {
        self.voies.keys().fold(0, |count, _| count + 1)
    }

    /// Returns the sum of the advantages (positive).
    #[must_use]
    pub fn advantages(&self) -> i32 {
        self.trait_values().filter(|value| *value > 0).sum()
    }

    /// Returns the sum of the disadvantages (negative).
    #[must_use]
    pub fn disadvantages(&self) -> i32 {
        self.trait_values().filter(|value| *value < 0).sum()
    }

    /// Returns the social class, or `None` when the class traits make it
    /// uncomputable.
    #[must_use]
    pub fn social_class(&self) -> Option<SocialClass> {
        traits::social_class(self)
    }

    /// Returns every violation, in mechanic order; the report sorts them.
    #[must_use]
    pub fn violations(&self) -> Vec<Violation> {
        let mut violations = Vec::new();
        age::check(self, &mut violations);
        domaine::check(self, &mut violations);
        voie::check(self, &mut violations);
        traits::check(self, &mut violations);
        characteristics::check(self, &mut violations);
        violations.extend(
            self.adjustments
                .iter()
                .map(|(target, adjustment)| adjustment_notice(target.subject(), adjustment)),
        );
        violations
    }

    /// Returns the violations matched against the character's waivers.
    #[must_use]
    pub fn report(&self) -> ViolationReport {
        ViolationReport::new(self.violations(), &self.waivers)
    }

    pub(crate) const fn age(&self) -> Option<Age> {
        self.age
    }

    pub(crate) const fn domaine(&self) -> Option<Domaine> {
        self.domaine
    }

    pub(crate) fn voies(&self) -> impl Iterator<Item = (Voie, &BTreeSet<Rank>)> {
        self.voies.iter().map(|(voie, ranks)| (*voie, ranks))
    }

    pub(crate) fn has_voie(&self, voie: Voie) -> bool {
        self.voies.contains_key(&voie)
    }

    pub(crate) fn has_rank(&self, voie: Voie, rank: Rank) -> bool {
        self.voies
            .get(&voie)
            .is_some_and(|ranks| ranks.contains(&rank))
    }

    pub(crate) fn highest_rank(&self, voie: Voie) -> Option<Rank> {
        self.voies
            .get(&voie)
            .and_then(|ranks| ranks.last().copied())
    }

    pub(crate) fn traits(&self) -> impl Iterator<Item = (Trait, TraitPoints)> + '_ {
        self.traits.iter().map(|(t, points)| (*t, *points))
    }

    pub(crate) fn has_trait(&self, t: Trait) -> bool {
        self.traits.contains_key(&t)
    }

    pub(crate) fn trait_points(&self, t: Trait) -> Option<TraitPoints> {
        self.traits.get(&t).copied()
    }

    pub(crate) fn allocated_points(&self, characteristic: Characteristic) -> i32 {
        self.allocated[characteristic].get()
    }

    fn trait_values(&self) -> impl Iterator<Item = i32> + '_ {
        self.traits().map(|(t, points)| t.value(points))
    }

    /// Appends the adjustments made to `target`, in the order they were added.
    fn adjusted(&self, target: Target, value: DerivedValue) -> DerivedValue {
        self.adjustments
            .iter()
            .filter(|(t, _)| *t == target)
            .fold(value, |value, (_, adjustment)| {
                value.with_adjustment(adjustment)
            })
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Author, Category, Severity};

    use super::*;
    use crate::rules::CARAC_LIMITS;

    #[test]
    fn empty_character_is_incomplete() {
        let incomplete: Vec<_> = Character::new()
            .violations()
            .into_iter()
            .filter(|v| v.category() == Category::Incomplete)
            .map(|v| v.rule().as_str().to_owned())
            .collect();
        assert_eq!(incomplete, ["COC.CRE.AGE-BRACKET", "COC.CRE.DOMAINE"]);
    }

    #[test]
    fn budgets_need_an_age_bracket() {
        let character = Character::new();
        assert_eq!(character.value(Value::CaracPoints), None);
        assert_eq!(
            character.value(Value::Initiative).map(|v| v.total()),
            Some(10)
        );
        let character = character.with_age(25);
        for (value, expected) in [
            (Value::CaracPoints, 10),
            (Value::CapacityPoints, 4),
            (Value::MaxVoies, 4),
            (Value::MaxRank, 2),
        ] {
            assert_eq!(character.value(value).map(|v| v.total()), Some(expected));
        }
    }

    #[test]
    fn adjustment_changes_the_value_and_surfaces_as_a_notice() {
        let adjustment = Adjustment::new(1, Author::new("gm"), "test reason");
        let target = Target::Value(Value::Defence);
        let character = Character::new().with_adjustment(target, adjustment.clone());
        assert_eq!(character.value(Value::Defence).map(|v| v.total()), Some(11));
        let notices: Vec<_> = character
            .violations()
            .into_iter()
            .filter(|v| v.category() == Category::Notice)
            .collect();
        assert_eq!(
            notices,
            [adjustment_notice(
                Subject::Value(Value::Defence.id()),
                &adjustment
            )]
        );
    }

    #[test]
    fn adjusted_budget_moves_the_limit() {
        let adjustment = Adjustment::new(1, Author::new("gm"), "test reason");
        let character = Character::new()
            .with_age(25)
            .with_rank(Voie::Crime, 1)
            .with_rank(Voie::Crime, 2)
            .with_rank(Voie::Crime, 3)
            .with_rank(Voie::Survie, 1);
        let max_rank_deviation = |c: &Character| {
            c.violations().iter().any(|v| {
                v.rule().as_str() == "COC.CRE.CAPACITY-POINTS"
                    && v.subject() == &Subject::Voie(Voie::Crime.id())
            })
        };
        assert!(max_rank_deviation(&character));
        let adjusted = character.with_adjustment(Target::Value(Value::MaxRank), adjustment);
        assert_eq!(adjusted.value(Value::MaxRank).map(|v| v.total()), Some(3));
        assert!(!max_rank_deviation(&adjusted));
    }

    #[test]
    fn adjustment_on_an_uncomputable_budget_still_surfaces() {
        let adjustment = Adjustment::new(1, Author::new("gm"), "test reason");
        let character = Character::new().with_adjustment(Target::Value(Value::MaxRank), adjustment);
        assert_eq!(character.value(Value::MaxRank), None);
        assert!(
            character
                .violations()
                .iter()
                .any(|v| v.category() == Category::Notice)
        );
    }

    #[test]
    fn waiver_accepts_a_deviation() {
        let subject = Subject::Characteristic(Characteristic::Int);
        let character = Character::new().with_carac_points(Characteristic::Int, 6);
        let report = character.report();
        let open: Vec<_> = report
            .open()
            .filter(|entry| entry.violation().rule() == &CARAC_LIMITS)
            .collect();
        assert_eq!(open.len(), 1);
        assert_eq!(
            open[0].violation().category(),
            Category::Deviation(Severity::Absolute)
        );

        let waiver = Waiver::new(CARAC_LIMITS, subject, Author::new("gm"), "test reason");
        let report = character.with_waiver(waiver).report();
        assert!(
            report
                .open()
                .all(|entry| entry.violation().rule() != &CARAC_LIMITS)
        );
        assert_eq!(report.unused_waivers(), []);
    }

    #[test]
    fn violations_do_not_depend_on_the_order_of_choices() {
        let forward = Character::new()
            .with_age(8)
            .with_rank(Voie::Crime, 3)
            .with_rank(Voie::Survie, 1)
            .with_trait(Trait::Rapide, 1)
            .with_trait(Trait::Croyant, 2)
            .with_trait(Trait::Hypercartesien, 1);
        let backward = Character::new()
            .with_trait(Trait::Hypercartesien, 1)
            .with_trait(Trait::Croyant, 2)
            .with_trait(Trait::Rapide, 1)
            .with_rank(Voie::Survie, 1)
            .with_rank(Voie::Crime, 3)
            .with_age(8);
        assert_eq!(forward.violations(), backward.violations());
        assert!(
            forward
                .violations()
                .iter()
                .any(|v| v.category() == Category::Uncomputable)
        );
    }

    #[test]
    fn choices_can_be_given_up() {
        let character = Character::new()
            .with_rank(Voie::Crime, 1)
            .with_trait(Trait::Rapide, 2)
            .without_rank(Voie::Crime, 1)
            .without_trait(Trait::Rapide)
            .without_rank(Voie::Survie, 1);
        assert_eq!(character, Character::new());
    }
}
