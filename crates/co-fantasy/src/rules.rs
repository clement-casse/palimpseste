//! Rule, mechanic and value identifiers.
//!
//! Rule identifiers trace to the co-rules skill. Soft limits of different
//! severities carry distinct rule identifiers, because a waiver matches on
//! rule and subject only.

use co_kernel::{Mechanic, RuleId, ValueId};

/// CO.VOIE.STRUCTURE: ranks 1 to 5.
pub(crate) const VOIE_STRUCTURE: RuleId = RuleId::from_static("CO.VOIE.STRUCTURE");
/// CO.VOIE.ORDER: lower ranks first, no gaps.
pub(crate) const VOIE_ORDER: RuleId = RuleId::from_static("CO.VOIE.ORDER");
/// COF.CRE.PROFIL: a profil, and with it a family.
pub(crate) const PROFIL: RuleId = RuleId::from_static("COF.CRE.PROFIL");
/// COF.CRE.PEUPLE: a peuple.
pub(crate) const PEUPLE: RuleId = RuleId::from_static("COF.CRE.PEUPLE");
/// COF.CRE.CARAC-SERIES: the seven values of one series.
pub(crate) const CARAC_SERIES: RuleId = RuleId::from_static("COF.CRE.CARAC-SERIES");
/// COF.CRE.CARAC-KEY: the best values on the profil's key characteristics.
pub(crate) const CARAC_KEY: RuleId = RuleId::from_static("COF.CRE.CARAC-KEY");
/// COF.CRE.CARAC-MAGIC: a spellcaster's magic characteristic at +1 or more.
pub(crate) const CARAC_MAGIC: RuleId = RuleId::from_static("COF.CRE.CARAC-MAGIC");
/// COF.CRE.CARAC-PEUPLE: the peuple modifiers.
pub(crate) const CARAC_PEUPLE: RuleId = RuleId::from_static("COF.CRE.CARAC-PEUPLE");
/// COF.CRE.CARAC-BOUNDS: values within [-2, +4] (human) or [-2, +5].
pub(crate) const CARAC_BOUNDS: RuleId = RuleId::from_static("COF.CRE.CARAC-BOUNDS");
/// COF.CRE.VOIES: two profil voies and the voie de peuple.
pub(crate) const VOIES: RuleId = RuleId::from_static("COF.CRE.VOIES");
/// COF.CRE.MAGE-EXTRA: the mage family's rank-2 capacité.
pub(crate) const MAGE_EXTRA: RuleId = RuleId::from_static("COF.CRE.MAGE-EXTRA");
/// COF.CRE.VOIE-DU-MAGE: only the mage family replaces its voie de peuple.
pub(crate) const VOIE_DU_MAGE: RuleId = RuleId::from_static("COF.CRE.VOIE-DU-MAGE");
/// COF.PRG.LEVEL-REQ: level required per rank.
pub(crate) const LEVEL_REQ: RuleId = RuleId::from_static("COF.PRG.LEVEL-REQ");
/// COF.CRE.LEVEL1: every character starts at level 1.
pub(crate) const LEVEL1: RuleId = RuleId::from_static("COF.CRE.LEVEL1");
/// COF.DER.DR: DR = 2 + CON.
pub(crate) const DR: RuleId = RuleId::from_static("COF.DER.DR");
/// COF.DER.PC: PC = 2 + CHA.
pub(crate) const PC: RuleId = RuleId::from_static("COF.DER.PC");
/// COF.DER.PM: PM = VOL + spells known (D-11).
pub(crate) const PM: RuleId = RuleId::from_static("COF.DER.PM");
/// COF.DER.INIT: Init = 10 + PER.
pub(crate) const INIT: RuleId = RuleId::from_static("COF.DER.INIT");
/// COF.DER.DEF: DEF = 10 + AGI + armour + shield.
pub(crate) const DEF: RuleId = RuleId::from_static("COF.DER.DEF");
/// COF.DER.ARMOUR-LIMIT: the profil's armour and shield limits (D-24).
pub(crate) const ARMOUR_LIMIT: RuleId = RuleId::from_static("COF.DER.ARMOUR-LIMIT");

/// Profil checks.
pub(crate) const PROFIL_MECHANIC: Mechanic = Mechanic::new("cof.profil");
/// Peuple checks.
pub(crate) const PEUPLE_MECHANIC: Mechanic = Mechanic::new("cof.peuple");
/// Characteristic checks.
pub(crate) const CARAC_MECHANIC: Mechanic = Mechanic::new("cof.characteristics");
/// Voie and rank checks.
pub(crate) const VOIE_MECHANIC: Mechanic = Mechanic::new("cof.voies");
/// Armour and shield checks.
pub(crate) const ARMOUR_MECHANIC: Mechanic = Mechanic::new("cof.armour");
/// Derived value checks.
pub(crate) const DERIVED_MECHANIC: Mechanic = Mechanic::new("cof.derived");

/// The character's profil.
pub(crate) const PROFIL_VALUE: ValueId = ValueId::from_static("cof.profil");
/// The character's peuple.
pub(crate) const PEUPLE_VALUE: ValueId = ValueId::from_static("cof.peuple");
/// The value series and its assignment.
pub(crate) const SERIES: ValueId = ValueId::from_static("cof.series");
/// The characteristic raised by the peuple.
pub(crate) const PEUPLE_BONUS: ValueId = ValueId::from_static("cof.peuple-bonus");
/// The characteristic lowered by the peuple.
pub(crate) const PEUPLE_MALUS: ValueId = ValueId::from_static("cof.peuple-malus");
/// The voies chosen besides the voie de peuple.
pub(crate) const PROFIL_VOIES: ValueId = ValueId::from_static("cof.profil-voies");
/// The voie de peuple, or the voie du mage replacing it.
pub(crate) const PEUPLE_VOIE: ValueId = ValueId::from_static("cof.peuple-voie");
/// The armour worn.
pub(crate) const ARMOUR: ValueId = ValueId::from_static("cof.armour");
/// The shield carried.
pub(crate) const SHIELD: ValueId = ValueId::from_static("cof.shield");
