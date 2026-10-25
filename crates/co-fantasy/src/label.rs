//! The French display names of the identifiers the engine emits as
//! violation subjects and breakdown sources (ADR ENG-CATALOGUE): content,
//! choices, values and capacités, and the terms a formula sets. Whoever
//! displays them restates no identifier.

use co_kernel::{RuleId, ValueId};

use crate::{
    Armour, Family, Peuple, Profil, SERIES_VALUE, Series, Shield, Value, Voie, Weapon,
    level::LEVEL_PREFIX,
    rules::{
        ARMOUR, ATTACK, PEUPLE_BONUS, PEUPLE_MALUS, PEUPLE_VALUE, PEUPLE_VOIE, PROFIL_VALUE,
        PROFIL_VOIES, PURSE, SERIES, SHIELD,
    },
};

/// The choices the violations name, with their display name.
const CHOICES: [(ValueId, &str); 11] = [
    (PROFIL_VALUE, "Profil"),
    (PEUPLE_VALUE, "Peuple"),
    (SERIES, "Série de valeurs"),
    (SERIES_VALUE, "Valeur de série"),
    (PEUPLE_BONUS, "Bonus de peuple"),
    (PEUPLE_MALUS, "Malus de peuple"),
    (PROFIL_VOIES, "Voies de profil"),
    (PEUPLE_VOIE, "Voie de peuple"),
    (ARMOUR, "Armure"),
    (SHIELD, "Bouclier"),
    (PURSE, "Bourse"),
];

/// The French display name of what `id` names: a content (profil, peuple,
/// series, family, armour, shield, weapon, voie), a choice, a value, a
/// level-up (`cof.level.<level>`), or a capacité (`<voie>.<rank>`, ADR
/// ENG-VALUE-IDS). `None` for an identifier the engine does not emit.
#[must_use]
pub fn label(id: &str) -> Option<String> {
    CHOICES
        .iter()
        .find(|(choice, _)| choice.as_str() == id)
        .map(|(_, name)| *name)
        .or_else(|| content_name(id))
        .map(str::to_owned)
        .or_else(|| Value::from_id(id).map(Value::name))
        .or_else(|| level_name(id))
        .or_else(|| ability_name(id))
}

/// The display name of the level-up `id`, `cof.level.<level>`.
fn level_name(id: &str) -> Option<String> {
    let level: u8 = id.strip_prefix(LEVEL_PREFIX)?.parse().ok()?;
    Some(format!("Niveau {level}"))
}

/// The French name of the term `rule`'s formula sets itself, when it names
/// one: the level of the attack values. `None` for a plain constant.
#[must_use]
pub fn term_label(rule: &RuleId) -> Option<&'static str> {
    (*rule == ATTACK).then_some("Niveau")
}

/// The display name of the content `id` names.
fn content_name(id: &str) -> Option<&'static str> {
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
}

/// The display name of the capacité `id`, `<voie>.<rank>`: its name, then
/// its voie and rank, since names repeat across voies.
fn ability_name(id: &str) -> Option<String> {
    let (voie, rank) = Voie::from_ability_id(id)?;
    let place = format!("{}, rang {}", voie.name(), rank.get());
    Some(match voie.ability_name(rank) {
        Some(name) => format!("{name} ({place})"),
        None => place,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use co_kernel::{Adjustment, Author, Characteristic, Source, Subject};

    use super::*;
    use crate::{Character, LevelUp, Money, Target, WeaponUse, rules::DEF};

    /// Every identifier `label` looks up, content, choices and values.
    fn identifiers() -> Vec<String> {
        let content = Profil::ALL
            .iter()
            .map(|p| p.id())
            .chain(Peuple::ALL.iter().map(|p| p.id()))
            .chain(Series::ALL.iter().map(|s| s.id()))
            .chain(Family::ALL.iter().map(|f| f.id()))
            .chain(Armour::ALL.iter().map(|a| a.id()))
            .chain(Shield::ALL.iter().map(|s| s.id()))
            .chain(Weapon::ALL.iter().map(|w| w.id()))
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
            label("cof.peuple-bonus").as_deref(),
            Some("Bonus de peuple")
        );
        assert_eq!(label("cof.profil.pretre").as_deref(), Some("Prêtre"));
        assert_eq!(
            label(
                Value::Damage(Weapon::EpeeLongue, WeaponUse::OneHanded)
                    .id()
                    .as_str()
            )
            .as_deref(),
            Some("DM Épée longue (à une main)")
        );
        assert_eq!(
            label(Voie::Rage.ability(1.into()).as_str()).as_deref(),
            Some("Cri de guerre (Voie de la rage, rang 1)")
        );
        assert_eq!(
            label(Voie::Rage.ability(6.into()).as_str()).as_deref(),
            Some("Voie de la rage, rang 6")
        );
        assert_eq!(label("cof.unknown"), None);
        assert_eq!(label("cof.unknown.1"), None);
        assert_eq!(label("cof.rage.x"), None);
        assert_eq!(label("cof.level.3").as_deref(), Some("Niveau 3"));
        assert_eq!(label("cof.level.x"), None);
        assert_eq!(term_label(&ATTACK), Some("Niveau"));
        assert_eq!(term_label(&DEF), None);
    }

    /// A character emitting many subjects and sources: every voie at ranks
    /// 1 and 2, level-ups, every weapon, an armour, a shield and adjustments.
    fn busy() -> Character {
        let adjustment = || Adjustment::new(1, Author::new("gm"), "test reason");
        let character = Character::new()
            .with_profil(Profil::Magicien)
            .with_peuple(Peuple::Halfelin)
            .with_series(Series::Expert)
            .with_series_value(Characteristic::Vol, 3)
            .with_peuple_bonus(Characteristic::Agi)
            .with_peuple_malus(Characteristic::Cha)
            .with_armour(Armour::PlaqueComplete)
            .with_shield(Shield::GrandBouclier)
            .with_purse(Money::pa(20))
            .with_adjustment(Target::Value(Value::Money), adjustment())
            .with_adjustment(Target::Characteristic(Characteristic::For), adjustment());
        let character = Voie::ALL
            .into_iter()
            .fold(character, |c, voie| c.with_rank(voie, 1).with_rank(voie, 2))
            .with_level_up(LevelUp::new().with_rank(Voie::Rage, 3))
            .with_level_up(LevelUp::new());
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
