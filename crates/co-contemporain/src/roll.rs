//! The rolls a character makes in play (ADR ENG-ROLLS): a characteristic
//! test, an attack test, the DM of a weapon. The engine says what a roll
//! offers and resolves it from the faces drawn; it never says whether the
//! roll succeeds, since only the Game Master knows the difficulty.

use co_kernel::{Characteristic, DerivedValue, Dice, RollMode, Rolled, Source, TestRoll};

use crate::{
    AttackMode, BonusDie, Character, HEALTH_SCALE_STEPS, HealthState, Situational, Target,
    TestBonus, Value, Weapon, weapon,
};

/// What each ally helping gives to a test (COC.RES.COOPERATION).
const COOPERATION: i32 = 2;

/// The lowest natural roll of a critical success without a weapon
/// (CO.RES.CRIT-NATURAL).
const NATURAL_CRITICAL: i32 = 20;

/// The characteristics whose tests are physical actions, taking the malus
/// of the state of health (D-56).
const PHYSICAL: [Characteristic; 3] = [
    Characteristic::For,
    Characteristic::Agi,
    Characteristic::Con,
];

/// A roll of the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Roll {
    /// A test of a characteristic (CO.RES.TEST).
    Test(Characteristic),
    /// An attack test with a weapon category.
    Attack(AttackMode),
    /// An attack test with a weapon, critical in the weapon's range.
    WeaponAttack(Weapon),
    /// The DM of a weapon.
    Damage(Weapon),
}

impl Roll {
    /// Every roll: the tests, the attacks, then each weapon's.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        let tests = Characteristic::ALL.into_iter().map(Self::Test);
        let attacks = [
            AttackMode::Unarmed,
            AttackMode::ContactWeapon,
            AttackMode::RangedWeapon,
            AttackMode::HeavyWeapon,
        ]
        .into_iter()
        .map(Self::Attack);
        let weapons = Weapon::ALL
            .into_iter()
            .flat_map(|weapon| [Self::WeaponAttack(weapon), Self::Damage(weapon)]);
        tests.chain(attacks).chain(weapons)
    }

    /// Returns the stable identifier of the roll, such as
    /// `coc.roll.test.for` or `coc.roll.damage.<weapon>`.
    #[must_use]
    pub fn id(self) -> String {
        match self {
            Self::Test(c) => format!("coc.roll.test.{}", c.abbreviation().to_lowercase()),
            Self::Attack(mode) => format!("coc.roll.attack.{}", mode.id()),
            Self::WeaponAttack(w) => weapon::value_id("roll.weapon-attack", w)
                .as_str()
                .to_owned(),
            Self::Damage(w) => weapon::value_id("roll.damage", w).as_str().to_owned(),
        }
    }

    /// Returns the roll whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().find(|roll| roll.id() == id)
    }

    /// The French display name of the roll.
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Test(c) => format!("Test de {}", c.abbreviation()),
            Self::Attack(mode) => format!("Test d’attaque : {}", mode.name()),
            Self::WeaponAttack(weapon) => format!("Test d’attaque : {}", weapon.name()),
            Self::Damage(weapon) => format!("DM : {}", weapon.name()),
        }
    }
}

/// A modifier a roll may take, picked by the player when it applies: the
/// engine does not know the circumstances of the roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollOption {
    /// A capacité's bonus to some tests of the characteristic (D-50).
    TestBonus(TestBonus),
    /// The malus of the state of health reached (COC.CNT.SCALES, D-56).
    HealthState(HealthState),
}

/// An option a roll offers, with its amount, whether it is picked
/// unless the player says otherwise, and whether it is put forward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    option: RollOption,
    amount: i32,
    picked: bool,
    featured: bool,
}

impl Offered {
    /// The option.
    #[must_use]
    pub const fn option(&self) -> &RollOption {
        &self.option
    }

    /// The amount it adds to the roll when picked.
    #[must_use]
    pub const fn amount(&self) -> i32 {
        self.amount
    }

    /// Whether it is picked by default.
    #[must_use]
    pub const fn picked(&self) -> bool {
        self.picked
    }

    /// Whether it is put forward: a test bonus naming the characteristic
    /// tested or counting it among its usual ones (D-60), the malus of the
    /// state of health where it is picked.
    #[must_use]
    pub const fn featured(&self) -> bool {
        self.featured
    }
}

/// What a roll offers within the rules: its base with its breakdown, the
/// options to pick, and what the player should know before rolling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollOffer {
    roll: Roll,
    base: DerivedValue,
    damage_dice: Dice,
    options: Vec<Offered>,
    notes: Vec<Situational>,
    bonus_dice: Vec<BonusDie>,
    featured_dice: Vec<BonusDie>,
    mode: RollMode,
    critical_from: Option<i32>,
    critical_bonus: Option<i32>,
}

impl RollOffer {
    /// The roll offered.
    #[must_use]
    pub const fn roll(&self) -> Roll {
        self.roll
    }

    /// What is added to the dice whatever the circumstances: the
    /// characteristic, the attack value or the DM of the weapon, computed
    /// with the equipment in play.
    #[must_use]
    pub const fn base(&self) -> &DerivedValue {
        &self.base
    }

    /// The dice to roll in `mode`: the d20 of a test, alone or with its dé
    /// bonus or malus, or the weapon's dice for its DM, whatever the mode.
    #[must_use]
    pub fn dice(&self, mode: RollMode) -> Dice {
        match self.roll {
            Roll::Damage(_) => self.damage_dice.clone(),
            Roll::Test(_) | Roll::Attack(_) | Roll::WeaponAttack(_) => mode.dice(),
        }
    }

    /// The options the player may pick, in a stable order.
    #[must_use]
    pub fn options(&self) -> &[Offered] {
        &self.options
    }

    /// The situational notes of the roll's target, none with an amount in
    /// COC2: shown, never picked.
    #[must_use]
    pub fn notes(&self) -> &[Situational] {
        &self.notes
    }

    /// The capacités' dé bonus that may apply to the test: shown for the
    /// player to choose the mode.
    #[must_use]
    pub fn bonus_dice(&self) -> &[BonusDie] {
        &self.bonus_dice
    }

    /// Those of [`bonus_dice`](Self::bonus_dice) put forward: naming the
    /// characteristic tested or counting it among their usual ones (D-60).
    #[must_use]
    pub fn featured_dice(&self) -> &[BonusDie] {
        &self.featured_dice
    }

    /// The mode the roll starts in: a plain d20, since no capacité gives a
    /// dé bonus to every test of a characteristic.
    #[must_use]
    pub const fn mode(&self) -> RollMode {
        self.mode
    }

    /// The lowest natural roll of a critical success, for an attack.
    #[must_use]
    pub const fn critical_from(&self) -> Option<i32> {
        self.critical_from
    }

    /// The bonus critique added to the DM of a critical success
    /// (COC.DER.CRIT-BONUS); `None` when the roll is not a DM or the weapon
    /// has none.
    #[must_use]
    pub const fn critical_bonus(&self) -> Option<i32> {
        self.critical_bonus
    }

    /// What each ally helping adds to the roll (COC.RES.COOPERATION);
    /// `None` when the roll takes no help.
    #[must_use]
    pub const fn cooperation(&self) -> Option<i32> {
        match self.roll {
            Roll::Test(_) => Some(COOPERATION),
            Roll::Attack(_) | Roll::WeaponAttack(_) | Roll::Damage(_) => None,
        }
    }
}

/// What the player chose for one roll.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RollChoices {
    options: Vec<RollOption>,
    mode: RollMode,
    modifier: i32,
    helpers: u32,
    critical: bool,
}

impl RollChoices {
    /// Creates the choices of a plain roll: no option, no modifier.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Picks `option`; one the roll does not offer adds nothing.
    #[must_use]
    pub fn with_option(mut self, option: RollOption) -> Self {
        self.options.push(option);
        self
    }

    /// Rolls the test in `mode`.
    #[must_use]
    pub const fn with_mode(mut self, mode: RollMode) -> Self {
        self.mode = mode;
        self
    }

    /// Adds the modifier the Game Master gives to this roll.
    #[must_use]
    pub const fn with_modifier(mut self, modifier: i32) -> Self {
        self.modifier = modifier;
        self
    }

    /// Counts `helpers` allies helping (COC.RES.COOPERATION).
    #[must_use]
    pub const fn with_helpers(mut self, helpers: u32) -> Self {
        self.helpers = helpers;
        self
    }

    /// Rolls the DM of a critical success.
    #[must_use]
    pub const fn with_critical(mut self) -> Self {
        self.critical = true;
        self
    }
}

/// Where a line of a roll's total comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollSource {
    /// The dice counted: the die kept of a test, every die of a DM.
    Dice(Vec<Rolled>),
    /// A contribution of the base.
    Base(Source),
    /// An option picked.
    Option(RollOption),
    /// This many allies helping.
    Cooperation(u32),
    /// The Game Master's modifier.
    Modifier,
    /// The weapon's bonus critique.
    CriticalBonus,
}

/// A line of a roll's total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollLine {
    source: RollSource,
    amount: i32,
}

impl RollLine {
    /// Where the line comes from.
    #[must_use]
    pub const fn source(&self) -> &RollSource {
        &self.source
    }

    /// What it adds to the total.
    #[must_use]
    pub const fn amount(&self) -> i32 {
        self.amount
    }
}

/// A roll resolved: every line of its total, and what was left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollOutcome {
    test: Option<TestRoll>,
    lines: Vec<RollLine>,
    critical: bool,
    unpicked: Vec<Offered>,
}

impl RollOutcome {
    /// The d20 of a test or an attack: the die kept, the one set aside and
    /// its natural 20 or 1; `None` for a DM.
    #[must_use]
    pub const fn test(&self) -> Option<TestRoll> {
        self.test
    }

    /// The lines of the total, in order: the dice, the base, the options
    /// picked, the help, the Game Master's modifier, the bonus critique.
    #[must_use]
    pub fn lines(&self) -> &[RollLine] {
        &self.lines
    }

    /// The total: the sum of the lines, clamped to the `i32` range.
    #[must_use]
    pub fn total(&self) -> i32 {
        let sum = self.lines.iter().fold(0_i64, |sum, line| {
            sum.saturating_add(i64::from(line.amount))
        });
        clamp(sum)
    }

    /// Whether an attack's die is in the critical range, or a DM counts
    /// its bonus critique.
    #[must_use]
    pub const fn critical(&self) -> bool {
        self.critical
    }

    /// The options offered and not picked: they may still apply, and the
    /// Game Master should hear of them.
    #[must_use]
    pub fn unpicked(&self) -> &[Offered] {
        &self.unpicked
    }
}

fn clamp(sum: i64) -> i32 {
    i32::try_from(sum).unwrap_or(if sum < 0 { i32::MIN } else { i32::MAX })
}

/// The state of health reached with `damage` steps filled: the last state
/// whose step is filled, never one on a blackened step (COC.CNT.SCALES).
pub(crate) fn health_state(character: &Character, damage: i32) -> Option<HealthState> {
    let length = character
        .value(Value::HealthScale)
        .map_or(HEALTH_SCALE_STEPS, |v| v.total());
    let blackened = HEALTH_SCALE_STEPS.saturating_sub(length).max(0);
    let reached = blackened.saturating_add(damage);
    HealthState::ALL
        .into_iter()
        .rfind(|state| state.step() > blackened && state.step() <= reached)
}

/// The malus of the state of health, picked for a physical action (D-56).
fn health(state: Option<HealthState>, picked: bool) -> Option<Offered> {
    state.map(|state| Offered {
        option: RollOption::HealthState(state),
        amount: state.malus(),
        picked,
        featured: picked,
    })
}

/// The characteristics named and the usual ones (D-60) of the tests `die`
/// applies to: those of the test chosen for it, or its own; `None` while
/// its test is not chosen, the die then going to no test yet
/// (COC.PRG.BONUS-DIE).
fn die_characteristics(character: &Character, die: BonusDie) -> Option<Characteristics> {
    if die.on_chosen_ability() {
        character
            .bonus_die_test(die)
            .map(|test| (test.characteristics(), test.usual_characteristics()))
    } else {
        Some((die.characteristics(), die.usual_characteristics()))
    }
}

/// Characteristics named, then usual.
type Characteristics = (&'static [Characteristic], &'static [Characteristic]);

fn test_offer(
    character: &Character,
    characteristic: Characteristic,
    state: Option<HealthState>,
) -> RollOffer {
    let mut base = character.characteristic(characteristic);
    if characteristic == Characteristic::Agi {
        let malus = character.value(Value::AgiTestModifier).unwrap_or_default();
        base.extend(malus.contributions().iter().cloned());
    }
    // Named, a bonus goes to the tests of its characteristics only; else to
    // every test, put forward on those of its usual ones (D-60).
    let named = |characteristics: &[Characteristic]| {
        characteristics.is_empty() || characteristics.contains(&characteristic)
    };
    let featured = |(named, usual): Characteristics| {
        named.contains(&characteristic) || usual.contains(&characteristic)
    };
    let mut options = character
        .test_bonuses()
        .into_iter()
        .filter(|bonus| named(bonus.characteristics()))
        .filter_map(|bonus| {
            let amount = character.value(Value::TestBonus(bonus))?.total();
            Some(Offered {
                option: RollOption::TestBonus(bonus),
                amount,
                picked: false,
                featured: featured((bonus.characteristics(), bonus.usual_characteristics())),
            })
        })
        .collect::<Vec<_>>();
    options.extend(health(state, PHYSICAL.contains(&characteristic)));
    let bonus_dice = character
        .bonus_dice()
        .into_iter()
        .filter(|die| die_characteristics(character, *die).is_some_and(|(n, _)| named(n)))
        .collect::<Vec<_>>();
    let featured_dice = bonus_dice
        .iter()
        .copied()
        .filter(|die| die_characteristics(character, *die).is_some_and(featured))
        .collect();
    RollOffer {
        roll: Roll::Test(characteristic),
        base,
        damage_dice: Dice::default(),
        options,
        notes: character.situational(Target::Characteristic(characteristic)),
        bonus_dice,
        featured_dice,
        mode: RollMode::Plain,
        critical_from: None,
        critical_bonus: None,
    }
}

fn attack_offer(
    character: &Character,
    roll: Roll,
    mode: AttackMode,
    critical_from: i32,
    state: Option<HealthState>,
) -> RollOffer {
    let value = Value::Attack(mode);
    let options = health(state, true).into_iter().collect();
    RollOffer {
        roll,
        base: character.value(value).unwrap_or_default(),
        damage_dice: Dice::default(),
        options,
        notes: character.situational(Target::Value(value)),
        bonus_dice: Vec::new(),
        featured_dice: Vec::new(),
        mode: RollMode::Plain,
        critical_from: Some(critical_from),
        critical_bonus: None,
    }
}

fn damage_offer(character: &Character, weapon: Weapon) -> RollOffer {
    let value = Value::Damage(weapon);
    RollOffer {
        roll: Roll::Damage(weapon),
        base: character.value(value).unwrap_or_default(),
        damage_dice: character.damage_dice(weapon),
        options: Vec::new(),
        notes: character.situational(Target::Value(value)),
        bonus_dice: Vec::new(),
        featured_dice: Vec::new(),
        mode: RollMode::Plain,
        critical_from: None,
        critical_bonus: character
            .value(Value::CriticalBonus(weapon))
            .map(|v| v.total()),
    }
}

/// What `roll` offers the character in the state of health `state`.
pub(crate) fn offer(character: &Character, roll: Roll, state: Option<HealthState>) -> RollOffer {
    match roll {
        Roll::Test(characteristic) => test_offer(character, characteristic, state),
        Roll::Attack(mode) => attack_offer(character, roll, mode, NATURAL_CRITICAL, state),
        Roll::WeaponAttack(weapon) => {
            let critical_from = character
                .value(Value::CriticalRange(weapon))
                .map_or(NATURAL_CRITICAL, |v| v.total());
            attack_offer(character, roll, weapon.attack_mode(), critical_from, state)
        }
        Roll::Damage(weapon) => damage_offer(character, weapon),
    }
}

impl RollOffer {
    /// Resolves the roll with `choices` from the dice `rolled`, those of
    /// [`dice`](Self::dice) for these choices; `None` when they are not.
    #[must_use]
    pub fn resolve(&self, choices: &RollChoices, rolled: &[Rolled]) -> Option<RollOutcome> {
        if !Rolled::are_of(rolled, &self.dice(choices.mode)) {
            return None;
        }
        let line = |source, amount| RollLine { source, amount };
        let mut lines = Vec::new();
        let test = if matches!(self.roll, Roll::Damage(_)) {
            lines.push(line(RollSource::Dice(rolled.to_vec()), Rolled::sum(rolled)));
            None
        } else {
            let test = TestRoll::new(choices.mode, rolled)?;
            let kept = test.kept();
            lines.push(line(RollSource::Dice(vec![kept]), i32::from(kept.face())));
            Some(test)
        };
        lines.extend(
            self.base
                .contributions()
                .iter()
                .map(|c| line(RollSource::Base(c.source().clone()), c.amount())),
        );
        let (picked, unpicked) = self
            .options
            .iter()
            .cloned()
            .partition::<Vec<_>, _>(|offered| choices.options.contains(&offered.option));
        lines.extend(
            picked
                .into_iter()
                .map(|offered| line(RollSource::Option(offered.option), offered.amount)),
        );
        if let Some(each) = self.cooperation()
            && choices.helpers > 0
        {
            let amount = i64::from(each).saturating_mul(i64::from(choices.helpers));
            lines.push(line(
                RollSource::Cooperation(choices.helpers),
                clamp(amount),
            ));
        }
        if choices.modifier != 0 {
            lines.push(line(RollSource::Modifier, choices.modifier));
        }
        let critical = match (test, self.critical_from, self.critical_bonus) {
            (Some(test), Some(from), _) => i32::from(test.kept().face()) >= from,
            (None, _, Some(bonus)) if choices.critical => {
                lines.push(line(RollSource::CriticalBonus, bonus));
                true
            }
            _ => false,
        };
        Some(RollOutcome {
            test,
            lines,
            critical,
            unpicked,
        })
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Die, Natural, Rank};

    use super::*;
    use crate::{Equipment, InPlay, Protection, Sequel, Trait, Voie};

    fn d20(face: u8) -> Rolled {
        Rolled::new(Die::new(20), face)
    }

    fn played(character: &Character) -> InPlay {
        character.in_play(Equipment::new())
    }

    fn ranks(character: Character, voie: Voie, up_to: u8) -> Character {
        (1..=up_to).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    fn amounts(outcome: &RollOutcome) -> Vec<i32> {
        outcome.lines().iter().map(RollLine::amount).collect()
    }

    #[test]
    fn rolls_have_distinct_identifiers_read_back() {
        let ids = Roll::all().map(Roll::id).collect::<Vec<_>>();
        let unique = ids.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), ids.len());
        for roll in Roll::all() {
            assert_eq!(Roll::from_id(&roll.id()), Some(roll));
            assert_ne!(roll.name(), "");
        }
        assert_eq!(Roll::Test(Characteristic::For).id(), "coc.roll.test.for");
        assert_eq!(
            Roll::Damage(Weapon::MainsNues).id(),
            "coc.roll.damage.mains-nues"
        );
        assert_eq!(Roll::Test(Characteristic::Int).name(), "Test de INT");
        assert_eq!(Roll::from_id("coc.roll.test.none"), None);
    }

    #[test]
    fn a_test_adds_the_die_kept_to_the_characteristic() {
        let character = Character::new().with_carac_points(Characteristic::Int, 2);
        let played = played(&character);
        let roll = Roll::Test(Characteristic::Int);
        let offer = played.roll_offer(roll, None);
        assert_eq!(offer.base().total(), 2);
        assert_eq!(offer.dice(RollMode::Plain), Dice::new([20]));
        assert_eq!(offer.mode(), RollMode::Plain);
        assert_eq!(offer.cooperation(), Some(2));

        let outcome = played
            .roll_offer(roll, None)
            .resolve(&RollChoices::new(), &[d20(11)])
            .unwrap();
        assert_eq!(outcome.total(), 13);
        assert_eq!(
            outcome.lines()[0].source(),
            &RollSource::Dice(vec![d20(11)])
        );
        assert!(!outcome.critical());
        assert_eq!(outcome.test().and_then(TestRoll::natural), None);
    }

    #[test]
    fn dice_of_another_roll_resolve_nothing() {
        let played = played(&Character::new());
        let roll = Roll::Test(Characteristic::Int);
        let two = [d20(3), d20(4)];
        assert_eq!(
            played
                .roll_offer(roll, None)
                .resolve(&RollChoices::new(), &two),
            None
        );
        let bonus = RollChoices::new().with_mode(RollMode::BonusDie);
        assert_eq!(
            played.roll_offer(roll, None).resolve(&bonus, &[d20(3)]),
            None
        );
    }

    #[test]
    fn a_bonus_die_keeps_the_higher_and_shows_naturals() {
        let played = played(&Character::new());
        let choices = RollChoices::new().with_mode(RollMode::BonusDie);
        let outcome = played
            .roll_offer(Roll::Test(Characteristic::Cha), None)
            .resolve(&choices, &[d20(4), d20(20)])
            .unwrap();
        let test = outcome.test().unwrap();
        assert_eq!(test.kept(), d20(20));
        assert_eq!(test.discarded(), Some(d20(4)));
        assert_eq!(test.natural(), Some(Natural::Twenty));
        // A test has no critical range: the natural 20 is all there is.
        assert!(!outcome.critical());
    }

    #[test]
    fn test_bonuses_are_offered_unpicked_and_stay_visible() {
        // Pickpocket: a bonus to some AGI tests.
        let character = ranks(Character::new(), Voie::Crime, 1);
        let played = played(&character);
        let bonus = character.test_bonuses()[0];
        let characteristic = bonus.characteristics()[0];
        let roll = Roll::Test(characteristic);
        let offer = played.roll_offer(roll, None);
        let [offered] = offer.options() else {
            panic!("one option expected")
        };
        assert_eq!(offered.option(), &RollOption::TestBonus(bonus));
        assert!(!offered.picked());
        let amount = offered.amount();
        assert!(amount > 0);

        let plain = played
            .roll_offer(roll, None)
            .resolve(&RollChoices::new(), &[d20(10)])
            .unwrap();
        assert_eq!(plain.total(), 10 + offer.base().total());
        assert_eq!(plain.unpicked(), offer.options());

        let choices = RollChoices::new().with_option(RollOption::TestBonus(bonus));
        let picked = played
            .roll_offer(roll, None)
            .resolve(&choices, &[d20(10)])
            .unwrap();
        assert_eq!(picked.total(), plain.total() + amount);
        assert_eq!(picked.unpicked(), []);
        assert!(
            picked
                .lines()
                .iter()
                .any(|l| l.source() == &RollSource::Option(RollOption::TestBonus(bonus)))
        );
        // A test of another characteristic does not offer it.
        let other = Characteristic::ALL
            .into_iter()
            .find(|c| !bonus.characteristics().contains(c))
            .unwrap();
        assert_eq!(played.roll_offer(Roll::Test(other), None).options(), []);
    }

    #[test]
    fn an_option_not_offered_adds_nothing() {
        let played = played(&Character::new());
        let choices = RollChoices::new().with_option(RollOption::HealthState(HealthState::Blesse));
        let outcome = played
            .roll_offer(Roll::Test(Characteristic::For), None)
            .resolve(&choices, &[d20(10)])
            .unwrap();
        assert_eq!(outcome.total(), 10);
    }

    #[test]
    fn helpers_and_the_modifier_are_lines_of_a_test() {
        let played = played(&Character::new());
        let choices = RollChoices::new().with_helpers(3).with_modifier(-2);
        let outcome = played
            .roll_offer(Roll::Test(Characteristic::Vol), None)
            .resolve(&choices, &[d20(10)])
            .unwrap();
        assert_eq!(amounts(&outcome), [10, 0, 6, -2]);
        assert_eq!(outcome.lines()[2].source(), &RollSource::Cooperation(3));
        assert_eq!(outcome.lines()[3].source(), &RollSource::Modifier);
        assert_eq!(outcome.total(), 14);
    }

    #[test]
    fn attacks_and_damage_take_no_help() {
        let played = played(&Character::new());
        let choices = RollChoices::new().with_helpers(2);
        let attack = Roll::Attack(AttackMode::Unarmed);
        assert_eq!(played.roll_offer(attack, None).cooperation(), None);
        let outcome = played
            .roll_offer(attack, None)
            .resolve(&choices, &[d20(10)])
            .unwrap();
        assert_eq!(outcome.total(), 10);
    }

    #[test]
    fn notes_without_an_amount_are_shown_never_picked() {
        // Gangster: FOR tested in place of CHA to coerce.
        let character = ranks(Character::new(), Voie::Crime, 2);
        let offer = played(&character).roll_offer(Roll::Test(Characteristic::For), None);
        let [note] = offer.notes() else {
            panic!("one note expected")
        };
        assert_eq!(note.amount(), None);
        assert_eq!(offer.roll(), Roll::Test(Characteristic::For));
        // Nothing to pick: the note has no amount to add.
        assert_eq!(offer.options(), []);
    }

    #[test]
    fn a_bonus_goes_to_every_test_and_is_put_forward_on_its_usual_ones() {
        // Convaincant: an approche sociale, usually CHA (D-60); Hyperpolyglotte:
        // a dé bonus usually INT.
        let character = ranks(Character::new(), Voie::Discours, 1);
        let character = ranks(character, Voie::Langues, 5);
        let played = played(&character);
        let featured = |c| {
            let offer = played.roll_offer(Roll::Test(c), None);
            let speech = offer
                .options()
                .iter()
                .find(|o| matches!(o.option(), RollOption::TestBonus(b) if b.voie() == Voie::Discours))
                .map(Offered::featured);
            (
                speech,
                offer.bonus_dice().len(),
                offer.featured_dice().len(),
            )
        };
        assert_eq!(featured(Characteristic::Cha), (Some(true), 1, 0));
        assert_eq!(featured(Characteristic::Int), (Some(false), 1, 1));
        // The malus of the state of health is put forward where it is picked.
        let offer = played.roll_offer(Roll::Test(Characteristic::Int), Some(HealthState::Affaibli));
        let health = offer
            .options()
            .iter()
            .find(|o| matches!(o.option(), RollOption::HealthState(_)))
            .unwrap();
        assert!(!health.featured());
        let offer = played.roll_offer(Roll::Test(Characteristic::Con), Some(HealthState::Affaibli));
        assert!(offer.options().iter().any(|o| o.picked() && o.featured()));
    }

    #[test]
    fn a_bonus_die_is_shown_on_the_tests_it_names_only() {
        // Spécialiste: a dé bonus to INT tests of a speciality.
        let character = ranks(Character::new(), Voie::Investigation, 2);
        let played = played(&character);
        let shown = |characteristic| {
            played
                .roll_offer(Roll::Test(characteristic), None)
                .bonus_dice()
                .len()
        };
        assert_eq!(shown(Characteristic::Int), 1);
        assert_eq!(shown(Characteristic::For), 0);
        // Never applied by itself: it covers a speciality.
        let offer = played.roll_offer(Roll::Test(Characteristic::Int), None);
        assert_eq!(offer.mode(), RollMode::Plain);
    }

    #[test]
    fn the_state_of_health_is_picked_for_physical_actions_only() {
        let played = played(&Character::new());
        let state = Some(HealthState::Affaibli);
        let picked = |roll| {
            played
                .roll_offer(roll, state)
                .options()
                .iter()
                .find(|o| o.option() == &RollOption::HealthState(HealthState::Affaibli))
                .map(|o| (o.amount(), o.picked()))
        };
        assert_eq!(picked(Roll::Test(Characteristic::Con)), Some((-3, true)));
        assert_eq!(picked(Roll::Test(Characteristic::Int)), Some((-3, false)));
        assert_eq!(
            picked(Roll::Attack(AttackMode::RangedWeapon)),
            Some((-3, true))
        );
        assert_eq!(picked(Roll::Damage(Weapon::Couteau)), None);
    }

    #[test]
    fn the_state_of_health_follows_the_steps_filled() {
        let whole = played(&Character::new());
        assert_eq!(whole.health_state(0), None);
        assert_eq!(whole.health_state(4), None);
        assert_eq!(whole.health_state(5), Some(HealthState::Contusionne));
        assert_eq!(whole.health_state(14), Some(HealthState::Affaibli));
        assert_eq!(whole.health_state(40), Some(HealthState::Mourant));
        // A shorter scale: its first steps are blackened, and a state on a
        // blackened step never applies.
        let shorter = Character::new().with_trait(Trait::Sequelle(Sequel::DiminueCorps), 0);
        let length = shorter.value(Value::HealthScale).unwrap().total();
        assert!(length < HEALTH_SCALE_STEPS);
        let blackened = HEALTH_SCALE_STEPS - length;
        let shorter = played(&shorter);
        assert_eq!(shorter.health_state(0), None);
        assert_eq!(
            shorter.health_state(10 - blackened),
            Some(HealthState::Affaibli)
        );
    }

    #[test]
    fn an_agi_test_takes_the_malus_of_the_protections_worn() {
        let character = Character::new();
        let played = character.in_play(Equipment::new().with_protection(Protection::GiletRigide));
        let offer = played.roll_offer(Roll::Test(Characteristic::Agi), None);
        let malus = played.value(Value::AgiTestModifier).unwrap().total();
        assert!(malus < 0);
        assert_eq!(offer.base().total(), malus);
        let other = played.roll_offer(Roll::Test(Characteristic::For), None);
        assert_eq!(other.base().total(), 0);
    }

    #[test]
    fn a_weapon_attack_is_critical_in_the_weapon_range() {
        let character = ranks(Character::new(), Voie::ArmesDeContact, 4);
        let played = played(&character);
        let roll = Roll::WeaponAttack(Weapon::Couteau);
        let offer = played.roll_offer(roll, None);
        assert_eq!(offer.critical_from(), Some(18));
        assert_eq!(
            Some(offer.base().total()),
            played
                .value(Value::Attack(AttackMode::ContactWeapon))
                .map(|v| v.total())
        );
        let roll_of = |face| {
            played
                .roll_offer(roll, None)
                .resolve(&RollChoices::new(), &[d20(face)])
                .unwrap()
        };
        assert!(roll_of(18).critical());
        assert!(!roll_of(17).critical());
        assert_eq!(roll_of(18).total(), 18 + offer.base().total());
        // The category alone is critical on a natural 20.
        let mode = Roll::Attack(AttackMode::ContactWeapon);
        assert_eq!(played.roll_offer(mode, None).critical_from(), Some(20));
    }

    #[test]
    fn damage_sums_the_weapon_dice_and_adds_the_bonus_critique() {
        let character = Character::new().with_carac_points(Characteristic::For, 2);
        let played = played(&character);
        let roll = Roll::Damage(Weapon::Fleuret);
        let offer = played.roll_offer(roll, None);
        assert_eq!(offer.dice(RollMode::BonusDie), Dice::new([8]));
        let bonus = offer.critical_bonus().unwrap();
        let rolled = [Rolled::new(Die::new(8), 5)];

        let plain = played
            .roll_offer(roll, None)
            .resolve(&RollChoices::new(), &rolled)
            .unwrap();
        assert_eq!(plain.total(), 5 + offer.base().total());
        assert_eq!(plain.test(), None);
        assert!(!plain.critical());

        let critical = played
            .roll_offer(roll, None)
            .resolve(&RollChoices::new().with_critical(), &rolled)
            .unwrap();
        assert!(critical.critical());
        assert_eq!(critical.total(), plain.total() + bonus);
        assert_eq!(
            critical.lines().last().map(RollLine::source),
            Some(&RollSource::CriticalBonus)
        );
        assert_eq!(
            played
                .roll_offer(roll, None)
                .resolve(&RollChoices::new(), &[d20(5)]),
            None
        );
    }

    #[test]
    fn a_die_for_a_capacite_chosen_follows_its_test() {
        // Boss: a dé bonus to a test of the ranks 2 to 4, once chosen.
        let character = ranks(Character::new(), Voie::Corporations, 5);
        let die = character.bonus_dice()[0];
        assert!(die.on_chosen_ability());
        let rank = Voie::Corporations.bonus_die_choice()[0];
        let chosen = character
            .clone()
            .with_bonus_die_target(Voie::Corporations, rank);
        let test = chosen.bonus_die_test(die).unwrap();
        for characteristic in Characteristic::ALL {
            let offered = played(&chosen)
                .roll_offer(Roll::Test(characteristic), None)
                .bonus_dice()
                .contains(&die);
            let named = test.characteristics();
            assert_eq!(offered, named.is_empty() || named.contains(&characteristic));
            // Never applied by itself: its test is one among others.
            assert_eq!(
                played(&chosen)
                    .roll_offer(Roll::Test(characteristic), None)
                    .mode(),
                RollMode::Plain
            );
        }
        assert_eq!(
            Rank::new(rank),
            chosen.bonus_die_target(Voie::Corporations).unwrap()
        );
        // Not chosen yet, it goes to no test.
        for characteristic in Characteristic::ALL {
            let offer = played(&character).roll_offer(Roll::Test(characteristic), None);
            assert_eq!(offer.bonus_dice(), []);
        }
    }
}
