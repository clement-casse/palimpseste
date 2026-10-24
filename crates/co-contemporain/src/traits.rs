//! Traits distinctifs (COC.CRE.TRAITS, COC.CNT.TRAITS) and social class
//! (COC.CRE.SOCIAL-CLASS).

use std::collections::BTreeMap;

use co_kernel::{Severity, Subject, ValueId, Violation};

use crate::{
    Character,
    rules::{
        ADVANTAGES, DISADVANTAGES, SOCIAL_CLASS, SOCIAL_CLASS_VALUE, TRAIT_MECHANIC, TRAIT_POINTS,
        TRAITS,
    },
};

/// A trait distinctif: an advantage or a disadvantage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Trait {
    /// Ange gardien (advantage).
    AngeGardien,
    /// Chance insolente (advantage).
    ChanceInsolente,
    /// Charisme naturel (advantage).
    CharismeNaturel,
    /// Classe supérieure (advantage), sets the social class.
    ClasseSuperieure,
    /// Croyant (advantage), incompatible with Hypercartésien.
    Croyant,
    /// Endurant (advantage).
    Endurant,
    /// Force de la nature (advantage): CON +1, contact DM +1.
    ForceDeLaNature,
    /// Hypercartésien (advantage), incompatible with Croyant.
    Hypercartesien,
    /// Inébranlable (advantage).
    Inebranlable,
    /// Instinct de préservation (advantage).
    InstinctDePreservation,
    /// Logique élémentaire (advantage).
    LogiqueElementaire,
    /// Mémoire eidétique (advantage).
    MemoireEidetique,
    /// Nyctalope (advantage).
    Nyctalope,
    /// Rapide (advantage): Init +3.
    Rapide,
    /// Remarquable (advantage).
    Remarquable,
    /// Code de conduite (disadvantage).
    CodeDeConduite,
    /// Classe inférieure (disadvantage), sets the social class.
    ClasseInferieure,
    /// Innocent (disadvantage).
    Innocent,
    /// D'un autre âge (disadvantage): adolescent or elderly character.
    DUnAutreAge,
    /// Double vie (disadvantage).
    DoubleVie,
    /// Haine viscérale (disadvantage).
    HaineViscerale,
    /// Ignorant (disadvantage).
    Ignorant,
    /// Inapte au combat (disadvantage).
    InapteAuCombat,
    /// Nemesis, ennemi juré (disadvantage).
    Nemesis,
    /// Ostracisé (disadvantage).
    Ostracise,
    /// Séquelle (disadvantage), with the sequel chosen.
    Sequelle(Sequel),
    /// Superstitieux (disadvantage).
    Superstitieux,
    /// Syndrome de l'imposteur (disadvantage).
    SyndromeDeLImposteur,
    /// Toxicomane (disadvantage).
    Toxicomane,
    /// Vilain défaut (disadvantage).
    VilainDefaut,
}

/// The sequel chosen with the trait Séquelle (COC.CNT.SEQUELAE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sequel {
    /// Diminué (corps): CON -2, CON at most +3, health scale -4 steps.
    DiminueCorps,
    /// Diminué (esprit): INT -2, INT at most +3.
    DiminueEsprit,
    /// Any other sequel; none changes the character at creation (D-21).
    Autre,
}

impl Sequel {
    /// Every sequel, in declaration order.
    pub const ALL: [Self; 3] = [Self::DiminueCorps, Self::DiminueEsprit, Self::Autre];

    /// The identifier of the sequel acquired in play, apart from the trait
    /// Séquelle taken at creation.
    #[must_use]
    pub const fn acquired_id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::DiminueCorps => "coc.acquired-sequel.diminue-corps",
            Self::DiminueEsprit => "coc.acquired-sequel.diminue-esprit",
            Self::Autre => "coc.acquired-sequel.autre",
        })
    }

    /// Returns the sequel whose [`acquired_id`](Self::acquired_id) is `id`.
    #[must_use]
    pub fn from_acquired_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|sequel| sequel.acquired_id().as_str() == id)
    }

    /// The French display name of the sequel acquired in play.
    #[must_use]
    pub const fn acquired_name(self) -> &'static str {
        match self {
            Self::DiminueCorps => "Séquelle acquise : diminué (corps)",
            Self::DiminueEsprit => "Séquelle acquise : diminué (esprit)",
            Self::Autre => "Séquelle acquise (autre)",
        }
    }
}

/// The points of a trait, as a magnitude: the trait's side gives the sign.
///
/// No game bounds: the points each trait allows are checked by validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitPoints(u8);

impl TraitPoints {
    /// Creates trait points.
    #[must_use]
    pub const fn new(points: u8) -> Self {
        Self(points)
    }

    /// Returns the magnitude.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl From<u8> for TraitPoints {
    fn from(points: u8) -> Self {
        Self(points)
    }
}

/// Social status (COC.CRE.SOCIAL-CLASS), middle class by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SocialClass {
    /// Marginal (classe inférieure 2).
    Marginal,
    /// Classe populaire (classe inférieure 1).
    Populaire,
    /// Classe moyenne, the default.
    Moyenne,
    /// Classe moyenne supérieure (classe supérieure 1).
    MoyenneSuperieure,
    /// Grande bourgeoisie (classe supérieure 2).
    GrandeBourgeoisie,
    /// Grande fortune (classe supérieure 3).
    GrandeFortune,
}

impl SocialClass {
    /// The French display name of the social class.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Marginal => "Marginal",
            Self::Populaire => "Classe populaire",
            Self::Moyenne => "Classe moyenne",
            Self::MoyenneSuperieure => "Classe moyenne supérieure",
            Self::GrandeBourgeoisie => "Grande bourgeoisie",
            Self::GrandeFortune => "Grande fortune",
        }
    }
}

/// Points allowed on each side (COC.CRE.TRAITS).
const MAX_SIDE: i32 = 5;

impl Trait {
    /// Every trait, the sequels apart: the advantages, then the disadvantages.
    pub const ALL: [Self; 32] = [
        Self::AngeGardien,
        Self::ChanceInsolente,
        Self::CharismeNaturel,
        Self::ClasseSuperieure,
        Self::Croyant,
        Self::Endurant,
        Self::ForceDeLaNature,
        Self::Hypercartesien,
        Self::Inebranlable,
        Self::InstinctDePreservation,
        Self::LogiqueElementaire,
        Self::MemoireEidetique,
        Self::Nyctalope,
        Self::Rapide,
        Self::Remarquable,
        Self::CodeDeConduite,
        Self::ClasseInferieure,
        Self::Innocent,
        Self::DUnAutreAge,
        Self::DoubleVie,
        Self::HaineViscerale,
        Self::Ignorant,
        Self::InapteAuCombat,
        Self::Nemesis,
        Self::Ostracise,
        Self::Sequelle(Sequel::DiminueCorps),
        Self::Sequelle(Sequel::DiminueEsprit),
        Self::Sequelle(Sequel::Autre),
        Self::Superstitieux,
        Self::SyndromeDeLImposteur,
        Self::Toxicomane,
        Self::VilainDefaut,
    ];

    /// Returns the t whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.id().as_str() == id)
    }

    /// The French display name of the trait.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AngeGardien => "Ange gardien",
            Self::ChanceInsolente => "Chance insolente",
            Self::CharismeNaturel => "Charisme naturel",
            Self::ClasseSuperieure => "Classe supérieure",
            Self::Croyant => "Croyant",
            Self::Endurant => "Endurant",
            Self::ForceDeLaNature => "Force de la nature",
            Self::Hypercartesien => "Hypercartésien",
            Self::Inebranlable => "Inébranlable",
            Self::InstinctDePreservation => "Instinct de préservation",
            Self::LogiqueElementaire => "Logique élémentaire",
            Self::MemoireEidetique => "Mémoire eidétique",
            Self::Nyctalope => "Nyctalope",
            Self::Rapide => "Rapide",
            Self::Remarquable => "Remarquable",
            Self::CodeDeConduite => "Code de conduite",
            Self::ClasseInferieure => "Classe inférieure",
            Self::Innocent => "Innocent",
            Self::DUnAutreAge => "D'un autre âge",
            Self::DoubleVie => "Double vie",
            Self::HaineViscerale => "Haine viscérale",
            Self::Ignorant => "Ignorant",
            Self::InapteAuCombat => "Inapte au combat",
            Self::Nemesis => "Nemesis",
            Self::Ostracise => "Ostracisé",
            Self::Sequelle(Sequel::DiminueCorps) => "Séquelle : diminué (corps)",
            Self::Sequelle(Sequel::DiminueEsprit) => "Séquelle : diminué (esprit)",
            Self::Sequelle(Sequel::Autre) => "Séquelle (autre)",
            Self::Superstitieux => "Superstitieux",
            Self::SyndromeDeLImposteur => "Syndrome de l'imposteur",
            Self::Toxicomane => "Toxicomane",
            Self::VilainDefaut => "Vilain défaut",
        }
    }

    /// Returns the identifier of the trait, used as a violation subject and
    /// a breakdown source.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::AngeGardien => "coc.trait.ange-gardien",
            Self::ChanceInsolente => "coc.trait.chance-insolente",
            Self::CharismeNaturel => "coc.trait.charisme-naturel",
            Self::ClasseSuperieure => "coc.trait.classe-superieure",
            Self::Croyant => "coc.trait.croyant",
            Self::Endurant => "coc.trait.endurant",
            Self::ForceDeLaNature => "coc.trait.force-de-la-nature",
            Self::Hypercartesien => "coc.trait.hypercartesien",
            Self::Inebranlable => "coc.trait.inebranlable",
            Self::InstinctDePreservation => "coc.trait.instinct-de-preservation",
            Self::LogiqueElementaire => "coc.trait.logique-elementaire",
            Self::MemoireEidetique => "coc.trait.memoire-eidetique",
            Self::Nyctalope => "coc.trait.nyctalope",
            Self::Rapide => "coc.trait.rapide",
            Self::Remarquable => "coc.trait.remarquable",
            Self::CodeDeConduite => "coc.trait.code-de-conduite",
            Self::ClasseInferieure => "coc.trait.classe-inferieure",
            Self::Innocent => "coc.trait.innocent",
            Self::DUnAutreAge => "coc.trait.d-un-autre-age",
            Self::DoubleVie => "coc.trait.double-vie",
            Self::HaineViscerale => "coc.trait.haine-viscerale",
            Self::Ignorant => "coc.trait.ignorant",
            Self::InapteAuCombat => "coc.trait.inapte-au-combat",
            Self::Nemesis => "coc.trait.nemesis",
            Self::Ostracise => "coc.trait.ostracise",
            Self::Sequelle(Sequel::DiminueCorps) => "coc.trait.sequelle.diminue-corps",
            Self::Sequelle(Sequel::DiminueEsprit) => "coc.trait.sequelle.diminue-esprit",
            Self::Sequelle(Sequel::Autre) => "coc.trait.sequelle.autre",
            Self::Superstitieux => "coc.trait.superstitieux",
            Self::SyndromeDeLImposteur => "coc.trait.syndrome-de-l-imposteur",
            Self::Toxicomane => "coc.trait.toxicomane",
            Self::VilainDefaut => "coc.trait.vilain-defaut",
        })
    }

    /// Whether the trait is an advantage rather than a disadvantage.
    #[must_use]
    pub const fn is_advantage(self) -> bool {
        matches!(
            self,
            Self::AngeGardien
                | Self::ChanceInsolente
                | Self::CharismeNaturel
                | Self::ClasseSuperieure
                | Self::Croyant
                | Self::Endurant
                | Self::ForceDeLaNature
                | Self::Hypercartesien
                | Self::Inebranlable
                | Self::InstinctDePreservation
                | Self::LogiqueElementaire
                | Self::MemoireEidetique
                | Self::Nyctalope
                | Self::Rapide
                | Self::Remarquable
        )
    }

    /// The points the catalogue allows for the trait (data, COC.CNT.TRAITS).
    #[must_use]
    pub const fn allowed_points(self) -> &'static [u8] {
        match self {
            Self::AngeGardien | Self::ClasseSuperieure | Self::Nemesis => &[1, 2, 3],
            Self::CharismeNaturel
            | Self::Endurant
            | Self::Inebranlable
            | Self::LogiqueElementaire
            | Self::MemoireEidetique
            | Self::Remarquable
            | Self::ClasseInferieure
            | Self::Innocent => &[1, 2],
            Self::Toxicomane => &[1, 3],
            Self::Hypercartesien
            | Self::InstinctDePreservation
            | Self::Nyctalope
            | Self::Ignorant
            | Self::Ostracise
            | Self::Superstitieux
            | Self::SyndromeDeLImposteur => &[1],
            Self::ChanceInsolente
            | Self::Croyant
            | Self::ForceDeLaNature
            | Self::Rapide
            | Self::CodeDeConduite
            | Self::DUnAutreAge
            | Self::DoubleVie
            | Self::HaineViscerale
            | Self::InapteAuCombat
            | Self::Sequelle(_)
            | Self::VilainDefaut => &[2],
        }
    }

    /// The signed value of the trait with `points`: positive for an
    /// advantage, negative for a disadvantage.
    pub(crate) fn value(self, points: TraitPoints) -> i32 {
        let magnitude = i32::from(points.get());
        if self.is_advantage() {
            magnitude
        } else {
            -magnitude
        }
    }
}

/// The social class set by the class traits chosen at creation, or `None`
/// when they contradict each other or carry points outside the catalogue
/// (D-19).
pub(crate) fn social_class(character: &Character) -> Option<SocialClass> {
    social_class_of(&character.creation_traits())
}

/// The social class set by the class traits of `traits` (D-19).
pub(crate) fn social_class_of(traits: &BTreeMap<Trait, TraitPoints>) -> Option<SocialClass> {
    let points = |t| traits.get(&t).map(|points| points.get());
    match (
        points(Trait::ClasseSuperieure),
        points(Trait::ClasseInferieure),
    ) {
        (None, None) => Some(SocialClass::Moyenne),
        (Some(1), None) => Some(SocialClass::MoyenneSuperieure),
        (Some(2), None) => Some(SocialClass::GrandeBourgeoisie),
        (Some(3), None) => Some(SocialClass::GrandeFortune),
        (None, Some(1)) => Some(SocialClass::Populaire),
        (None, Some(2)) => Some(SocialClass::Marginal),
        _ => None,
    }
}

/// Checks the points of each trait, the balance, the incompatibilities and
/// the social class.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    for (t, points) in character.traits() {
        if !t.allowed_points().contains(&points.get()) {
            violations.push(
                Violation::deviation(
                    TRAIT_POINTS,
                    TRAIT_MECHANIC,
                    Subject::Value(t.id()),
                    Severity::Standard,
                )
                .with_parameter("points", i32::from(points.get())),
            );
        }
    }

    let advantages = character.advantages();
    let disadvantages = character.disadvantages();
    let balance = |subject, value: i32| {
        Violation::deviation(TRAITS, TRAIT_MECHANIC, subject, Severity::Standard)
            .with_parameter("value", value)
    };
    if advantages > MAX_SIDE {
        violations
            .push(balance(Subject::Value(ADVANTAGES), advantages).with_parameter("max", MAX_SIDE));
    }
    if -disadvantages > MAX_SIDE {
        violations.push(
            balance(Subject::Value(DISADVANTAGES), disadvantages).with_parameter("min", -MAX_SIDE),
        );
    }
    if advantages + disadvantages != 0 {
        violations.push(balance(Subject::Character, advantages + disadvantages));
    }
    if character.has_trait(Trait::Croyant) && character.has_trait(Trait::Hypercartesien) {
        violations.push(Violation::deviation(
            TRAITS,
            TRAIT_MECHANIC,
            Subject::Value(Trait::Croyant.id()),
            Severity::Standard,
        ));
    }

    // A deviation, not an uncomputable value, so the Game Master can waive it.
    if social_class(character).is_none() {
        violations.push(Violation::deviation(
            SOCIAL_CLASS,
            TRAIT_MECHANIC,
            Subject::Value(SOCIAL_CLASS_VALUE),
            Severity::Standard,
        ));
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    fn trait_violations(character: &Character) -> Vec<Violation> {
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

    const DEVIATION: Category = Category::Deviation(Severity::Standard);

    fn with(traits: &[(Trait, u8)]) -> Character {
        traits
            .iter()
            .fold(Character::new(), |c, (t, points)| c.with_trait(*t, *points))
    }

    #[test]
    fn no_trait_is_balanced() {
        assert_eq!(trait_violations(&Character::new()), []);
        assert_eq!(social_class(&Character::new()), Some(SocialClass::Moyenne));
    }

    #[test]
    fn sides_sum_with_their_sign() {
        let character = with(&[
            (Trait::Rapide, 2),
            (Trait::Nyctalope, 1),
            (Trait::Toxicomane, 3),
        ]);
        assert_eq!(character.advantages(), 3);
        assert_eq!(character.disadvantages(), -3);
        assert_eq!(trait_violations(&character), []);
    }

    #[test]
    fn unbalanced_traits_deviate() {
        let violations = trait_violations(&with(&[(Trait::Rapide, 2)]));
        assert_eq!(rules(&violations), [("COC.CRE.TRAITS", DEVIATION)]);
        assert_eq!(violations[0].subject(), &Subject::Character);
    }

    #[test]
    fn more_than_five_points_on_a_side_deviates() {
        let character = with(&[
            (Trait::Rapide, 2),
            (Trait::ForceDeLaNature, 2),
            (Trait::ChanceInsolente, 2),
            (Trait::CodeDeConduite, 2),
            (Trait::DoubleVie, 2),
            (Trait::VilainDefaut, 2),
        ]);
        let subjects: Vec<_> = trait_violations(&character)
            .into_iter()
            .map(|v| v.subject().clone())
            .collect();
        assert_eq!(
            subjects,
            [Subject::Value(ADVANTAGES), Subject::Value(DISADVANTAGES)]
        );
    }

    #[test]
    fn points_outside_the_catalogue_deviate() {
        let character = with(&[(Trait::Toxicomane, 2), (Trait::Rapide, 2)]);
        let violations = trait_violations(&character);
        assert_eq!(rules(&violations), [("COC.CNT.TRAITS", DEVIATION)]);
        assert_eq!(
            violations[0].subject(),
            &Subject::Value(Trait::Toxicomane.id())
        );
    }

    #[test]
    fn croyant_and_hypercartesien_are_incompatible() {
        let character = with(&[
            (Trait::Croyant, 2),
            (Trait::Hypercartesien, 1),
            (Trait::Ignorant, 1),
            (Trait::CodeDeConduite, 2),
        ]);
        assert_eq!(
            rules(&trait_violations(&character)),
            [("COC.CRE.TRAITS", DEVIATION)]
        );
    }

    #[test]
    fn class_traits_set_the_social_class() {
        let class = |t, points| social_class(&with(&[(t, points)]));
        assert_eq!(
            class(Trait::ClasseSuperieure, 1),
            Some(SocialClass::MoyenneSuperieure)
        );
        assert_eq!(
            class(Trait::ClasseSuperieure, 2),
            Some(SocialClass::GrandeBourgeoisie)
        );
        assert_eq!(
            class(Trait::ClasseSuperieure, 3),
            Some(SocialClass::GrandeFortune)
        );
        assert_eq!(
            class(Trait::ClasseInferieure, 1),
            Some(SocialClass::Populaire)
        );
        assert_eq!(
            class(Trait::ClasseInferieure, 2),
            Some(SocialClass::Marginal)
        );
    }

    #[test]
    fn invalid_class_points_leave_no_social_class() {
        let character = with(&[
            (Trait::ClasseSuperieure, 4),
            (Trait::Nemesis, 3),
            (Trait::Ignorant, 1),
        ]);
        assert_eq!(social_class(&character), None);
        assert_eq!(
            rules(&trait_violations(&character)),
            [
                ("COC.CNT.TRAITS", DEVIATION),
                ("COC.CRE.SOCIAL-CLASS", DEVIATION)
            ]
        );
    }

    #[test]
    fn both_class_traits_deviate() {
        let character = with(&[(Trait::ClasseSuperieure, 1), (Trait::ClasseInferieure, 1)]);
        assert_eq!(social_class(&character), None);
        assert_eq!(
            rules(&trait_violations(&character)),
            [("COC.CRE.SOCIAL-CLASS", DEVIATION)]
        );
    }

    #[test]
    fn each_sequel_has_its_own_identifier() {
        assert_ne!(
            Trait::Sequelle(Sequel::DiminueCorps).id(),
            Trait::Sequelle(Sequel::DiminueEsprit).id()
        );
    }
}
