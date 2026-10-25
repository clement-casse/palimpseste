//! The content an engine offers, for the editor to present the options the
//! rules allow. Identifiers are those of the choices document.

use palimpseste_document::{Characteristic, Engine};
use serde::Serialize;
use tsify::Tsify;

/// A named entry: an identifier and its French display name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct Named {
    /// The stable identifier.
    pub id: String,
    /// The French display name.
    pub name: String,
}

impl Named {
    pub(crate) fn new(id: impl Into<String>, name: &str) -> Self {
        Self {
            id: id.into(),
            name: name.to_owned(),
        }
    }
}

/// The content of an engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(tag = "engine", rename_all = "lowercase")]
pub enum Catalog {
    /// COC2.
    Contemporain(ContemporainCatalog),
    /// COF2.
    Fantasy(FantasyCatalog),
}

/// The content of COF2 read at creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FantasyCatalog {
    /// The creation steps, in the book order.
    pub steps: Vec<Named>,
    /// The four families.
    pub families: Vec<Named>,
    /// The 14 profils.
    pub profils: Vec<ProfilEntry>,
    /// The 8 peuples.
    pub peuples: Vec<PeupleEntry>,
    /// The 3 value series.
    pub series: Vec<SeriesEntry>,
    /// The 78 voies reachable at level 1.
    pub voies: Vec<FantasyVoieEntry>,
    /// The armours, from the lightest.
    pub armours: Vec<ProtectionEntry>,
    /// The shields, from the smallest.
    pub shields: Vec<ProtectionEntry>,
    /// The weapons of the tables.
    pub weapons: Vec<FantasyWeaponEntry>,
    /// The dice rolled for the purse, in pièces d'argent.
    pub purse_dice: String,
    /// The steps of a level-up, in its form's order.
    pub level_steps: Vec<Named>,
    /// Every capacité of every voie, by identifier (ADR ENG-VALUE-IDS).
    pub abilities: Vec<AbilityEntry>,
    /// The options a capacité may take.
    pub ability_options: Vec<Named>,
    /// What an orphan point may be exchanged for.
    pub orphan_exchanges: Vec<Named>,
}

/// A capacité, named by its identifier (ADR ENG-VALUE-IDS).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct AbilityEntry {
    /// Identifier, `<voie>.<rank>`.
    pub id: String,
    /// The voie's identifier.
    pub voie: String,
    /// The rank in the voie.
    pub rank: u8,
    /// The capacité's name, such as `Défier la mort`, for lists under its
    /// voie.
    pub name: String,
    /// Display label naming the voie too, such as `Défier la mort (Voie de
    /// la rage, rang 2)`.
    pub label: String,
}

/// A COF2 profil.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ProfilEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The family's identifier.
    pub family: String,
    /// The five voies' identifiers.
    pub voies: Vec<String>,
    /// The three key characteristics; a slot may offer alternatives.
    pub key_characteristics: Vec<Vec<Characteristic>>,
    /// The characteristic of the profil's spells.
    pub magic_characteristic: Option<Characteristic>,
    /// The heaviest armour allowed, `None` for no armour.
    pub armour_limit: Option<String>,
    /// The largest shield allowed, `None` for no shield.
    pub shield_limit: Option<String>,
    /// The weapons the profil masters.
    pub mastered_weapons: Vec<String>,
    /// The starting equipment, line by line: one bundle per line to choose.
    pub starting_equipment: Vec<Vec<Vec<StartingItemEntry>>>,
}

/// An item of a starting equipment bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct StartingItemEntry {
    /// The weapon, armour or shield identifiers it may be.
    pub options: Vec<String>,
    /// How many are given.
    pub quantity: u32,
}

/// A COF2 peuple.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PeupleEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The characteristics the peuple may raise by 1.
    pub bonus_options: Vec<Characteristic>,
    /// The characteristics the peuple may lower by 1; a single one applies
    /// without a choice.
    pub malus_options: Vec<Characteristic>,
    /// The voie de peuple; a demi-elfe has none of its own.
    pub voie: Option<String>,
}

/// A COF2 value series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct SeriesEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The seven values, from the highest.
    pub values: Vec<i32>,
}

/// A COF2 voie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct FantasyVoieEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The profil it belongs to; `None` for a voie de peuple and the voie
    /// du mage.
    pub profil: Option<String>,
}

/// An armour or a shield.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ProtectionEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The DEF bonus.
    pub defence: i32,
}

/// A COF2 weapon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct FantasyWeaponEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The ways it can be used, with their dice.
    pub uses: Vec<WeaponUseEntry>,
}

/// A use of a COF2 weapon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct WeaponUseEntry {
    /// Display name of the use.
    pub name: String,
    /// The damage dice, such as `1d8`.
    pub dice: String,
}

/// The content of COC2 read at creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ContemporainCatalog {
    /// The creation steps, in the book order.
    pub steps: Vec<Named>,
    /// The 18 domaines.
    pub domaines: Vec<DomaineEntry>,
    /// The 18 voies.
    pub voies: Vec<ContemporainVoieEntry>,
    /// The traits distinctifs.
    pub traits: Vec<TraitOption>,
    /// The rows of the weapon tables.
    pub weapons: Vec<ContemporainWeaponEntry>,
    /// The protections.
    pub protections: Vec<Named>,
    /// The échelle de santé and its states.
    pub health_scale: HealthScaleEntry,
    /// The steps of a spending of XP, in its form's order.
    pub spending_steps: Vec<Named>,
    /// Every capacité of every voie, by identifier (ADR ENG-VALUE-IDS).
    pub abilities: Vec<AbilityEntry>,
    /// The sequels a character may acquire in play, by their identifier once
    /// acquired.
    pub sequels: Vec<Named>,
}

/// A COC2 domaine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DomaineEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Whether the domaine is extra-professional.
    pub extra_professional: bool,
    /// The voies to retain from the slots.
    pub minimum_retained: i32,
    /// The four voie slots; a slot may offer alternatives.
    pub slots: Vec<Vec<String>>,
}

/// A COC2 voie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ContemporainVoieEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The weapon category its starting equipment gives one weapon of.
    pub starting_weapon: Option<Named>,
    /// The characteristics its rank 5 raises whatever the choice.
    pub rank_five_raises: Vec<Characteristic>,
    /// The characteristics its rank 5 offers to choose from.
    pub rank_five_choice: Vec<Characteristic>,
}

/// A COC2 trait distinctif.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TraitOption {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// An advantage rather than a disadvantage.
    pub advantage: bool,
    /// The points the catalogue allows.
    pub allowed_points: Vec<u8>,
}

/// A COC2 weapon row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ContemporainWeaponEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The attack value it rolls against.
    pub attack_mode: Named,
}

/// The COC2 échelle de santé: its full length and the states its steps
/// reach. A character's scale may be shorter, its first steps blackened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct HealthScaleEntry {
    /// The steps of a full scale.
    pub steps: i32,
    /// The states, from the first step reached.
    pub states: Vec<HealthStateEntry>,
}

/// A state of the COC2 échelle de santé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct HealthStateEntry {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The step whose filling reaches the state.
    pub step: i32,
    /// The malus to Init, DEF and physical actions, negative.
    pub malus: i32,
}

/// The catalogue of `engine`.
#[must_use]
pub fn catalog(engine: Engine) -> Catalog {
    match engine {
        Engine::Contemporain => Catalog::Contemporain(contemporain()),
        Engine::Fantasy => Catalog::Fantasy(fantasy()),
    }
}

fn characteristics(list: &[co_kernel::Characteristic]) -> Vec<Characteristic> {
    list.iter().map(|c| (*c).into()).collect()
}

/// The identifier of a piece of starting gear.
fn gear_id(gear: co_fantasy::Gear) -> String {
    use co_fantasy::Gear;
    match gear {
        Gear::Weapon(weapon) => weapon.id().as_str().to_owned(),
        Gear::Armour(armour) => armour.id().as_str().to_owned(),
        Gear::Shield(shield) => shield.id().as_str().to_owned(),
    }
}

fn profil_entry(profil: co_fantasy::Profil) -> ProfilEntry {
    use co_fantasy::Weapon;
    ProfilEntry {
        id: profil.id().as_str().to_owned(),
        name: profil.name().to_owned(),
        family: profil.family().id().as_str().to_owned(),
        voies: profil
            .voies()
            .iter()
            .map(|voie| voie.id().as_str().to_owned())
            .collect(),
        key_characteristics: profil
            .key_characteristics()
            .iter()
            .map(|slot| characteristics(slot))
            .collect(),
        magic_characteristic: profil.magic_characteristic().map(Into::into),
        armour_limit: profil.armour_limit().map(|a| a.id().as_str().to_owned()),
        shield_limit: profil.shield_limit().map(|s| s.id().as_str().to_owned()),
        mastered_weapons: Weapon::ALL
            .into_iter()
            .filter(|weapon| profil.masters(*weapon))
            .map(|weapon| weapon.id().as_str().to_owned())
            .collect(),
        starting_equipment: profil
            .starting_equipment()
            .iter()
            .map(|line| {
                line.bundles()
                    .iter()
                    .map(|bundle| {
                        bundle
                            .iter()
                            .map(|item| StartingItemEntry {
                                options: item.options().iter().copied().map(gear_id).collect(),
                                quantity: item.quantity(),
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect(),
    }
}

fn fantasy() -> FantasyCatalog {
    use co_fantasy::{
        Armour, Family, Peuple, Profil, Series, Shield, Step, Voie, Weapon, WeaponUse,
    };
    FantasyCatalog {
        steps: Step::ALL
            .iter()
            .map(|step| Named::new(step.id(), step.name()))
            .collect(),
        families: Family::ALL
            .iter()
            .map(|family| Named::new(family.id().as_str(), family.name()))
            .collect(),
        profils: Profil::ALL.into_iter().map(profil_entry).collect(),
        peuples: Peuple::ALL
            .into_iter()
            .map(|peuple| PeupleEntry {
                id: peuple.id().as_str().to_owned(),
                name: peuple.name().to_owned(),
                bonus_options: characteristics(peuple.bonus_options()),
                malus_options: characteristics(peuple.malus_options()),
                voie: peuple.voie().map(|voie| voie.id().as_str().to_owned()),
            })
            .collect(),
        series: Series::ALL
            .into_iter()
            .map(|series| SeriesEntry {
                id: series.id().as_str().to_owned(),
                name: series.name().to_owned(),
                values: series.values().to_vec(),
            })
            .collect(),
        voies: Voie::ALL
            .into_iter()
            .map(|voie| FantasyVoieEntry {
                id: voie.id().as_str().to_owned(),
                name: voie.name().to_owned(),
                profil: voie.profil().map(|p| p.id().as_str().to_owned()),
            })
            .collect(),
        armours: Armour::ALL
            .into_iter()
            .map(|armour| ProtectionEntry {
                id: armour.id().as_str().to_owned(),
                name: armour.name().to_owned(),
                defence: armour.defence(),
            })
            .collect(),
        shields: Shield::ALL
            .into_iter()
            .map(|shield| ProtectionEntry {
                id: shield.id().as_str().to_owned(),
                name: shield.name().to_owned(),
                defence: shield.defence(),
            })
            .collect(),
        weapons: Weapon::ALL
            .into_iter()
            .map(|weapon| FantasyWeaponEntry {
                id: weapon.id().as_str().to_owned(),
                name: weapon.name().to_owned(),
                uses: WeaponUse::ALL
                    .into_iter()
                    .filter_map(|weapon_use| {
                        weapon.dice(weapon_use).map(|dice| WeaponUseEntry {
                            name: weapon_use.name().to_owned(),
                            dice: dice.to_string(),
                        })
                    })
                    .collect(),
            })
            .collect(),
        purse_dice: co_fantasy::purse_dice().to_string(),
        level_steps: co_fantasy::LevelStep::ALL
            .iter()
            .map(|step| Named::new(step.id(), step.name()))
            .collect(),
        abilities: Voie::ALL
            .into_iter()
            .flat_map(|voie| {
                (1..=5).map(move |rank| {
                    let ability = co_kernel::Rank::new(rank);
                    ability_entry(
                        voie.ability(ability).as_str(),
                        voie.id().as_str(),
                        rank,
                        voie.ability_name(ability).unwrap_or_default(),
                        co_fantasy::label,
                    )
                })
            })
            .collect(),
        ability_options: co_fantasy::AbilityOption::ALL
            .into_iter()
            .map(|option| Named::new(option.id(), option.name()))
            .collect(),
        orphan_exchanges: co_fantasy::OrphanExchange::ALL
            .into_iter()
            .map(|exchange| Named::new(exchange.id(), exchange.name()))
            .collect(),
    }
}

/// The catalog entry of the capacité `id`, named `name` and labelled by the
/// engine's `label`.
fn ability_entry(
    id: &str,
    voie: &str,
    rank: u8,
    name: &str,
    label: fn(&str) -> Option<String>,
) -> AbilityEntry {
    AbilityEntry {
        id: id.to_owned(),
        voie: voie.to_owned(),
        rank,
        name: name.to_owned(),
        label: label(id).unwrap_or_else(|| id.to_owned()),
    }
}

/// The identifier and display name of a COC2 attack mode.
fn attack_mode(mode: co_contemporain::AttackMode) -> Named {
    Named::new(mode.id(), mode.name())
}

fn contemporain() -> ContemporainCatalog {
    use co_contemporain::{
        Domaine, HEALTH_SCALE_STEPS, HealthState, Protection, Step, Trait, Voie, Weapon,
    };
    ContemporainCatalog {
        steps: Step::ALL
            .iter()
            .map(|step| Named::new(step.id(), step.name()))
            .collect(),
        domaines: Domaine::ALL
            .into_iter()
            .map(|domaine| DomaineEntry {
                id: domaine.id().as_str().to_owned(),
                name: domaine.name().to_owned(),
                extra_professional: domaine.is_extra_professional(),
                minimum_retained: domaine.minimum_retained(),
                slots: domaine
                    .slots()
                    .iter()
                    .map(|slot| slot.iter().map(|v| v.id().as_str().to_owned()).collect())
                    .collect(),
            })
            .collect(),
        voies: Voie::ALL
            .into_iter()
            .map(|voie| ContemporainVoieEntry {
                id: voie.id().as_str().to_owned(),
                name: voie.name().to_owned(),
                starting_weapon: voie.starting_weapon().map(attack_mode),
                rank_five_raises: characteristics(voie.rank_five_raises()),
                rank_five_choice: characteristics(voie.rank_five_choice()),
            })
            .collect(),
        traits: Trait::ALL
            .into_iter()
            .map(|t| TraitOption {
                id: t.id().as_str().to_owned(),
                name: t.name().to_owned(),
                advantage: t.is_advantage(),
                allowed_points: t.allowed_points().to_vec(),
            })
            .collect(),
        weapons: Weapon::ALL
            .into_iter()
            .map(|weapon| ContemporainWeaponEntry {
                id: weapon.id().as_str().to_owned(),
                name: weapon.name().to_owned(),
                attack_mode: attack_mode(weapon.attack_mode()),
            })
            .collect(),
        protections: Protection::ALL
            .into_iter()
            .map(|p| Named::new(p.id().as_str(), p.name()))
            .collect(),
        health_scale: HealthScaleEntry {
            steps: HEALTH_SCALE_STEPS,
            states: HealthState::ALL
                .into_iter()
                .map(|state| HealthStateEntry {
                    id: state.id().as_str().to_owned(),
                    name: state.name().to_owned(),
                    step: state.step(),
                    malus: state.malus(),
                })
                .collect(),
        },
        spending_steps: co_contemporain::SpendingStep::ALL
            .iter()
            .map(|step| Named::new(step.id(), step.name()))
            .collect(),
        abilities: Voie::ALL
            .into_iter()
            .flat_map(|voie| {
                (1..=5).map(move |rank| {
                    let ability = co_kernel::Rank::new(rank);
                    ability_entry(
                        voie.ability(ability).as_str(),
                        voie.id().as_str(),
                        rank,
                        voie.ability_name(ability).unwrap_or_default(),
                        co_contemporain::label,
                    )
                })
            })
            .collect(),
        sequels: co_contemporain::Sequel::ALL
            .into_iter()
            .map(|sequel| Named::new(sequel.acquired_id().as_str(), sequel.acquired_name()))
            .collect(),
    }
}
