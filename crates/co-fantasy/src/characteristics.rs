//! Starting characteristics: a value series assigned freely, then the peuple
//! modifiers (COF.CRE.CARAC-SERIES, COF.CRE.CARAC-PEUPLE,
//! COF.CRE.CARAC-BOUNDS, D-15).

use co_kernel::{Characteristic, DerivedValue, Severity, Source, Subject, ValueId, Violation};

use crate::{
    Character, Peuple, Profil,
    rules::{
        CARAC_BOUNDS, CARAC_KEY, CARAC_MAGIC, CARAC_MECHANIC, CARAC_PEUPLE, CARAC_SERIES,
        PEUPLE_BONUS, PEUPLE_MALUS, SERIES,
    },
};

/// Breakdown source of the series value assigned by the player.
pub const SERIES_VALUE: ValueId = ValueId::from_static("cof.series-value");

/// One of the three value series to assign to the seven characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Series {
    /// Polyvalent.
    Polyvalent,
    /// Expert.
    Expert,
    /// Spécialiste.
    Specialiste,
}

impl Series {
    /// Every series, in declaration order.
    pub const ALL: [Self; 3] = [Self::Polyvalent, Self::Expert, Self::Specialiste];

    /// Returns the identifier of the series, used by the choices document.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Polyvalent => "cof.series.polyvalent",
            Self::Expert => "cof.series.expert",
            Self::Specialiste => "cof.series.specialiste",
        })
    }

    /// Returns the series whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|series| series.id().as_str() == id)
    }

    /// The French display name of the series.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Polyvalent => "Polyvalent",
            Self::Expert => "Expert",
            Self::Specialiste => "Spécialiste",
        }
    }

    /// The seven values of the series, from the highest.
    #[must_use]
    pub const fn values(self) -> [i32; 7] {
        match self {
            Self::Polyvalent => [2, 2, 2, 1, 1, 0, -1],
            Self::Expert => [3, 2, 1, 1, 0, 0, -1],
            Self::Specialiste => [4, 2, 1, 0, 0, -1, -1],
        }
    }
}

/// Lowest starting value.
const MIN: i32 = -2;
/// Highest starting value of a human.
const HUMAN_MAX: i32 = 4;
/// Highest starting value of a non-human.
const MAX: i32 = 5;
/// Lowest advised value of a spellcaster's magic characteristic.
const MAGIC_MIN: i32 = 1;

/// The characteristic the peuple lowers: the one chosen, or the only one the
/// peuple offers.
fn applied_malus(character: &Character, peuple: Peuple) -> Option<Characteristic> {
    character.peuple_malus().or(match peuple.malus_options() {
        [only] => Some(*only),
        _ => None,
    })
}

/// The value of `c` before adjustments: the assigned series value (0 while
/// unassigned), then the peuple modifiers.
pub(crate) fn characteristic(character: &Character, c: Characteristic) -> DerivedValue {
    let mut value = DerivedValue::new().with(
        Source::Value(SERIES_VALUE),
        character.assigned_value(c).unwrap_or(0),
    );
    if let Some(peuple) = character.peuple() {
        if character.peuple_bonus() == Some(c) {
            value = value.with(Source::Value(peuple.id()), 1);
        }
        if applied_malus(character, peuple) == Some(c) {
            value = value.with(Source::Value(peuple.id()), -1);
        }
    }
    value
}

/// Checks the series, its assignment, the peuple modifiers and the starting
/// values.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    check_series(character, violations);
    check_key(character, violations);
    check_peuple(character, violations);
    check_values(character, violations);
}

/// The assigned values, in [`Characteristic::ALL`] order.
fn assigned(character: &Character) -> impl Iterator<Item = (Characteristic, i32)> + '_ {
    Characteristic::ALL
        .into_iter()
        .filter_map(|c| character.assigned_value(c).map(|value| (c, value)))
}

/// Whether every assigned value can be drawn from the series, each value of
/// the series at most once.
fn is_drawn_from(character: &Character, series: Series) -> bool {
    let mut remaining = series.values().to_vec();
    assigned(character).all(|(_, value)| {
        remaining
            .iter()
            .position(|r| *r == value)
            .map(|index| remaining.swap_remove(index))
            .is_some()
    })
}

/// COF.CRE.CARAC-SERIES: a series, all seven values assigned from it.
fn check_series(character: &Character, violations: &mut Vec<Violation>) {
    let subject = || Subject::Value(SERIES);
    match character.series() {
        None => violations.push(Violation::incomplete(
            CARAC_SERIES,
            CARAC_MECHANIC,
            subject(),
        )),
        Some(series) if !is_drawn_from(character, series) => violations.push(Violation::deviation(
            CARAC_SERIES,
            CARAC_MECHANIC,
            subject(),
            Severity::Standard,
        )),
        Some(_) => {}
    }
    let count = assigned(character).fold(0_i32, |count, _| count + 1);
    if count < 7 {
        violations.push(
            Violation::incomplete(CARAC_SERIES, CARAC_MECHANIC, subject())
                .with_parameter("assigned", count),
        );
    }
}

/// COF.CRE.CARAC-KEY (recommended): no other assigned value above a key
/// characteristic's. In a slot offering alternatives, the highest is the key.
fn check_key(character: &Character, violations: &mut Vec<Violation>) {
    let Some(profil) = character.profil() else {
        return;
    };
    let keys: Vec<(Characteristic, i32)> = profil
        .key_characteristics()
        .iter()
        .filter_map(|slot| {
            slot.iter()
                .filter_map(|c| character.assigned_value(*c).map(|value| (*c, value)))
                .max_by_key(|(_, value)| *value)
        })
        .collect();
    let lowest_key = keys.iter().map(|(_, value)| *value).min();
    let highest_other = assigned(character)
        .filter(|(c, _)| keys.iter().all(|(key, _)| key != c))
        .map(|(_, value)| value)
        .max();
    if let (Some(lowest_key), Some(highest_other)) = (lowest_key, highest_other)
        && highest_other > lowest_key
    {
        violations.push(
            Violation::deviation(
                CARAC_KEY,
                CARAC_MECHANIC,
                Subject::Character,
                Severity::Recommended,
            )
            .with_parameter("lowest_key", lowest_key)
            .with_parameter("highest_other", highest_other),
        );
    }
}

/// Whether a humain may raise `c`: its assigned value is one of the two
/// lowest, ties included (D-15). Undecidable, hence accepted, until all
/// seven values are assigned.
fn is_among_two_lowest(character: &Character, c: Characteristic) -> bool {
    let mut values: Vec<i32> = assigned(character).map(|(_, value)| value).collect();
    let Some(value) = character.assigned_value(c) else {
        return true;
    };
    if values.len() < 7 {
        return true;
    }
    values.sort_unstable();
    values.dedup();
    values
        .get(1)
        .or(values.first())
        .is_none_or(|threshold| value <= *threshold)
}

/// COF.CRE.CARAC-PEUPLE: the bonus and the malus the peuple offers.
fn check_peuple(character: &Character, violations: &mut Vec<Violation>) {
    let Some(peuple) = character.peuple() else {
        return;
    };
    let bonus_subject = || Subject::Value(PEUPLE_BONUS);
    match character.peuple_bonus() {
        None => violations.push(Violation::incomplete(
            CARAC_PEUPLE,
            CARAC_MECHANIC,
            bonus_subject(),
        )),
        Some(c)
            if !peuple.bonus_options().contains(&c)
                || (peuple.is_human() && !is_among_two_lowest(character, c)) =>
        {
            violations.push(Violation::deviation(
                CARAC_PEUPLE,
                CARAC_MECHANIC,
                bonus_subject(),
                Severity::Standard,
            ));
        }
        Some(_) => {}
    }

    let malus_subject = || Subject::Value(PEUPLE_MALUS);
    let options = peuple.malus_options();
    match character.peuple_malus() {
        None if options.len() > 1 => violations.push(Violation::incomplete(
            CARAC_PEUPLE,
            CARAC_MECHANIC,
            malus_subject(),
        )),
        Some(c) if !options.contains(&c) => violations.push(Violation::deviation(
            CARAC_PEUPLE,
            CARAC_MECHANIC,
            malus_subject(),
            Severity::Standard,
        )),
        _ => {}
    }
}

/// COF.CRE.CARAC-BOUNDS and COF.CRE.CARAC-MAGIC (recommended), on the
/// adjusted starting values.
fn check_values(character: &Character, violations: &mut Vec<Violation>) {
    let values = character.characteristics();
    let max = if character.peuple().is_some_and(Peuple::is_human) {
        HUMAN_MAX
    } else {
        MAX
    };
    for (c, value) in values.iter() {
        let value = value.get();
        if !(MIN..=max).contains(&value) {
            violations.push(
                Violation::deviation(
                    CARAC_BOUNDS,
                    CARAC_MECHANIC,
                    Subject::Characteristic(c),
                    Severity::Standard,
                )
                .with_parameter("value", value)
                .with_parameter("min", MIN)
                .with_parameter("max", max),
            );
        }
    }

    if let Some(magic) = character.profil().and_then(Profil::magic_characteristic)
        && character.assigned_value(magic).is_some()
        && values[magic].get() < MAGIC_MIN
    {
        violations.push(
            Violation::deviation(
                CARAC_MAGIC,
                CARAC_MECHANIC,
                Subject::Characteristic(magic),
                Severity::Recommended,
            )
            .with_parameter("value", values[magic].get())
            .with_parameter("min", MAGIC_MIN),
        );
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Category};

    use super::*;
    use crate::Target;

    fn carac_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
    }

    fn rules(violations: &[Violation]) -> Vec<(&str, Category)> {
        violations
            .iter()
            .map(|v| (v.rule().as_str(), v.category()))
            .collect()
    }

    fn assign(character: Character, values: [(Characteristic, i32); 7]) -> Character {
        values
            .into_iter()
            .fold(character, |c, (characteristic, value)| {
                c.with_series_value(characteristic, value)
            })
    }

    /// A nain guerrier, polyvalent series on its key characteristics.
    fn nain() -> Character {
        let character = Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(Peuple::Nain)
            .with_series(Series::Polyvalent)
            .with_peuple_bonus(Characteristic::Con);
        assign(
            character,
            [
                (Characteristic::For, 2),
                (Characteristic::Con, 2),
                (Characteristic::Agi, 2),
                (Characteristic::Vol, 1),
                (Characteristic::Per, 1),
                (Characteristic::Int, 0),
                (Characteristic::Cha, -1),
            ],
        )
    }

    /// A humain magicien, spécialiste series.
    fn humain() -> Character {
        let character = Character::new()
            .with_profil(Profil::Magicien)
            .with_peuple(Peuple::Humain)
            .with_series(Series::Specialiste);
        assign(
            character,
            [
                (Characteristic::Int, 4),
                (Characteristic::Vol, 2),
                (Characteristic::Agi, 1),
                (Characteristic::Con, 0),
                (Characteristic::Per, 0),
                (Characteristic::For, -1),
                (Characteristic::Cha, -1),
            ],
        )
    }

    #[test]
    fn series_and_peuple_within_the_rules_are_clean() {
        assert_eq!(carac_violations(&nain()), []);
        // The humain may raise a 0 as well as a -1 (D-15).
        assert_eq!(
            carac_violations(&humain().with_peuple_bonus(Characteristic::Per)),
            []
        );
    }

    #[test]
    fn peuple_modifiers_apply_on_the_series() {
        let character = nain();
        let con = characteristic(&character, Characteristic::Con);
        let breakdown: Vec<_> = con
            .contributions()
            .iter()
            .map(|c| (c.source().clone(), c.amount()))
            .collect();
        assert_eq!(
            breakdown,
            [
                (Source::Value(SERIES_VALUE), 2),
                (Source::Value(Peuple::Nain.id()), 1),
            ]
        );
        // The nain's only malus applies without a choice.
        assert_eq!(characteristic(&character, Characteristic::Agi).total(), 1);
    }

    #[test]
    fn missing_choices_are_incomplete() {
        let character = Character::new()
            .with_peuple(Peuple::DemiOrc)
            .with_series_value(Characteristic::For, 3);
        assert_eq!(
            rules(&carac_violations(&character)),
            [
                ("COF.CRE.CARAC-SERIES", Category::Incomplete),
                ("COF.CRE.CARAC-SERIES", Category::Incomplete),
                ("COF.CRE.CARAC-PEUPLE", Category::Incomplete),
                ("COF.CRE.CARAC-PEUPLE", Category::Incomplete),
            ]
        );
    }

    #[test]
    fn values_outside_the_series_deviate() {
        let character = nain().with_series_value(Characteristic::Cha, 2);
        assert_eq!(
            rules(&carac_violations(&character)),
            [(
                "COF.CRE.CARAC-SERIES",
                Category::Deviation(Severity::Standard)
            )]
        );
    }

    #[test]
    fn key_characteristics_below_another_deviate_as_recommended() {
        let character = nain()
            .with_series_value(Characteristic::Agi, 0)
            .with_series_value(Characteristic::Int, 2);
        assert_eq!(
            rules(&carac_violations(&character)),
            [(
                "COF.CRE.CARAC-KEY",
                Category::Deviation(Severity::Recommended)
            )]
        );
    }

    #[test]
    fn druide_third_key_is_the_higher_of_con_and_agi() {
        let druide = |con, agi| {
            let character = Character::new()
                .with_profil(Profil::Druide)
                .with_series(Series::Expert);
            assign(
                character,
                [
                    (Characteristic::Per, 3),
                    (Characteristic::Vol, 2),
                    (Characteristic::Con, con),
                    (Characteristic::Agi, agi),
                    (Characteristic::For, 1),
                    (Characteristic::Int, 0),
                    (Characteristic::Cha, -1),
                ],
            )
        };
        let key_violations = |c: &Character| {
            carac_violations(c)
                .into_iter()
                .filter(|v| v.rule() == &CARAC_KEY)
                .count()
        };
        assert_eq!(key_violations(&druide(1, 0)), 0);
        assert_eq!(key_violations(&druide(0, 1)), 0);
    }

    #[test]
    fn humain_bonus_must_go_to_one_of_the_two_lowest() {
        let character = humain().with_peuple_bonus(Characteristic::Agi);
        assert_eq!(
            rules(&carac_violations(&character)),
            [(
                "COF.CRE.CARAC-PEUPLE",
                Category::Deviation(Severity::Standard)
            )]
        );
    }

    #[test]
    fn humain_bonus_waits_for_the_full_assignment() {
        let peuple_violations = |c: &Character| {
            carac_violations(c)
                .into_iter()
                .filter(|v| v.rule() == &CARAC_PEUPLE)
                .count()
        };
        let partial = humain()
            .without_series_value(Characteristic::Cha)
            .with_peuple_bonus(Characteristic::Int);
        assert_eq!(peuple_violations(&partial), 0);
        let unassigned = humain()
            .without_series_value(Characteristic::Cha)
            .with_peuple_bonus(Characteristic::Cha);
        assert_eq!(peuple_violations(&unassigned), 0);
    }

    #[test]
    fn modifiers_the_peuple_does_not_offer_deviate() {
        let character = nain()
            .with_peuple_bonus(Characteristic::For)
            .with_peuple_malus(Characteristic::Cha);
        let subjects: Vec<_> = carac_violations(&character)
            .into_iter()
            .map(|v| v.subject().clone())
            .collect();
        assert_eq!(
            subjects,
            [Subject::Value(PEUPLE_BONUS), Subject::Value(PEUPLE_MALUS)]
        );
        // The chosen malus replaces the peuple's own.
        assert_eq!(characteristic(&character, Characteristic::Cha).total(), -2);
        assert_eq!(characteristic(&character, Characteristic::Agi).total(), 2);
    }

    #[test]
    fn values_outside_the_bounds_deviate() {
        let character = humain()
            .with_peuple_bonus(Characteristic::Cha)
            .with_peuple(Peuple::ElfeHaut);
        assert_eq!(carac_violations(&character), []);
        let human = humain()
            .with_peuple_bonus(Characteristic::Cha)
            .with_adjustment(
                Target::Characteristic(Characteristic::Int),
                Adjustment::new(1, Author::new("gm"), "test reason"),
            );
        let violations = carac_violations(&human);
        assert_eq!(
            rules(&violations),
            [(
                "COF.CRE.CARAC-BOUNDS",
                Category::Deviation(Severity::Standard)
            )]
        );
        assert_eq!(
            violations[0].subject(),
            &Subject::Characteristic(Characteristic::Int)
        );
    }

    #[test]
    fn low_magic_characteristic_deviates_as_recommended() {
        let character = humain()
            .with_peuple_bonus(Characteristic::Per)
            .with_series_value(Characteristic::Int, -1)
            .with_series_value(Characteristic::Cha, 4);
        let violations = carac_violations(&character);
        assert!(violations.iter().any(|v| v.rule() == &CARAC_MAGIC
            && v.category() == Category::Deviation(Severity::Recommended)
            && v.subject() == &Subject::Characteristic(Characteristic::Int)));
    }
}
