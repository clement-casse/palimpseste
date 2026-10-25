//! A COF2 character: its creation choices at level 1 and its level-ups,
//! everything else computed.

use std::collections::{BTreeMap, BTreeSet};

use co_kernel::{
    Adjustment, Characteristic, Characteristics, DerivedValue, Dice, Quantity, Rank, Speciality,
    Subject, ValueId, Violation, ViolationReport, Waiver, adjustment_notice,
};

use crate::{
    AbilityOption, Armour, BonusDie, Equipment, Family, InPlay, Item, Level, LevelOffer, LevelUp,
    Money, OrphanExchange, Peuple, Placement, Profil, RecoveryDie, Series, Shield, TestBonus, Voie,
    Weapon, WeaponUse, ability, armour, characteristics, derived, equipment, level, money, peuple,
    profil, step, test_bonus, voie, weapon,
};

/// The kind of attack an attack value applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttackMode {
    /// Attaque au contact: level (at most 10) + FOR.
    Contact,
    /// Attaque à distance: level + AGI.
    Ranged,
    /// Attaque magique: level + VOL.
    Magic,
}

/// A value the engine computes, other than a characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// PV maximum (COF.DER.PV-L1, COF.PRG.PV).
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
    /// RD given by the capacités (COF.CNT.EFFECT-SHAPES).
    DamageReduction,
    /// Attack value (COF.DER.ATTACK).
    Attack(AttackMode),
    /// Attack with a weapon in a use (COF.DER.ATTACK, D-46).
    WeaponAttack(Weapon, WeaponUse),
    /// DM added to the weapon's dice in a use (COF.DER.DM).
    Damage(Weapon, WeaponUse),
    /// Lowest natural roll of a critical success with a weapon
    /// (COF.DER.CRIT-RANGE).
    CriticalRange(Weapon),
    /// Money held, in pc: the purse at creation (COF.CRE.EQUIPMENT).
    Money,
    /// A capacité's permanent bonus to some tests (COF.DER.COMPETENCE,
    /// D-50).
    TestBonus(TestBonus),
}

impl Value {
    /// Every value but the weapon values and the test bonuses, in the book
    /// order (PV to the attack values, then the money held). [`from_id`](Self::from_id)
    /// searches it, so it lists every such value, shown or not.
    pub const FIXED: [Self; 11] = [
        Self::VigourPoints,
        Self::RecoveryDice,
        Self::LuckPoints,
        Self::ManaPoints,
        Self::Initiative,
        Self::Defence,
        Self::DamageReduction,
        Self::Attack(AttackMode::Contact),
        Self::Attack(AttackMode::Ranged),
        Self::Attack(AttackMode::Magic),
        Self::Money,
    ];

    /// Returns the value whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().find(|value| value.id().as_str() == id)
    }

    /// Every value, the weapon values for every weapon and use, and the test
    /// bonuses of every capacité.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        let weapons = Weapon::ALL.into_iter().flat_map(|weapon| {
            WeaponUse::ALL
                .into_iter()
                .flat_map(move |weapon_use| {
                    [
                        Self::WeaponAttack(weapon, weapon_use),
                        Self::Damage(weapon, weapon_use),
                    ]
                })
                .chain([Self::CriticalRange(weapon)])
        });
        Self::FIXED
            .into_iter()
            .chain(weapons)
            .chain(TestBonus::all().map(Self::TestBonus))
    }

    /// The French display name of the value.
    #[must_use]
    pub fn name(self) -> String {
        let name = match self {
            Self::VigourPoints => "Points de vigueur (PV)",
            Self::RecoveryDice => "Dés de récupération (DR)",
            Self::LuckPoints => "Points de chance (PC)",
            Self::ManaPoints => "Points de mana (PM)",
            Self::Initiative => "Initiative",
            Self::Defence => "Défense (DEF)",
            Self::DamageReduction => "Réduction des dommages (RD)",
            Self::Attack(AttackMode::Contact) => "Attaque au contact",
            Self::Attack(AttackMode::Ranged) => "Attaque à distance",
            Self::Attack(AttackMode::Magic) => "Attaque magique",
            Self::Money => "Argent (pc)",
            Self::WeaponAttack(weapon, weapon_use) => {
                return format!(
                    "Attaque {} ({})",
                    weapon.name(),
                    weapon_use.name().to_lowercase()
                );
            }
            Self::Damage(weapon, weapon_use) => {
                return format!(
                    "DM {} ({})",
                    weapon.name(),
                    weapon_use.name().to_lowercase()
                );
            }
            Self::CriticalRange(weapon) => return format!("Zone de critique {}", weapon.name()),
            Self::TestBonus(bonus) => return bonus.name(),
        };
        name.to_owned()
    }

    /// Returns the kernel identifier of the value.
    #[must_use]
    pub fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::WeaponAttack(weapon, weapon_use) => return weapon::attack_id(weapon, weapon_use),
            Self::Damage(weapon, weapon_use) => return weapon::damage_id(weapon, weapon_use),
            Self::CriticalRange(weapon) => return weapon::critical_range_id(weapon),
            Self::TestBonus(bonus) => return bonus.id(),
            Self::Money => "cof.money",
            Self::VigourPoints => "cof.vigour-points",
            Self::RecoveryDice => "cof.recovery-dice",
            Self::LuckPoints => "cof.luck-points",
            Self::ManaPoints => "cof.mana-points",
            Self::Initiative => "cof.initiative",
            Self::Defence => "cof.defence",
            Self::DamageReduction => "cof.damage-reduction",
            Self::Attack(AttackMode::Contact) => "cof.attack.contact",
            Self::Attack(AttackMode::Ranged) => "cof.attack.ranged",
            Self::Attack(AttackMode::Magic) => "cof.attack.magic",
        })
    }
}

/// What a manual adjustment applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    /// A characteristic.
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

/// The choices made for a character at level 1 and at each level-up, with
/// the adjustments and waivers granted on them.
///
/// Every value is computed from these choices, never stored: the level
/// counts the level-ups, the current ranks join the ranks chosen at creation
/// and those acquired since. Any choice can be made in any order, and
/// out-of-rule choices are kept: they show up as violations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Character {
    profil: Option<Profil>,
    peuple: Option<Peuple>,
    series: Option<Series>,
    assigned: BTreeMap<Characteristic, i32>,
    peuple_bonus: Option<Characteristic>,
    peuple_malus: Option<Characteristic>,
    voies: BTreeMap<Voie, BTreeSet<Rank>>,
    levels: Vec<LevelUp>,
    ability_options: BTreeMap<(Voie, Rank), AbilityOption>,
    specialities: BTreeMap<(Voie, Rank), Speciality>,
    equipment: Equipment,
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

    /// Acquires `rank` in `voie` at creation.
    #[must_use]
    pub fn with_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.voies.entry(voie).or_default().insert(rank.into());
        self
    }

    /// Gives up `rank` in `voie` at creation; the voie is dropped once it has
    /// no rank left.
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

    /// Gains a level, acquiring the capacités of `level_up`
    /// (COF.PRG.LEVEL-UP).
    #[must_use]
    pub fn with_level_up(mut self, level_up: LevelUp) -> Self {
        self.levels.push(level_up);
        self
    }

    /// Gives up the last level gained, and the capacités acquired with it.
    #[must_use]
    pub fn without_level_up(mut self) -> Self {
        self.levels.pop();
        self
    }

    /// Chooses `option` for the capacité at `rank` in `voie`, replacing any
    /// option chosen for it.
    #[must_use]
    pub fn with_ability_option(
        mut self,
        voie: Voie,
        rank: impl Into<Rank>,
        option: AbilityOption,
    ) -> Self {
        self.ability_options.insert((voie, rank.into()), option);
        self
    }

    /// Gives up the option chosen for the capacité at `rank` in `voie`.
    #[must_use]
    pub fn without_ability_option(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.ability_options.remove(&(voie, rank.into()));
        self
    }

    /// Writes the speciality chosen for the capacité at `rank` in `voie`,
    /// for a capacité asking for one ([`Voie::speciality`]).
    #[must_use]
    pub fn with_speciality(
        mut self,
        voie: Voie,
        rank: impl Into<Rank>,
        speciality: impl Into<Speciality>,
    ) -> Self {
        self.specialities
            .insert((voie, rank.into()), speciality.into());
        self
    }

    /// Gives up the speciality chosen for the capacité at `rank` in `voie`.
    #[must_use]
    pub fn without_speciality(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.specialities.remove(&(voie, rank.into()));
        self
    }

    /// Puts on `armour`, replacing any armour worn.
    #[must_use]
    pub const fn with_armour(mut self, armour: Armour) -> Self {
        self.equipment.armour = Some(armour);
        self
    }

    /// Takes off the armour.
    #[must_use]
    pub const fn without_armour(mut self) -> Self {
        self.equipment.armour = None;
        self
    }

    /// Takes up `shield`, replacing any shield carried.
    #[must_use]
    pub const fn with_shield(mut self, shield: Shield) -> Self {
        self.equipment.shield = Some(shield);
        self
    }

    /// Puts down the shield.
    #[must_use]
    pub const fn without_shield(mut self) -> Self {
        self.equipment.shield = None;
        self
    }

    /// Carries `quantity` of `item`, replacing any quantity already carried;
    /// a quantity of 0 gives the item up.
    #[must_use]
    pub fn with_item(mut self, item: impl Into<Item>, quantity: impl Into<Quantity>) -> Self {
        self.equipment = self.equipment.with_item(item, quantity);
        self
    }

    /// Gives up `item`.
    #[must_use]
    pub fn without_item(mut self, item: impl Into<Item>) -> Self {
        self.equipment.items.remove(&item.into());
        self
    }

    /// Wears and carries `equipment`, replacing the armour, the shield and
    /// every item.
    #[must_use]
    pub fn with_equipment(mut self, equipment: Equipment) -> Self {
        self.equipment = equipment;
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

    /// Returns the level: 1, plus 1 per level-up.
    #[must_use]
    pub fn level(&self) -> Level {
        let level_ups = u8::try_from(self.levels.len()).unwrap_or(u8::MAX);
        Level::from(level_ups.saturating_add(1))
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

    /// Returns what the level-up that reached `level` offers within the
    /// rules (ADR ENG-COF-PROGRESSION); `None` for level 1 or a level not
    /// reached.
    #[must_use]
    pub fn level_offer(&self, level: Level) -> Option<LevelOffer> {
        level::offer(self, level)
    }

    /// Returns where `violation` is settled: a creation step, or a step of
    /// the level-up it is about (ADR ENG-COF-PLACEMENT).
    #[must_use]
    pub fn placement(&self, violation: &Violation) -> Placement {
        step::placement(self, violation)
    }

    /// Returns the capacités held now, in voie and rank order: the ranks
    /// chosen at creation, then each level-up's, forgetting before
    /// acquiring.
    #[must_use]
    pub fn ranks(&self) -> Vec<(Voie, Rank)> {
        self.voies()
            .into_iter()
            .flat_map(|(voie, ranks)| ranks.into_iter().map(move |rank| (voie, rank)))
            .collect()
    }

    /// Returns the level whose level-up acquired the capacité at `rank` in
    /// `voie`, the last one when several did; `None` when it comes from
    /// creation or is not held.
    #[must_use]
    pub fn acquired_at(&self, voie: Voie, rank: impl Into<Rank>) -> Option<Level> {
        let rank = rank.into();
        self.levels()
            .filter(|(_, level_up)| level_up.ranks().any(|acquired| acquired == (voie, rank)))
            .map(|(level, _)| level)
            .last()
    }

    /// Returns the options the capacité at `rank` in `voie` offers the
    /// character, empty when it offers none. A capacité raising the lowest
    /// characteristic offers the lowest starting values, ties included.
    #[must_use]
    pub fn ability_options_offered(&self, voie: Voie, rank: impl Into<Rank>) -> Vec<AbilityOption> {
        ability::options(self, voie, rank.into())
    }

    /// Whether the capacité at `rank` in `voie` has no effect until one of its
    /// [`ability_options_offered`](Self::ability_options_offered) is chosen;
    /// a capacité offering options with a default keeps it.
    #[must_use]
    pub fn ability_option_required(&self, voie: Voie, rank: impl Into<Rank>) -> bool {
        ability::option_required(voie, rank.into())
    }

    /// Whether the character masters `weapon`: through the profil
    /// (`Profil::masters`) or a capacité (Haches et marteaux, Archer émérite
    /// of the elfe sylvain, Merveille technologique). Mastery is for display
    /// (D-28).
    #[must_use]
    pub fn masters(&self, weapon: Weapon) -> bool {
        self.profil.is_some_and(|profil| profil.masters(weapon)) || ability::masters(self, weapon)
    }

    /// Returns the permanent test bonuses of the capacités held now, in voie
    /// and rank order; [`value`](Self::value) computes each as a
    /// [`Value::TestBonus`] (D-50).
    #[must_use]
    pub fn test_bonuses(&self) -> Vec<TestBonus> {
        test_bonus::held(self)
    }

    /// Returns the dé bonus to tests of the capacités held now, with the
    /// equipment chosen at creation, in voie and rank order (D-50).
    #[must_use]
    pub fn bonus_dice(&self) -> Vec<BonusDie> {
        test_bonus::bonus_dice(self)
    }

    /// Returns the dice rolled for the DM of `weapon` in `weapon_use`: the
    /// table's ([`Weapon::dice`]), the dé évolutif with Bâton de mage, twice
    /// as many powder dice with Canon double; `None` when the weapon cannot
    /// be used that way.
    #[must_use]
    pub fn damage_dice(&self, weapon: Weapon, weapon_use: WeaponUse) -> Option<Dice> {
        weapon::damage_dice(self, weapon, weapon_use)
    }

    /// Returns the die of the dés de récupération, if the profil is set.
    #[must_use]
    pub fn recovery_die(&self) -> Option<RecoveryDie> {
        self.family().map(Family::recovery_die)
    }

    /// Returns the value of `characteristic` and its breakdown: the starting
    /// value, raised by the capacités held.
    #[must_use]
    pub fn characteristic(&self, characteristic: Characteristic) -> DerivedValue {
        self.adjusted(
            Target::Characteristic(characteristic),
            ability::raise(
                characteristics::characteristic(self, characteristic),
                self,
                characteristic,
            ),
        )
    }

    /// Returns the seven values.
    #[must_use]
    pub fn characteristics(&self) -> Characteristics {
        Characteristics::from_fn(|c| self.characteristic(c).total().into())
    }

    /// The seven starting values, adjusted, before any capacité raises them:
    /// the values the creation rules read.
    pub(crate) fn starting_characteristics(&self) -> Characteristics {
        Characteristics::from_fn(|c| {
            self.adjusted(
                Target::Characteristic(c),
                characteristics::characteristic(self, c),
            )
            .total()
            .into()
        })
    }

    /// Returns `value` and its breakdown.
    ///
    /// PV, DR and PC are `None` while the profil is unknown, PM while no
    /// spell is known, the money while the purse is not rolled, a weapon's
    /// attack and DM when the weapon cannot be used that way, a test bonus while its
    /// capacité is not held or its option not chosen. Their adjustments then
    /// still surface as notices.
    #[must_use]
    pub fn value(&self, value: Value) -> Option<DerivedValue> {
        let values = self.characteristics();
        let base = match value {
            Value::VigourPoints => derived::vigour_points(self, self.family()?, &values),
            Value::RecoveryDice => derived::recovery_dice(self, self.family()?, &values),
            Value::LuckPoints => derived::luck_points(self, self.family()?, &values),
            Value::ManaPoints => derived::mana_points(self, &values)?,
            Value::Initiative => derived::initiative(self, &values),
            Value::Defence => derived::defence(self, &values),
            Value::DamageReduction => derived::damage_reduction(self, &values)?,
            Value::Attack(mode) => derived::attack(self, &values, mode),
            Value::WeaponAttack(weapon, weapon_use) => {
                weapon::attack(self, &values, weapon, weapon_use)?
            }
            Value::Damage(weapon, weapon_use) => weapon::damage(self, &values, weapon, weapon_use)?,
            Value::CriticalRange(weapon) => weapon::critical_range(self, &values, weapon),
            Value::Money => money::money(self)?,
            Value::TestBonus(bonus) => test_bonus::value(self, &values, bonus)?,
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
        level::check(self, &mut violations);
        ability::check(self, &mut violations);
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

    /// Returns the character in play, wearing and carrying `equipment`
    /// instead of the equipment chosen at creation (ADR ENG-IN-PLAY). Its
    /// adjustments and waivers still apply.
    #[must_use]
    pub fn in_play(&self, equipment: Equipment) -> InPlay {
        InPlay::new(self.clone().with_equipment(equipment))
    }

    pub(crate) fn waivers(&self) -> &[Waiver] {
        &self.waivers
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

    /// The speciality chosen for the capacité at `rank` in `voie`.
    pub(crate) fn speciality(&self, voie: Voie, rank: Rank) -> Option<&Speciality> {
        self.specialities.get(&(voie, rank))
    }

    /// The option chosen for the capacité at `rank` in `voie`.
    pub(crate) fn ability_option(&self, voie: Voie, rank: Rank) -> Option<AbilityOption> {
        self.ability_options.get(&(voie, rank)).copied()
    }

    /// The options chosen, in voie and rank order.
    pub(crate) fn ability_options(&self) -> impl Iterator<Item = (Voie, Rank, AbilityOption)> + '_ {
        self.ability_options
            .iter()
            .map(|((voie, rank), option)| (*voie, *rank, *option))
    }

    /// The ranks chosen at creation, in voie order.
    pub(crate) fn creation_voies(&self) -> impl Iterator<Item = (Voie, &BTreeSet<Rank>)> {
        self.voies.iter().map(|(voie, ranks)| (*voie, ranks))
    }

    /// The last level-up acquiring or forgetting a rank in `voie`.
    pub(crate) fn last_change(&self, voie: Voie) -> Option<Level> {
        self.levels()
            .filter(|(_, level_up)| {
                level_up
                    .ranks()
                    .chain(level_up.forgotten())
                    .any(|(changed, _)| changed == voie)
            })
            .map(|(level, _)| level)
            .last()
    }

    /// The level-ups, with the level each one reached.
    pub(crate) fn levels(&self) -> impl Iterator<Item = (Level, &LevelUp)> {
        self.levels.iter().enumerate().map(|(index, level_up)| {
            let level = u8::try_from(index).map_or(u8::MAX, |index| index.saturating_add(2));
            (Level::from(level), level_up)
        })
    }

    /// The current ranks: those chosen at creation, then each level-up's in
    /// order, forgetting before acquiring.
    pub(crate) fn voies(&self) -> BTreeMap<Voie, BTreeSet<Rank>> {
        let mut voies = self.voies.clone();
        for level_up in &self.levels {
            level_up.apply(&mut voies);
        }
        voies
    }

    /// Whether a level-up forgets a capacité: until one does, the current
    /// ranks are every rank chosen or acquired, read without replaying the
    /// level-ups.
    fn forgets(&self) -> bool {
        self.levels
            .iter()
            .any(|level_up| level_up.forgotten().next().is_some())
    }

    /// The ranks acquired, at creation or at a level-up, in `voie`.
    fn acquired(&self, voie: Voie) -> impl Iterator<Item = Rank> + '_ {
        let created = self.voies.get(&voie).into_iter().flatten().copied();
        let acquired = self
            .levels
            .iter()
            .flat_map(LevelUp::ranks)
            .filter(move |(acquired, _)| *acquired == voie)
            .map(|(_, rank)| rank);
        created.chain(acquired)
    }

    /// The highest rank held in `voie`.
    pub(crate) fn highest_rank(&self, voie: Voie) -> Option<Rank> {
        if self.forgets() {
            self.voies()
                .get(&voie)
                .and_then(|ranks| ranks.last().copied())
        } else {
            self.acquired(voie).max()
        }
    }

    pub(crate) fn has_rank(&self, voie: Voie, rank: Rank) -> bool {
        if self.forgets() {
            self.voies()
                .get(&voie)
                .is_some_and(|ranks| ranks.contains(&rank))
        } else {
            self.acquired(voie).any(|acquired| acquired == rank)
        }
    }

    /// The orphan points exchanged, with the level-up that exchanged each.
    pub(crate) fn orphan_exchanges(&self) -> impl Iterator<Item = (Level, OrphanExchange)> + '_ {
        self.levels()
            .filter_map(|(level, level_up)| level_up.orphan().map(|exchange| (level, exchange)))
    }

    pub(crate) const fn armour(&self) -> Option<Armour> {
        self.equipment.armour
    }

    pub(crate) const fn shield(&self) -> Option<Shield> {
        self.equipment.shield
    }

    pub(crate) const fn purse(&self) -> Option<Money> {
        self.purse
    }

    /// The weapons carried, with their quantity, in weapon order.
    pub(crate) fn weapons_carried(&self) -> impl Iterator<Item = (Weapon, Quantity)> + '_ {
        self.equipment
            .items
            .iter()
            .filter_map(|(item, quantity)| match item {
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
            .with_level_up(LevelUp::new().with_rank(Voie::Rage, 2))
            .with_ability_option(Voie::Archer, 1, AbilityOption::Thrown)
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
            .without_purse()
            .without_ability_option(Voie::Archer, 1)
            .without_level_up()
            .without_level_up();
        assert_eq!(character, Character::new());
        assert_eq!(
            LevelUp::new()
                .with_rank(Voie::Rage, 2)
                .without_rank(Voie::Rage, 2),
            LevelUp::new()
        );
    }
}
