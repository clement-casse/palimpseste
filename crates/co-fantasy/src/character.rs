//! A COF2 character at creation (level 1): choices only, everything else
//! computed.

use std::collections::{BTreeMap, BTreeSet};

use co_kernel::{
    Adjustment, Characteristic, Characteristics, DerivedValue, Rank, Subject, ValueId, Violation,
    ViolationReport, Waiver, adjustment_notice,
};

use crate::{
    Armour, Family, Item, Money, Peuple, Profil, Quantity, RecoveryDie, Series, Shield, Voie,
    Weapon, WeaponUse, armour, characteristics, derived, equipment, money, peuple, profil, voie,
    weapon,
};

/// The kind of attack an attack value applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttackMode {
    /// Attaque au contact: level + FOR.
    Contact,
    /// Attaque à distance: level + AGI.
    Ranged,
    /// Attaque magique: level + VOL.
    Magic,
}

/// A value the engine computes, other than a characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// PV maximum (COF.DER.PV-L1).
    VigourPoints,
    /// Number of dés de récupération (COF.DER.DR).
    RecoveryDice,
    /// PC (COF.DER.PC).
    LuckPoints,
    /// PM (COF.DER.PM).
    ManaPoints,
    /// Init (COF.DER.INIT).
    Initiative,
    /// DEF (COF.DER.DEF).
    Defence,
    /// Attack value (COF.DER.ATTACK).
    Attack(AttackMode),
    /// DM added to the weapon's dice in a use (COF.DER.DM).
    Damage(Weapon, WeaponUse),
    /// Lowest natural roll of a critical success with a weapon
    /// (COF.DER.CRIT-RANGE).
    CriticalRange(Weapon),
    /// Money held, in pc: the purse at creation (COF.CRE.EQUIPMENT).
    Money,
}

impl Value {
    /// Returns the value whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().find(|value| value.id().as_str() == id)
    }

    /// Every value, the weapon values for every weapon and use.
    fn all() -> impl Iterator<Item = Self> {
        let fixed = [
            Self::VigourPoints,
            Self::RecoveryDice,
            Self::LuckPoints,
            Self::ManaPoints,
            Self::Initiative,
            Self::Defence,
            Self::Attack(AttackMode::Contact),
            Self::Attack(AttackMode::Ranged),
            Self::Attack(AttackMode::Magic),
            Self::Money,
        ];
        let weapons = Weapon::ALL.into_iter().flat_map(|weapon| {
            WeaponUse::ALL
                .into_iter()
                .map(move |weapon_use| Self::Damage(weapon, weapon_use))
                .chain([Self::CriticalRange(weapon)])
        });
        fixed.into_iter().chain(weapons)
    }

    /// Returns the kernel identifier of the value.
    #[must_use]
    pub fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Damage(weapon, weapon_use) => return weapon::damage_id(weapon, weapon_use),
            Self::CriticalRange(weapon) => return weapon::critical_range_id(weapon),
            Self::Money => "cof.money",
            Self::VigourPoints => "cof.vigour-points",
            Self::RecoveryDice => "cof.recovery-dice",
            Self::LuckPoints => "cof.luck-points",
            Self::ManaPoints => "cof.mana-points",
            Self::Initiative => "cof.initiative",
            Self::Defence => "cof.defence",
            Self::Attack(AttackMode::Contact) => "cof.attack.contact",
            Self::Attack(AttackMode::Ranged) => "cof.attack.ranged",
            Self::Attack(AttackMode::Magic) => "cof.attack.magic",
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

/// The choices made for a level-1 character, with the adjustments and
/// waivers granted on them.
///
/// Every value is computed from these choices, never stored. Any choice can
/// be made in any order, and out-of-rule choices are kept: they show up as
/// violations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Character {
    profil: Option<Profil>,
    peuple: Option<Peuple>,
    series: Option<Series>,
    assigned: BTreeMap<Characteristic, i32>,
    peuple_bonus: Option<Characteristic>,
    peuple_malus: Option<Characteristic>,
    voies: BTreeMap<Voie, BTreeSet<Rank>>,
    armour: Option<Armour>,
    shield: Option<Shield>,
    items: BTreeMap<Item, Quantity>,
    purse: Option<Money>,
    adjustments: Vec<(Target, Adjustment)>,
    waivers: Vec<Waiver>,
}

impl Character {
    /// Creates a character with no choice made.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the profil, and with it the family.
    #[must_use]
    pub const fn with_profil(mut self, profil: Profil) -> Self {
        self.profil = Some(profil);
        self
    }

    /// Gives up the profil.
    #[must_use]
    pub const fn without_profil(mut self) -> Self {
        self.profil = None;
        self
    }

    /// Sets the peuple. The peuple bonus and malus already chosen are kept.
    #[must_use]
    pub const fn with_peuple(mut self, peuple: Peuple) -> Self {
        self.peuple = Some(peuple);
        self
    }

    /// Gives up the peuple.
    #[must_use]
    pub const fn without_peuple(mut self) -> Self {
        self.peuple = None;
        self
    }

    /// Sets the value series.
    #[must_use]
    pub const fn with_series(mut self, series: Series) -> Self {
        self.series = Some(series);
        self
    }

    /// Gives up the value series; the assigned values are kept.
    #[must_use]
    pub const fn without_series(mut self) -> Self {
        self.series = None;
        self
    }

    /// Assigns a series value to `characteristic`, replacing any earlier one.
    #[must_use]
    pub fn with_series_value(mut self, characteristic: Characteristic, value: i32) -> Self {
        self.assigned.insert(characteristic, value);
        self
    }

    /// Takes back the series value assigned to `characteristic`.
    #[must_use]
    pub fn without_series_value(mut self, characteristic: Characteristic) -> Self {
        self.assigned.remove(&characteristic);
        self
    }

    /// Chooses the characteristic the peuple raises by 1.
    #[must_use]
    pub const fn with_peuple_bonus(mut self, characteristic: Characteristic) -> Self {
        self.peuple_bonus = Some(characteristic);
        self
    }

    /// Gives up the peuple bonus.
    #[must_use]
    pub const fn without_peuple_bonus(mut self) -> Self {
        self.peuple_bonus = None;
        self
    }

    /// Chooses the characteristic the peuple lowers by 1. A peuple offering
    /// a single malus applies it without this choice; a chosen malus
    /// replaces it.
    #[must_use]
    pub const fn with_peuple_malus(mut self, characteristic: Characteristic) -> Self {
        self.peuple_malus = Some(characteristic);
        self
    }

    /// Gives up the chosen peuple malus; the peuple's single malus, if any,
    /// applies again.
    #[must_use]
    pub const fn without_peuple_malus(mut self) -> Self {
        self.peuple_malus = None;
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

    /// Puts on `armour`, replacing any armour worn.
    #[must_use]
    pub const fn with_armour(mut self, armour: Armour) -> Self {
        self.armour = Some(armour);
        self
    }

    /// Takes off the armour.
    #[must_use]
    pub const fn without_armour(mut self) -> Self {
        self.armour = None;
        self
    }

    /// Takes up `shield`, replacing any shield carried.
    #[must_use]
    pub const fn with_shield(mut self, shield: Shield) -> Self {
        self.shield = Some(shield);
        self
    }

    /// Puts down the shield.
    #[must_use]
    pub const fn without_shield(mut self) -> Self {
        self.shield = None;
        self
    }

    /// Carries `quantity` of `item`, replacing any quantity already carried;
    /// a quantity of 0 gives the item up.
    #[must_use]
    pub fn with_item(mut self, item: impl Into<Item>, quantity: impl Into<Quantity>) -> Self {
        let item = item.into();
        let quantity = quantity.into();
        if quantity.get() == 0 {
            self.items.remove(&item);
        } else {
            self.items.insert(item, quantity);
        }
        self
    }

    /// Gives up `item`.
    #[must_use]
    pub fn without_item(mut self, item: impl Into<Item>) -> Self {
        self.items.remove(&item.into());
        self
    }

    /// Sets the purse rolled for the starting equipment (2d6 pa).
    #[must_use]
    pub const fn with_purse(mut self, purse: Money) -> Self {
        self.purse = Some(purse);
        self
    }

    /// Gives up the purse.
    #[must_use]
    pub const fn without_purse(mut self) -> Self {
        self.purse = None;
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

    /// Returns the family of the profil, if the profil is set.
    #[must_use]
    pub fn family(&self) -> Option<Family> {
        self.profil.map(Profil::family)
    }

    /// Returns the voies the voie de peuple slot accepts (COF.CRE.VOIES,
    /// COF.CRE.VOIE-DU-MAGE): the peuple's own, a demi-elfe's by upbringing,
    /// and the voie du mage for the mage family.
    #[must_use]
    pub fn peuple_slot_options(&self) -> Vec<Voie> {
        let mut options: Vec<Voie> = self
            .peuple
            .map(|peuple| {
                Voie::ALL
                    .into_iter()
                    .filter(|voie| peuple.allows_voie(*voie))
                    .collect()
            })
            .unwrap_or_default();
        if self.family() == Some(Family::Mages) {
            options.push(Voie::Mage);
        }
        options
    }

    /// Returns the die of the dés de récupération, if the profil is set.
    #[must_use]
    pub fn recovery_die(&self) -> Option<RecoveryDie> {
        self.family().map(Family::recovery_die)
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
    /// PV, DR and PC are `None` while the profil is unknown, PM while no
    /// spell is known, the money while the purse is not rolled, a weapon's
    /// DM when the weapon cannot be used that way. Their adjustments then
    /// still surface as notices.
    #[must_use]
    pub fn value(&self, value: Value) -> Option<DerivedValue> {
        let values = self.characteristics();
        let base = match value {
            Value::VigourPoints => derived::vigour_points(self.family()?, &values),
            Value::RecoveryDice => derived::recovery_dice(self.family()?, &values),
            Value::LuckPoints => derived::luck_points(self, self.family()?, &values),
            Value::ManaPoints => derived::mana_points(self, &values)?,
            Value::Initiative => derived::initiative(self, &values),
            Value::Defence => derived::defence(self, &values),
            Value::Attack(mode) => derived::attack(self, &values, mode),
            Value::Damage(weapon, weapon_use) => weapon::damage(&values, weapon, weapon_use)?,
            Value::CriticalRange(weapon) => weapon::critical_range(weapon),
            Value::Money => money::money(self)?,
        };
        Some(self.adjusted(Target::Value(value), base))
    }

    /// Returns every violation, in mechanic order; the report sorts them.
    #[must_use]
    pub fn violations(&self) -> Vec<Violation> {
        let mut violations = Vec::new();
        profil::check(self, &mut violations);
        peuple::check(self, &mut violations);
        characteristics::check(self, &mut violations);
        voie::check(self, &mut violations);
        armour::check(self, &mut violations);
        equipment::check(self, &mut violations);
        money::check(self, &mut violations);
        weapon::check(self, &mut violations);
        derived::check(self, &mut violations);
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

    pub(crate) const fn profil(&self) -> Option<Profil> {
        self.profil
    }

    pub(crate) const fn peuple(&self) -> Option<Peuple> {
        self.peuple
    }

    pub(crate) const fn series(&self) -> Option<Series> {
        self.series
    }

    pub(crate) fn assigned_value(&self, characteristic: Characteristic) -> Option<i32> {
        self.assigned.get(&characteristic).copied()
    }

    pub(crate) const fn peuple_bonus(&self) -> Option<Characteristic> {
        self.peuple_bonus
    }

    pub(crate) const fn peuple_malus(&self) -> Option<Characteristic> {
        self.peuple_malus
    }

    pub(crate) fn voies(&self) -> impl Iterator<Item = (Voie, &BTreeSet<Rank>)> {
        self.voies.iter().map(|(voie, ranks)| (*voie, ranks))
    }

    pub(crate) fn has_rank(&self, voie: Voie, rank: Rank) -> bool {
        self.voies
            .get(&voie)
            .is_some_and(|ranks| ranks.contains(&rank))
    }

    pub(crate) const fn armour(&self) -> Option<Armour> {
        self.armour
    }

    pub(crate) const fn shield(&self) -> Option<Shield> {
        self.shield
    }

    pub(crate) const fn purse(&self) -> Option<Money> {
        self.purse
    }

    /// The weapons carried, with their quantity, in weapon order.
    pub(crate) fn weapons_carried(&self) -> impl Iterator<Item = (Weapon, Quantity)> + '_ {
        self.items.iter().filter_map(|(item, quantity)| match item {
            Item::Weapon(weapon) => Some((*weapon, *quantity)),
            Item::Other(_) => None,
        })
    }

    /// The weapons carried, in weapon order.
    pub(crate) fn weapons(&self) -> impl Iterator<Item = Weapon> + '_ {
        self.weapons_carried().map(|(weapon, _)| weapon)
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
    use crate::rules::CARAC_BOUNDS;

    #[test]
    fn empty_character_is_incomplete() {
        let violations = Character::new().violations();
        assert!(
            violations
                .iter()
                .all(|v| v.category() == Category::Incomplete)
        );
        let rules: Vec<_> = violations.iter().map(|v| v.rule().as_str()).collect();
        assert_eq!(
            rules,
            [
                "COF.CRE.PROFIL",
                "COF.CRE.PEUPLE",
                "COF.CRE.CARAC-SERIES",
                "COF.CRE.CARAC-SERIES",
                "COF.CRE.VOIES",
                "COF.CRE.VOIES",
                "COF.CRE.EQUIPMENT",
            ]
        );
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
    fn adjustment_on_an_uncomputable_value_still_surfaces() {
        let adjustment = Adjustment::new(2, Author::new("gm"), "test reason");
        let character =
            Character::new().with_adjustment(Target::Value(Value::ManaPoints), adjustment);
        assert_eq!(character.value(Value::ManaPoints), None);
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
        let character = Character::new().with_series_value(Characteristic::Int, 6);
        let report = character.report();
        let open: Vec<_> = report
            .open()
            .filter(|entry| entry.violation().rule() == &CARAC_BOUNDS)
            .collect();
        assert_eq!(open.len(), 1);
        assert_eq!(
            open[0].violation().category(),
            Category::Deviation(Severity::Standard)
        );

        let waiver = Waiver::new(CARAC_BOUNDS, subject, Author::new("gm"), "test reason");
        let report = character.with_waiver(waiver).report();
        assert!(
            report
                .open()
                .all(|entry| entry.violation().rule() != &CARAC_BOUNDS)
        );
        assert_eq!(report.unused_waivers(), []);
    }

    #[test]
    fn violations_do_not_depend_on_the_order_of_choices() {
        let forward = Character::new()
            .with_profil(Profil::Voleur)
            .with_peuple(Peuple::DemiOrc)
            .with_rank(Voie::Rage, 1)
            .with_rank(Voie::Assassin, 3)
            .with_armour(Armour::CotteDeMailles)
            .with_item(Weapon::ArcLong, 2)
            .with_purse(Money::pa(20))
            .with_series_value(Characteristic::For, 7);
        let backward = Character::new()
            .with_series_value(Characteristic::For, 7)
            .with_purse(Money::pa(20))
            .with_item(Weapon::ArcLong, 2)
            .with_armour(Armour::CotteDeMailles)
            .with_rank(Voie::Assassin, 3)
            .with_rank(Voie::Rage, 1)
            .with_peuple(Peuple::DemiOrc)
            .with_profil(Profil::Voleur);
        assert_eq!(forward.violations(), backward.violations());
    }

    #[test]
    fn choices_can_be_given_up() {
        let character = Character::new()
            .with_profil(Profil::Barde)
            .with_peuple(Peuple::DemiElfe)
            .with_series(Series::Expert)
            .with_peuple_bonus(Characteristic::Cha)
            .with_peuple_malus(Characteristic::Con)
            .with_series_value(Characteristic::Agi, 2)
            .with_rank(Voie::Rage, 1)
            .with_armour(Armour::CuirSimple)
            .with_shield(Shield::PetitBouclier)
            .with_item(Weapon::Dague, 2)
            .with_item("rope", 1)
            .with_purse(Money::pa(7))
            .without_profil()
            .without_peuple()
            .without_series()
            .without_peuple_bonus()
            .without_peuple_malus()
            .without_series_value(Characteristic::Agi)
            .without_rank(Voie::Rage, 1)
            .without_rank(Voie::Brute, 1)
            .without_armour()
            .without_shield()
            .without_item(Weapon::Dague)
            .with_item("rope", 0)
            .without_item("lantern")
            .without_purse();
        assert_eq!(character, Character::new());
    }
}
