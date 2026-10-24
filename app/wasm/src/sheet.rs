//! The evaluation of a choices document: every value with its breakdown,
//! the violations placed in their creation step, and the status of each
//! step. Everything is computed and named by the engines; this module only
//! maps their results to plain data.

use std::collections::BTreeSet;

use co_kernel::{Contribution, DerivedValue, ParameterValue, RuleId, Source, ViolationReport};
use palimpseste_document::{
    Characteristic, ContemporainDocument, ContemporainEquipment, Document, Equipment, Error,
    FantasyDocument, FantasyEquipment, ItemEntry, Subject, WaiverEntry,
};
use serde::Serialize;
use tsify::Tsify;

use crate::catalog::Named;

/// A character evaluated by its engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    /// The creation steps, in the book order, with what is left to settle.
    pub steps: Vec<StepStatus>,
    /// The COF2 level-ups or COC2 spendings of XP, in order, each with the
    /// status of its steps.
    pub progression: Vec<ProgressionStatus>,
    /// The seven starting characteristics.
    pub characteristics: Vec<CharacteristicLine>,
    /// The computed values, budgets included.
    pub values: Vec<ValueLine>,
    /// Every violation, waived or not, in the engine's stable order.
    pub violations: Vec<ReportedViolation>,
    /// The waivers that accept no violation.
    pub unused_waivers: Vec<WaiverEntry>,
    /// What only one engine computes.
    pub details: Details,
    /// The findings of the rules that hold in play, about the equipment in
    /// play (ADR ENG-IN-PLAY); empty from [`evaluate`].
    pub in_play: Vec<ReportedViolation>,
}

/// What is left to settle in a creation step: violations no waiver accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct StepStatus {
    /// The step's identifier.
    pub id: String,
    /// Its French display name.
    pub name: String,
    /// Missing choices, and values that cannot be computed.
    pub incomplete: u32,
    /// Rules broken and not waived.
    pub deviations: u32,
    /// Manual adjustments.
    pub notices: u32,
}

/// A level-up (COF2) or a spending of XP (COC2), and what is left to settle
/// in each of its steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ProgressionStatus {
    /// The level it reaches (COF2) or its number from 1 (COC2).
    pub number: u32,
    /// Its French display name, such as `Niveau 3`.
    pub name: String,
    /// Its steps, in its form's order.
    pub steps: Vec<StepStatus>,
}

/// A starting characteristic and its breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct CharacteristicLine {
    /// The characteristic.
    pub characteristic: Characteristic,
    /// Its value, adjustments included.
    pub total: i32,
    /// The terms that produced it.
    pub contributions: Vec<ContributionLine>,
}

/// A computed value and its breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ValueLine {
    /// The value's identifier, the target of its adjustments.
    pub id: String,
    /// Its French display name.
    pub label: String,
    /// The value, `None` while it cannot be computed.
    pub total: Option<i32>,
    /// The terms that produced it.
    pub contributions: Vec<ContributionLine>,
    /// The dice rolled with it, for weapon damage and dés de récupération.
    pub dice: Option<String>,
}

/// One term of a breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ContributionLine {
    /// Where the term comes from.
    pub source: SourceKind,
    /// The identifier of a capacité or value source (`cof.rage.2`), the key
    /// its name will attach to.
    pub source_id: Option<String>,
    /// A French label for the source.
    pub label: String,
    /// The signed amount.
    pub amount: i32,
}

/// The kind of source of a contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// A characteristic.
    Characteristic,
    /// A capacité.
    Ability,
    /// A term of the rule's formula.
    Rule,
    /// Another value: content, a choice, a budget.
    Value,
    /// A manual adjustment.
    Adjustment,
}

/// One violation of the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ReportedViolation {
    /// The stable rule identifier, such as `COF.CRE.PROFIL`.
    pub rule: String,
    /// The mechanic that emitted the violation.
    pub mechanic: String,
    /// What the violation is about; a waiver names the same subject.
    pub subject: Subject,
    /// A French label for the subject, when the engine names it.
    pub subject_label: Option<String>,
    /// Its kind.
    pub category: Category,
    /// Whether a waiver accepts it.
    pub waived: bool,
    /// The level-up or spending it is about (see
    /// [`ProgressionStatus::number`]), `None` for creation.
    pub progression: Option<u32>,
    /// The identifier of the step where it is settled: a creation step, or
    /// a step of its level-up or spending.
    pub step: String,
    /// The values the violation reports, such as the points spent.
    pub parameters: Vec<ReportedParameter>,
}

/// A named value of a violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ReportedParameter {
    /// The parameter's name.
    pub name: String,
    /// Its value.
    pub value: ReportedValue,
}

/// The value of a violation parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(untagged)]
pub enum ReportedValue {
    /// A number.
    Integer(i64),
    /// A text.
    Text(String),
}

/// The kind of a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Category {
    /// A required choice is missing.
    Incomplete,
    /// A rule is broken.
    Deviation {
        /// How strongly the book states the rule.
        severity: Severity,
    },
    /// A value cannot be computed from the current inputs.
    Uncomputable,
    /// Information only.
    Notice,
}

/// How strongly the book states a broken rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The book calls the rule indicative.
    Recommended,
    /// A book rule the Game Master may waive.
    Standard,
    /// The book says never.
    Absolute,
}

/// What only one engine computes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(tag = "engine", rename_all = "lowercase")]
pub enum Details {
    /// COC2.
    Contemporain(ContemporainDetails),
    /// COF2.
    Fantasy(FantasyDetails),
}

/// What COF2 computes besides values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FantasyDetails {
    /// The profil's family.
    pub family: Option<Named>,
    /// The die of the dés de récupération, such as `d10`.
    pub recovery_die: Option<String>,
    /// The voies the voie de peuple slot accepts.
    pub peuple_slot_options: Vec<String>,
    /// The money held, with the fewest coins, such as `7 pa`.
    pub money: Option<String>,
    /// The level reached.
    pub level: u8,
    /// The die a dé évolutif d4° stands for at that level, such as `d6`.
    pub evolutive_die: String,
    /// What each level-up offers within the rules, in order.
    pub levels: Vec<LevelLine>,
    /// The capacités held that offer options.
    pub ability_options: Vec<AbilityOptionLine>,
    /// Every capacité held, with the level-up that acquired it.
    pub abilities_held: Vec<HeldAbility>,
}

/// What a COF2 level-up offers within the rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LevelLine {
    /// The level it reaches.
    pub level: u8,
    /// Its points to spend: 2, and those its forgotten ranks refund.
    pub points_available: i32,
    /// Its points spent, the orphan point included.
    pub points_spent: i32,
    /// The capacités it may acquire within the rules, with their cost.
    pub acquirable: Vec<AbilityCost>,
    /// The capacités it may forget within the rules.
    pub forgettable: Vec<String>,
    /// The top ranks never forgotten, with the reason (`free`,
    /// `level-one`).
    pub protected: Vec<ProtectedAbility>,
    /// How many capacités it may forget.
    pub forget_max: u32,
    /// Whether its orphan point may be exchanged.
    pub orphan_allowed: bool,
}

/// A capacité and what acquiring it costs: capacity points (COF2) or XP
/// (COC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct AbilityCost {
    /// The capacité's identifier.
    pub ability: String,
    /// Its cost.
    pub cost: i32,
}

/// A capacité the rules never let forget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ProtectedAbility {
    /// The capacité's identifier.
    pub ability: String,
    /// Why: `free` or `level-one`.
    pub reason: String,
}

/// A COF2 capacité held that offers options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AbilityOptionLine {
    /// The capacité's identifier.
    pub ability: String,
    /// The identifiers of the options it offers the character.
    pub offered: Vec<String>,
    /// Whether it has no effect until one is chosen.
    pub required: bool,
    /// The level-up that acquired it, `None` for creation.
    pub progression: Option<u32>,
}

/// A capacité held, and where it was acquired.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct HeldAbility {
    /// The capacité's identifier.
    pub ability: String,
    /// The level-up (COF2) or spending (COC2) that acquired it, `None` for
    /// creation.
    pub progression: Option<u32>,
}

/// What COC2 computes besides values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ContemporainDetails {
    /// The age bracket's display name.
    pub age_bracket: Option<String>,
    /// The social class's display name, `None` when contradictory.
    pub social_class: Option<String>,
    /// Characteristic points allocated.
    pub carac_points_spent: i64,
    /// Capacity points spent: one per rank.
    pub capacity_points_spent: i32,
    /// Voies with at least one rank.
    pub voie_count: i32,
    /// The sum of the advantages.
    pub advantages: i32,
    /// The sum of the disadvantages, negative.
    pub disadvantages: i32,
    /// What each spending of XP offers within the rules, in order.
    pub spendings: Vec<SpendingLine>,
    /// The rank 5 capacités held, with the characteristic each raises.
    pub rank_five: Vec<RankFiveLine>,
    /// Every capacité held, with the spending that bought it.
    pub abilities_held: Vec<HeldAbility>,
}

/// What a COC2 spending of XP offers within the rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SpendingLine {
    /// Its number, from 1.
    pub number: u32,
    /// The XP left once every spending is paid.
    pub xp_left: i32,
    /// The ranks it may buy within the rules, with their XP.
    pub acquirable: Vec<AbilityCost>,
    /// The trait change raising the social class one class.
    pub class_rise: Option<ClassRise>,
    /// The XP the cure of Toxicomane costs, when the trait is held.
    pub cure: Option<i32>,
}

/// The trait change raising the social class one class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ClassRise {
    /// The class trait's identifier.
    pub id: String,
    /// Its new points; none gives the trait up.
    pub points: Option<u8>,
    /// The XP it costs.
    pub cost: i32,
}

/// A COC2 rank 5 held and the characteristic it raises.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct RankFiveLine {
    /// The voie's identifier.
    pub voie: String,
    /// The characteristics raised whatever the choice.
    pub raises: Vec<Characteristic>,
    /// The characteristics to choose from; empty when fixed.
    pub choice: Vec<Characteristic>,
    /// The spending that bought the rank, `None` for creation.
    pub progression: Option<u32>,
}

/// Evaluates `document` with its engine.
///
/// # Errors
///
/// Returns why the document cannot be read.
pub fn evaluate(document: &Document) -> Result<Sheet, Error> {
    match document {
        Document::Contemporain(document) => contemporain(document, None),
        Document::Fantasy(document) => fantasy(document, None),
    }
}

/// Evaluates a choices document with the character in play, wearing and
/// carrying `equipment` (ADR ENG-IN-PLAY): the steps and violations of its
/// creation, its values with the equipment in play, and the findings of the
/// rules that hold in play.
///
/// # Errors
///
/// Returns why the document or the equipment cannot be read, and
/// [`Error::OtherEngine`] for an equipment of another engine.
pub fn evaluate_in_play(document: &Document, equipment: &Equipment) -> Result<Sheet, Error> {
    match (document, equipment) {
        (Document::Contemporain(document), Equipment::Contemporain(equipment)) => {
            contemporain(document, Some(equipment))
        }
        (Document::Fantasy(document), Equipment::Fantasy(equipment)) => {
            fantasy(document, Some(equipment))
        }
        _ => Err(Error::OtherEngine),
    }
}

/// The parts every engine shares, from its report and the engine's labels
/// (`label` and `term_label`, ADR ENG-CATALOGUE).
struct Common<'a> {
    report: &'a ViolationReport,
    steps: Vec<(&'static str, &'static str)>,
    /// The level-ups or spendings, by number, with their display name.
    progression: Vec<(u32, String)>,
    /// The steps of a level-up or spending.
    progression_steps: Vec<(&'static str, &'static str)>,
    /// Where a violation is settled: its level-up or spending, if any, and
    /// its step.
    place: &'a dyn Fn(&co_kernel::Violation) -> (Option<u32>, &'static str),
    label: &'a dyn Fn(&str) -> Option<String>,
    term_label: &'a dyn Fn(&RuleId) -> Option<&'static str>,
}

impl Common<'_> {
    fn sheet(
        &self,
        characteristics: impl Fn(co_kernel::Characteristic) -> DerivedValue,
        values: Vec<ValueLine>,
        details: Details,
        in_play: Option<&ViolationReport>,
    ) -> Sheet {
        let violations: Vec<ReportedViolation> = self
            .report
            .entries()
            .iter()
            .map(|entry| self.violation(entry))
            .collect();
        let status = |progression: Option<u32>, (id, name): &(&str, &str)| {
            let mut status = StepStatus {
                id: (*id).to_owned(),
                name: (*name).to_owned(),
                incomplete: 0,
                deviations: 0,
                notices: 0,
            };
            for violation in violations
                .iter()
                .filter(|v| v.progression == progression && v.step == *id && !v.waived)
            {
                match violation.category {
                    Category::Incomplete | Category::Uncomputable => status.incomplete += 1,
                    Category::Deviation { .. } => status.deviations += 1,
                    Category::Notice => status.notices += 1,
                }
            }
            status
        };
        let steps = self.steps.iter().map(|step| status(None, step)).collect();
        let progression = self
            .progression
            .iter()
            .map(|(number, name)| ProgressionStatus {
                number: *number,
                name: name.clone(),
                steps: self
                    .progression_steps
                    .iter()
                    .map(|step| status(Some(*number), step))
                    .collect(),
            })
            .collect();
        Sheet {
            steps,
            progression,
            characteristics: co_kernel::Characteristic::ALL
                .into_iter()
                .map(|c| {
                    let value = characteristics(c);
                    CharacteristicLine {
                        characteristic: c.into(),
                        total: value.total(),
                        contributions: self.contributions(&value),
                    }
                })
                .collect(),
            values,
            violations,
            unused_waivers: self
                .report
                .unused_waivers()
                .iter()
                .map(|waiver| WaiverEntry {
                    rule: waiver.rule().as_str().to_owned(),
                    subject: waiver.subject().into(),
                    author: waiver.author().as_str().to_owned(),
                    reason: waiver.reason().to_owned(),
                })
                .collect(),
            details,
            in_play: in_play
                .map(|report| {
                    report
                        .entries()
                        .iter()
                        .map(|entry| self.violation(entry))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn violation(&self, entry: &co_kernel::ReportEntry) -> ReportedViolation {
        let violation = entry.violation();
        let subject_label = match violation.subject() {
            co_kernel::Subject::Character => None,
            co_kernel::Subject::Characteristic(c) => Some(c.abbreviation().to_owned()),
            co_kernel::Subject::Voie(id) => (self.label)(id.as_str()),
            co_kernel::Subject::Ability(id) => (self.label)(id.as_str()),
            co_kernel::Subject::Value(id) => (self.label)(id.as_str()),
        };
        let (progression, step) = (self.place)(violation);
        ReportedViolation {
            rule: violation.rule().as_str().to_owned(),
            mechanic: violation.mechanic().as_str().to_owned(),
            subject: violation.subject().into(),
            subject_label,
            category: violation.category().into(),
            waived: entry.is_waived(),
            progression,
            step: step.to_owned(),
            parameters: violation
                .parameters()
                .iter()
                .map(|parameter| ReportedParameter {
                    name: parameter.name().to_owned(),
                    value: match parameter.value() {
                        ParameterValue::Integer(value) => ReportedValue::Integer(*value),
                        ParameterValue::Text(text) => ReportedValue::Text(text.clone()),
                    },
                })
                .collect(),
        }
    }

    fn contributions(&self, value: &DerivedValue) -> Vec<ContributionLine> {
        value
            .contributions()
            .iter()
            .map(|contribution| self.contribution(contribution))
            .collect()
    }

    fn contribution(&self, contribution: &Contribution) -> ContributionLine {
        let labelled = |id: &str| (self.label)(id).unwrap_or_else(|| id.to_owned());
        let (source, source_id, label) = match contribution.source() {
            Source::Characteristic(c) => (
                SourceKind::Characteristic,
                None,
                c.abbreviation().to_owned(),
            ),
            Source::Ability(id) => (
                SourceKind::Ability,
                Some(id.as_str().to_owned()),
                labelled(id.as_str()),
            ),
            Source::Rule(rule) => (
                SourceKind::Rule,
                None,
                (self.term_label)(rule).unwrap_or("Base").to_owned(),
            ),
            Source::Value(id) => (
                SourceKind::Value,
                Some(id.as_str().to_owned()),
                labelled(id.as_str()),
            ),
            Source::Adjustment { author, reason } => (
                SourceKind::Adjustment,
                None,
                format!("Ajustement de {} : {reason}", author.as_str()),
            ),
        };
        ContributionLine {
            source,
            source_id,
            label,
            amount: contribution.amount(),
        }
    }

    fn line(&self, id: String, label: String, value: Option<&DerivedValue>) -> ValueLine {
        ValueLine {
            id,
            label,
            total: value.map(DerivedValue::total),
            contributions: value
                .map(|value| self.contributions(value))
                .unwrap_or_default(),
            dice: None,
        }
    }
}

/// The weapons the document carries, in document order without repeats.
fn carried<T: Ord + Copy>(items: &[ItemEntry], from_id: impl Fn(&str) -> Option<T>) -> Vec<T> {
    let mut seen = BTreeSet::new();
    items
        .iter()
        .filter_map(|item| match item {
            ItemEntry::Weapon { id, quantity } if *quantity > 0 => from_id(id),
            _ => None,
        })
        .filter(|weapon| seen.insert(*weapon))
        .collect()
}

fn fantasy(
    document: &FantasyDocument,
    equipment: Option<&FantasyEquipment>,
) -> Result<Sheet, Error> {
    use co_fantasy::{Family, LevelStep, Money, Placement, Step, Value, Weapon, WeaponUse};
    let character = document.to_character()?;
    let report = character.report();
    // In play, the values follow the equipment in play; the steps and
    // violations stay those of the creation.
    let played = equipment
        .map(|equipment| equipment.to_equipment().map(|e| character.in_play(e)))
        .transpose()?;
    let value_of = |value: Value| match &played {
        Some(played) => played.value(value),
        None => character.value(value),
    };
    let items = equipment.map_or(&document.items, |equipment| &equipment.items);
    let place = |violation: &co_kernel::Violation| match character.placement(violation) {
        Placement::Creation(step) => (None, step.id()),
        Placement::Level(level, step) => (Some(u32::from(level.get())), step.id()),
    };
    let level = character.level().get();
    let common = Common {
        report: &report,
        steps: Step::ALL
            .iter()
            .map(|step| (step.id(), step.name()))
            .collect(),
        progression: (2..=level)
            .map(|level| (u32::from(level), format!("Niveau {level}")))
            .collect(),
        progression_steps: LevelStep::ALL
            .iter()
            .map(|step| (step.id(), step.name()))
            .collect(),
        place: &place,
        label: &co_fantasy::label,
        term_label: &co_fantasy::term_label,
    };
    let line = |value: Value| {
        common.line(
            value.id().as_str().to_owned(),
            value.name(),
            value_of(value).as_ref(),
        )
    };
    let recovery_die = character.recovery_die().map(|die| die.to_string());
    let mut values: Vec<ValueLine> = Value::FIXED.into_iter().map(line).collect();
    for value in &mut values {
        if value.id == Value::RecoveryDice.id().as_str() {
            value.dice.clone_from(&recovery_die);
        }
    }
    for weapon in carried(items, Weapon::from_id) {
        for weapon_use in WeaponUse::ALL {
            if let Some(dice) = weapon.dice(weapon_use) {
                let mut damage = line(Value::Damage(weapon, weapon_use));
                damage.dice = Some(dice.to_string());
                values.push(damage);
            }
        }
        values.push(line(Value::CriticalRange(weapon)));
    }
    let details = Details::Fantasy(FantasyDetails {
        family: character
            .family()
            .map(|family: Family| Named::new(family.id().as_str(), family.name())),
        recovery_die,
        peuple_slot_options: character
            .peuple_slot_options()
            .iter()
            .map(|voie| voie.id().as_str().to_owned())
            .collect(),
        money: value_of(Value::Money).map(|money| Money::pc(money.total()).to_string()),
        level,
        evolutive_die: format!("d{}", character.level().evolutive_die().faces()),
        levels: fantasy_levels(&character),
        ability_options: fantasy_options(&character),
        abilities_held: character
            .ranks()
            .into_iter()
            .map(|(voie, rank)| HeldAbility {
                ability: voie.ability(rank).as_str().to_owned(),
                progression: character
                    .acquired_at(voie, rank)
                    .map(|level| u32::from(level.get())),
            })
            .collect(),
    });
    let in_play = played.as_ref().map(co_fantasy::InPlay::report);
    Ok(common.sheet(
        |c| character.characteristic(c),
        values,
        details,
        in_play.as_ref(),
    ))
}

fn contemporain(
    document: &ContemporainDocument,
    equipment: Option<&ContemporainEquipment>,
) -> Result<Sheet, Error> {
    use co_contemporain::{Placement, SpendingStep, Step, Value, Weapon};
    let character = document.to_character()?;
    let report = character.report();
    // In play, the values follow the equipment in play; the steps and
    // violations stay those of the creation. No COC2 equipment rule holds in
    // play.
    let played = equipment
        .map(|equipment| equipment.to_equipment().map(|e| character.in_play(e)))
        .transpose()?;
    let value_of = |value: Value| match &played {
        Some(played) => played.value(value),
        None => character.value(value),
    };
    let damage_dice = |weapon: Weapon| match &played {
        Some(played) => played.damage_dice(weapon),
        None => character.damage_dice(weapon),
    };
    let items = equipment.map_or(&document.items, |equipment| &equipment.items);
    let place = |violation: &co_kernel::Violation| match character.placement(violation) {
        Placement::Creation(step) => (None, step.id()),
        Placement::Spending(spending, step) => (Some(count(spending)), step.id()),
    };
    let spendings = document.xp_spendings.len();
    let common = Common {
        report: &report,
        steps: Step::ALL
            .iter()
            .map(|step| (step.id(), step.name()))
            .collect(),
        progression: (1..=spendings)
            .map(|spending| (count(spending), format!("Dépense d'XP {spending}")))
            .collect(),
        progression_steps: SpendingStep::ALL
            .iter()
            .map(|step| (step.id(), step.name()))
            .collect(),
        place: &place,
        label: &co_contemporain::label,
        term_label: &co_contemporain::term_label,
    };
    let line = |value: Value| {
        common.line(
            value.id().as_str().to_owned(),
            value.name(),
            value_of(value).as_ref(),
        )
    };
    let mut values: Vec<ValueLine> = Value::FIXED.into_iter().map(line).collect();
    let mut weapons = vec![Weapon::MainsNues];
    weapons.extend(
        carried(items, Weapon::from_id)
            .into_iter()
            .filter(|weapon| *weapon != Weapon::MainsNues),
    );
    for weapon in weapons {
        let mut damage = line(Value::Damage(weapon));
        damage.dice = Some(damage_dice(weapon).to_string());
        values.push(damage);
        values.push(line(Value::CriticalBonus(weapon)));
        values.push(line(Value::CriticalRange(weapon)));
    }
    let details = Details::Contemporain(ContemporainDetails {
        age_bracket: character
            .age_bracket()
            .map(|bracket| bracket.name().to_owned()),
        social_class: character
            .social_class()
            .map(|class| class.name().to_owned()),
        carac_points_spent: character.carac_points_spent(),
        capacity_points_spent: character.capacity_points_spent(),
        voie_count: character.voie_count(),
        advantages: character.advantages(),
        disadvantages: character.disadvantages(),
        spendings: contemporain_spendings(&character, spendings),
        rank_five: contemporain_rank_five(&character),
        abilities_held: character
            .ranks()
            .into_iter()
            .map(|(voie, rank)| HeldAbility {
                ability: voie.ability(rank).as_str().to_owned(),
                progression: character.bought_in(voie, rank).map(count),
            })
            .collect(),
    });
    Ok(common.sheet(|c| character.characteristic(c), values, details, None))
}

/// A count or number from the engines, as the bindings send it.
fn count(number: usize) -> u32 {
    u32::try_from(number).unwrap_or(u32::MAX)
}

/// What each COF2 level-up offers within the rules.
fn fantasy_levels(character: &co_fantasy::Character) -> Vec<LevelLine> {
    let id = |voie: co_fantasy::Voie, rank| voie.ability(rank).as_str().to_owned();
    (2..=character.level().get())
        .filter_map(|level| {
            let offer = character.level_offer(co_fantasy::Level::from(level))?;
            Some(LevelLine {
                level,
                points_available: offer.points_available(),
                points_spent: offer.points_spent(),
                acquirable: offer
                    .acquirable()
                    .iter()
                    .map(|(voie, rank)| AbilityCost {
                        ability: id(*voie, *rank),
                        cost: co_fantasy::rank_cost(*rank),
                    })
                    .collect(),
                forgettable: offer
                    .forgettable()
                    .iter()
                    .map(|(voie, rank)| id(*voie, *rank))
                    .collect(),
                protected: offer
                    .protected()
                    .iter()
                    .map(|(voie, rank, reason)| ProtectedAbility {
                        ability: id(*voie, *rank),
                        reason: (*reason).to_owned(),
                    })
                    .collect(),
                forget_max: count(offer.forget_max()),
                orphan_allowed: offer.orphan_allowed(),
            })
        })
        .collect()
}

/// The COF2 capacités held that offer options.
fn fantasy_options(character: &co_fantasy::Character) -> Vec<AbilityOptionLine> {
    character
        .ranks()
        .into_iter()
        .filter_map(|(voie, rank)| {
            let offered = character.ability_options_offered(voie, rank);
            (!offered.is_empty()).then(|| AbilityOptionLine {
                ability: voie.ability(rank).as_str().to_owned(),
                offered: offered
                    .iter()
                    .map(|option| option.id().to_owned())
                    .collect(),
                required: character.ability_option_required(voie, rank),
                progression: character
                    .acquired_at(voie, rank)
                    .map(|level| u32::from(level.get())),
            })
        })
        .collect()
}

/// What each of the `spendings` COC2 spendings of XP offers within the rules.
fn contemporain_spendings(
    character: &co_contemporain::Character,
    spendings: usize,
) -> Vec<SpendingLine> {
    (1..=spendings)
        .filter_map(|spending| {
            let offer = character.spending_offer(spending)?;
            Some(SpendingLine {
                number: count(spending),
                xp_left: offer.xp_left(),
                acquirable: offer
                    .acquirable()
                    .iter()
                    .map(|(voie, rank)| AbilityCost {
                        ability: voie.ability(*rank).as_str().to_owned(),
                        cost: co_contemporain::xp_cost(*rank),
                    })
                    .collect(),
                class_rise: offer.class_rise().map(|(t, points, cost)| ClassRise {
                    id: t.id().as_str().to_owned(),
                    points: points.map(co_contemporain::TraitPoints::get),
                    cost,
                }),
                cure: offer.cure(),
            })
        })
        .collect()
}

/// The COC2 rank 5 capacités held and the characteristic each raises.
fn contemporain_rank_five(character: &co_contemporain::Character) -> Vec<RankFiveLine> {
    character
        .ranks()
        .into_iter()
        .filter(|(_, rank)| rank.get() == 5)
        .map(|(voie, rank)| RankFiveLine {
            voie: voie.id().as_str().to_owned(),
            raises: voie
                .rank_five_raises()
                .iter()
                .map(|c| (*c).into())
                .collect(),
            choice: voie
                .rank_five_choice()
                .iter()
                .map(|c| (*c).into())
                .collect(),
            progression: character.bought_in(voie, rank).map(count),
        })
        .collect()
}

impl From<co_kernel::Category> for Category {
    fn from(category: co_kernel::Category) -> Self {
        match category {
            co_kernel::Category::Incomplete => Self::Incomplete,
            co_kernel::Category::Deviation(severity) => Self::Deviation {
                severity: severity.into(),
            },
            co_kernel::Category::Uncomputable => Self::Uncomputable,
            co_kernel::Category::Notice => Self::Notice,
        }
    }
}

impl From<co_kernel::Severity> for Severity {
    fn from(severity: co_kernel::Severity) -> Self {
        match severity {
            co_kernel::Severity::Recommended => Self::Recommended,
            co_kernel::Severity::Standard => Self::Standard,
            co_kernel::Severity::Absolute => Self::Absolute,
        }
    }
}
