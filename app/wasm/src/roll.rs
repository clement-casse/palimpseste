//! The rolls of a character in play (ADR ENG-ROLLS, ADR APP-ROLLS): what a
//! roll offers, and its resolution. The engines say what a roll offers and
//! resolve it; this module names their results and draws the faces, the
//! one thing the engines are given.

use co_kernel::{Dice, Natural, RollMode, Rolled, TestRoll};
use palimpseste_document::{
    ContemporainDocument, Document, Equipment, Error, FantasyDocument, SpecialityEntry,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::sheet::{
    ContributionLine, contemporain_creation, contribution_line, fantasy_creation, speciality_of,
};

/// What the device knows of the character in play: the engines store none
/// of it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Play {
    /// What the character wears and carries now; the equipment of its
    /// document when absent.
    #[serde(default)]
    pub equipment: Option<Equipment>,
    /// The steps of the échelle de santé filled (COC2).
    #[serde(default)]
    pub damage: i32,
}

/// How the d20 of a test is rolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// One d20.
    #[default]
    Plain,
    /// Two d20, the higher kept (dé bonus).
    Bonus,
    /// Two d20, the lower kept (dé malus).
    Malus,
}

impl From<Mode> for RollMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Plain => Self::Plain,
            Mode::Bonus => Self::BonusDie,
            Mode::Malus => Self::MalusDie,
        }
    }
}

impl From<RollMode> for Mode {
    fn from(mode: RollMode) -> Self {
        match mode {
            RollMode::Plain => Self::Plain,
            RollMode::BonusDie => Self::Bonus,
            RollMode::MalusDie => Self::Malus,
        }
    }
}

/// What kind of roll it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum RollKind {
    /// A characteristic test.
    Test,
    /// An attack test.
    Attack,
    /// The DM of a weapon.
    Damage,
}

/// A modifier the player may pick for a roll.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Tsify)]
pub struct RollOptionLine {
    /// What it is, in French: the tests a bonus covers, a situation, a
    /// state of health.
    pub label: String,
    /// Where it comes from, in French: its capacité, or what it is.
    pub origin: String,
    /// The amount added, `None` for dice.
    pub amount: Option<i32>,
    /// The dice added to a DM, such as `1d6`.
    pub dice: Option<String>,
    /// Whether it starts picked.
    pub picked: bool,
    /// Whether it is put forward: a test bonus naming the characteristic
    /// tested or counting it among its usual ones (D-60), and whatever
    /// aims at the roll itself (a situational modifier, the state of
    /// health); the others are offered all the same.
    pub featured: bool,
}

/// A capacité's dé bonus that may apply to a test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct RollBonusDie {
    /// The capacité giving it, in French.
    pub origin: String,
    /// The tests it covers, in a few French words.
    pub scope: String,
    /// Whether it is put forward, as [`RollOptionLine::featured`] says.
    pub featured: bool,
    /// Whether the character has it whatever the test is about: the roll
    /// then starts with a dé bonus. Otherwise it depends on a circumstance
    /// the player knows.
    pub granted: bool,
}

/// A situational note without an amount: shown, never picked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct RollNote {
    /// The capacité it comes from, in French.
    pub origin: String,
    /// The situation, in a few French words.
    pub scope: String,
}

/// What a roll offers within the rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RollOffer {
    /// The roll's identifier.
    pub roll: String,
    /// Its French display name.
    pub name: String,
    /// What kind of roll it is.
    pub kind: RollKind,
    /// What is added to the dice whatever the circumstances.
    pub base: i32,
    /// The terms of the base.
    pub contributions: Vec<ContributionLine>,
    /// The dice rolled with no option picked, such as `1d20` or `2d6`.
    pub dice: String,
    /// The options to pick, named by their position in a request.
    pub options: Vec<RollOptionLine>,
    /// Situational notes without an amount, in French.
    pub notes: Vec<RollNote>,
    /// The capacités' dé bonus that may apply.
    pub bonus_dice: Vec<RollBonusDie>,
    /// The mode the roll starts in.
    pub mode: Mode,
    /// The lowest natural roll of a critical success, for an attack.
    pub critical_from: Option<i32>,
    /// What a critical success does to this DM, in French; `None` when the
    /// roll is no DM or the weapon has no critical.
    pub critical: Option<String>,
    /// What each ally helping adds; `None` when the roll takes no help.
    pub helper_bonus: Option<i32>,
    /// What the count of allies counts, in French.
    pub helpers_label: Option<String>,
    /// The DM roll following this attack, if any.
    pub damage_roll: Option<String>,
}

/// A roll asked for: what the player chose.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RollRequest {
    /// The roll's identifier.
    pub roll: String,
    /// The options picked, by their position in the offer.
    #[serde(default)]
    pub options: Vec<u32>,
    /// How the d20 is rolled.
    #[serde(default)]
    pub mode: Mode,
    /// The Game Master's modifier.
    #[serde(default)]
    pub modifier: i32,
    /// The allies helping.
    #[serde(default)]
    pub helpers: u32,
    /// Whether the DM are those of a critical success.
    #[serde(default)]
    pub critical: bool,
}

/// A die rolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
pub struct RolledDie {
    /// Its number of faces.
    pub faces: u8,
    /// The face it showed.
    pub face: u8,
    /// Whether it counts: `false` for the d20 a dé bonus or malus set
    /// aside.
    pub kept: bool,
}

/// A natural result of the d20 kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum NaturalRoll {
    /// A natural 20.
    Twenty,
    /// A natural 1.
    One,
}

/// What a line of a roll's total is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum RollLineKind {
    /// The dice counted.
    Dice,
    /// A term of the base.
    Base,
    /// An option picked.
    Option,
    /// The allies helping.
    Cooperation,
    /// The Game Master's modifier.
    Modifier,
    /// The competence bonuses beyond their cap, taken back.
    Cap,
    /// What a critical success adds to the DM.
    Critical,
}

/// A line of a roll's total.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct RollLine {
    /// What the line is.
    pub kind: RollLineKind,
    /// Its French label.
    pub label: String,
    /// Where it comes from, in French, for an option.
    pub origin: Option<String>,
    /// What it adds to the total.
    pub amount: i32,
    /// The dice rolled for it.
    pub dice: Vec<RolledDie>,
}

/// A roll resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RollOutcome {
    /// The roll's identifier.
    pub roll: String,
    /// Its French display name.
    pub name: String,
    /// What kind of roll it is.
    pub kind: RollKind,
    /// How the d20 was rolled.
    pub mode: Mode,
    /// Every die rolled, in order.
    pub dice: Vec<RolledDie>,
    /// The natural 20 or 1 of the d20 kept.
    pub natural: Option<NaturalRoll>,
    /// The lines of the total, in order.
    pub lines: Vec<RollLine>,
    /// The total.
    pub total: i32,
    /// Whether an attack's d20 is in the critical range, or the DM are
    /// those of a critical success.
    pub critical: bool,
    /// The lowest natural roll of a critical success, for an attack.
    pub critical_from: Option<i32>,
    /// The options offered and not picked: they may still apply.
    pub unpicked: Vec<RollOptionLine>,
    /// The competence bonuses picked and not counted, another of the same
    /// source being higher.
    pub set_aside: Vec<RollOptionLine>,
    /// The DM roll following this attack, if any.
    pub damage_roll: Option<String>,
}

/// Draws the face of a die with `faces` faces from `random` bytes, each
/// face as likely as another: bytes beyond the last multiple of `faces` are
/// drawn again.
fn face(faces: u8, random: &mut dyn FnMut() -> u8) -> u8 {
    if faces == 0 {
        return 0;
    }
    let limit = u8::MAX - (u8::MAX % faces + 1) % faces;
    loop {
        let byte = random();
        if byte <= limit {
            return byte % faces + 1;
        }
    }
}

/// Rolls `dice` with `random` bytes.
fn draw(dice: &Dice, random: &mut dyn FnMut() -> u8) -> Vec<Rolled> {
    dice.dice()
        .iter()
        .map(|die| Rolled::new(*die, face(die.faces(), random)))
        .collect()
}

fn rolled_die(rolled: Rolled, kept: bool) -> RolledDie {
    RolledDie {
        faces: rolled.die().faces(),
        face: rolled.face(),
        kept,
    }
}

/// Every die of a roll: those of a test with the one kept, or all of them
/// counted.
fn rolled_dice(rolled: &[Rolled], test: Option<TestRoll>) -> Vec<RolledDie> {
    let mut kept = test.map(TestRoll::kept);
    rolled
        .iter()
        .map(|die| match test {
            None => rolled_die(*die, true),
            // Of two equal dice, the first is the one kept.
            Some(_) => rolled_die(*die, kept.take_if(|kept| kept == die).is_some()),
        })
        .collect()
}

fn natural(test: Option<TestRoll>) -> Option<NaturalRoll> {
    test.and_then(TestRoll::natural)
        .map(|natural| match natural {
            Natural::Twenty => NaturalRoll::Twenty,
            Natural::One => NaturalRoll::One,
        })
}

/// The options at the positions `indices` of an offer's `options`.
fn picked<'a, T>(options: &'a [T], indices: &[u32]) -> Result<Vec<&'a T>, Error> {
    indices
        .iter()
        .map(|index| {
            usize::try_from(*index)
                .ok()
                .and_then(|index| options.get(index))
                .ok_or_else(|| unknown("roll option", &index.to_string()))
        })
        .collect()
}

fn unknown(kind: &'static str, id: &str) -> Error {
    Error::UnknownContent {
        kind,
        id: id.to_string(),
    }
}

/// The French count of `helpers` allies.
fn helpers_text(helpers: u32) -> String {
    if helpers == 1 {
        "Coopération : 1 allié".to_owned()
    } else {
        format!("Coopération : {helpers} alliés")
    }
}

fn plain_line(kind: RollLineKind, label: &str, amount: i32) -> RollLine {
    RollLine {
        kind,
        label: label.to_owned(),
        origin: None,
        amount,
        dice: Vec::new(),
    }
}

fn dice_line(rolled: &[Rolled], amount: i32) -> RollLine {
    RollLine {
        kind: RollLineKind::Dice,
        label: rolled.iter().map(|r| r.die()).collect::<Dice>().to_string(),
        origin: None,
        amount,
        dice: rolled.iter().map(|r| rolled_die(*r, true)).collect(),
    }
}

fn base_line(contribution: ContributionLine) -> RollLine {
    RollLine {
        kind: RollLineKind::Base,
        label: contribution.label,
        origin: None,
        amount: contribution.amount,
        dice: Vec::new(),
    }
}

fn option_line(option: RollOptionLine, amount: i32, rolled: &[Rolled]) -> RollLine {
    RollLine {
        kind: RollLineKind::Option,
        label: option.label,
        origin: Some(option.origin),
        amount,
        dice: rolled.iter().map(|r| rolled_die(*r, true)).collect(),
    }
}

/// The tests a bonus covers: the speciality written for its capacité when
/// it applies to it, then its scope.
fn bonus_label(
    specialities: &[SpecialityEntry],
    ability: &str,
    on_speciality: bool,
    scope: &str,
) -> String {
    let speciality = on_speciality
        .then(|| speciality_of(specialities, ability))
        .flatten();
    match speciality {
        Some(speciality) if scope.is_empty() => speciality,
        Some(speciality) => format!("{speciality} ({scope})"),
        None if scope.is_empty() => "Bonus aux tests".to_owned(),
        None => scope.to_owned(),
    }
}

mod fantasy {
    use co_fantasy::{
        InPlay, Offered, Roll, RollChoices, RollOffer as Offer, RollOption, RollSource, label,
        term_label,
    };

    use super::{
        Error, FantasyDocument, Play, RollBonusDie, RollKind, RollLine, RollLineKind, RollMode,
        RollOffer, RollOptionLine, RollOutcome, RollRequest, base_line, bonus_label,
        contribution_line, dice_line, draw, fantasy_creation, helpers_text, natural, option_line,
        picked, plain_line, rolled_dice, unknown,
    };

    fn named(id: &str) -> String {
        label(id).unwrap_or_else(|| id.to_owned())
    }

    fn played(document: &FantasyDocument, play: &Play) -> Result<InPlay, Error> {
        let creation = fantasy_creation(document);
        let equipment = match &play.equipment {
            None => &creation,
            Some(super::Equipment::Fantasy(equipment)) => equipment,
            Some(super::Equipment::Contemporain(_)) => return Err(Error::OtherEngine),
        };
        Ok(document.to_character()?.in_play(equipment.to_equipment()?))
    }

    const fn kind(roll: Roll) -> RollKind {
        match roll {
            Roll::Test(_) => RollKind::Test,
            Roll::Attack(_) | Roll::WeaponAttack(..) => RollKind::Attack,
            Roll::Damage(..) => RollKind::Damage,
        }
    }

    fn damage_roll(roll: Roll) -> Option<String> {
        match roll {
            Roll::WeaponAttack(weapon, weapon_use) => Some(Roll::Damage(weapon, weapon_use).id()),
            Roll::Test(_) | Roll::Attack(_) | Roll::Damage(..) => None,
        }
    }

    fn option(document: &FantasyDocument, offered: &Offered) -> RollOptionLine {
        let (label, origin) = match offered.option() {
            RollOption::TestBonus(bonus) => {
                let ability = bonus.ability();
                (
                    bonus_label(
                        &document.specialities,
                        ability.as_str(),
                        bonus.on_speciality(),
                        bonus.scope(),
                    ),
                    named(ability.as_str()),
                )
            }
            RollOption::Situational(situational) => (
                situational.situation().name().to_owned(),
                named(situational.ability().as_str()),
            ),
        };
        RollOptionLine {
            label,
            origin,
            amount: offered.dice().is_none().then(|| offered.amount()),
            dice: offered.dice().map(ToString::to_string),
            picked: offered.picked(),
            featured: offered.featured(),
        }
    }

    fn roll_of(id: &str) -> Result<Roll, Error> {
        Roll::from_id(id).ok_or_else(|| unknown("roll", id))
    }

    pub(super) fn offer(
        document: &FantasyDocument,
        play: &Play,
        roll: &str,
    ) -> Result<RollOffer, Error> {
        let roll = roll_of(roll)?;
        let offer = played(document, play)?.roll_offer(roll);
        Ok(RollOffer {
            roll: roll.id(),
            name: roll.name(),
            kind: kind(roll),
            base: offer.base().total(),
            contributions: offer
                .base()
                .contributions()
                .iter()
                .map(|c| contribution_line(&label, &term_label, c))
                .collect(),
            dice: offer
                .dice(&RollChoices::new().with_mode(offer.mode()))
                .to_string(),
            options: offer
                .options()
                .iter()
                .map(|o| option(document, o))
                .collect(),
            notes: Vec::new(),
            bonus_dice: offer
                .bonus_dice()
                .iter()
                .map(|offered| {
                    let die = offered.die();
                    RollBonusDie {
                        origin: named(die.ability().as_str()),
                        scope: bonus_label(
                            &document.specialities,
                            die.ability().as_str(),
                            die.on_speciality(),
                            die.scope(),
                        ),
                        featured: offered.featured(),
                        granted: offered.granted(),
                    }
                })
                .collect(),
            mode: offer.mode().into(),
            critical_from: offer.critical_from(),
            critical: offer.doubles_on_critical().then(|| "DM doublés".to_owned()),
            helper_bonus: offer.cooperation(),
            helpers_label: offer
                .cooperation()
                .map(|_| "Alliés ayant réussi leur aide".to_owned()),
            damage_roll: damage_roll(roll),
        })
    }

    fn choices(offer: &Offer, request: &RollRequest) -> Result<RollChoices, Error> {
        let mut choices = RollChoices::new()
            .with_mode(RollMode::from(request.mode))
            .with_modifier(request.modifier)
            .with_helpers(request.helpers);
        if request.critical {
            choices = choices.with_critical();
        }
        for offered in picked(offer.options(), &request.options)? {
            choices = choices.with_option(offered.option().clone());
        }
        Ok(choices)
    }

    pub(super) fn resolve(
        document: &FantasyDocument,
        play: &Play,
        request: &RollRequest,
        random: &mut dyn FnMut() -> u8,
    ) -> Result<RollOutcome, Error> {
        let roll = roll_of(&request.roll)?;
        let played = played(document, play)?;
        let offer = played.roll_offer(roll);
        let choices = choices(&offer, request)?;
        let rolled = draw(&offer.dice(&choices), random);
        let outcome = offer
            .resolve(&choices, &rolled)
            .ok_or_else(|| unknown("roll dice", &roll.id()))?;
        let lines = outcome
            .lines()
            .iter()
            .map(|line| match line.source() {
                RollSource::Dice(dice) => dice_line(dice, line.amount()),
                RollSource::Base(source) => base_line(contribution_line(
                    &label,
                    &term_label,
                    &co_kernel::Contribution::new(source.clone(), line.amount()),
                )),
                RollSource::Option(picked, dice) => {
                    let offered = offer.options().iter().find(|o| o.option() == picked);
                    let named = offered.map_or_default(|o| option(document, o));
                    option_line(named, line.amount(), dice)
                }
                RollSource::CompetenceCap => plain_line(
                    RollLineKind::Cap,
                    "Bonus de compétence plafonnés à +15",
                    line.amount(),
                ),
                RollSource::Cooperation(helpers) => plain_line(
                    RollLineKind::Cooperation,
                    &helpers_text(*helpers),
                    line.amount(),
                ),
                RollSource::Modifier => {
                    plain_line(RollLineKind::Modifier, "Modificateur du MJ", line.amount())
                }
                RollSource::Critical => plain_line(
                    RollLineKind::Critical,
                    "Critique : DM doublés",
                    line.amount(),
                ),
            })
            .collect::<Vec<RollLine>>();
        Ok(RollOutcome {
            roll: roll.id(),
            name: roll.name(),
            kind: kind(roll),
            mode: request.mode,
            dice: rolled_dice(&rolled, outcome.test()),
            natural: natural(outcome.test()),
            lines,
            total: outcome.total(),
            critical: outcome.critical(),
            critical_from: offer.critical_from(),
            unpicked: outcome
                .unpicked()
                .iter()
                .map(|o| option(document, o))
                .collect(),
            set_aside: outcome
                .set_aside()
                .iter()
                .map(|o| option(document, o))
                .collect(),
            damage_roll: damage_roll(roll),
        })
    }
}

mod contemporain {
    use co_contemporain::{
        HealthState, InPlay, Offered, Roll, RollChoices, RollOffer as Offer, RollOption,
        RollSource, label, term_label,
    };

    use super::{
        ContemporainDocument, Error, Play, RollBonusDie, RollKind, RollLine, RollLineKind,
        RollMode, RollNote, RollOffer, RollOptionLine, RollOutcome, RollRequest, base_line,
        bonus_label, contemporain_creation, contribution_line, dice_line, draw, helpers_text,
        natural, option_line, picked, plain_line, rolled_dice, unknown,
    };

    fn named(id: &str) -> String {
        label(id).unwrap_or_else(|| id.to_owned())
    }

    fn played(document: &ContemporainDocument, play: &Play) -> Result<InPlay, Error> {
        let creation = contemporain_creation(document);
        let equipment = match &play.equipment {
            None => &creation,
            Some(super::Equipment::Contemporain(equipment)) => equipment,
            Some(super::Equipment::Fantasy(_)) => return Err(Error::OtherEngine),
        };
        Ok(document.to_character()?.in_play(equipment.to_equipment()?))
    }

    const fn kind(roll: Roll) -> RollKind {
        match roll {
            Roll::Test(_) => RollKind::Test,
            Roll::Attack(_) | Roll::WeaponAttack(_) => RollKind::Attack,
            Roll::Damage(_) => RollKind::Damage,
        }
    }

    fn damage_roll(roll: Roll) -> Option<String> {
        match roll {
            Roll::WeaponAttack(weapon) => Some(Roll::Damage(weapon).id()),
            Roll::Test(_) | Roll::Attack(_) | Roll::Damage(_) => None,
        }
    }

    fn option(document: &ContemporainDocument, offered: &Offered) -> RollOptionLine {
        let (label, origin) = match offered.option() {
            RollOption::TestBonus(bonus) => {
                let ability = bonus.ability();
                (
                    bonus_label(
                        &document.specialities,
                        ability.as_str(),
                        bonus.on_speciality(),
                        bonus.scope(),
                    ),
                    named(ability.as_str()),
                )
            }
            RollOption::HealthState(state) => (state.name().to_owned(), "État de santé".to_owned()),
        };
        RollOptionLine {
            label,
            origin,
            amount: Some(offered.amount()),
            dice: None,
            picked: offered.picked(),
            featured: offered.featured(),
        }
    }

    fn roll_of(id: &str) -> Result<Roll, Error> {
        Roll::from_id(id).ok_or_else(|| unknown("roll", id))
    }

    fn state(played: &InPlay, play: &Play) -> Option<HealthState> {
        played.health_state(play.damage)
    }

    pub(super) fn offer(
        document: &ContemporainDocument,
        play: &Play,
        roll: &str,
    ) -> Result<RollOffer, Error> {
        let roll = roll_of(roll)?;
        let played = played(document, play)?;
        let offer = played.roll_offer(roll, state(&played, play));
        Ok(RollOffer {
            roll: roll.id(),
            name: roll.name(),
            kind: kind(roll),
            base: offer.base().total(),
            contributions: offer
                .base()
                .contributions()
                .iter()
                .map(|c| contribution_line(&label, &term_label, c))
                .collect(),
            dice: offer.dice(offer.mode()).to_string(),
            options: offer
                .options()
                .iter()
                .map(|o| option(document, o))
                .collect(),
            notes: offer
                .notes()
                .iter()
                .map(|note| RollNote {
                    origin: named(note.ability().as_str()),
                    scope: note.situation().name().to_owned(),
                })
                .collect(),
            bonus_dice: offer
                .bonus_dice()
                .iter()
                .map(|offered| {
                    let die = offered.die();
                    RollBonusDie {
                        origin: named(die.ability().as_str()),
                        scope: bonus_label(
                            &document.specialities,
                            die.ability().as_str(),
                            die.on_speciality(),
                            die.scope(),
                        ),
                        featured: offered.featured(),
                        // No COC2 dé bonus holds whatever the test is about.
                        granted: false,
                    }
                })
                .collect(),
            mode: offer.mode().into(),
            critical_from: offer.critical_from(),
            critical: offer
                .critical_bonus()
                .map(|bonus| format!("bonus critique {bonus:+}")),
            helper_bonus: offer.cooperation(),
            helpers_label: offer.cooperation().map(|_| "Alliés aidant".to_owned()),
            damage_roll: damage_roll(roll),
        })
    }

    fn choices(offer: &Offer, request: &RollRequest) -> Result<RollChoices, Error> {
        let mut choices = RollChoices::new()
            .with_mode(RollMode::from(request.mode))
            .with_modifier(request.modifier)
            .with_helpers(request.helpers);
        if request.critical {
            choices = choices.with_critical();
        }
        for offered in picked(offer.options(), &request.options)? {
            choices = choices.with_option(offered.option().clone());
        }
        Ok(choices)
    }

    pub(super) fn resolve(
        document: &ContemporainDocument,
        play: &Play,
        request: &RollRequest,
        random: &mut dyn FnMut() -> u8,
    ) -> Result<RollOutcome, Error> {
        let roll = roll_of(&request.roll)?;
        let played = played(document, play)?;
        let state = state(&played, play);
        let offer = played.roll_offer(roll, state);
        let choices = choices(&offer, request)?;
        let rolled = draw(&offer.dice(request.mode.into()), random);
        let outcome = offer
            .resolve(&choices, &rolled)
            .ok_or_else(|| unknown("roll dice", &roll.id()))?;
        let lines = outcome
            .lines()
            .iter()
            .map(|line| match line.source() {
                RollSource::Dice(dice) => dice_line(dice, line.amount()),
                RollSource::Base(source) => base_line(contribution_line(
                    &label,
                    &term_label,
                    &co_kernel::Contribution::new(source.clone(), line.amount()),
                )),
                RollSource::Option(picked) => {
                    let offered = offer.options().iter().find(|o| o.option() == picked);
                    let named = offered.map_or_default(|o| option(document, o));
                    option_line(named, line.amount(), &[])
                }
                RollSource::Cooperation(helpers) => plain_line(
                    RollLineKind::Cooperation,
                    &helpers_text(*helpers),
                    line.amount(),
                ),
                RollSource::Modifier => {
                    plain_line(RollLineKind::Modifier, "Modificateur du MJ", line.amount())
                }
                RollSource::CriticalBonus => plain_line(
                    RollLineKind::Critical,
                    "Critique : bonus critique",
                    line.amount(),
                ),
            })
            .collect::<Vec<RollLine>>();
        Ok(RollOutcome {
            roll: roll.id(),
            name: roll.name(),
            kind: kind(roll),
            mode: request.mode,
            dice: rolled_dice(&rolled, outcome.test()),
            natural: natural(outcome.test()),
            lines,
            total: outcome.total(),
            critical: outcome.critical(),
            critical_from: offer.critical_from(),
            unpicked: outcome
                .unpicked()
                .iter()
                .map(|o| option(document, o))
                .collect(),
            set_aside: Vec::new(),
            damage_roll: damage_roll(roll),
        })
    }
}

/// What the roll `roll` offers the character of `document` in play.
///
/// # Errors
///
/// Returns why the document or the equipment cannot be read, an unknown
/// roll, and [`Error::OtherEngine`] for an equipment of another engine.
pub fn roll_offer(document: &Document, play: &Play, roll: &str) -> Result<RollOffer, Error> {
    match document {
        Document::Contemporain(document) => contemporain::offer(document, play, roll),
        Document::Fantasy(document) => fantasy::offer(document, play, roll),
    }
}

/// Rolls `request` for the character of `document` in play, the faces
/// drawn from `random` bytes.
///
/// # Errors
///
/// Returns why the document or the equipment cannot be read, an unknown
/// roll or option, and [`Error::OtherEngine`] for an equipment of another
/// engine.
pub fn roll(
    document: &Document,
    play: &Play,
    request: &RollRequest,
    random: &mut dyn FnMut() -> u8,
) -> Result<RollOutcome, Error> {
    match document {
        Document::Contemporain(document) => contemporain::resolve(document, play, request, random),
        Document::Fantasy(document) => fantasy::resolve(document, play, request, random),
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Die;

    use super::*;

    #[test]
    fn every_face_is_as_likely_and_bytes_beyond_are_drawn_again() {
        for faces in [1_u8, 4, 6, 20, 255] {
            let mut counts = vec![0_u32; usize::from(faces) + 1];
            for byte in 0..=u8::MAX {
                // A rejected byte is followed by one that is not.
                let mut bytes = [byte, 0].into_iter();
                let drawn = face(faces, &mut || bytes.next().unwrap());
                if bytes.next().is_some() {
                    counts[usize::from(drawn)] += 1;
                }
            }
            assert_eq!(counts[0], 0);
            assert!(counts[1..].windows(2).all(|pair| pair[0] == pair[1]));
            assert!(counts[1] > 0);
        }
        assert_eq!(face(0, &mut || 0), 0);
    }

    #[test]
    fn modes_convert_both_ways() {
        for mode in [Mode::Plain, Mode::Bonus, Mode::Malus] {
            assert_eq!(Mode::from(RollMode::from(mode)), mode);
        }
    }

    #[test]
    fn naturals_and_allies_are_named() {
        let d20 = |face| Rolled::new(Die::new(20), face);
        let one = TestRoll::new(RollMode::Plain, &[d20(1)]);
        assert_eq!(natural(one), Some(NaturalRoll::One));
        assert_eq!(natural(None), None);
        assert_eq!(helpers_text(1), "Coopération : 1 allié");
        assert_eq!(helpers_text(3), "Coopération : 3 alliés");
    }

    #[test]
    fn a_bonus_is_named_by_the_speciality_written_then_its_scope() {
        let written = [SpecialityEntry {
            ability: "coc.investigation.2".into(),
            speciality: "généalogie".into(),
        }];
        let label =
            |ability, on_speciality, scope| bonus_label(&written, ability, on_speciality, scope);
        assert_eq!(
            label("coc.investigation.2", true, "enquête"),
            "généalogie (enquête)"
        );
        assert_eq!(label("coc.investigation.2", true, ""), "généalogie");
        // Not written for this capacité, or not about it.
        assert_eq!(label("coc.crime.1", true, "vol"), "vol");
        assert_eq!(label("coc.investigation.2", false, "enquête"), "enquête");
        assert_eq!(label("coc.crime.1", false, ""), "Bonus aux tests");
    }

    #[test]
    fn dice_are_drawn_in_order() {
        let mut bytes = [2_u8, 5].into_iter();
        let rolled = draw(&Dice::new([6, 20]), &mut || bytes.next().unwrap());
        assert_eq!(
            rolled,
            [Rolled::new(Die::new(6), 3), Rolled::new(Die::new(20), 6)]
        );
    }
}
