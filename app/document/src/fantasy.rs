//! The choices of a COF2 character at level 1.

use co_fantasy::{
    Armour, Character, Money, Peuple, Profil, Series, Shield, Target as EngineTarget, Value, Voie,
    Weapon,
};
use co_kernel::Characteristic as KernelCharacteristic;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts")]
use tsify::Tsify;

use crate::{
    AdjustmentEntry, Characteristic, DocumentError, ItemEntry, PerCharacteristic, Target, VERSION,
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
    /// The ranks acquired per voie.
    #[serde(default)]
    pub voies: Vec<VoieEntry>,
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
    pub fn to_character(&self) -> Result<Character, DocumentError> {
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
        if let Some(id) = &self.armour {
            character = character.with_armour(content("armour", id, Armour::from_id)?);
        }
        if let Some(id) = &self.shield {
            character = character.with_shield(content("shield", id, Shield::from_id)?);
        }
        for item in &self.items {
            character = match item {
                ItemEntry::Weapon { id, quantity } => {
                    character.with_item(content("weapon", id, Weapon::from_id)?, *quantity)
                }
                ItemEntry::Other { name, quantity } => {
                    character.with_item(filled("item name", name)?, *quantity)
                }
            };
        }
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
