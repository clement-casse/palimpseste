//! Rule, mechanic and value identifiers.
//!
//! Rule identifiers trace to the co-rules skill. Soft limits of different
//! severities carry distinct rule identifiers, because a waiver matches on
//! rule and subject only.

use co_kernel::{Mechanic, RuleId, ValueId};

/// COC.CRE.AGE-BRACKET: limits by age bracket.
pub(crate) const AGE_BRACKET: RuleId = RuleId::from_static("COC.CRE.AGE-BRACKET");
/// COC.CRE.DOMAINE: voies retained from the domaine.
pub(crate) const DOMAINE: RuleId = RuleId::from_static("COC.CRE.DOMAINE");
/// COC.CNT.DOMAINES: social class required or recommended by a domaine.
pub(crate) const DOMAINES: RuleId = RuleId::from_static("COC.CNT.DOMAINES");
/// COC.CRE.CAPACITY-POINTS: ranks bought at creation, number of voies, maximum rank.
pub(crate) const CAPACITY_POINTS: RuleId = RuleId::from_static("COC.CRE.CAPACITY-POINTS");
/// COC.CRE.MAX-VOIES: six voies in total.
pub(crate) const MAX_VOIES: RuleId = RuleId::from_static("COC.CRE.MAX-VOIES");
/// COC.CRE.TRAITS: advantages and disadvantages balance.
pub(crate) const TRAITS: RuleId = RuleId::from_static("COC.CRE.TRAITS");
/// COC.CNT.TRAITS: points allowed for each trait.
pub(crate) const TRAIT_POINTS: RuleId = RuleId::from_static("COC.CNT.TRAITS");
/// COC.CRE.SOCIAL-CLASS: social class from traits.
pub(crate) const SOCIAL_CLASS: RuleId = RuleId::from_static("COC.CRE.SOCIAL-CLASS");
/// COC.CRE.CARAC-POINTS: characteristic points by age bracket.
pub(crate) const CARAC_POINTS: RuleId = RuleId::from_static("COC.CRE.CARAC-POINTS");
/// COC.CRE.CARAC-LOWERED: one characteristic lowered below 0 (D-07).
pub(crate) const CARAC_LOWERED: RuleId = RuleId::from_static("COC.CRE.CARAC-LOWERED");
/// COC.CRE.CARAC-PHYSICAL: recommended bounds of FOR, AGI, CON (D-17).
pub(crate) const CARAC_PHYSICAL: RuleId = RuleId::from_static("COC.CRE.CARAC-PHYSICAL");
/// COC.CRE.CARAC-LIMITS: starting values within [-2, +5].
pub(crate) const CARAC_LIMITS: RuleId = RuleId::from_static("COC.CRE.CARAC-LIMITS");
/// COC.DER.INIT: Init = 10 + INT + PER.
pub(crate) const INIT: RuleId = RuleId::from_static("COC.DER.INIT");
/// COC.DER.DEF: DEF = 10 + AGI + PER.
pub(crate) const DEF: RuleId = RuleId::from_static("COC.DER.DEF");
/// COC.DER.CG: CG = CON + 3.
pub(crate) const CG: RuleId = RuleId::from_static("COC.DER.CG");
/// COC.DER.HEALTH-SCALE: 20 steps by default.
pub(crate) const HEALTH_SCALE: RuleId = RuleId::from_static("COC.DER.HEALTH-SCALE");
/// COC.DER.CRIT-RANGE: critical success on a natural 20 by default.
pub(crate) const CRIT_RANGE: RuleId = RuleId::from_static("COC.DER.CRIT-RANGE");
/// COC.CRE.EQUIPMENT: the starting equipment of the voies acquired (D-26,
/// D-30).
pub(crate) const EQUIPMENT: RuleId = RuleId::from_static("COC.CRE.EQUIPMENT");

/// Age and age bracket checks.
pub(crate) const AGE_MECHANIC: Mechanic = Mechanic::new("coc.age");
/// Domaine checks.
pub(crate) const DOMAINE_MECHANIC: Mechanic = Mechanic::new("coc.domaine");
/// Voie and capacity point checks.
pub(crate) const VOIE_MECHANIC: Mechanic = Mechanic::new("coc.voies");
/// Trait and social class checks.
pub(crate) const TRAIT_MECHANIC: Mechanic = Mechanic::new("coc.traits");
/// Characteristic checks.
pub(crate) const CARAC_MECHANIC: Mechanic = Mechanic::new("coc.characteristics");
/// Starting equipment checks.
pub(crate) const EQUIPMENT_MECHANIC: Mechanic = Mechanic::new("coc.equipment");

/// The character's age.
pub(crate) const AGE: ValueId = ValueId::from_static("coc.age");
/// The character's domaine.
pub(crate) const DOMAINE_VALUE: ValueId = ValueId::from_static("coc.domaine");
/// The sum of the advantages.
pub(crate) const ADVANTAGES: ValueId = ValueId::from_static("coc.advantages");
/// The sum of the disadvantages.
pub(crate) const DISADVANTAGES: ValueId = ValueId::from_static("coc.disadvantages");
/// The character's social class.
pub(crate) const SOCIAL_CLASS_VALUE: ValueId = ValueId::from_static("coc.social-class");
