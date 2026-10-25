//! The choices of a COC2 character: at creation, then the experience spent
//! and the sequels acquired in play.

use co_contemporain::{
    Character, Domaine, Equipment, Protection, Sequel, Target as EngineTarget, Trait, Value, Voie,
    Weapon, XpSpending,
};
use co_kernel::Characteristic as KernelCharacteristic;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts")]
use tsify::Tsify;

use crate::{
    AdjustmentEntry, Characteristic, Error, ItemEntry, PerCharacteristic, SpecialityEntry, Target,
    VERSION, VoieEntry, WaiverEntry, check_version, content, filled,
};

/// The choices of a COC2 character, mirroring `co_contemporain::Character`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ContemporainDocument {
    /// The document version, [`VERSION`].
    pub version: u32,
    /// The age, in years.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u16>,
    /// The domaine's identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domaine: Option<String>,
    /// The ranks acquired per voie at creation.
    #[serde(default)]
    pub voies: Vec<VoieEntry>,
    /// The XP earned since creation.
    #[serde(default)]
    pub experience: u32,
    /// The spendings of XP, in order.
    #[serde(default)]
    pub xp_spendings: Vec<XpSpendingEntry>,
    /// The characteristic chosen for each rank 5 offering a choice.
    #[serde(default)]
    pub raised_characteristics: Vec<RaisedCharacteristicEntry>,
    /// The speciality written for each capacité asking for one.
    #[serde(default)]
    pub specialities: Vec<SpecialityEntry>,
    /// The identifiers of the sequels acquired in play
    /// (`coc.acquired-sequel.diminue-corps`).
    #[serde(default)]
    pub acquired_sequels: Vec<String>,
    /// The traits distinctifs taken.
    #[serde(default)]
    pub traits: Vec<TraitEntry>,
    /// The characteristic points allocated; negative points lower.
    #[serde(default)]
    pub carac_points: PerCharacteristic<i32>,
    /// The items carried.
    #[serde(default)]
    pub items: Vec<ItemEntry>,
    /// The identifiers of the protections worn.
    #[serde(default)]
    pub protections: Vec<String>,
    /// The manual adjustments.
    #[serde(default)]
    pub adjustments: Vec<AdjustmentEntry>,
    /// The waivers granted.
    #[serde(default)]
    pub waivers: Vec<WaiverEntry>,
}

/// What one spending of XP buys.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct XpSpendingEntry {
    /// The identifiers of the capacités bought (`coc.crime.3`).
    #[serde(default)]
    pub abilities: Vec<String>,
    /// The trait changes bought.
    #[serde(default)]
    pub traits: Vec<TraitChangeEntry>,
}

/// A trait set to new points, or given up, by a spending of XP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct TraitChangeEntry {
    /// The trait's identifier.
    pub id: String,
    /// The trait's new points; none gives the trait up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<u8>,
}

/// The characteristic a voie's rank 5 raises, chosen among its options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct RaisedCharacteristicEntry {
    /// The voie's identifier.
    pub voie: String,
    /// The characteristic raised.
    pub characteristic: Characteristic,
}

/// A trait distinctif taken, with its points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct TraitEntry {
    /// The trait's identifier.
    pub id: String,
    /// The points, as a magnitude: the trait's side gives the sign.
    pub points: u8,
}

impl ContemporainDocument {
    /// The document of a character with no choice made.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            version: VERSION,
            age: None,
            domaine: None,
            voies: Vec::new(),
            experience: 0,
            xp_spendings: Vec::new(),
            raised_characteristics: Vec::new(),
            specialities: Vec::new(),
            acquired_sequels: Vec::new(),
            traits: Vec::new(),
            carac_points: PerCharacteristic::default(),
            items: Vec::new(),
            protections: Vec::new(),
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
        if let Some(age) = self.age {
            character = character.with_age(age);
        }
        if let Some(id) = &self.domaine {
            character = character.with_domaine(content("domaine", id, Domaine::from_id)?);
        }
        for entry in &self.voies {
            let voie = content("voie", &entry.id, Voie::from_id)?;
            for rank in &entry.ranks {
                character = character.with_rank(voie, *rank);
            }
        }
        for entry in &self.traits {
            let t = content("trait", &entry.id, Trait::from_id)?;
            character = character.with_trait(t, entry.points);
        }
        for characteristic in KernelCharacteristic::ALL {
            let points = self.carac_points.get(characteristic);
            if points != 0 {
                character = character.with_carac_points(characteristic, points);
            }
        }
        character = character.with_experience(self.experience);
        for entry in &self.xp_spendings {
            character = character.with_xp_spending(entry.spending()?);
        }
        for entry in &self.raised_characteristics {
            let voie = content("voie", &entry.voie, Voie::from_id)?;
            character = character.with_raised_characteristic(voie, entry.characteristic.into());
        }
        for entry in &self.specialities {
            let (voie, rank) = content("ability", &entry.ability, Voie::from_ability_id)?;
            character =
                character.with_speciality(voie, rank, filled("speciality", &entry.speciality)?);
        }
        for id in &self.acquired_sequels {
            character =
                character.with_acquired_sequel(content("sequel", id, Sequel::from_acquired_id)?);
        }
        character = character.with_equipment(equipment(&self.protections, &self.items)?);
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

impl XpSpendingEntry {
    fn spending(&self) -> Result<XpSpending, Error> {
        let mut spending = XpSpending::new();
        for id in &self.abilities {
            let (voie, rank) = content("ability", id, Voie::from_ability_id)?;
            spending = spending.with_rank(voie, rank);
        }
        for change in &self.traits {
            let t = content("trait", &change.id, Trait::from_id)?;
            spending = match change.points {
                Some(points) => spending.with_trait(t, points),
                None => spending.with_trait_given_up(t),
            };
        }
        Ok(spending)
    }
}

/// What a COC2 character wears and carries in play, in place of its
/// creation equipment (ADR ENG-IN-PLAY). The browser keeps it; the backend
/// never stores it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ContemporainEquipment {
    /// The identifiers of the protections worn.
    #[serde(default)]
    pub protections: Vec<String>,
    /// The items carried.
    #[serde(default)]
    pub items: Vec<ItemEntry>,
}

impl ContemporainEquipment {
    /// Converts the equipment to the engine's.
    ///
    /// # Errors
    ///
    /// Returns why the equipment cannot be read: an unknown identifier, a
    /// blank item name.
    pub fn to_equipment(&self) -> Result<Equipment, Error> {
        equipment(&self.protections, &self.items)
    }
}

/// The engine's equipment for protections and items, from a document or an
/// equipment in play.
fn equipment(protections: &[String], items: &[ItemEntry]) -> Result<Equipment, Error> {
    let mut equipment = Equipment::new();
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
    for id in protections {
        equipment = equipment.with_protection(content("protection", id, Protection::from_id)?);
    }
    Ok(equipment)
}
