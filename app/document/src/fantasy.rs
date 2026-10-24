//! The choices of a COF2 character: at level 1, then at each level-up.

use co_fantasy::{
    AbilityOption, Armour, Character, Equipment, LevelUp, Money, OrphanExchange, Peuple, Profil,
    Series, Shield, Target as EngineTarget, Value, Voie, Weapon,
};
use co_kernel::Characteristic as KernelCharacteristic;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts")]
use tsify::Tsify;

use crate::{
    AdjustmentEntry, Characteristic, Error, ItemEntry, PerCharacteristic, Target, VERSION,
    VoieEntry, WaiverEntry, check_version, content, filled,
};

/// The choices of a COF2 character, mirroring `co_fantasy::Character`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FantasyDocument {
    /// The document version, [`VERSION`].
    pub version: u32,
    /// The profil's identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profil: Option<String>,
    /// The peuple's identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peuple: Option<String>,
    /// The value series' identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    /// The series value assigned to each characteristic, `None` until
    /// assigned.
    #[serde(default)]
    pub series_values: PerCharacteristic<Option<i32>>,
    /// The characteristic the peuple raises.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peuple_bonus: Option<Characteristic>,
    /// The characteristic the peuple lowers, when chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peuple_malus: Option<Characteristic>,
    /// The ranks acquired per voie at creation.
    #[serde(default)]
    pub voies: Vec<VoieEntry>,
    /// The level-ups, in order: the first reaches level 2.
    #[serde(default)]
    pub level_ups: Vec<LevelUpEntry>,
    /// The option chosen for each capacité offering one.
    #[serde(default)]
    pub ability_options: Vec<AbilityOptionEntry>,
    /// The armour worn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armour: Option<String>,
    /// The shield held.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shield: Option<String>,
    /// The items carried.
    #[serde(default)]
    pub items: Vec<ItemEntry>,
    /// The purse rolled, in pièces de cuivre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purse: Option<i32>,
    /// The manual adjustments.
    #[serde(default)]
    pub adjustments: Vec<AdjustmentEntry>,
    /// The waivers granted.
    #[serde(default)]
    pub waivers: Vec<WaiverEntry>,
}

impl FantasyDocument {
    /// The document of a character with no choice made.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            version: VERSION,
            profil: None,
            peuple: None,
            series: None,
            series_values: PerCharacteristic::default(),
            peuple_bonus: None,
            peuple_malus: None,
            voies: Vec::new(),
            level_ups: Vec::new(),
            ability_options: Vec::new(),
            armour: None,
            shield: None,
            items: Vec::new(),
            purse: None,
            adjustments: Vec::new(),
            waivers: Vec::new(),
        }
    }

    /// Converts the document to the engine's character.
    ///
    /// # Errors
    ///
    /// Returns why the document cannot be read.
    pub fn to_character(&self) -> Result<Character, Error> {
        check_version(self.version)?;
        let mut character = Character::new();
        if let Some(id) = &self.profil {
            character = character.with_profil(content("profil", id, Profil::from_id)?);
        }
        if let Some(id) = &self.peuple {
            character = character.with_peuple(content("peuple", id, Peuple::from_id)?);
        }
        if let Some(id) = &self.series {
            character = character.with_series(content("series", id, Series::from_id)?);
        }
        for characteristic in KernelCharacteristic::ALL {
            if let Some(value) = self.series_values.get(characteristic) {
                character = character.with_series_value(characteristic, value);
            }
        }
        if let Some(bonus) = self.peuple_bonus {
            character = character.with_peuple_bonus(bonus.into());
        }
        if let Some(malus) = self.peuple_malus {
            character = character.with_peuple_malus(malus.into());
        }
        for entry in &self.voies {
            let voie = content("voie", &entry.id, Voie::from_id)?;
            for rank in &entry.ranks {
                character = character.with_rank(voie, *rank);
            }
        }
        for entry in &self.level_ups {
            character = character.with_level_up(entry.level_up()?);
        }
        for entry in &self.ability_options {
            let (voie, rank) = ability(&entry.ability)?;
            let option = content("ability option", &entry.option, AbilityOption::from_id)?;
            character = character.with_ability_option(voie, rank, option);
        }
        character = character.with_equipment(equipment(
            self.armour.as_deref(),
            self.shield.as_deref(),
            &self.items,
        )?);
        if let Some(purse) = self.purse {
            character = character.with_purse(Money::pc(purse));
        }
        for entry in &self.adjustments {
            let target = match &entry.target {
                Target::Characteristic(c) => EngineTarget::Characteristic((*c).into()),
                Target::Value(id) => EngineTarget::Value(content("value", id, Value::from_id)?),
            };
            character = character.with_adjustment(target, entry.adjustment()?);
        }
        for entry in &self.waivers {
            character = character.with_waiver(entry.waiver()?);
        }
        Ok(character)
    }
}

/// The capacités acquired and forgotten at one level-up, and the exchange of
/// its orphan point.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct LevelUpEntry {
    /// The identifiers of the capacités acquired (`cof.rage.2`).
    #[serde(default)]
    pub abilities: Vec<String>,
    /// The identifiers of the capacités forgotten (changement
    /// d'orientation).
    #[serde(default)]
    pub forgotten: Vec<String>,
    /// The identifier of what the orphan point is exchanged for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orphan: Option<String>,
}

impl LevelUpEntry {
    fn level_up(&self) -> Result<LevelUp, Error> {
        let mut level_up = LevelUp::new();
        for id in &self.abilities {
            let (voie, rank) = ability(id)?;
            level_up = level_up.with_rank(voie, rank);
        }
        for id in &self.forgotten {
            let (voie, rank) = ability(id)?;
            level_up = level_up.with_forgotten(voie, rank);
        }
        if let Some(id) = &self.orphan {
            level_up =
                level_up.with_orphan(content("orphan exchange", id, OrphanExchange::from_id)?);
        }
        Ok(level_up)
    }
}

/// The option chosen for a capacité.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct AbilityOptionEntry {
    /// The capacité's identifier (`cof.elfe-haut.5`).
    pub ability: String,
    /// The option's identifier (`cof.option.int`).
    pub option: String,
}

/// The voie and rank of the capacité `id` names.
fn ability(id: &str) -> Result<(Voie, co_kernel::Rank), Error> {
    content("ability", id, Voie::from_ability_id)
}

/// What a COF2 character wears and carries in play, in place of its
/// creation equipment (ADR ENG-IN-PLAY). The browser keeps it; the backend
/// never stores it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FantasyEquipment {
    /// The armour worn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armour: Option<String>,
    /// The shield held.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shield: Option<String>,
    /// The items carried.
    #[serde(default)]
    pub items: Vec<ItemEntry>,
}

impl FantasyEquipment {
    /// Converts the equipment to the engine's.
    ///
    /// # Errors
    ///
    /// Returns why the equipment cannot be read: an unknown identifier, a
    /// blank item name.
    pub fn to_equipment(&self) -> Result<Equipment, Error> {
        equipment(self.armour.as_deref(), self.shield.as_deref(), &self.items)
    }
}

/// The engine's equipment for an armour, a shield and items, from a
/// document or an equipment in play.
fn equipment(
    armour: Option<&str>,
    shield: Option<&str>,
    items: &[ItemEntry],
) -> Result<Equipment, Error> {
    let mut equipment = Equipment::new();
    if let Some(id) = armour {
        equipment = equipment.with_armour(content("armour", id, Armour::from_id)?);
    }
    if let Some(id) = shield {
        equipment = equipment.with_shield(content("shield", id, Shield::from_id)?);
    }
    for item in items {
        equipment = match item {
            ItemEntry::Weapon { id, quantity } => {
                equipment.with_item(content("weapon", id, Weapon::from_id)?, *quantity)
            }
            ItemEntry::Other { name, quantity } => {
                equipment.with_item(filled("item name", name)?, *quantity)
            }
        };
    }
    Ok(equipment)
}
