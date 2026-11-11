//! What a capacité does, as short texts in our own words, each saying
//! whether the engine computes it (ADR KER-ABILITY-EFFECTS). Which
//! capacités have which effects stays with the engines.

/// Whether the engine computes an effect, or only describes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Application {
    /// The engine counts it in a total of the sheet: a characteristic, a
    /// value (PV, DEF, Init, an attack, DM, dice, critical range, RD, PC,
    /// PM), an option chosen once included. The player need not read it.
    Computed,
    /// No total includes it: a bonus to some tests, a dé bonus, a mastery, a
    /// limit the engine checks, a situational modifier, or what the engine
    /// does not compute. The player reads it to play.
    Described,
}

/// The bonus a described effect gives, computed by the engine, so the
/// sheet can show its amount in place of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bonus {
    /// The capacité's test bonus numbered `n` from 1, or all of them for 0
    /// (one per characteristic chosen, such as Tatouages).
    Test(u8),
    /// The capacité's dé bonus to tests.
    Die,
}

/// What a formula of an effect counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Measure {
    /// The rank reached in the capacité's voie.
    Rank,
    /// The value of a characteristic.
    Characteristic(crate::Characteristic),
    /// The character's level.
    Level,
    /// The voies of a group (a profil, by its identifier) in which the
    /// character holds `rank`.
    VoiesAt {
        /// The group's identifier, such as `cof.profil.forgesort`.
        group: &'static str,
        /// The rank held.
        rank: u8,
    },
}

/// A formula of an effect, without dice: a measure, times a factor, plus
/// an amount, at most a maximum, written with a unit; and the formula as
/// the text shows it (`CHA min`, `2 m par rang`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Term {
    measure: Measure,
    times: i32,
    plus: i32,
    max: Option<i32>,
    unit: &'static str,
    signed: bool,
    formula: &'static str,
}

impl Term {
    /// The measure itself, written `formula`.
    #[must_use]
    pub const fn new(measure: Measure, formula: &'static str) -> Self {
        Self {
            measure,
            times: 1,
            plus: 0,
            max: None,
            unit: "",
            signed: false,
            formula,
        }
    }

    /// The same term times `times`.
    #[must_use]
    pub const fn times(mut self, times: i32) -> Self {
        self.times = times;
        self
    }

    /// The same term plus `plus`.
    #[must_use]
    pub const fn plus(mut self, plus: i32) -> Self {
        self.plus = plus;
        self
    }

    /// The same term, at most `max`.
    #[must_use]
    pub const fn at_most(mut self, max: i32) -> Self {
        self.max = Some(max);
        self
    }

    /// The same term written with `unit` after its value (` min`).
    #[must_use]
    pub const fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    /// The same term written with its sign (`+2`).
    #[must_use]
    pub const fn signed(mut self) -> Self {
        self.signed = true;
        self
    }

    /// What the term counts.
    #[must_use]
    pub const fn measure(self) -> Measure {
        self.measure
    }

    /// The formula as the text shows it.
    #[must_use]
    pub const fn formula(self) -> &'static str {
        self.formula
    }

    /// The value written for `quantity`, its unit included.
    #[must_use]
    pub fn write(self, quantity: i32) -> String {
        let value = quantity * self.times + self.plus;
        let value = self.max.map_or(value, |max| value.min(max));
        if self.signed && value >= 0 {
            format!("+{value}{}", self.unit)
        } else {
            format!("{value}{}", self.unit)
        }
    }
}

/// A part of an effect written for a character: text, or a value computed
/// from a formula, with the formula.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectPart {
    /// Text, as written.
    Text(String),
    /// A value computed from a formula.
    Value {
        /// The value, its unit included (`3 min`).
        value: String,
        /// The formula (`CHA min`).
        formula: &'static str,
    },
}

/// One effect of a capacité: a short French text, never the book's, and
/// whether the engine computes it. Its text may hold a `{}` slot per
/// [`Term`], which the sheet fills with the value for the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbilityEffect {
    text: &'static str,
    application: Application,
    bonus: Option<Bonus>,
    choice: bool,
    terms: &'static [Term],
}

impl AbilityEffect {
    /// An effect the engine computes.
    #[must_use]
    pub const fn computed(text: &'static str) -> Self {
        Self {
            text,
            application: Application::Computed,
            bonus: None,
            choice: false,
            terms: &[],
        }
    }

    /// A computed effect that depends on a choice recorded for its capacité
    /// (the characteristic it raises), which the sheet shows beside it.
    #[must_use]
    pub const fn chosen(text: &'static str) -> Self {
        Self {
            text,
            application: Application::Computed,
            bonus: None,
            choice: true,
            terms: &[],
        }
    }

    /// An effect the engine only describes.
    #[must_use]
    pub const fn described(text: &'static str) -> Self {
        Self {
            text,
            application: Application::Described,
            bonus: None,
            choice: false,
            terms: &[],
        }
    }

    /// A described effect giving `bonus`, which the engine computes.
    #[must_use]
    pub const fn giving(bonus: Bonus, text: &'static str) -> Self {
        Self {
            text,
            application: Application::Described,
            bonus: Some(bonus),
            choice: false,
            terms: &[],
        }
    }

    /// The same effect, its text's `{}` slots filled by `terms` in order.
    #[must_use]
    pub const fn with(mut self, terms: &'static [Term]) -> Self {
        self.terms = terms;
        self
    }

    /// The text, its formulas written as formulas, such as `pendant CHA
    /// min`.
    #[must_use]
    pub fn text(self) -> String {
        self.parts(|_| None)
            .into_iter()
            .map(|part| match part {
                EffectPart::Text(text) => text,
                EffectPart::Value { formula, .. } => formula.to_owned(),
            })
            .collect()
    }

    /// The text as written, with a `{}` slot per term.
    #[must_use]
    pub const fn template(self) -> &'static str {
        self.text
    }

    /// The terms filling the slots, in order.
    #[must_use]
    pub const fn terms(self) -> &'static [Term] {
        self.terms
    }

    /// The effect written for a character whose quantities `value_of`
    /// gives: each slot a value, or its formula when `value_of` has none.
    #[must_use]
    pub fn parts(self, value_of: impl Fn(Measure) -> Option<i32>) -> Vec<EffectPart> {
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut terms = self.terms.iter();
        let mut pieces = self.text.split("{}").peekable();
        while let Some(piece) = pieces.next() {
            text.push_str(piece);
            if pieces.peek().is_none() {
                break;
            }
            let Some(term) = terms.next() else {
                text.push_str("{}");
                continue;
            };
            match value_of(term.measure) {
                Some(quantity) => {
                    if !text.is_empty() {
                        parts.push(EffectPart::Text(std::mem::take(&mut text)));
                    }
                    parts.push(EffectPart::Value {
                        value: term.write(quantity),
                        formula: term.formula,
                    });
                }
                None => text.push_str(term.formula),
            }
        }
        if !text.is_empty() {
            parts.push(EffectPart::Text(text));
        }
        parts
    }

    /// Whether the engine computes it.
    #[must_use]
    pub const fn application(self) -> Application {
        self.application
    }

    /// The bonus it gives, when the engine computes its amount.
    #[must_use]
    pub const fn bonus(self) -> Option<Bonus> {
        self.bonus
    }

    /// Whether it depends on a choice recorded for its capacité, shown
    /// beside its text.
    #[must_use]
    pub const fn choice(self) -> bool {
        self.choice
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Characteristic;

    #[test]
    fn formulas_fill_their_slots_with_values_or_formulas() {
        const TERMS: [Term; 2] = [
            Term::new(Measure::Rank, "2 m par rang").times(2).unit(" m"),
            Term::new(Measure::Characteristic(Characteristic::Cha), "CHA min").unit(" min"),
        ];
        let effect = AbilityEffect::described("illusion ({} de côté) pendant {}").with(&TERMS);
        assert_eq!(
            effect.text(),
            "illusion (2 m par rang de côté) pendant CHA min"
        );
        let parts = effect.parts(|quantity| match quantity {
            Measure::Rank => Some(2),
            _ => None,
        });
        assert_eq!(
            parts,
            [
                EffectPart::Text("illusion (".into()),
                EffectPart::Value {
                    value: "4 m".into(),
                    formula: "2 m par rang"
                },
                EffectPart::Text(" de côté) pendant CHA min".into()),
            ]
        );
        let capped =
            Term::new(Measure::Characteristic(Characteristic::Int), "INT (max. 4)").at_most(4);
        assert_eq!(capped.write(6), "4");
        let bonus = Term::new(Measure::Rank, "+1/rang 5").signed().plus(5);
        assert_eq!(bonus.write(2), "+7");
        assert_eq!(effect.template(), "illusion ({} de côté) pendant {}");
    }

    #[test]
    fn effects_say_whether_they_are_computed() {
        let computed = AbilityEffect::computed("+1 DEF");
        assert_eq!(
            (computed.text().as_str(), computed.application()),
            ("+1 DEF", Application::Computed)
        );
        let described = AbilityEffect::described("vision dans le noir 30 m");
        assert_eq!(
            (described.application(), described.bonus()),
            (Application::Described, None)
        );
        let giving = AbilityEffect::giving(Bonus::Test(1), "rang + 2 escalade");
        assert_eq!(
            (giving.application(), giving.bonus()),
            (Application::Described, Some(Bonus::Test(1)))
        );
        let chosen = AbilityEffect::chosen("PER ou VOL +1");
        assert_eq!(chosen.terms(), []);
        assert_eq!(
            (chosen.application(), chosen.choice(), described.choice()),
            (Application::Computed, true, false)
        );
    }
}
