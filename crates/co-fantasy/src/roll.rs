//! The rolls a character makes in play (ADR ENG-ROLLS): a characteristic
//! test, an attack test, the DM of a weapon. The engine says what a roll
//! offers and resolves it from the faces drawn; it never says whether the
//! roll succeeds, since only the Game Master knows the difficulty.

use co_kernel::{Characteristic, DerivedValue, Dice, RollMode, Rolled, Source, TestRoll};

use crate::{
    AttackMode, BonusDie, Character, Situational, Target, TestBonus, Value, Weapon, WeaponUse,
    test_bonus::{self, Stacking},
};

/// What each ally whose help succeeded gives to a test
/// (COF.RES.COOPERATION, D-59).
const COOPERATION: i32 = 2;

/// The lowest natural roll of a critical success without a weapon
/// (CO.RES.CRIT-NATURAL).
const NATURAL_CRITICAL: i32 = 20;

/// The most the competence bonuses add to one test (COF.DER.COMPETENCE).
const COMPETENCE_CAP: i32 = 15;

/// A roll of the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Roll {
    /// A test of a characteristic (CO.RES.TEST).
    Test(Characteristic),
    /// An attack test with an attack value.
    Attack(AttackMode),
    /// An attack test with a weapon in a use, critical in the weapon's
    /// range.
    WeaponAttack(Weapon, WeaponUse),
    /// The DM of a weapon in a use.
    Damage(Weapon, WeaponUse),
}

impl Roll {
    /// Every roll: the tests, the attacks, then each weapon's in each use.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        let tests = Characteristic::ALL.into_iter().map(Self::Test);
        let attacks = [AttackMode::Contact, AttackMode::Ranged, AttackMode::Magic]
            .into_iter()
            .map(Self::Attack);
        let weapons = Weapon::ALL.into_iter().flat_map(|weapon| {
            WeaponUse::ALL.into_iter().flat_map(move |weapon_use| {
                [
                    Self::WeaponAttack(weapon, weapon_use),
                    Self::Damage(weapon, weapon_use),
                ]
            })
        });
        tests.chain(attacks).chain(weapons)
    }

    /// Returns the stable identifier of the roll: `cof.roll.test.for`, or
    /// `cof.roll.` and the end of the identifier of its value
    /// (`cof.roll.damage.<weapon>.<use>`).
    #[must_use]
    pub fn id(self) -> String {
        let of_value = |value: Value| {
            let id = value.id();
            let id = id.as_str();
            format!("cof.roll.{}", id.strip_prefix("cof.").unwrap_or(id))
        };
        match self {
            Self::Test(c) => format!("cof.roll.test.{}", c.abbreviation().to_lowercase()),
            Self::Attack(mode) => of_value(Value::Attack(mode)),
            Self::WeaponAttack(weapon, weapon_use) => {
                of_value(Value::WeaponAttack(weapon, weapon_use))
            }
            Self::Damage(weapon, weapon_use) => of_value(Value::Damage(weapon, weapon_use)),
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
            Self::Attack(mode) => format!("Test d’attaque : {}", Value::Attack(mode).name()),
            Self::WeaponAttack(weapon, weapon_use) => format!(
                "Test d’attaque : {} ({})",
                weapon.name(),
                weapon_use.name().to_lowercase()
            ),
            Self::Damage(weapon, weapon_use) => format!(
                "DM : {} ({})",
                weapon.name(),
                weapon_use.name().to_lowercase()
            ),
        }
    }
}

/// A modifier a roll may take, picked by the player when it applies: the
/// engine does not know the circumstances of the roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollOption {
    /// A capacité's bonus to some tests of the characteristic (D-50).
    TestBonus(TestBonus),
    /// A capacité's modifier holding in a situation (ADR ENG-SITUATIONAL):
    /// an amount, or dice added to a DM.
    Situational(Situational),
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

    /// The amount it adds to the roll when picked; 0 for dice.
    #[must_use]
    pub const fn amount(&self) -> i32 {
        self.amount
    }

    /// The dice it adds to a DM when picked.
    #[must_use]
    pub const fn dice(&self) -> Option<&Dice> {
        match &self.option {
            RollOption::Situational(situational) => situational.dice(),
            RollOption::TestBonus(_) => None,
        }
    }

    /// Whether it is picked by default.
    #[must_use]
    pub const fn picked(&self) -> bool {
        self.picked
    }

    /// Whether it is put forward: a test bonus naming the characteristic
    /// tested or counting it among its usual ones (D-60), and every other
    /// option, which aims at the roll itself.
    #[must_use]
    pub const fn featured(&self) -> bool {
        self.featured
    }
}

/// A capacité's dé bonus a test offers, with whether it is put forward and
/// whether it is granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfferedDie {
    die: BonusDie,
    featured: bool,
    granted: bool,
}

impl OfferedDie {
    /// The dé bonus.
    #[must_use]
    pub const fn die(&self) -> BonusDie {
        self.die
    }

    /// Whether it is put forward: it names the characteristic tested or
    /// counts it among its usual ones (D-60).
    #[must_use]
    pub const fn featured(&self) -> bool {
        self.featured
    }

    /// Whether the character has it whatever the test is about, the roll
    /// then starting with a dé bonus; otherwise it depends on a
    /// circumstance the engine does not know.
    #[must_use]
    pub const fn granted(&self) -> bool {
        self.granted
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
    bonus_dice: Vec<OfferedDie>,
    mode: RollMode,
    critical_from: Option<i32>,
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

    /// The dice to roll with `choices`: the d20 of a test, alone or with
    /// its dé bonus or malus, or the weapon's dice for its DM, then those of
    /// each option picked, in the order offered.
    #[must_use]
    pub fn dice(&self, choices: &RollChoices) -> Dice {
        self.own_dice(choices.mode)
            .dice()
            .iter()
            .copied()
            .chain(
                self.options
                    .iter()
                    .filter(|offered| choices.options.contains(&offered.option))
                    .filter_map(Offered::dice)
                    .flat_map(|dice| dice.dice().iter().copied()),
            )
            .collect()
    }

    /// The dice of the roll itself, before those of the options picked.
    fn own_dice(&self, mode: RollMode) -> Dice {
        match self.roll {
            Roll::Damage(..) => self.damage_dice.clone(),
            Roll::Test(_) | Roll::Attack(_) | Roll::WeaponAttack(..) => mode.dice(),
        }
    }

    /// The options the player may pick, in a stable order.
    #[must_use]
    pub fn options(&self) -> &[Offered] {
        &self.options
    }

    /// The capacités' dé bonus that may apply to the test: shown for the
    /// player to choose the mode.
    #[must_use]
    pub fn bonus_dice(&self) -> &[OfferedDie] {
        &self.bonus_dice
    }

    /// The mode the roll starts in: a dé bonus when a capacité gives one to
    /// every test of the characteristic.
    #[must_use]
    pub const fn mode(&self) -> RollMode {
        self.mode
    }

    /// The lowest natural roll of a critical success, for an attack.
    #[must_use]
    pub const fn critical_from(&self) -> Option<i32> {
        self.critical_from
    }

    /// Whether the roll is a DM, doubled on a critical success (D-57).
    #[must_use]
    pub const fn doubles_on_critical(&self) -> bool {
        matches!(self.roll, Roll::Damage(..))
    }

    /// What each ally whose help succeeded adds to the roll
    /// (COF.RES.COOPERATION, D-59); `None` when the roll takes no help.
    #[must_use]
    pub const fn cooperation(&self) -> Option<i32> {
        match self.roll {
            Roll::Test(_) => Some(COOPERATION),
            Roll::Attack(_) | Roll::WeaponAttack(..) | Roll::Damage(..) => None,
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

    /// Counts `helpers` allies whose help succeeded (COF.RES.COOPERATION).
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
    /// An option picked, with the dice rolled for it when it adds dice.
    Option(RollOption, Vec<Rolled>),
    /// What the competence bonuses picked exceed +15 by, taken back
    /// (COF.DER.COMPETENCE, D-58).
    CompetenceCap,
    /// This many allies helping.
    Cooperation(u32),
    /// The Game Master's modifier.
    Modifier,
    /// The DM of a critical success, doubled: the sum of the lines before
    /// it (D-57).
    Critical,
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
    set_aside: Vec<Offered>,
}

impl RollOutcome {
    /// The d20 of a test or an attack: the die kept, the one set aside and
    /// its natural 20 or 1; `None` for a DM.
    #[must_use]
    pub const fn test(&self) -> Option<TestRoll> {
        self.test
    }

    /// The lines of the total, in order: the dice, the base, the options
    /// picked, the competence cap, the help, the Game Master's modifier,
    /// the doubling of a critical DM.
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

    /// Whether an attack's die is in the critical range, or a DM is
    /// doubled.
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

    /// The competence bonuses picked and not counted: another of the same
    /// source is higher (COF.DER.COMPETENCE, D-58).
    #[must_use]
    pub fn set_aside(&self) -> &[Offered] {
        &self.set_aside
    }
}

fn clamp(sum: i64) -> i32 {
    i32::try_from(sum).unwrap_or(if sum < 0 { i32::MIN } else { i32::MAX })
}

/// The situational modifiers to `target`, as options.
fn situational(character: &Character, target: Target) -> Vec<Offered> {
    character
        .situational(target)
        .into_iter()
        .map(|s| Offered {
            amount: s.amount().unwrap_or(0),
            option: RollOption::Situational(s),
            picked: false,
            featured: true,
        })
        .collect()
}

/// Whether `die` goes to every test of `characteristic`, whatever the test
/// is about.
fn unconditional(die: BonusDie, characteristic: Characteristic) -> bool {
    // A dé bonus naming a characteristic covers every test of it.
    die.characteristics().contains(&characteristic)
}

fn test_offer(character: &Character, characteristic: Characteristic) -> RollOffer {
    // Named, a bonus goes to the tests of its characteristics only; else to
    // every test, put forward on those of its usual ones (D-60).
    let named = |characteristics: &[Characteristic]| {
        characteristics.is_empty() || characteristics.contains(&characteristic)
    };
    let featured = |named: &[Characteristic], usual: &[Characteristic]| {
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
                // A bonus to every test of the characteristic needs no
                // circumstance.
                picked: test_bonus::stacking(bonus) == Stacking::Apart
                    && bonus.characteristics().contains(&characteristic),
                featured: featured(bonus.characteristics(), bonus.usual_characteristics()),
            })
        })
        .collect::<Vec<_>>();
    options.extend(situational(
        character,
        Target::Characteristic(characteristic),
    ));
    let bonus_dice = character
        .bonus_dice()
        .into_iter()
        .filter(|die| named(die.characteristics()))
        .map(|die| OfferedDie {
            die,
            featured: featured(die.characteristics(), die.usual_characteristics()),
            granted: unconditional(die, characteristic),
        })
        .collect::<Vec<_>>();
    let mode = if bonus_dice.iter().any(OfferedDie::granted) {
        RollMode::BonusDie
    } else {
        RollMode::Plain
    };
    RollOffer {
        roll: Roll::Test(characteristic),
        base: character.characteristic(characteristic),
        damage_dice: Dice::default(),
        options,
        bonus_dice,
        mode,
        critical_from: None,
    }
}

/// What `roll` offers the character.
pub(crate) fn offer(character: &Character, roll: Roll) -> RollOffer {
    let total = |value| character.value(value).map(|v| v.total());
    let (value, damage_dice, critical_from) = match roll {
        Roll::Test(characteristic) => return test_offer(character, characteristic),
        Roll::Attack(mode) => (Value::Attack(mode), Dice::default(), Some(NATURAL_CRITICAL)),
        Roll::WeaponAttack(weapon, weapon_use) => (
            Value::WeaponAttack(weapon, weapon_use),
            Dice::default(),
            total(Value::CriticalRange(weapon)).or(Some(NATURAL_CRITICAL)),
        ),
        Roll::Damage(weapon, weapon_use) => (
            Value::Damage(weapon, weapon_use),
            character
                .damage_dice(weapon, weapon_use)
                .unwrap_or_default(),
            None,
        ),
    };
    RollOffer {
        roll,
        base: character.value(value).unwrap_or_default(),
        damage_dice,
        options: situational(character, Target::Value(value)),
        bonus_dice: Vec::new(),
        mode: RollMode::Plain,
        critical_from,
    }
}

/// The competence bonuses picked that count: the highest of each source
/// (COF.DER.COMPETENCE, D-58), the others set aside.
fn stacked(picked: Vec<Offered>) -> (Vec<Offered>, Vec<Offered>) {
    let source = |offered: &Offered| match offered.option {
        RollOption::TestBonus(bonus) => test_bonus::stacking(bonus),
        RollOption::Situational(_) => Stacking::Apart,
    };
    let mut counted = Vec::new();
    let mut set_aside = Vec::new();
    for offered in picked {
        let stacking = source(&offered);
        let rival = counted
            .iter_mut()
            .find(|other| stacking != Stacking::Apart && source(other) == stacking);
        match rival {
            None => counted.push(offered),
            Some(other) if other.amount < offered.amount => {
                set_aside.push(std::mem::replace(other, offered));
            }
            Some(_) => set_aside.push(offered),
        }
    }
    (counted, set_aside)
}

impl RollOffer {
    /// Resolves the roll with `choices` from the dice `rolled`, those of
    /// [`dice`](Self::dice) for these choices; `None` when they are not.
    #[must_use]
    pub fn resolve(&self, choices: &RollChoices, rolled: &[Rolled]) -> Option<RollOutcome> {
        if !Rolled::are_of(rolled, &self.dice(choices)) {
            return None;
        }
        let line = |source, amount| RollLine { source, amount };
        let mut lines = Vec::new();
        // The dice of the roll itself, then those of the options picked.
        let (rolled, mut extra) =
            rolled.split_at_checked(self.own_dice(choices.mode).dice().len())?;
        let test = if self.doubles_on_critical() {
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
        let (counted, set_aside) = stacked(picked);
        let mut competence = 0_i64;
        for offered in counted {
            let count = offered.dice().map_or(0, |dice| dice.dice().len());
            let (dice, rest) = extra.split_at_checked(count)?;
            extra = rest;
            let amount = offered.amount.saturating_add(Rolled::sum(dice));
            if let RollOption::TestBonus(bonus) = offered.option
                && test_bonus::stacking(bonus) != Stacking::Apart
            {
                competence = competence.saturating_add(i64::from(amount));
            }
            lines.push(line(
                RollSource::Option(offered.option, dice.to_vec()),
                amount,
            ));
        }
        if competence > i64::from(COMPETENCE_CAP) {
            let excess = i64::from(COMPETENCE_CAP).saturating_sub(competence);
            lines.push(line(RollSource::CompetenceCap, clamp(excess)));
        }
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
        let critical = match (test, self.critical_from) {
            (Some(test), Some(from)) => i32::from(test.kept().face()) >= from,
            (None, _) if choices.critical => {
                let sum = lines
                    .iter()
                    .fold(0_i64, |sum, l| sum.saturating_add(i64::from(l.amount)));
                lines.push(line(RollSource::Critical, clamp(sum)));
                true
            }
            _ => false,
        };
        Some(RollOutcome {
            test,
            lines,
            critical,
            unpicked,
            set_aside,
        })
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Die, Natural};

    use super::*;
    use crate::{Equipment, InPlay, LevelUp, Peuple, Profil, Voie};

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

    fn pick(offer: &RollOffer) -> RollChoices {
        offer
            .options()
            .iter()
            .fold(RollChoices::new(), |choices, o| {
                choices.with_option(o.option().clone())
            })
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
        assert_eq!(Roll::Test(Characteristic::For).id(), "cof.roll.test.for");
        assert_eq!(
            Roll::Attack(AttackMode::Magic).id(),
            format!(
                "cof.roll.{}",
                Value::Attack(AttackMode::Magic)
                    .id()
                    .as_str()
                    .trim_start_matches("cof.")
            )
        );
        assert_eq!(Roll::from_id("cof.roll.test.none"), None);
    }

    #[test]
    fn a_test_adds_the_die_kept_the_help_and_the_modifier() {
        let played = played(&Character::new());
        let roll = Roll::Test(Characteristic::Int);
        let offer = played.roll_offer(roll);
        assert_eq!(offer.cooperation(), Some(2));
        assert_eq!(offer.mode(), RollMode::Plain);
        let choices = RollChoices::new()
            .with_mode(RollMode::MalusDie)
            .with_helpers(2)
            .with_modifier(-1);
        assert_eq!(offer.dice(&choices), Dice::new([20, 20]));
        let outcome = played
            .roll_offer(roll)
            .resolve(&choices, &[d20(1), d20(14)])
            .unwrap();
        let test = outcome.test().unwrap();
        assert_eq!(test.kept(), d20(1));
        assert_eq!(test.natural(), Some(Natural::One));
        assert_eq!(outcome.total(), 1 + offer.base().total() + 4 - 1);
        assert!(
            outcome
                .lines()
                .iter()
                .any(|l| l.source() == &RollSource::Cooperation(2))
        );
        assert!(!outcome.critical());
        assert_eq!(played.roll_offer(roll).resolve(&choices, &[d20(1)]), None);
    }

    #[test]
    fn a_heroic_characteristic_starts_the_roll_with_a_bonus_die() {
        // Force héroïque: a dé bonus to every FOR test.
        let character = ranks(Character::new(), Voie::Brute, 4);
        let played = played(&character);
        let offer = played.roll_offer(Roll::Test(Characteristic::For));
        assert_eq!(offer.mode(), RollMode::BonusDie);
        let [heroic] = offer.bonus_dice() else {
            panic!("one dé bonus expected")
        };
        assert!(heroic.granted() && heroic.featured());
        let other = played.roll_offer(Roll::Test(Characteristic::Agi));
        assert_eq!(other.mode(), RollMode::Plain);
        assert_eq!(other.bonus_dice(), []);
        // Never on an attack (COF.RES.TEST-KINDS).
        let attack = played.roll_offer(Roll::Attack(AttackMode::Contact));
        assert_eq!(attack.mode(), RollMode::Plain);
        assert_eq!(attack.bonus_dice(), []);
    }

    #[test]
    fn a_scoped_bonus_die_is_offered_and_never_starts_the_roll() {
        // Expert en criminalité: a dé bonus to investigation, whatever the
        // characteristic; the engine cannot tell, so the roll starts plain.
        let rogue = ranks(Character::new(), Voie::Roublard, 4);
        let offer = played(&rogue).roll_offer(Roll::Test(Characteristic::Int));
        let [expert] = offer.bonus_dice() else {
            panic!("one dé bonus expected")
        };
        assert!(!expert.granted() && expert.featured());
        assert_eq!(offer.mode(), RollMode::Plain);
    }

    #[test]
    fn a_scoped_bonus_is_offered_unpicked_and_stays_visible() {
        // Intimidation: "rang + 2", naming no characteristic.
        let character = ranks(Character::new(), Voie::Spadassin, 1);
        let played = played(&character);
        let roll = Roll::Test(Characteristic::Cha);
        let offer = played.roll_offer(roll);
        let [offered] = offer.options() else {
            panic!("one option expected")
        };
        assert!(!offered.picked());
        assert_eq!(offered.amount(), 3);

        let plain = played
            .roll_offer(roll)
            .resolve(&RollChoices::new(), &[d20(10)])
            .unwrap();
        assert_eq!(plain.unpicked(), offer.options());
        let picked = played
            .roll_offer(roll)
            .resolve(&pick(&offer), &[d20(10)])
            .unwrap();
        assert_eq!(picked.total(), plain.total() + 3);
        assert_eq!(picked.unpicked(), []);
        // Never offered to an attack (COF.RES.TEST-KINDS).
        let attack = played.roll_offer(Roll::Attack(AttackMode::Contact));
        assert!(
            attack
                .options()
                .iter()
                .all(|o| !matches!(o.option(), RollOption::TestBonus(_)))
        );
    }

    #[test]
    fn a_bonus_goes_to_every_test_and_is_put_forward_on_its_usual_ones() {
        // Discrétion: usually AGI or CHA (D-60); Constitution héroïque: CON
        // named.
        let character = ranks(Character::new(), Voie::Assassin, 1);
        let character = ranks(character, Voie::Mercenaire, 4);
        let played = played(&character);
        let featured = |c| {
            let offer = played.roll_offer(Roll::Test(c));
            let discretion = offer
                .options()
                .iter()
                .find(|o| matches!(o.option(), RollOption::TestBonus(b) if b.voie() == Voie::Assassin))
                .map(Offered::featured);
            (
                discretion,
                offer.bonus_dice().len(),
                offer.bonus_dice().iter().filter(|d| d.featured()).count(),
            )
        };
        assert_eq!(featured(Characteristic::Agi), (Some(true), 0, 0));
        assert_eq!(featured(Characteristic::Int), (Some(false), 0, 0));
        assert_eq!(featured(Characteristic::Con), (Some(false), 1, 1));
        // Every situational option aims at its roll.
        let attack = played.roll_offer(Roll::Attack(AttackMode::Contact));
        assert!(attack.options().iter().all(Offered::featured));
    }

    #[test]
    fn a_bonus_to_every_test_of_the_characteristic_starts_picked() {
        // Tatouages: +3 to the tests of the characteristic chosen.
        let character = ranks(Character::new(), Voie::Pagne, 3).with_ability_option(
            Voie::Pagne,
            3,
            crate::AbilityOption::Characteristic(Characteristic::For),
        );
        let offer = played(&character).roll_offer(Roll::Test(Characteristic::For));
        let tattoo = offer
            .options()
            .iter()
            .find(|o| o.picked())
            .expect("the tattoo is picked");
        assert_eq!(tattoo.amount(), 3);
        assert!(tattoo.featured());
    }

    #[test]
    fn a_higher_bonus_picked_later_takes_the_place_of_the_lower() {
        // Explosifs rank 1 gives +3, Spadassin rank 2 gives +4.
        let character = ranks(
            ranks(Character::new(), Voie::Explosifs, 1),
            Voie::Spadassin,
            2,
        );
        let played = played(&character);
        let roll = Roll::Test(Characteristic::Int);
        let offer = played.roll_offer(roll);
        assert_eq!(offer.roll(), roll);
        assert_eq!(
            offer
                .options()
                .iter()
                .map(Offered::amount)
                .collect::<Vec<_>>(),
            [3, 4]
        );
        let outcome = played
            .roll_offer(roll)
            .resolve(&pick(&offer), &[d20(10)])
            .unwrap();
        assert_eq!(outcome.total(), 10 + offer.base().total() + 4);
        assert_eq!(
            outcome
                .set_aside()
                .iter()
                .map(Offered::amount)
                .collect::<Vec<_>>(),
            [3]
        );
    }

    #[test]
    fn tatouages_add_beside_the_competence_bonuses() {
        let character = ranks(Character::new(), Voie::Pagne, 3).with_ability_option(
            Voie::Pagne,
            3,
            crate::AbilityOption::Characteristic(Characteristic::For),
        );
        let played = played(&character);
        let roll = Roll::Test(Characteristic::For);
        let offer = played.roll_offer(roll);
        assert!(offer.options().len() >= 2);
        let outcome = played
            .roll_offer(roll)
            .resolve(&pick(&offer), &[d20(10)])
            .unwrap();
        let added = offer.options().iter().map(Offered::amount).sum::<i32>();
        assert_eq!(outcome.total(), 10 + offer.base().total() + added);
        assert_eq!(outcome.set_aside(), []);
    }

    #[test]
    fn one_competence_bonus_counts_per_source() {
        // Two voies de profil, "rang + 2" each: +3 and +4.
        let character = ranks(
            ranks(Character::new(), Voie::Spadassin, 1),
            Voie::Explosifs,
            2,
        );
        let played = played(&character);
        let roll = Roll::Test(Characteristic::Int);
        let offer = played.roll_offer(roll);
        assert_eq!(
            offer
                .options()
                .iter()
                .map(Offered::amount)
                .collect::<Vec<_>>(),
            [4, 3]
        );
        let outcome = played
            .roll_offer(roll)
            .resolve(&pick(&offer), &[d20(10)])
            .unwrap();
        assert_eq!(outcome.total(), 10 + offer.base().total() + 4);
        let [aside] = outcome.set_aside() else {
            panic!("one bonus set aside expected")
        };
        assert_eq!(aside.amount(), 3);
        // The lower one picked first changes nothing.
        let reversed = offer
            .options()
            .iter()
            .rev()
            .fold(RollChoices::new(), |c, o| c.with_option(o.option().clone()));
        assert_eq!(
            played.roll_offer(roll).resolve(&reversed, &[d20(10)]),
            Some(outcome)
        );
    }

    #[test]
    fn a_peuple_bonus_adds_to_a_profil_one_and_the_cap_holds() {
        let character = ranks(
            ranks(
                Character::new()
                    .with_profil(Profil::Voleur)
                    .with_peuple(Peuple::Halfelin),
                Voie::Spadassin,
                1,
            ),
            Voie::Halfelin,
            1,
        );
        let roll = Roll::Test(Characteristic::Agi);
        let offer = played(&character).roll_offer(roll);
        let competences = offer
            .options()
            .iter()
            .filter(|o| matches!(o.option(), RollOption::TestBonus(_)))
            .map(Offered::amount)
            .collect::<Vec<_>>();
        assert_eq!(competences, [3, 3]);
        let outcome = played(&character)
            .roll_offer(roll)
            .resolve(&pick(&offer), &[d20(10)])
            .unwrap();
        assert_eq!(outcome.set_aside(), []);
        assert_eq!(outcome.total(), 10 + offer.base().total() + 6);

        // A Game Master's adjustment beyond the cap is taken back.
        let RollOption::TestBonus(bonus) = offer.options()[0].option().clone() else {
            panic!("a test bonus expected")
        };
        let adjusted = character.with_adjustment(
            Target::Value(Value::TestBonus(bonus)),
            Adjustment::new(12, Author::new("gm"), "a legend"),
        );
        let offer = played(&adjusted).roll_offer(roll);
        let outcome = played(&adjusted)
            .roll_offer(roll)
            .resolve(&pick(&offer), &[d20(10)])
            .unwrap();
        assert_eq!(outcome.total(), 10 + offer.base().total() + 15);
        assert!(
            outcome
                .lines()
                .iter()
                .any(|l| l.source() == &RollSource::CompetenceCap && l.amount() == -3)
        );
    }

    #[test]
    fn a_weapon_attack_is_critical_in_the_weapon_range() {
        let played = played(&Character::new());
        let roll = Roll::WeaponAttack(Weapon::Rapiere, WeaponUse::OneHanded);
        let offer = played.roll_offer(roll);
        assert_eq!(offer.critical_from(), Some(19));
        assert_eq!(offer.cooperation(), None);
        let roll_of = |face| {
            played
                .roll_offer(roll)
                .resolve(&RollChoices::new().with_helpers(3), &[d20(face)])
                .unwrap()
        };
        assert!(roll_of(19).critical());
        assert!(!roll_of(18).critical());
        assert_eq!(roll_of(18).total(), 18 + offer.base().total());
        let value = played.roll_offer(Roll::Attack(AttackMode::Ranged));
        assert_eq!(value.critical_from(), Some(20));
    }

    #[test]
    fn a_critical_doubles_the_whole_damage() {
        let character = Character::new();
        let played = played(&character);
        let roll = Roll::Damage(Weapon::EpeeLongue, WeaponUse::OneHanded);
        let offer = played.roll_offer(roll);
        let dice = offer.dice(&RollChoices::new());
        assert_eq!(
            Some(dice.clone()),
            played.damage_dice(Weapon::EpeeLongue, WeaponUse::OneHanded)
        );
        let rolled = dice
            .dice()
            .iter()
            .map(|die| Rolled::new(*die, 3))
            .collect::<Vec<_>>();
        let plain = played
            .roll_offer(roll)
            .resolve(&RollChoices::new().with_modifier(2), &rolled)
            .unwrap();
        assert!(!plain.critical());
        assert_eq!(plain.test(), None);
        let critical = played
            .roll_offer(roll)
            .resolve(
                &RollChoices::new().with_modifier(2).with_critical(),
                &rolled,
            )
            .unwrap();
        assert!(critical.critical());
        assert_eq!(critical.total(), plain.total() * 2);
        assert_eq!(
            critical.lines().last().map(RollLine::source),
            Some(&RollSource::Critical)
        );
        assert_eq!(amounts(&critical).last(), Some(&plain.total()));
    }

    #[test]
    fn situational_dice_are_rolled_when_picked() {
        // Attaque sournoise: dés évolutifs added to the DM in its situation.
        let character = Voie::ALL
            .into_iter()
            .flat_map(|voie| (1..=5).map(move |rank| (voie, rank)))
            .map(|(voie, rank)| ranks(Character::new(), voie, rank))
            .find(|character| {
                played(character)
                    .roll_offer(Roll::Damage(Weapon::Dague, WeaponUse::OneHanded))
                    .options()
                    .iter()
                    .any(|o| o.dice().is_some())
            })
            .expect("a capacité adds dice to the DM in a situation");
        let character = character.with_level_up(LevelUp::new());
        let played = played(&character);
        let roll = Roll::Damage(Weapon::Dague, WeaponUse::OneHanded);
        let offer = played.roll_offer(roll);
        let option = offer.options().iter().find(|o| o.dice().is_some()).unwrap();
        let extra = option.dice().unwrap().dice().len();
        let choices = RollChoices::new().with_option(option.option().clone());
        let plain = offer.dice(&RollChoices::new());
        let dice = offer.dice(&choices);
        assert_eq!(dice.dice().len(), plain.dice().len() + extra);
        let rolled = dice
            .dice()
            .iter()
            .map(|die| Rolled::new(*die, 2))
            .collect::<Vec<_>>();
        let outcome = played.roll_offer(roll).resolve(&choices, &rolled).unwrap();
        assert_eq!(
            outcome.total(),
            2 * i32::try_from(rolled.len()).unwrap() + offer.base().total()
        );
        let line = outcome
            .lines()
            .iter()
            .find(|l| matches!(l.source(), RollSource::Option(..)))
            .unwrap();
        assert_eq!(
            line.source(),
            &RollSource::Option(
                option.option().clone(),
                rolled[plain.dice().len()..].to_vec()
            )
        );
        // Without the option, its dice are not those of the roll.
        assert_eq!(
            played
                .roll_offer(roll)
                .resolve(&RollChoices::new(), &rolled),
            None
        );
    }
}
