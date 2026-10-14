//! Domaines and the voies they suggest (COC.CRE.DOMAINE, COC.CNT.DOMAINES).

use co_kernel::{Severity, Subject, Violation};

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
    /// Whether the domaine is extra-professional.
    const fn is_extra_professional(self) -> bool {
        matches!(
            self,
            Self::Criminalite | Self::Dilettantisme | Self::Marginalite
        )
    }

    /// Voies to retain from the domaine: 2 (professional) or 3 (extra-professional).
    const fn minimum_retained(self) -> usize {
        if self.is_extra_professional() { 3 } else { 2 }
    }

    /// The four voie slots of the domaine; a slot may offer alternatives.
    const fn slots(self) -> [&'static [Voie]; 4] {
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
    let subject = || Subject::Value(DOMAINE_VALUE);
    let Some(domaine) = character.domaine() else {
        violations.push(Violation::incomplete(DOMAINE, DOMAINE_MECHANIC, subject()));
        return;
    };

    let retained = domaine
        .slots()
        .iter()
        .filter(|slot| slot.iter().any(|voie| character.has_voie(*voie)))
        .count();
    if retained < domaine.minimum_retained() {
        violations.push(
            Violation::deviation(DOMAINE, DOMAINE_MECHANIC, subject(), Severity::Standard)
                .with_parameter("retained", crate::saturate(retained))
                .with_parameter("min", crate::saturate(domaine.minimum_retained())),
        );
    }

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
