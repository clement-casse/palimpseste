//! The 18 voies (COC.CNT.VOIES) and the checks on acquired ranks.

use co_kernel::{
    AbilityId, Characteristic, Rank, Severity, Subject, Violation, VoieId, voie_rank_violations,
};

use crate::{
    AttackMode, Character, Value,
    rules::{CAPACITY_POINTS, CONTENT_VOIES, MAX_VOIES, VOIE_MECHANIC},
};

/// One of the 18 voies of COC2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Voie {
    /// Voie des armes de contact.
    ArmesDeContact,
    /// Voie des armes à distance.
    ArmesADistance,
    /// Voie des armes lourdes.
    ArmesLourdes,
    /// Voie des arts et artisanats.
    ArtsEtArtisanats,
    /// Voie de l'athlétisme.
    Athletisme,
    /// Voie du combat à mains nues.
    CombatAMainsNues,
    /// Voie des corporations.
    Corporations,
    /// Voie du crime.
    Crime,
    /// Voie du discours.
    Discours,
    /// Voie de la furtivité.
    Furtivite,
    /// Voie de l'investigation.
    Investigation,
    /// Voie des langues.
    Langues,
    /// Voie de la médecine.
    Medecine,
    /// Voie de la psychologie.
    Psychologie,
    /// Voie de la technologie.
    Technologie,
    /// Voie des sciences.
    Sciences,
    /// Voie de la survie.
    Survie,
    /// Voie des véhicules.
    Vehicules,
}

impl Voie {
    /// Every voie, in declaration order.
    pub const ALL: [Self; 18] = [
        Self::ArmesDeContact,
        Self::ArmesADistance,
        Self::ArmesLourdes,
        Self::ArtsEtArtisanats,
        Self::Athletisme,
        Self::CombatAMainsNues,
        Self::Corporations,
        Self::Crime,
        Self::Discours,
        Self::Furtivite,
        Self::Investigation,
        Self::Langues,
        Self::Medecine,
        Self::Psychologie,
        Self::Technologie,
        Self::Sciences,
        Self::Survie,
        Self::Vehicules,
    ];

    /// Returns the voie whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|voie| voie.slug() == id)
    }

    /// The French display name of the voie.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ArmesDeContact => "Voie des armes de contact",
            Self::ArmesADistance => "Voie des armes à distance",
            Self::ArmesLourdes => "Voie des armes lourdes",
            Self::ArtsEtArtisanats => "Voie des arts et artisanats",
            Self::Athletisme => "Voie de l'athlétisme",
            Self::CombatAMainsNues => "Voie du combat à mains nues",
            Self::Corporations => "Voie des corporations",
            Self::Crime => "Voie du crime",
            Self::Discours => "Voie du discours",
            Self::Furtivite => "Voie de la furtivité",
            Self::Investigation => "Voie de l'investigation",
            Self::Langues => "Voie des langues",
            Self::Medecine => "Voie de la médecine",
            Self::Psychologie => "Voie de la psychologie",
            Self::Technologie => "Voie de la technologie",
            Self::Sciences => "Voie des sciences",
            Self::Survie => "Voie de la survie",
            Self::Vehicules => "Voie des véhicules",
        }
    }

    /// The weapon category the voie's starting equipment gives one weapon of;
    /// the other voies give free text items only (COC.CNT.VOIES).
    #[must_use]
    pub const fn starting_weapon(self) -> Option<AttackMode> {
        match self {
            Self::ArmesDeContact => Some(AttackMode::ContactWeapon),
            Self::ArmesADistance => Some(AttackMode::RangedWeapon),
            Self::ArmesLourdes => Some(AttackMode::HeavyWeapon),
            _ => None,
        }
    }

    /// The characteristics the voie's rank 5 raises by 1 whatever the choice
    /// (COC.PRG.CARAC, COC.CNT.VOIES); empty when it raises the one chosen
    /// in [`rank_five_choice`](Self::rank_five_choice).
    #[must_use]
    pub const fn rank_five_raises(self) -> &'static [Characteristic] {
        use Characteristic::{Agi, Cha, Con, Int};
        match self {
            Self::ArtsEtArtisanats => &[Cha],
            Self::Corporations => &[Cha, Int],
            Self::Langues | Self::Medecine | Self::Technologie | Self::Sciences => &[Int],
            Self::Survie => &[Con],
            Self::Vehicules => &[Agi],
            _ => &[],
        }
    }

    /// The characteristics the voie's rank 5 offers to raise by 1, one to
    /// choose; empty when it raises fixed ones.
    #[must_use]
    pub const fn rank_five_choice(self) -> &'static [Characteristic] {
        use Characteristic::{Agi, Cha, Con, For, Int, Per, Vol};
        match self {
            Self::ArmesDeContact | Self::CombatAMainsNues => &[For, Agi],
            Self::ArmesADistance => &[Agi, Per],
            Self::ArmesLourdes => &[For, Per],
            Self::Athletisme => &[For, Agi, Con],
            Self::Crime => &[Per, Cha],
            Self::Discours => &[Cha, Int],
            Self::Furtivite => &[Per, Agi],
            Self::Investigation => &[Int, Per],
            Self::Psychologie => &[Per, Vol],
            _ => &[],
        }
    }

    /// What the capacité at `rank` asks the player to choose, in French
    /// (`discipline`, `approche`, `véhicule`), as a speciality written with
    /// [`Character::with_speciality`]; `None` when it asks for none
    /// (COC.CNT.VOIES).
    #[must_use]
    pub const fn speciality(self, rank: Rank) -> Option<&'static str> {
        Some(match (self, rank.get()) {
            (Self::ArtsEtArtisanats, 1..=5) => "discipline",
            (Self::Athletisme, 2..=5) => "discipline sportive",
            (Self::CombatAMainsNues, 2) => "art martial",
            (Self::Corporations, 1 | 3 | 5) => "corporation",
            (Self::Discours, 1..=5) => "approche",
            (Self::Investigation, 2 | 4) => "spécialité",
            (Self::Medecine, 2 | 3 | 5) => "spécialité médicale",
            (Self::Medecine, 4) => "médecine traditionnelle",
            (Self::Technologie, 2..=5) => "spécialité technique",
            (Self::Sciences, 1..=5) => "discipline scientifique",
            (Self::Survie, 2..=5) => "milieu",
            (Self::Vehicules, 1..=5) => "véhicule",
            _ => return None,
        })
    }

    /// Returns the kernel identifier of the voie.
    #[must_use]
    pub fn id(self) -> VoieId {
        VoieId::new(self.slug())
    }

    /// Returns the identifier of the capacité at `rank` in this voie,
    /// `<voie>.<rank>` (ADR ENG-VALUE-IDS): the one key every capacité is
    /// named by.
    #[must_use]
    pub fn ability(self, rank: Rank) -> AbilityId {
        AbilityId::new(format!("{}.{}", self.slug(), rank.get()))
    }

    /// Returns the voie and rank of the capacité whose
    /// [`ability`](Self::ability) identifier is `id`.
    #[must_use]
    pub fn from_ability_id(id: &str) -> Option<(Self, Rank)> {
        let (voie, rank) = id.rsplit_once('.')?;
        let rank: u8 = rank.parse().ok()?;
        Some((Self::from_id(voie)?, Rank::new(rank)))
    }

    /// The French display name of the capacité at `rank` in this voie
    /// (COC.CNT.VOIES, D-45); `None` outside ranks 1 to 5.
    #[must_use]
    pub fn ability_name(self, rank: Rank) -> Option<&'static str> {
        let index = usize::from(rank.get()).checked_sub(1)?;
        ABILITY_NAMES
            .iter()
            .find(|(voie, _)| *voie == self)
            .and_then(|(_, names)| names.get(index).copied())
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::ArmesDeContact => "coc.armes-de-contact",
            Self::ArmesADistance => "coc.armes-a-distance",
            Self::ArmesLourdes => "coc.armes-lourdes",
            Self::ArtsEtArtisanats => "coc.arts-et-artisanats",
            Self::Athletisme => "coc.athletisme",
            Self::CombatAMainsNues => "coc.combat-a-mains-nues",
            Self::Corporations => "coc.corporations",
            Self::Crime => "coc.crime",
            Self::Discours => "coc.discours",
            Self::Furtivite => "coc.furtivite",
            Self::Investigation => "coc.investigation",
            Self::Langues => "coc.langues",
            Self::Medecine => "coc.medecine",
            Self::Psychologie => "coc.psychologie",
            Self::Technologie => "coc.technologie",
            Self::Sciences => "coc.sciences",
            Self::Survie => "coc.survie",
            Self::Vehicules => "coc.vehicules",
        }
    }
}

/// The names of the capacités of each voie, ranks 1 to 5 (COC.CNT.VOIES, D-45).
const ABILITY_NAMES: [(Voie, [&str; 5]); 18] = [
    (
        Voie::ArmesDeContact,
        [
            "Amateur",
            "Entraîné",
            "Tacticien",
            "Fin bretteur",
            "Maître d'armes",
        ],
    ),
    (
        Voie::ArmesADistance,
        [
            "Amateur",
            "Entraîné",
            "Tacticien",
            "Sniper",
            "Tireur d'élite",
        ],
    ),
    (
        Voie::ArmesLourdes,
        [
            "Artilleur",
            "Mitrailleur",
            "Tacticien d'impact",
            "Canonnier",
            "Seigneur du feu",
        ],
    ),
    (
        Voie::ArtsEtArtisanats,
        [
            "Artiste/artisan",
            "Copiste",
            "Inspiré",
            "Célébrité",
            "Génie",
        ],
    ),
    (
        Voie::Athletisme,
        [
            "Sportif amateur",
            "Sportif confirmé",
            "Déterminé",
            "Athlète",
            "Champion",
        ],
    ),
    (
        Voie::CombatAMainsNues,
        ["Amateur", "Entraîné", "Tacticien", "Disciple", "Maître"],
    ),
    (
        Voie::Corporations,
        ["Membre", "Connecté", "Cadre", "Influenceur", "Boss"],
    ),
    (
        Voie::Crime,
        [
            "Pickpocket",
            "Gangster",
            "Cambrioleur",
            "Imposteur",
            "Cerveau",
        ],
    ),
    (
        Voie::Discours,
        [
            "Convaincant",
            "Éloquent",
            "Rhétoricien",
            "Meneur",
            "Marionnettiste",
        ],
    ),
    (
        Voie::Furtivite,
        ["Furtif", "Vigilant", "Passe-partout", "Infiltré", "Fantôme"],
    ),
    (
        Voie::Investigation,
        [
            "Fin limier",
            "Spécialiste",
            "Intuitif",
            "Mémoire vivante",
            "Expert",
        ],
    ),
    (
        Voie::Langues,
        [
            "Bilingue",
            "Philologue",
            "Linguiste",
            "Cryptographe",
            "Hyperpolyglotte",
        ],
    ),
    (
        Voie::Medecine,
        [
            "Secouriste",
            "Médecin",
            "Chirurgien",
            "Homme-médecine",
            "Professeur",
        ],
    ),
    (
        Voie::Psychologie,
        [
            "Observateur",
            "Intuitif",
            "Introspectif",
            "Profileur",
            "Clairvoyant",
        ],
    ),
    (
        Voie::Technologie,
        ["Geek", "Ingénieur", "Connecté", "Optimisateur", "Génie"],
    ),
    (
        Voie::Sciences,
        [
            "Alumni",
            "Professionnel",
            "Esprit pratique",
            "Expert reconnu",
            "Personnalité publique",
        ],
    ),
    (
        Voie::Survie,
        [
            "Voyageur du monde",
            "Explorateur",
            "Baroudeur",
            "Guide",
            "Aventurier",
        ],
    ),
    (
        Voie::Vehicules,
        [
            "Chauffeur",
            "Conducteur expérimenté",
            "Casse-cou",
            "Pilote émérite",
            "As des as",
        ],
    ),
];

/// Voies allowed in total, whatever the age bracket (COC.CRE.MAX-VOIES).
const ABSOLUTE_MAX_VOIES: i32 = 6;

/// Checks the current ranks against the voie structure (ADR KER-VOIE), and
/// the ranks chosen at creation against the creation budgets.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        violations.extend(voie_rank_violations(&voie.id(), &ranks, VOIE_MECHANIC));
    }
    check_max_rank(character, violations);
    check_voie_budget(character, violations);
    check_max_voies(character, violations);
    check_capacity_points(character, violations);
    check_specialities(character, violations);
}

/// COC.CNT.VOIES: a capacité held that asks for a speciality has one.
fn check_specialities(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        for rank in ranks {
            if let Some(asked) = voie.speciality(rank)
                && character.speciality(voie, rank).is_none()
            {
                violations.push(
                    Violation::incomplete(
                        CONTENT_VOIES,
                        VOIE_MECHANIC,
                        Subject::Ability(voie.ability(rank)),
                    )
                    .with_parameter("speciality", asked),
                );
            }
        }
    }
}

/// COC.CRE.CAPACITY-POINTS: no rank above the bracket's maximum.
fn check_max_rank(character: &Character, violations: &mut Vec<Violation>) {
    let Some(max) = character.value(Value::MaxRank).map(|v| v.total()) else {
        return;
    };
    for (voie, ranks) in character.creation_voies() {
        if let Some(highest) = ranks.last().map(|rank| i32::from(rank.get()))
            && highest > max
        {
            violations.push(
                Violation::deviation(
                    CAPACITY_POINTS,
                    VOIE_MECHANIC,
                    Subject::Voie(voie.id()),
                    Severity::Standard,
                )
                .with_parameter("rank", highest)
                .with_parameter("max", max),
            );
        }
    }
}

/// COC.CRE.CAPACITY-POINTS: no more voies than the bracket allows.
fn check_voie_budget(character: &Character, violations: &mut Vec<Violation>) {
    let count = character.voie_count();
    if let Some(max) = character.value(Value::MaxVoies).map(|v| v.total())
        && count > max
    {
        violations.push(
            Violation::deviation(
                CAPACITY_POINTS,
                VOIE_MECHANIC,
                Subject::Value(Value::MaxVoies.id()),
                Severity::Standard,
            )
            .with_parameter("voies", count)
            .with_parameter("max", max),
        );
    }
}

/// COC.CRE.MAX-VOIES: six voies in total, whatever the bracket.
fn check_max_voies(character: &Character, violations: &mut Vec<Violation>) {
    let count = character.voie_count();
    if count > ABSOLUTE_MAX_VOIES {
        violations.push(
            Violation::deviation(
                MAX_VOIES,
                VOIE_MECHANIC,
                Subject::Character,
                Severity::Standard,
            )
            .with_parameter("voies", count)
            .with_parameter("max", ABSOLUTE_MAX_VOIES),
        );
    }
}

/// COC.CRE.CAPACITY-POINTS: the capacity points spent exactly.
fn check_capacity_points(character: &Character, violations: &mut Vec<Violation>) {
    let Some(budget) = character.value(Value::CapacityPoints).map(|v| v.total()) else {
        return;
    };
    let spent = character.capacity_points_spent();
    let subject = Subject::Value(Value::CapacityPoints.id());
    let violation = match spent.cmp(&budget) {
        std::cmp::Ordering::Less => Violation::incomplete(CAPACITY_POINTS, VOIE_MECHANIC, subject),
        std::cmp::Ordering::Greater => {
            Violation::deviation(CAPACITY_POINTS, VOIE_MECHANIC, subject, Severity::Standard)
        }
        std::cmp::Ordering::Equal => return,
    };
    violations.push(
        violation
            .with_parameter("spent", spent)
            .with_parameter("budget", budget),
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use co_kernel::Category;

    use super::*;

    /// The voie violations besides the specialities, tested on their own.
    fn voie_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations.retain(|v| v.rule() != &CONTENT_VOIES);
        violations
    }

    fn speciality_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check_specialities(character, &mut violations);
        violations
    }

    #[test]
    fn capacites_name_the_speciality_they_ask_for() {
        let rank = Rank::new;
        assert_eq!(
            Voie::ArtsEtArtisanats.speciality(rank(1)),
            Some("discipline")
        );
        assert_eq!(
            Voie::Medecine.speciality(rank(4)),
            Some("médecine traditionnelle")
        );
        assert_eq!(Voie::Medecine.speciality(rank(1)), None);
        assert_eq!(Voie::Crime.speciality(rank(1)), None);
        let asking = Voie::ALL
            .iter()
            .flat_map(|voie| (1..=5).filter(|r| voie.speciality(rank(*r)).is_some()))
            .count();
        assert_eq!(asking, 42);
    }

    #[test]
    fn a_missing_speciality_is_incomplete() {
        let character = ranks(Character::new(), Voie::Discours, 2);
        let violations = speciality_violations(&character);
        assert_eq!(violations.len(), 2);
        let first = &violations[0];
        assert_eq!(first.rule(), &CONTENT_VOIES);
        assert_eq!(first.category(), Category::Incomplete);
        assert_eq!(
            first.subject(),
            &Subject::Ability(Voie::Discours.ability(Rank::new(1)))
        );
        assert_eq!(
            first
                .parameters()
                .first()
                .map(|p| (p.name(), p.value().clone())),
            Some(("speciality", "approche".into()))
        );
        let written = character
            .with_speciality(Voie::Discours, 1, "persuasion")
            .with_speciality(Voie::Discours, 2, "négociation");
        assert_eq!(speciality_violations(&written), []);
        let erased = written.without_speciality(Voie::Discours, 2);
        assert_eq!(speciality_violations(&erased).len(), 1);
    }

    #[test]
    fn a_speciality_without_its_capacite_is_kept_silently() {
        let character = Character::new().with_speciality(Voie::Discours, 1, "persuasion");
        assert_eq!(speciality_violations(&character), []);
        assert_eq!(
            character
                .speciality(Voie::Discours, Rank::new(1))
                .map(co_kernel::Speciality::as_str),
            Some("persuasion")
        );
    }

    fn ranks(character: Character, voie: Voie, highest: u8) -> Character {
        (1..=highest).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    /// A 31-50 character spending exactly its 7 points on 5 voies.
    fn adult() -> Character {
        let character = Character::new().with_age(40);
        let character = ranks(character, Voie::Corporations, 1);
        let character = ranks(character, Voie::ArmesDeContact, 1);
        let character = ranks(character, Voie::Sciences, 1);
        let character = ranks(character, Voie::Survie, 2);
        ranks(character, Voie::Vehicules, 2)
    }

    fn rules(violations: &[Violation]) -> Vec<(&str, Category)> {
        violations
            .iter()
            .map(|v| (v.rule().as_str(), v.category()))
            .collect()
    }

    #[test]
    fn spending_the_budget_within_limits_is_clean() {
        assert_eq!(voie_violations(&adult()), []);
    }

    #[test]
    fn ability_ids_name_voie_and_rank() {
        assert_eq!(Voie::Crime.ability(Rank::new(2)).as_str(), "coc.crime.2");
        assert_eq!(Voie::Survie.id().as_str(), "coc.survie");
        assert_eq!(
            Voie::from_ability_id("coc.crime.5"),
            Some((Voie::Crime, Rank::new(5)))
        );
        for bad in ["coc.crime", "coc.crime.x", "coc.unknown.1", ""] {
            assert_eq!(Voie::from_ability_id(bad), None, "{bad}");
        }
    }

    #[test]
    fn every_capacite_is_named() {
        for voie in Voie::ALL {
            let names: BTreeSet<_> = (1..=5)
                .map(|rank| voie.ability_name(Rank::new(rank)))
                .collect();
            assert_eq!(names.len(), 5, "{voie:?}");
            assert!(names.iter().all(|name| name.is_some_and(|n| !n.is_empty())));
            assert_eq!(voie.ability_name(Rank::new(0)), None);
            assert_eq!(voie.ability_name(Rank::new(6)), None);
        }
        assert_eq!(
            Voie::ArmesDeContact.ability_name(Rank::new(4)),
            Some("Fin bretteur")
        );
        assert_eq!(Voie::Crime.ability_name(Rank::new(2)), Some("Gangster"));
    }

    #[test]
    fn unspent_points_are_incomplete() {
        let character = ranks(Character::new().with_age(40), Voie::Survie, 1);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("COC.CRE.CAPACITY-POINTS", Category::Incomplete)]
        );
    }

    #[test]
    fn overspent_points_deviate() {
        let character = ranks(adult(), Voie::Corporations, 2);
        assert_eq!(
            rules(&voie_violations(&character)),
            [(
                "COC.CRE.CAPACITY-POINTS",
                Category::Deviation(Severity::Standard)
            )]
        );
    }

    #[test]
    fn gap_breaks_the_rank_order() {
        let character = Character::new()
            .with_age(20)
            .with_rank(Voie::Crime, 2)
            .with_rank(Voie::Survie, 1)
            .with_rank(Voie::Survie, 2)
            .with_rank(Voie::Langues, 1);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [("CO.VOIE.ORDER", Category::Deviation(Severity::Standard))]
        );
        assert_eq!(violations[0].subject(), &Subject::Voie(Voie::Crime.id()));
    }

    #[test]
    fn several_gaps_in_a_voie_make_one_violation() {
        let character = Character::new().with_rank(Voie::Crime, 200);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [
                ("CO.VOIE.STRUCTURE", Category::Deviation(Severity::Standard)),
                ("CO.VOIE.ORDER", Category::Deviation(Severity::Standard)),
            ]
        );
        let parameters: Vec<_> = violations[1]
            .parameters()
            .iter()
            .map(|p| p.value().clone())
            .collect();
        assert_eq!(
            parameters,
            [
                co_kernel::ParameterValue::Integer(1),
                co_kernel::ParameterValue::Integer(199)
            ]
        );
    }

    #[test]
    fn rank_outside_the_structure_deviates() {
        let character = ranks(Character::new(), Voie::Crime, 5).with_rank(Voie::Crime, 6);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("CO.VOIE.STRUCTURE", Category::Deviation(Severity::Standard))]
        );
    }

    #[test]
    fn rank_above_the_bracket_maximum_deviates() {
        let character =
            ranks(Character::new().with_age(20), Voie::Crime, 3).with_rank(Voie::Survie, 1);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [(
                "COC.CRE.CAPACITY-POINTS",
                Category::Deviation(Severity::Standard)
            )]
        );
        assert_eq!(violations[0].subject(), &Subject::Voie(Voie::Crime.id()));
    }

    #[test]
    fn too_many_voies_deviate_twice_beyond_six() {
        let voies = [
            Voie::Crime,
            Voie::Discours,
            Voie::Furtivite,
            Voie::Langues,
            Voie::Medecine,
            Voie::Survie,
            Voie::Vehicules,
        ];
        let character = voies
            .into_iter()
            .fold(Character::new(), |c, voie| c.with_rank(voie, 1));
        let character = character.with_age(60);
        let subjects: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.subject().clone()))
            .collect();
        assert_eq!(
            subjects,
            [
                (
                    "COC.CRE.CAPACITY-POINTS".to_owned(),
                    Subject::Value(Value::MaxVoies.id())
                ),
                ("COC.CRE.MAX-VOIES".to_owned(), Subject::Character),
                (
                    "COC.CRE.CAPACITY-POINTS".to_owned(),
                    Subject::Value(Value::CapacityPoints.id())
                ),
            ]
        );
    }

    #[test]
    fn budgets_are_not_checked_without_a_bracket() {
        let character = ranks(Character::new(), Voie::Crime, 4);
        assert_eq!(voie_violations(&character), []);
    }
}
