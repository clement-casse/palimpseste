//! The French display names of the identifiers the engine emits as
//! violation subjects and breakdown sources (ADR ENG-CATALOGUE): content,
//! choices, values and capacités, and the terms a formula sets. Whoever
//! displays them restates no identifier.

use co_kernel::{RuleId, ValueId};

use crate::{
    ALLOCATED_POINTS, Domaine, Protection, Trait, Value, Voie, Weapon,
    rules::{
        ADVANTAGES, AGE, AGE_BRACKET, DISADVANTAGES, DOMAINE_VALUE, EXPERIENCE_EARNED,
        SOCIAL_CLASS_VALUE,
    },
};

/// The choices and sums the violations and breakdowns name, with their
/// display name.
const CHOICES: [(ValueId, &str); 7] = [
    (AGE, "Âge"),
    (DOMAINE_VALUE, "Domaine"),
    (ADVANTAGES, "Avantages"),
    (DISADVANTAGES, "Désavantages"),
    (SOCIAL_CLASS_VALUE, "Classe sociale"),
    (ALLOCATED_POINTS, "Points attribués"),
    (EXPERIENCE_EARNED, "Expérience gagnée"),
];

/// The French display name of what `id` names: a content (domaine, trait,
/// weapon, protection, voie), a choice, a value, or a capacité
/// (`<voie>.<rank>`, ADR ENG-VALUE-IDS). `None` for an identifier the
/// engine does not emit.
#[must_use]
pub fn label(id: &str) -> Option<String> {
    CHOICES
        .iter()
        .find(|(choice, _)| choice.as_str() == id)
        .map(|(_, name)| *name)
        .or_else(|| content_name(id))
        .map(str::to_owned)
        .or_else(|| Value::from_id(id).map(Value::name))
        .or_else(|| ability_name(id))
}

/// The French name of the term `rule`'s formula sets itself, when it names
/// one: the age bracket. `None` for a plain constant.
#[must_use]
pub fn term_label(rule: &RuleId) -> Option<&'static str> {
    (*rule == AGE_BRACKET).then_some("Tranche d'âge")
}

/// The display name of the content `id` names.
fn content_name(id: &str) -> Option<&'static str> {
    Domaine::from_id(id)
        .map(Domaine::name)
        .or_else(|| Trait::from_id(id).map(Trait::name))
        .or_else(|| Weapon::from_id(id).map(Weapon::name))
        .or_else(|| Protection::from_id(id).map(Protection::name))
        .or_else(|| Voie::from_id(id).map(Voie::name))
}

/// The display name of the capacité `id`, `<voie>.<rank>`.
fn ability_name(id: &str) -> Option<String> {
    let (voie, rank) = id.rsplit_once('.')?;
    let rank: u8 = rank.parse().ok()?;
    Some(format!("{}, rang {rank}", Voie::from_id(voie)?.name()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use co_kernel::{Adjustment, Author, Characteristic, Source, Subject};

    use super::*;
    use crate::{Character, Target, XpSpending, rules::INIT};

    /// Every identifier `label` looks up, content, choices and values.
    fn identifiers() -> Vec<String> {
        let content = Domaine::ALL
            .iter()
            .map(|d| d.id())
            .chain(Trait::ALL.iter().map(|t| t.id()))
            .chain(Weapon::ALL.iter().map(|w| w.id()))
            .chain(Protection::ALL.iter().map(|p| p.id()))
            .chain(CHOICES.into_iter().map(|(choice, _)| choice))
            .chain(Value::all().map(Value::id))
            .map(|id| id.as_str().to_owned());
        content
            .chain(Voie::ALL.iter().map(|v| v.id().as_str().to_owned()))
            .collect()
    }

    #[test]
    fn every_identifier_names_one_thing() {
        let ids = identifiers();
        let unique: BTreeSet<&String> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
        for id in &ids {
            assert!(label(id).is_some_and(|name| !name.is_empty()), "{id}");
        }
    }

    #[test]
    fn labels_name_choices_values_and_capacites() {
        assert_eq!(
            label("coc.allocated-points").as_deref(),
            Some("Points attribués")
        );
        assert_eq!(label("coc.domaine.sante").as_deref(), Some("Santé"));
        assert_eq!(
            label(Value::CriticalBonus(Weapon::Couteau).id().as_str()).as_deref(),
            Some("Bonus critique Couteau, poignard, baïonnette")
        );
        assert_eq!(
            label(Voie::Crime.ability(2.into()).as_str()).as_deref(),
            Some("Voie du crime, rang 2")
        );
        assert_eq!(label("coc.unknown"), None);
        assert_eq!(label("coc.unknown.1"), None);
        assert_eq!(label("coc.crime.x"), None);
        assert_eq!(term_label(&AGE_BRACKET), Some("Tranche d'âge"));
        assert_eq!(term_label(&INIT), None);
    }

    /// A character emitting many subjects and sources: every voie at ranks
    /// 1, 2 and 4, rank 5 bought with XP, every trait, weapon and
    /// protection, and adjustments.
    fn busy() -> Character {
        let adjustment = || Adjustment::new(1, Author::new("gm"), "test reason");
        let character = Character::new()
            .with_age(13)
            .with_domaine(Domaine::Marginalite)
            .with_carac_points(Characteristic::For, 9)
            .with_adjustment(Target::Value(Value::MaxRank), adjustment())
            .with_adjustment(Target::Characteristic(Characteristic::Con), adjustment());
        let character = Voie::ALL.into_iter().fold(character, |c, voie| {
            c.with_rank(voie, 1).with_rank(voie, 2).with_rank(voie, 4)
        });
        let character = Trait::ALL
            .into_iter()
            .fold(character, |c, t| c.with_trait(t, 9));
        let character = Protection::ALL
            .into_iter()
            .fold(character, Character::with_protection);
        let spending = Voie::ALL
            .into_iter()
            .fold(XpSpending::new(), |s, voie| s.with_rank(voie, 5))
            .with_trait(Trait::ClasseSuperieure, 3)
            .with_trait_given_up(Trait::Rapide);
        let character = character
            .with_xp_spending(spending)
            .with_raised_characteristic(Voie::Langues, Characteristic::For);
        Weapon::ALL
            .into_iter()
            .fold(character, |c, weapon| c.with_item(weapon, 1))
    }

    #[test]
    fn every_subject_and_source_the_engine_emits_has_a_label() {
        for character in [Character::new(), busy()] {
            let mut ids: Vec<String> = Vec::new();
            for violation in character.violations() {
                match violation.subject() {
                    Subject::Voie(id) => ids.push(id.as_str().to_owned()),
                    Subject::Ability(id) => ids.push(id.as_str().to_owned()),
                    Subject::Value(id) => ids.push(id.as_str().to_owned()),
                    Subject::Character | Subject::Characteristic(_) => {}
                }
            }
            let values = Value::all().filter_map(|value| character.value(value));
            let characteristics = Characteristic::ALL.map(|c| character.characteristic(c));
            for value in values.chain(characteristics) {
                for contribution in value.contributions() {
                    match contribution.source() {
                        Source::Ability(id) => ids.push(id.as_str().to_owned()),
                        Source::Value(id) => ids.push(id.as_str().to_owned()),
                        Source::Characteristic(_) | Source::Rule(_) | Source::Adjustment { .. } => {
                        }
                    }
                }
            }
            assert_ne!(ids, Vec::<String>::new());
            for id in ids {
                assert!(label(&id).is_some(), "{id}");
            }
        }
    }
}
