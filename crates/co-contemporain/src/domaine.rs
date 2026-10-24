//! Domaines and the voies they suggest (COC.CRE.DOMAINE, COC.CNT.DOMAINES).

use co_kernel::{Severity, Subject, ValueId, Violation};

use crate::{
    Character, Trait, Voie,
    rules::{DOMAINE, DOMAINE_MECHANIC, DOMAINE_VALUE, DOMAINES},
};

/// The activity domaine of a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Domaine {
    /// Art (professional).
    Art,
    /// Artisanat (professional).
    Artisanat,
    /// Aventure (professional).
    Aventure,
    /// Communication (professional).
    Communication,
    /// Culture et enseignement (professional).
    CultureEtEnseignement,
    /// Finance et droit (professional).
    FinanceEtDroit,
    /// Journalisme (professional).
    Journalisme,
    /// Marine (professional).
    Marine,
    /// Mysticisme (professional).
    Mysticisme,
    /// Pilotage (professional).
    Pilotage,
    /// Santé (professional).
    Sante,
    /// Sciences (professional).
    Sciences,
    /// Sécurité (professional).
    Securite,
    /// Service (professional).
    Service,
    /// Sport (professional).
    Sport,
    /// Criminalité (extra-professional).
    Criminalite,
    /// Dilettantisme (extra-professional).
    Dilettantisme,
    /// Marginalité (extra-professional).
    Marginalite,
}

impl Domaine {
    /// Every domaine: the professional ones, then the extra-professional ones.
    pub const ALL: [Self; 18] = [
        Self::Art,
        Self::Artisanat,
        Self::Aventure,
        Self::Communication,
        Self::CultureEtEnseignement,
        Self::FinanceEtDroit,
        Self::Journalisme,
        Self::Marine,
        Self::Mysticisme,
        Self::Pilotage,
        Self::Sante,
        Self::Sciences,
        Self::Securite,
        Self::Service,
        Self::Sport,
        Self::Criminalite,
        Self::Dilettantisme,
        Self::Marginalite,
    ];

    /// Returns the identifier of the domaine, used by the choices document.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Art => "coc.domaine.art",
            Self::Artisanat => "coc.domaine.artisanat",
            Self::Aventure => "coc.domaine.aventure",
            Self::Communication => "coc.domaine.communication",
            Self::CultureEtEnseignement => "coc.domaine.culture-et-enseignement",
            Self::FinanceEtDroit => "coc.domaine.finance-et-droit",
            Self::Journalisme => "coc.domaine.journalisme",
            Self::Marine => "coc.domaine.marine",
            Self::Mysticisme => "coc.domaine.mysticisme",
            Self::Pilotage => "coc.domaine.pilotage",
            Self::Sante => "coc.domaine.sante",
            Self::Sciences => "coc.domaine.sciences",
            Self::Securite => "coc.domaine.securite",
            Self::Service => "coc.domaine.service",
            Self::Sport => "coc.domaine.sport",
            Self::Criminalite => "coc.domaine.criminalite",
            Self::Dilettantisme => "coc.domaine.dilettantisme",
            Self::Marginalite => "coc.domaine.marginalite",
        })
    }

    /// Returns the domaine whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|domaine| domaine.id().as_str() == id)
    }

    /// The French display name of the domaine.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Art => "Art",
            Self::Artisanat => "Artisanat",
            Self::Aventure => "Aventure",
            Self::Communication => "Communication",
            Self::CultureEtEnseignement => "Culture et enseignement",
            Self::FinanceEtDroit => "Finance et droit",
            Self::Journalisme => "Journalisme",
            Self::Marine => "Marine",
            Self::Mysticisme => "Mysticisme",
            Self::Pilotage => "Pilotage",
            Self::Sante => "Santé",
            Self::Sciences => "Sciences",
            Self::Securite => "Sécurité",
            Self::Service => "Service",
            Self::Sport => "Sport",
            Self::Criminalite => "Criminalité",
            Self::Dilettantisme => "Dilettantisme",
            Self::Marginalite => "Marginalité",
        }
    }

    /// Whether the domaine is extra-professional.
    #[must_use]
    pub const fn is_extra_professional(self) -> bool {
        matches!(
            self,
            Self::Criminalite | Self::Dilettantisme | Self::Marginalite
        )
    }

    /// Voies to retain from the domaine: 2 (professional) or 3 (extra-professional).
    #[must_use]
    pub const fn minimum_retained(self) -> i32 {
        if self.is_extra_professional() { 3 } else { 2 }
    }

    /// The four voie slots of the domaine; a slot may offer alternatives.
    #[must_use]
    pub const fn slots(self) -> [&'static [Voie]; 4] {
        use Voie::{
            ArmesADistance, ArmesDeContact, ArmesLourdes, ArtsEtArtisanats, Athletisme,
            CombatAMainsNues, Corporations, Crime, Discours, Furtivite, Investigation, Langues,
            Medecine, Psychologie, Sciences, Survie, Technologie, Vehicules,
        };
        const ANY_COMBAT: &[Voie] = &[ArmesADistance, ArmesDeContact, CombatAMainsNues];
        match self {
            Self::Art => [
                &[ArtsEtArtisanats],
                &[Corporations],
                &[Discours],
                &[Psychologie],
            ],
            Self::Artisanat => [
                &[ArtsEtArtisanats],
                &[Athletisme],
                &[Corporations],
                &[Technologie],
            ],
            Self::Aventure => [ANY_COMBAT, &[Corporations], &[Sciences], &[Survie]],
            Self::Communication => [&[Corporations], &[Discours], &[Langues], &[Sciences]],
            Self::CultureEtEnseignement | Self::Sciences => {
                [&[Corporations], &[Investigation], &[Langues], &[Sciences]]
            }
            Self::FinanceEtDroit => [&[Corporations], &[Discours], &[Investigation], &[Sciences]],
            Self::Journalisme => [
                &[ArtsEtArtisanats],
                &[Corporations],
                &[Discours],
                &[Investigation],
            ],
            Self::Marine | Self::Pilotage => {
                [&[Corporations], &[Technologie], &[Survie], &[Vehicules]]
            }
            Self::Mysticisme => [&[Discours], &[Langues], &[Psychologie], &[Sciences]],
            Self::Sante => [&[Corporations], &[Medecine], &[Psychologie], &[Sciences]],
            Self::Securite => [
                &[ArmesADistance, ArmesLourdes],
                &[Corporations],
                &[Investigation],
                &[Sciences],
            ],
            Self::Service => [
                &[ArtsEtArtisanats],
                &[Corporations],
                &[Discours],
                &[Furtivite],
            ],
            Self::Sport => [ANY_COMBAT, &[Athletisme], &[Corporations], &[Vehicules]],
            Self::Criminalite => [ANY_COMBAT, &[Corporations], &[Crime], &[Furtivite]],
            Self::Dilettantisme => [&[ArtsEtArtisanats], &[Discours], &[Langues], &[Sciences]],
            Self::Marginalite => [&[ArtsEtArtisanats], &[Crime], &[Furtivite], &[Survie]],
        }
    }
}

/// Checks the domaine choice, the voies retained from it, and the social
/// class it calls for.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let Some(domaine) = character.domaine() else {
        violations.push(Violation::incomplete(
            DOMAINE,
            DOMAINE_MECHANIC,
            Subject::Value(DOMAINE_VALUE),
        ));
        return;
    };
    check_retained(character, domaine, violations);
    check_social_class(character, domaine, violations);
}

/// COC.CRE.DOMAINE: enough voies retained from the domaine's slots.
fn check_retained(character: &Character, domaine: Domaine, violations: &mut Vec<Violation>) {
    let retained = domaine
        .slots()
        .iter()
        .filter(|slot| slot.iter().any(|voie| character.has_creation_voie(*voie)))
        .fold(0_i32, |retained, _| retained + 1);
    if retained < domaine.minimum_retained() {
        violations.push(
            Violation::deviation(
                DOMAINE,
                DOMAINE_MECHANIC,
                Subject::Value(DOMAINE_VALUE),
                Severity::Standard,
            )
            .with_parameter("retained", retained)
            .with_parameter("min", domaine.minimum_retained()),
        );
    }
}

/// COC.CNT.DOMAINES: the class trait an extra-professional domaine calls for.
fn check_social_class(character: &Character, domaine: Domaine, violations: &mut Vec<Violation>) {
    // The subject is the missing trait: the two severities never share a
    // waiver.
    let (class_trait, severity) = match domaine {
        Domaine::Marginalite => (Trait::ClasseInferieure, Severity::Standard),
        Domaine::Dilettantisme => (Trait::ClasseSuperieure, Severity::Recommended),
        _ => return,
    };
    if !character.has_trait(class_trait) {
        violations.push(Violation::deviation(
            DOMAINES,
            DOMAINE_MECHANIC,
            Subject::Value(class_trait.id()),
            severity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    fn domaine_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
    }

    fn with_voies(domaine: Domaine, voies: &[Voie]) -> Character {
        voies
            .iter()
            .fold(Character::new().with_domaine(domaine), |c, voie| {
                c.with_rank(*voie, 1)
            })
    }

    fn categories(violations: &[Violation]) -> Vec<(&str, Category)> {
        violations
            .iter()
            .map(|v| (v.rule().as_str(), v.category()))
            .collect()
    }

    #[test]
    fn missing_domaine_is_incomplete() {
        assert_eq!(
            categories(&domaine_violations(&Character::new())),
            [("COC.CRE.DOMAINE", Category::Incomplete)]
        );
    }

    #[test]
    fn two_slots_satisfy_a_professional_domaine() {
        let character = with_voies(Domaine::Aventure, &[Voie::Survie, Voie::ArmesLourdes]);
        assert_eq!(
            categories(&domaine_violations(&character)),
            [("COC.CRE.DOMAINE", Category::Deviation(Severity::Standard))]
        );
        let character = character.with_rank(Voie::Sciences, 1);
        assert_eq!(domaine_violations(&character), []);
    }

    #[test]
    fn alternatives_of_one_slot_count_once() {
        let character = with_voies(
            Domaine::Aventure,
            &[Voie::ArmesADistance, Voie::ArmesDeContact],
        );
        let violations = domaine_violations(&character);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].parameters()[0].value(),
            &co_kernel::ParameterValue::Integer(1)
        );
    }

    #[test]
    fn extra_professional_domaine_needs_three_slots() {
        let character = with_voies(Domaine::Criminalite, &[Voie::Crime, Voie::Furtivite]);
        assert_eq!(domaine_violations(&character).len(), 1);
        let character = character.with_rank(Voie::CombatAMainsNues, 1);
        assert_eq!(domaine_violations(&character), []);
    }

    #[test]
    fn marginalite_requires_classe_inferieure() {
        let voies = [Voie::Crime, Voie::Furtivite, Voie::Survie];
        let character = with_voies(Domaine::Marginalite, &voies);
        assert_eq!(
            categories(&domaine_violations(&character)),
            [("COC.CNT.DOMAINES", Category::Deviation(Severity::Standard))]
        );
        let character = character.with_trait(Trait::ClasseInferieure, 1);
        assert_eq!(domaine_violations(&character), []);
    }

    #[test]
    fn dilettantisme_recommends_classe_superieure() {
        let voies = [Voie::Discours, Voie::Langues, Voie::Sciences];
        let character = with_voies(Domaine::Dilettantisme, &voies);
        assert_eq!(
            categories(&domaine_violations(&character)),
            [(
                "COC.CNT.DOMAINES",
                Category::Deviation(Severity::Recommended)
            )]
        );
        assert_eq!(
            domaine_violations(&character)[0].subject(),
            &Subject::Value(Trait::ClasseSuperieure.id())
        );
    }
}
