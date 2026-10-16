//! The evaluation of a choices document: every value with its breakdown,
//! the violations placed in their creation step, and the status of each
//! step. Everything is computed by the engines; this module only maps their
//! results to plain data, with French labels for display.

use std::collections::BTreeSet;

use co_kernel::{Contribution, DerivedValue, ParameterValue, Source, ViolationReport};
use palimpseste_document::{
    Characteristic, ContemporainDocument, Document, Error, FantasyDocument, ItemEntry, Subject,
    WaiverEntry,
};
use serde::Serialize;
use tsify::Tsify;

use crate::catalog::{Named, attack_mode};

/// A character evaluated by its engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    /// The creation steps, in the book order, with what is left to settle.
    pub steps: Vec<StepStatus>,
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
pub struct ContributionLine {
    /// Where the term comes from.
    pub source: SourceKind,
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
    /// The identifier of the creation step where it is settled.
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
}

/// Evaluates `document` with its engine.
///
/// # Errors
///
/// Returns why the document cannot be read.
pub fn evaluate(document: &Document) -> Result<Sheet, Error> {
    match document {
        Document::Contemporain(document) => contemporain(document),
        Document::Fantasy(document) => fantasy(document),
    }
}

/// The parts every engine shares, from its report and labels.
struct Common<'a> {
    report: &'a ViolationReport,
    steps: Vec<(&'static str, &'static str)>,
    step_of: &'a dyn Fn(&co_kernel::Violation) -> &'static str,
    label: &'a dyn Fn(&str) -> Option<String>,
}

impl Common<'_> {
    fn sheet(
        &self,
        characteristics: impl Fn(co_kernel::Characteristic) -> DerivedValue,
        values: Vec<ValueLine>,
        details: Details,
    ) -> Sheet {
        let violations: Vec<ReportedViolation> = self
            .report
            .entries()
            .iter()
            .map(|entry| self.violation(entry))
            .collect();
        let steps = self
            .steps
            .iter()
            .map(|(id, name)| {
                let mut status = StepStatus {
                    id: (*id).to_owned(),
                    name: (*name).to_owned(),
                    incomplete: 0,
                    deviations: 0,
                    notices: 0,
                };
                for violation in violations.iter().filter(|v| v.step == *id && !v.waived) {
                    match violation.category {
                        Category::Incomplete | Category::Uncomputable => status.incomplete += 1,
                        Category::Deviation { .. } => status.deviations += 1,
                        Category::Notice => status.notices += 1,
                    }
                }
                status
            })
            .collect();
        Sheet {
            steps,
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
        }
    }

    fn violation(&self, entry: &co_kernel::ReportEntry) -> ReportedViolation {
        let violation = entry.violation();
        let subject_label = match violation.subject() {
            co_kernel::Subject::Character => None,
            co_kernel::Subject::Characteristic(c) => {
                Some(abbreviation(Characteristic::from(*c)).to_owned())
            }
            co_kernel::Subject::Voie(id) => (self.label)(id.as_str()),
            co_kernel::Subject::Ability(id) => (self.label)(id.as_str()),
            co_kernel::Subject::Value(id) => (self.label)(id.as_str()),
        };
        ReportedViolation {
            rule: violation.rule().as_str().to_owned(),
            mechanic: violation.mechanic().as_str().to_owned(),
            subject: violation.subject().into(),
            subject_label,
            category: violation.category().into(),
            waived: entry.is_waived(),
            step: (self.step_of)(violation).to_owned(),
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
        let (source, label) = match contribution.source() {
            Source::Characteristic(c) => (
                SourceKind::Characteristic,
                abbreviation(Characteristic::from(*c)).to_owned(),
            ),
            Source::Ability(id) => (
                SourceKind::Ability,
                (self.label)(id.as_str()).unwrap_or_else(|| id.as_str().to_owned()),
            ),
            Source::Rule(rule) => (SourceKind::Rule, rule_label(rule.as_str()).to_owned()),
            Source::Value(id) => (
                SourceKind::Value,
                (self.label)(id.as_str()).unwrap_or_else(|| id.as_str().to_owned()),
            ),
            Source::Adjustment { author, reason } => (
                SourceKind::Adjustment,
                format!("Ajustement de {} : {reason}", author.as_str()),
            ),
        };
        ContributionLine {
            source,
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

/// The French label of a term a rule's formula sets itself.
fn rule_label(rule: &str) -> &'static str {
    match rule {
        "COF.CRE.LEVEL1" => "Niveau",
        "COC.CRE.AGE-BRACKET" => "Tranche d'âge",
        _ => "Base",
    }
}

/// The abbreviation the books use for `characteristic`.
pub(crate) const fn abbreviation(characteristic: Characteristic) -> &'static str {
    match characteristic {
        Characteristic::Agi => "AGI",
        Characteristic::Con => "CON",
        Characteristic::For => "FOR",
        Characteristic::Per => "PER",
        Characteristic::Cha => "CHA",
        Characteristic::Int => "INT",
        Characteristic::Vol => "VOL",
    }
}

/// The label of a capacité identifier, `<voie>.<rank>`.
fn ability_label(id: &str, voie: impl Fn(&str) -> Option<&'static str>) -> Option<String> {
    let (voie_id, rank) = id.rsplit_once('.')?;
    Some(format!("{}, rang {rank}", voie(voie_id)?))
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

fn fantasy_value_label(value: co_fantasy::Value) -> String {
    use co_fantasy::{AttackMode, Value};
    match value {
        Value::VigourPoints => "Points de vigueur (PV)".to_owned(),
        Value::RecoveryDice => "Dés de récupération (DR)".to_owned(),
        Value::LuckPoints => "Points de chance (PC)".to_owned(),
        Value::ManaPoints => "Points de mana (PM)".to_owned(),
        Value::Initiative => "Initiative".to_owned(),
        Value::Defence => "Défense (DEF)".to_owned(),
        Value::Attack(AttackMode::Contact) => "Attaque au contact".to_owned(),
        Value::Attack(AttackMode::Ranged) => "Attaque à distance".to_owned(),
        Value::Attack(AttackMode::Magic) => "Attaque magique".to_owned(),
        Value::Money => "Argent (pc)".to_owned(),
        Value::Damage(weapon, weapon_use) => {
            format!(
                "DM {} ({})",
                weapon.name(),
                weapon_use.name().to_lowercase()
            )
        }
        Value::CriticalRange(weapon) => format!("Zone de critique {}", weapon.name()),
    }
}

/// The French label of a COF2 identifier: content, a choice or a value.
fn fantasy_label(id: &str) -> Option<String> {
    use co_fantasy::{Armour, Family, Peuple, Profil, Series, Shield, Value, Voie, Weapon};
    let fixed = match id {
        "cof.profil" => Some("Profil"),
        "cof.peuple" => Some("Peuple"),
        "cof.series" => Some("Série de valeurs"),
        "cof.series-value" => Some("Valeur de série"),
        "cof.peuple-bonus" => Some("Bonus de peuple"),
        "cof.peuple-malus" => Some("Malus de peuple"),
        "cof.profil-voies" => Some("Voies de profil"),
        "cof.peuple-voie" => Some("Voie de peuple"),
        "cof.armour" => Some("Armure"),
        "cof.shield" => Some("Bouclier"),
        "cof.purse" => Some("Bourse"),
        _ => None,
    };
    let content = || {
        Profil::from_id(id)
            .map(Profil::name)
            .or_else(|| Peuple::from_id(id).map(Peuple::name))
            .or_else(|| Series::from_id(id).map(Series::name))
            .or_else(|| {
                Family::ALL
                    .into_iter()
                    .find(|family| family.id().as_str() == id)
                    .map(Family::name)
            })
            .or_else(|| Armour::from_id(id).map(Armour::name))
            .or_else(|| Shield::from_id(id).map(Shield::name))
            .or_else(|| Weapon::from_id(id).map(Weapon::name))
            .or_else(|| Voie::from_id(id).map(Voie::name))
    };
    fixed
        .or_else(content)
        .map(str::to_owned)
        .or_else(|| Value::from_id(id).map(fantasy_value_label))
        .or_else(|| ability_label(id, |voie| Voie::from_id(voie).map(Voie::name)))
}

fn fantasy(document: &FantasyDocument) -> Result<Sheet, Error> {
    use co_fantasy::{AttackMode, Family, Money, RecoveryDie, Step, Value, Weapon, WeaponUse};
    let character = document.to_character()?;
    let report = character.report();
    let step_of = |violation: &co_kernel::Violation| Step::of(violation).id();
    let common = Common {
        report: &report,
        steps: Step::ALL
            .iter()
            .map(|step| (step.id(), step.name()))
            .collect(),
        step_of: &step_of,
        label: &fantasy_label,
    };
    let line = |value: Value| {
        common.line(
            value.id().as_str().to_owned(),
            fantasy_value_label(value),
            character.value(value).as_ref(),
        )
    };
    let mut values: Vec<ValueLine> = [
        Value::VigourPoints,
        Value::RecoveryDice,
        Value::LuckPoints,
        Value::ManaPoints,
        Value::Initiative,
        Value::Defence,
        Value::Attack(AttackMode::Contact),
        Value::Attack(AttackMode::Ranged),
        Value::Attack(AttackMode::Magic),
        Value::Money,
    ]
    .into_iter()
    .map(line)
    .collect();
    let recovery_die = character.recovery_die().map(|die| {
        match die {
            RecoveryDie::D6 => "d6",
            RecoveryDie::D8 => "d8",
            RecoveryDie::D10 => "d10",
        }
        .to_owned()
    });
    if let Some(dice) = values.get_mut(1) {
        dice.dice.clone_from(&recovery_die);
    }
    for weapon in carried(&document.items, Weapon::from_id) {
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
        money: character
            .value(Value::Money)
            .map(|money| Money::pc(money.total()).to_string()),
    });
    Ok(common.sheet(|c| character.characteristic(c), values, details))
}

fn contemporain_value_label(value: co_contemporain::Value) -> String {
    use co_contemporain::Value;
    match value {
        Value::Initiative => "Initiative".to_owned(),
        Value::Defence => "Défense (DEF)".to_owned(),
        Value::Attack(mode) => attack_mode(mode).name,
        Value::ContactDamageBonus => "Bonus aux DM au contact".to_owned(),
        Value::HealingCapacity => "Capacité de guérison (CG)".to_owned(),
        Value::HealthScale => "Échelle de santé".to_owned(),
        Value::CaracPoints => "Points de caractéristique".to_owned(),
        Value::CapacityPoints => "Points de capacité".to_owned(),
        Value::MaxVoies => "Voies au maximum".to_owned(),
        Value::MaxRank => "Rang maximum".to_owned(),
        Value::Damage(weapon) => format!("DM {}", weapon.name()),
        Value::CriticalBonus(weapon) => format!("Bonus critique {}", weapon.name()),
        Value::CriticalRange(weapon) => format!("Zone de critique {}", weapon.name()),
        Value::DamageReduction => "Réduction des dommages (RD)".to_owned(),
        Value::AgiTestModifier => "Modificateur aux tests d'AGI".to_owned(),
    }
}

/// The French label of a COC2 identifier: content, a choice or a value.
fn contemporain_label(id: &str) -> Option<String> {
    use co_contemporain::{Domaine, Protection, Trait, Value, Voie, Weapon};
    let fixed = match id {
        "coc.age" => Some("Âge"),
        "coc.domaine" => Some("Domaine"),
        "coc.advantages" => Some("Avantages"),
        "coc.disadvantages" => Some("Désavantages"),
        "coc.social-class" => Some("Classe sociale"),
        "coc.allocated-points" => Some("Points attribués"),
        _ => None,
    };
    let content = || {
        Domaine::from_id(id)
            .map(Domaine::name)
            .or_else(|| Trait::from_id(id).map(Trait::name))
            .or_else(|| Weapon::from_id(id).map(Weapon::name))
            .or_else(|| Protection::from_id(id).map(Protection::name))
            .or_else(|| Voie::from_id(id).map(Voie::name))
    };
    fixed
        .or_else(content)
        .map(str::to_owned)
        .or_else(|| Value::from_id(id).map(contemporain_value_label))
        .or_else(|| ability_label(id, |voie| Voie::from_id(voie).map(Voie::name)))
}

fn contemporain(document: &ContemporainDocument) -> Result<Sheet, Error> {
    use co_contemporain::{AttackMode, Step, Value, Weapon};
    let character = document.to_character()?;
    let report = character.report();
    let step_of = |violation: &co_kernel::Violation| Step::of(violation).id();
    let common = Common {
        report: &report,
        steps: Step::ALL
            .iter()
            .map(|step| (step.id(), step.name()))
            .collect(),
        step_of: &step_of,
        label: &contemporain_label,
    };
    let line = |value: Value| {
        common.line(
            value.id().as_str().to_owned(),
            contemporain_value_label(value),
            character.value(value).as_ref(),
        )
    };
    let mut values: Vec<ValueLine> = [
        Value::CaracPoints,
        Value::CapacityPoints,
        Value::MaxVoies,
        Value::MaxRank,
        Value::Initiative,
        Value::Defence,
        Value::Attack(AttackMode::Unarmed),
        Value::Attack(AttackMode::ContactWeapon),
        Value::Attack(AttackMode::RangedWeapon),
        Value::Attack(AttackMode::HeavyWeapon),
        Value::ContactDamageBonus,
        Value::HealingCapacity,
        Value::HealthScale,
        Value::DamageReduction,
        Value::AgiTestModifier,
    ]
    .into_iter()
    .map(line)
    .collect();
    let mut weapons = vec![Weapon::MainsNues];
    weapons.extend(
        carried(&document.items, Weapon::from_id)
            .into_iter()
            .filter(|weapon| *weapon != Weapon::MainsNues),
    );
    for weapon in weapons {
        let mut damage = line(Value::Damage(weapon));
        damage.dice = Some(character.damage_dice(weapon).to_string());
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
    });
    Ok(common.sheet(|c| character.characteristic(c), values, details))
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
