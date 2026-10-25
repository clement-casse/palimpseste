//! Voies (COF.CNT.PROFILS, COF.CNT.PEUPLES) and the checks on the ranks
//! chosen at creation, at level 1.

use co_kernel::{AbilityId, Rank, Severity, Subject, Violation, VoieId, voie_rank_violations};

use crate::{
    Action, Character, Family, Profil, action, level,
    rules::{LEVEL_REQ, MAGE_EXTRA, PEUPLE_VOIE, PROFIL_VOIES, VOIE_DU_MAGE, VOIE_MECHANIC, VOIES},
};

/// One of the 78 voies a level-1 COF2 character can acquire: the five voies
/// of each profil, the voies de peuple and the voie du mage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Voie {
    /// Voie de l'artilleur (arquebusier).
    Artilleur,
    /// Voie des explosifs (arquebusier).
    Explosifs,
    /// Voie du mercenaire (arquebusier).
    Mercenaire,
    /// Voie du pistolero (arquebusier).
    Pistolero,
    /// Voie de la précision (arquebusier).
    Precision,
    /// Voie de l'escrime (barde).
    Escrime,
    /// Voie du musicien (barde).
    Musicien,
    /// Voie du saltimbanque (barde).
    Saltimbanque,
    /// Voie de la séduction (barde).
    Seduction,
    /// Voie du vagabond (barde).
    Vagabond,
    /// Voie de l'archer (rôdeur).
    Archer,
    /// Voie du compagnon animal (rôdeur).
    CompagnonAnimal,
    /// Voie de la survie (rôdeur).
    Survie,
    /// Voie du traqueur (rôdeur).
    Traqueur,
    /// Voie du combat à deux armes (rôdeur).
    CombatADeuxArmes,
    /// Voie de l'assassin (voleur).
    Assassin,
    /// Voie de l'aventurier (voleur).
    Aventurier,
    /// Voie du déplacement (voleur).
    Deplacement,
    /// Voie du roublard (voleur).
    Roublard,
    /// Voie du spadassin (voleur).
    Spadassin,
    /// Voie de la brute (barbare).
    Brute,
    /// Voie du pagne (barbare).
    Pagne,
    /// Voie du pourfendeur (barbare).
    Pourfendeur,
    /// Voie du primitif (barbare).
    Primitif,
    /// Voie de la rage (barbare).
    Rage,
    /// Voie du cavalier (chevalier).
    Cavalier,
    /// Voie de la guerre (chevalier).
    Guerre,
    /// Voie du preux (chevalier).
    Preux,
    /// Voie du meneur d'homme (chevalier).
    MeneurDHomme,
    /// Voie de la noblesse (chevalier).
    Noblesse,
    /// Voie du bouclier (guerrier).
    Bouclier,
    /// Voie du combat (guerrier).
    Combat,
    /// Voie du maître d'armes (guerrier).
    MaitreDArmes,
    /// Voie de la résistance (guerrier).
    Resistance,
    /// Voie du soldat (guerrier).
    Soldat,
    /// Voie de l'air (ensorceleur).
    Air,
    /// Voie de la divination (ensorceleur).
    Divination,
    /// Voie de l'envoûteur (ensorceleur).
    Envouteur,
    /// Voie des illusions (ensorceleur).
    Illusions,
    /// Voie de l'invocation (ensorceleur).
    Invocation,
    /// Voie des artefacts (forgesort).
    Artefacts,
    /// Voie des élixirs (forgesort).
    Elixirs,
    /// Voie du métal (forgesort).
    Metal,
    /// Voie du golem (forgesort).
    Golem,
    /// Voie des runes (forgesort).
    Runes,
    /// Voie de la magie des arcanes (magicien).
    MagieDesArcanes,
    /// Voie de la magie destructrice (magicien).
    MagieDestructrice,
    /// Voie de la magie élémentaire (magicien).
    MagieElementaire,
    /// Voie de la magie protectrice (magicien).
    MagieProtectrice,
    /// Voie de la magie universelle (magicien).
    MagieUniverselle,
    /// Voie du démon (sorcier).
    Demon,
    /// Voie de la mort (sorcier).
    Mort,
    /// Voie de l'outre-tombe (sorcier).
    OutreTombe,
    /// Voie du sang (sorcier).
    Sang,
    /// Voie de la sombre magie (sorcier).
    SombreMagie,
    /// Voie des animaux (druide).
    Animaux,
    /// Voie du fauve (druide).
    Fauve,
    /// Voie de la nature (druide).
    Nature,
    /// Voie du protecteur (druide).
    Protecteur,
    /// Voie des végétaux (druide).
    Vegetaux,
    /// Voie de l'énergie vitale (moine).
    EnergieVitale,
    /// Voie de la maîtrise (moine).
    Maitrise,
    /// Voie de la méditation (moine).
    Meditation,
    /// Voie du poing (moine).
    Poing,
    /// Voie du vent (moine).
    Vent,
    /// Voie de la foi (prêtre).
    Foi,
    /// Voie de la guerre sainte (prêtre).
    GuerreSainte,
    /// Voie de la prière (prêtre).
    Priere,
    /// Voie des soins (prêtre).
    Soins,
    /// Voie de la spiritualité (prêtre).
    Spiritualite,
    /// Voie du demi-orc (voie de peuple).
    DemiOrc,
    /// Voie de l'elfe haut (voie de peuple).
    ElfeHaut,
    /// Voie de l'elfe sylvain (voie de peuple).
    ElfeSylvain,
    /// Voie du gnome (voie de peuple).
    Gnome,
    /// Voie du halfelin (voie de peuple).
    Halfelin,
    /// Voie de l'humain (voie de peuple).
    Humain,
    /// Voie du nain (voie de peuple).
    Nain,
    /// Voie du mage (mage family, replaces the voie de peuple).
    Mage,
}

impl Voie {
    /// Every voie, in declaration order: the profil voies by profil, the
    /// voies de peuple, the voie du mage.
    pub const ALL: [Self; 78] = [
        Self::Artilleur,
        Self::Explosifs,
        Self::Mercenaire,
        Self::Pistolero,
        Self::Precision,
        Self::Escrime,
        Self::Musicien,
        Self::Saltimbanque,
        Self::Seduction,
        Self::Vagabond,
        Self::Archer,
        Self::CompagnonAnimal,
        Self::Survie,
        Self::Traqueur,
        Self::CombatADeuxArmes,
        Self::Assassin,
        Self::Aventurier,
        Self::Deplacement,
        Self::Roublard,
        Self::Spadassin,
        Self::Brute,
        Self::Pagne,
        Self::Pourfendeur,
        Self::Primitif,
        Self::Rage,
        Self::Cavalier,
        Self::Guerre,
        Self::Preux,
        Self::MeneurDHomme,
        Self::Noblesse,
        Self::Bouclier,
        Self::Combat,
        Self::MaitreDArmes,
        Self::Resistance,
        Self::Soldat,
        Self::Air,
        Self::Divination,
        Self::Envouteur,
        Self::Illusions,
        Self::Invocation,
        Self::Artefacts,
        Self::Elixirs,
        Self::Metal,
        Self::Golem,
        Self::Runes,
        Self::MagieDesArcanes,
        Self::MagieDestructrice,
        Self::MagieElementaire,
        Self::MagieProtectrice,
        Self::MagieUniverselle,
        Self::Demon,
        Self::Mort,
        Self::OutreTombe,
        Self::Sang,
        Self::SombreMagie,
        Self::Animaux,
        Self::Fauve,
        Self::Nature,
        Self::Protecteur,
        Self::Vegetaux,
        Self::EnergieVitale,
        Self::Maitrise,
        Self::Meditation,
        Self::Poing,
        Self::Vent,
        Self::Foi,
        Self::GuerreSainte,
        Self::Priere,
        Self::Soins,
        Self::Spiritualite,
        Self::DemiOrc,
        Self::ElfeHaut,
        Self::ElfeSylvain,
        Self::Gnome,
        Self::Halfelin,
        Self::Humain,
        Self::Nain,
        Self::Mage,
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
            Self::Artilleur => "Voie de l'artilleur",
            Self::Explosifs => "Voie des explosifs",
            Self::Mercenaire => "Voie du mercenaire",
            Self::Pistolero => "Voie du pistolero",
            Self::Precision => "Voie de la précision",
            Self::Escrime => "Voie de l'escrime",
            Self::Musicien => "Voie du musicien",
            Self::Saltimbanque => "Voie du saltimbanque",
            Self::Seduction => "Voie de la séduction",
            Self::Vagabond => "Voie du vagabond",
            Self::Archer => "Voie de l'archer",
            Self::CompagnonAnimal => "Voie du compagnon animal",
            Self::Survie => "Voie de la survie",
            Self::Traqueur => "Voie du traqueur",
            Self::CombatADeuxArmes => "Voie du combat à deux armes",
            Self::Assassin => "Voie de l'assassin",
            Self::Aventurier => "Voie de l'aventurier",
            Self::Deplacement => "Voie du déplacement",
            Self::Roublard => "Voie du roublard",
            Self::Spadassin => "Voie du spadassin",
            Self::Brute => "Voie de la brute",
            Self::Pagne => "Voie du pagne",
            Self::Pourfendeur => "Voie du pourfendeur",
            Self::Primitif => "Voie du primitif",
            Self::Rage => "Voie de la rage",
            Self::Cavalier => "Voie du cavalier",
            Self::Guerre => "Voie de la guerre",
            Self::Preux => "Voie du preux",
            Self::MeneurDHomme => "Voie du meneur d'homme",
            Self::Noblesse => "Voie de la noblesse",
            Self::Bouclier => "Voie du bouclier",
            Self::Combat => "Voie du combat",
            Self::MaitreDArmes => "Voie du maître d'armes",
            Self::Resistance => "Voie de la résistance",
            Self::Soldat => "Voie du soldat",
            Self::Air => "Voie de l'air",
            Self::Divination => "Voie de la divination",
            Self::Envouteur => "Voie de l'envoûteur",
            Self::Illusions => "Voie des illusions",
            Self::Invocation => "Voie de l'invocation",
            Self::Artefacts => "Voie des artefacts",
            Self::Elixirs => "Voie des élixirs",
            Self::Metal => "Voie du métal",
            Self::Golem => "Voie du golem",
            Self::Runes => "Voie des runes",
            Self::MagieDesArcanes => "Voie de la magie des arcanes",
            Self::MagieDestructrice => "Voie de la magie destructrice",
            Self::MagieElementaire => "Voie de la magie élémentaire",
            Self::MagieProtectrice => "Voie de la magie protectrice",
            Self::MagieUniverselle => "Voie de la magie universelle",
            Self::Demon => "Voie du démon",
            Self::Mort => "Voie de la mort",
            Self::OutreTombe => "Voie de l'outre-tombe",
            Self::Sang => "Voie du sang",
            Self::SombreMagie => "Voie de la sombre magie",
            Self::Animaux => "Voie des animaux",
            Self::Fauve => "Voie du fauve",
            Self::Nature => "Voie de la nature",
            Self::Protecteur => "Voie du protecteur",
            Self::Vegetaux => "Voie des végétaux",
            Self::EnergieVitale => "Voie de l'énergie vitale",
            Self::Maitrise => "Voie de la maîtrise",
            Self::Meditation => "Voie de la méditation",
            Self::Poing => "Voie du poing",
            Self::Vent => "Voie du vent",
            Self::Foi => "Voie de la foi",
            Self::GuerreSainte => "Voie de la guerre sainte",
            Self::Priere => "Voie de la prière",
            Self::Soins => "Voie des soins",
            Self::Spiritualite => "Voie de la spiritualité",
            Self::DemiOrc => "Voie du demi-orc",
            Self::ElfeHaut => "Voie de l'elfe haut",
            Self::ElfeSylvain => "Voie de l'elfe sylvain",
            Self::Gnome => "Voie du gnome",
            Self::Halfelin => "Voie du halfelin",
            Self::Humain => "Voie de l'humain",
            Self::Nain => "Voie du nain",
            Self::Mage => "Voie du mage",
        }
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
    /// (COF.CNT.VOIES, D-45); `None` outside ranks 1 to 5. The voie du
    /// mage's rank 1 is Occultisme: the peuple capacité it keeps has its own
    /// name, in the peuple's voie.
    #[must_use]
    pub fn ability_name(self, rank: Rank) -> Option<&'static str> {
        let index = usize::from(rank.get()).checked_sub(1)?;
        ABILITY_NAMES
            .iter()
            .find(|(voie, _)| *voie == self)
            .and_then(|(_, names)| names.get(index).copied())
    }

    /// The profil the voie belongs to: `None` for a voie de peuple and the
    /// voie du mage.
    #[must_use]
    pub fn profil(self) -> Option<Profil> {
        Profil::ALL
            .into_iter()
            .find(|profil| profil.voies().contains(&self))
    }

    /// Whether the voie fills the voie de peuple slot: a voie de peuple, or
    /// the voie du mage replacing it.
    pub(crate) const fn is_peuple_slot(self) -> bool {
        matches!(
            self,
            Self::DemiOrc
                | Self::ElfeHaut
                | Self::ElfeSylvain
                | Self::Gnome
                | Self::Halfelin
                | Self::Humain
                | Self::Nain
                | Self::Mage
        )
    }

    /// The actions the capacité at `rank` may take to use, the first usual
    /// (COF.CNT.VOIE): empty when it is passive, and outside ranks 1 to 5.
    #[must_use]
    pub fn actions(self, rank: Rank) -> &'static [Action] {
        action::of(self, rank)
    }

    /// What the capacité at `rank` asks the player to choose, in French
    /// (`origine`), as a speciality written with
    /// [`Character::with_speciality`](crate::Character::with_speciality);
    /// `None` when it asks for none (COF.CNT.VOIE).
    #[must_use]
    pub const fn speciality(self, rank: Rank) -> Option<&'static str> {
        match (self, rank.get()) {
            (Self::Humain, 1) => Some("origine"),
            _ => None,
        }
    }

    /// The PM the capacité at `rank` costs to cast, its rank, when it is a
    /// spell (COF.CNT.VOIE); `None` otherwise.
    #[must_use]
    pub fn spell_cost(self, rank: Rank) -> Option<i32> {
        self.is_spell(rank).then(|| i32::from(rank.get()))
    }

    /// Whether the capacité at `rank` is a spell (the `*` of the co-rules
    /// catalogue, D-23).
    #[must_use]
    pub fn is_spell(self, rank: Rank) -> bool {
        matches!(
            (self, rank.get()),
            (
                Self::Musicien
                    | Self::Air
                    | Self::Envouteur
                    | Self::Illusions
                    | Self::Invocation
                    | Self::MagieDesArcanes
                    | Self::MagieDestructrice
                    | Self::MagieElementaire
                    | Self::MagieProtectrice
                    | Self::MagieUniverselle
                    | Self::Vegetaux,
                1..=5
            ) | (Self::Seduction, 5)
                | (Self::Vagabond, 4 | 5)
                | (Self::Divination | Self::Priere, 1..=3)
                | (Self::Artefacts, 2 | 4)
                | (Self::Metal | Self::SombreMagie, 1..=4)
                | (Self::Golem, 4)
                | (Self::Runes | Self::Mort | Self::Foi, 2..=5)
                | (Self::Demon | Self::Sang, 1 | 2 | 4 | 5)
                | (Self::OutreTombe, 1 | 3..=5)
                | (Self::Animaux, 3..=5)
                | (Self::Protecteur | Self::Soins, 1..=3 | 5)
                | (Self::GuerreSainte, 1 | 4 | 5)
                | (Self::Spiritualite, 2 | 3 | 5)
                | (Self::Mage, 2 | 3)
        )
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Artilleur => "cof.artilleur",
            Self::Explosifs => "cof.explosifs",
            Self::Mercenaire => "cof.mercenaire",
            Self::Pistolero => "cof.pistolero",
            Self::Precision => "cof.precision",
            Self::Escrime => "cof.escrime",
            Self::Musicien => "cof.musicien",
            Self::Saltimbanque => "cof.saltimbanque",
            Self::Seduction => "cof.seduction",
            Self::Vagabond => "cof.vagabond",
            Self::Archer => "cof.archer",
            Self::CompagnonAnimal => "cof.compagnon-animal",
            Self::Survie => "cof.survie",
            Self::Traqueur => "cof.traqueur",
            Self::CombatADeuxArmes => "cof.combat-a-deux-armes",
            Self::Assassin => "cof.assassin",
            Self::Aventurier => "cof.aventurier",
            Self::Deplacement => "cof.deplacement",
            Self::Roublard => "cof.roublard",
            Self::Spadassin => "cof.spadassin",
            Self::Brute => "cof.brute",
            Self::Pagne => "cof.pagne",
            Self::Pourfendeur => "cof.pourfendeur",
            Self::Primitif => "cof.primitif",
            Self::Rage => "cof.rage",
            Self::Cavalier => "cof.cavalier",
            Self::Guerre => "cof.guerre",
            Self::Preux => "cof.preux",
            Self::MeneurDHomme => "cof.meneur-d-homme",
            Self::Noblesse => "cof.noblesse",
            Self::Bouclier => "cof.bouclier",
            Self::Combat => "cof.combat",
            Self::MaitreDArmes => "cof.maitre-d-armes",
            Self::Resistance => "cof.resistance",
            Self::Soldat => "cof.soldat",
            Self::Air => "cof.air",
            Self::Divination => "cof.divination",
            Self::Envouteur => "cof.envouteur",
            Self::Illusions => "cof.illusions",
            Self::Invocation => "cof.invocation",
            Self::Artefacts => "cof.artefacts",
            Self::Elixirs => "cof.elixirs",
            Self::Metal => "cof.metal",
            Self::Golem => "cof.golem",
            Self::Runes => "cof.runes",
            Self::MagieDesArcanes => "cof.magie-des-arcanes",
            Self::MagieDestructrice => "cof.magie-destructrice",
            Self::MagieElementaire => "cof.magie-elementaire",
            Self::MagieProtectrice => "cof.magie-protectrice",
            Self::MagieUniverselle => "cof.magie-universelle",
            Self::Demon => "cof.demon",
            Self::Mort => "cof.mort",
            Self::OutreTombe => "cof.outre-tombe",
            Self::Sang => "cof.sang",
            Self::SombreMagie => "cof.sombre-magie",
            Self::Animaux => "cof.animaux",
            Self::Fauve => "cof.fauve",
            Self::Nature => "cof.nature",
            Self::Protecteur => "cof.protecteur",
            Self::Vegetaux => "cof.vegetaux",
            Self::EnergieVitale => "cof.energie-vitale",
            Self::Maitrise => "cof.maitrise",
            Self::Meditation => "cof.meditation",
            Self::Poing => "cof.poing",
            Self::Vent => "cof.vent",
            Self::Foi => "cof.foi",
            Self::GuerreSainte => "cof.guerre-sainte",
            Self::Priere => "cof.priere",
            Self::Soins => "cof.soins",
            Self::Spiritualite => "cof.spiritualite",
            Self::DemiOrc => "cof.demi-orc",
            Self::ElfeHaut => "cof.elfe-haut",
            Self::ElfeSylvain => "cof.elfe-sylvain",
            Self::Gnome => "cof.gnome",
            Self::Halfelin => "cof.halfelin",
            Self::Humain => "cof.humain",
            Self::Nain => "cof.nain",
            Self::Mage => "cof.mage",
        }
    }
}

/// The names of the capacités of each voie, ranks 1 to 5 (COF.CNT.VOIES, D-45).
const ABILITY_NAMES: [(Voie, [&str; 5]); 78] = [
    (
        Voie::Artilleur,
        [
            "Mécanismes",
            "Arme à répétition",
            "Tir de barrage",
            "Canon double",
            "Couleuvrine",
        ],
    ),
    (
        Voie::Explosifs,
        [
            "Tir de grenaille",
            "Démolition",
            "Poudre puissante",
            "Piège explosif",
            "Boulet explosif",
        ],
    ),
    (
        Voie::Mercenaire,
        [
            "Pilier de bar",
            "Mort ou vif",
            "Combattant aguerri",
            "Constitution héroïque",
            "Combat de masse",
        ],
    ),
    (
        Voie::Pistolero,
        [
            "Plus vite que son ombre",
            "Ajuster le tir",
            "Tir double",
            "Agilité héroïque",
            "As de la gâchette",
        ],
    ),
    (
        Voie::Precision,
        [
            "Joli coup",
            "Défaut dans la cuirasse",
            "Tir précis",
            "Tireur d'élite",
            "Tir fatal",
        ],
    ),
    (
        Voie::Escrime,
        [
            "Précision",
            "Feinte",
            "Intelligence du combat",
            "Attaque flamboyante",
            "Botte mortelle",
        ],
    ),
    (
        Voie::Musicien,
        [
            "Chant des héros",
            "Chant de réconfort",
            "Attaque sonore",
            "Zone de silence",
            "Danse irrésistible",
        ],
    ),
    (
        Voie::Saltimbanque,
        [
            "Acrobate",
            "Grâce féline",
            "Lanceur de couteau",
            "Liberté d'action",
            "Esquive acrobatique",
        ],
    ),
    (
        Voie::Seduction,
        [
            "Charmant",
            "Dentelles et rapière",
            "Baratineur de génie",
            "Charisme héroïque",
            "Suggestion",
        ],
    ),
    (
        Voie::Vagabond,
        [
            "Rumeurs et légendes",
            "Éclectique",
            "Attirail",
            "Compréhension des langues",
            "Déguisement",
        ],
    ),
    (
        Voie::Archer,
        [
            "Archer émérite",
            "Tir chirurgical",
            "Dans le mille",
            "Tir rapide",
            "Flèche de mort",
        ],
    ),
    (
        Voie::CompagnonAnimal,
        [
            "Le loup",
            "Travail d'équipe",
            "Lien empathique",
            "Loup alpha",
            "Tactiques de meute",
        ],
    ),
    (
        Voie::Survie,
        [
            "Survie",
            "Nature nourricière",
            "Grand pas",
            "Constitution héroïque",
            "Increvable",
        ],
    ),
    (
        Voie::Traqueur,
        [
            "Éclaireur",
            "Attaque éclair",
            "Chasseur émérite",
            "Perception héroïque",
            "Repli",
        ],
    ),
    (
        Voie::CombatADeuxArmes,
        [
            "Attaque à suivre",
            "Parade croisée",
            "Droite - gauche",
            "Combattant héroïque",
            "Double peine",
        ],
    ),
    (
        Voie::Assassin,
        [
            "Discrétion",
            "Attaque sournoise",
            "Attaque par surprise",
            "Disparition",
            "Ouverture mortelle",
        ],
    ),
    (
        Voie::Aventurier,
        [
            "Baratin",
            "Provocation",
            "Souplesse du félin",
            "Charisme héroïque",
            "Attaque paralysante",
        ],
    ),
    (
        Voie::Deplacement,
        [
            "Agile",
            "Réflexes félins",
            "Acrobaties",
            "Agilité héroïque",
            "Esquive de la magie",
        ],
    ),
    (
        Voie::Roublard,
        [
            "Doigts agiles",
            "Aux aguets",
            "Feindre la mort",
            "Expert en criminalité",
            "Maître du poison",
        ],
    ),
    (
        Voie::Spadassin,
        [
            "Attaque en finesse",
            "Esquive fatale",
            "Frappe chirurgicale",
            "Ambidextrie",
            "Botte secrète",
        ],
    ),
    (
        Voie::Brute,
        [
            "Argument de taille",
            "Tour de force",
            "Attaque brutale",
            "Force héroïque",
            "Briseur d'os",
        ],
    ),
    (
        Voie::Pagne,
        [
            "Vigueur",
            "Peau de pierre",
            "Tatouages",
            "Constitution héroïque",
            "Peau d'acier",
        ],
    ),
    (
        Voie::Pourfendeur,
        [
            "Réflexes éclair",
            "Charge",
            "Enchaînement",
            "Déchaînement d'acier",
            "Attaque tourbillon",
        ],
    ),
    (
        Voie::Primitif,
        [
            "Proche de la nature",
            "Armure de vent",
            "Vigilance",
            "Résistance à la magie",
            "Vitalité débordante",
        ],
    ),
    (
        Voie::Rage,
        [
            "Cri de guerre",
            "Défier la mort",
            "Rage du berserk",
            "Même pas mal",
            "Furie du berserk",
        ],
    ),
    (
        Voie::Cavalier,
        [
            "Fidèle monture",
            "Cavalier émérite",
            "Charge",
            "Monture magique",
            "Monture fantastique",
        ],
    ),
    (
        Voie::Guerre,
        [
            "Armure sur mesure",
            "Encaisser un coup",
            "Frappe du justicier",
            "Force héroïque",
            "Mon armure est une arme",
        ],
    ),
    (
        Voie::Preux,
        [
            "Ignorer la douleur",
            "Piqûres d'insectes",
            "Laissez-le-moi",
            "Charisme héroïque",
            "Seul contre tous",
        ],
    ),
    (
        Voie::MeneurDHomme,
        [
            "Sans peur",
            "Intercepter",
            "Exemplaire",
            "Charge fantastique",
            "Ordre de bataille",
        ],
    ),
    (
        Voie::Noblesse,
        [
            "Éduqué",
            "Écuyer",
            "Autorité naturelle",
            "Massacrer la piétaille",
            "Formation d'élite",
        ],
    ),
    (
        Voie::Bouclier,
        [
            "Protéger un allié",
            "Parer un coup",
            "Défense au bouclier",
            "Absorber un sort",
            "Renvoi de sort",
        ],
    ),
    (
        Voie::Combat,
        [
            "Vivacité",
            "Manœuvre",
            "Attaque puissante",
            "Double attaque",
            "Attaque circulaire",
        ],
    ),
    (
        Voie::MaitreDArmes,
        [
            "Armes de prédilection",
            "Science du critique",
            "Spécialisation",
            "Attaque parfaite",
            "Riposte",
        ],
    ),
    (
        Voie::Resistance,
        [
            "Robustesse",
            "Résilient",
            "Armure lourde",
            "Constitution héroïque",
            "Dur à cuire",
        ],
    ),
    (
        Voie::Soldat,
        [
            "Teigneux",
            "Prouesse",
            "Piqûre de rappel",
            "Force héroïque",
            "Rempart",
        ],
    ),
    (
        Voie::Air,
        [
            "Murmures dans le vent",
            "Sous tension",
            "Télékinésie",
            "Foudre",
            "Forme éthérée",
        ],
    ),
    (
        Voie::Divination,
        [
            "Divination",
            "Détection de l'invisible",
            "Clairvoyance",
            "Perception héroïque",
            "Prescience",
        ],
    ),
    (
        Voie::Envouteur,
        ["Injonction", "Sommeil", "Confusion", "Amitié", "Domination"],
    ),
    (
        Voie::Illusions,
        [
            "Mirage",
            "Image décalée",
            "Sort illusoire",
            "Imitation",
            "Exécution mentale",
        ],
    ),
    (
        Voie::Invocation,
        [
            "Choc",
            "Serviteur invisible",
            "Arme de mana",
            "Porte dimensionnelle",
            "Mur de mana",
        ],
    ),
    (
        Voie::Artefacts,
        [
            "Bâton de mage",
            "Ouverture - fermeture",
            "Sac sans fond",
            "Frappe des arcanes",
            "Artefact étrange",
        ],
    ),
    (
        Voie::Elixirs,
        [
            "Fortifiant",
            "Feu grégeois",
            "Élixir de guérison",
            "Élixirs mineurs",
            "Élixirs majeurs",
        ],
    ),
    (
        Voie::Metal,
        [
            "Morsure de la forge",
            "Métal brûlant",
            "Magnétisme",
            "Métal hurlant",
            "Endurer",
        ],
    ),
    (
        Voie::Golem,
        [
            "Grosse tête",
            "Golem",
            "Protecteur",
            "Statuette",
            "Golem supérieur",
        ],
    ),
    (
        Voie::Runes,
        [
            "Runes de défense",
            "Rune de puissance",
            "Rune de protection",
            "Rune d'énergie",
            "Rune de garde",
        ],
    ),
    (
        Voie::MagieDesArcanes,
        [
            "Projectile de mana",
            "Lévitation",
            "Forme gazeuse",
            "Accélération",
            "Désintégration",
        ],
    ),
    (
        Voie::MagieDestructrice,
        [
            "Arc de feu",
            "Saper les forces",
            "Flèche de feu",
            "Explosion de feu",
            "Appel de la foudre",
        ],
    ),
    (
        Voie::MagieElementaire,
        [
            "Asphyxie",
            "Maîtrise des éléments",
            "Arme élémentaire",
            "Respiration aquatique",
            "Armure de pierre",
        ],
    ),
    (
        Voie::MagieProtectrice,
        [
            "Armure de mana",
            "Chute ralentie",
            "Déphasage",
            "Cercle de protection",
            "Interruption du temps",
        ],
    ),
    (
        Voie::MagieUniverselle,
        [
            "Lumière",
            "Familier",
            "Invisibilité",
            "Vol",
            "Téléportation",
        ],
    ),
    (
        Voie::Demon,
        [
            "Malédiction",
            "Beauté de la succube",
            "Pacte démoniaque",
            "Aspect du démon",
            "Invocation d'un démon",
        ],
    ),
    (
        Voie::Mort,
        [
            "Siphon des âmes",
            "Masque mortuaire",
            "Baiser du vampire",
            "Peur",
            "Briser les cœurs",
        ],
    ),
    (
        Voie::OutreTombe,
        [
            "Un pied dans la tombe",
            "Armure d'os",
            "Animation des morts",
            "Ensevelissement",
            "Armée des morts",
        ],
    ),
    (
        Voie::Sang,
        [
            "Saignements",
            "Sang mordant",
            "Exsangue",
            "Rituel de sang",
            "Lien de sang",
        ],
    ),
    (
        Voie::SombreMagie,
        [
            "Ténèbres",
            "Reptation",
            "Strangulation",
            "Manteau d'ombre",
            "Pacte ténébreux",
        ],
    ),
    (
        Voie::Animaux,
        [
            "Langage des animaux",
            "Petit compagnon",
            "Nuée d'insectes",
            "Masque du prédateur",
            "Forme animale",
        ],
    ),
    (
        Voie::Fauve,
        [
            "Vitesse du félin",
            "Panthère",
            "Attaque bondissante",
            "Grand félin",
            "Les sept vies du chat",
        ],
    ),
    (
        Voie::Nature,
        [
            "Maître de la survie",
            "Terrains difficiles",
            "Bâton de druide",
            "Constitution héroïque",
            "Résistant",
        ],
    ),
    (
        Voie::Protecteur,
        [
            "Baies magiques",
            "Forêt vivante",
            "Régénération",
            "Perception héroïque",
            "Forme d'arbre",
        ],
    ),
    (
        Voie::Vegetaux,
        [
            "Peau d'écorce",
            "Prison végétale",
            "Flèche vivante",
            "Animation d'un arbre",
            "Porte végétale",
        ],
    ),
    (
        Voie::EnergieVitale,
        [
            "Mains d'énergie",
            "Projection du ki",
            "Invulnérable",
            "Pression mortelle",
            "Ascétisme",
        ],
    ),
    (
        Voie::Maitrise,
        [
            "Agilité du singe",
            "Griffes du tigre",
            "Morsure du serpent",
            "Fureur du dragon",
            "Moment de perfection",
        ],
    ),
    (
        Voie::Meditation,
        [
            "Pacifisme",
            "Transe de guérison",
            "Maîtrise du ki",
            "Volonté héroïque",
            "Projection mentale",
        ],
    ),
    (
        Voie::Poing,
        [
            "Poings de fer",
            "Peau de fer",
            "Parade de projectiles",
            "Déluge de coups",
            "Puissance du ki",
        ],
    ),
    (
        Voie::Vent,
        [
            "Pas du vent",
            "Course du vent",
            "Course des airs",
            "Agilité héroïque",
            "Passe-muraille",
        ],
    ),
    (
        Voie::Foi,
        [
            "Prédicateur",
            "Miracle mineur",
            "Arme de lumière",
            "Ailes célestes",
            "Foudres divines",
        ],
    ),
    (
        Voie::GuerreSainte,
        [
            "Arme bénie",
            "Bouclier de la foi",
            "Châtiment divin",
            "Marteau de la foi",
            "Mot de pouvoir",
        ],
    ),
    (
        Voie::Priere,
        [
            "Bénédiction",
            "Sanctuaire",
            "Destruction du mal",
            "Volonté héroïque",
            "Intervention divine",
        ],
    ),
    (
        Voie::Soins,
        [
            "Récupération mineure",
            "Vigueur divine",
            "Récupération majeure",
            "Phénix",
            "Rétablissement",
        ],
    ),
    (
        Voie::Spiritualite,
        [
            "Vêtements sacrés",
            "Augure",
            "Délivrance",
            "Charisme héroïque",
            "Marche des plans",
        ],
    ),
    (
        Voie::DemiOrc,
        [
            "Impressionnant",
            "Talent pour la violence",
            "Critique brutal",
            "Attaque sanglante",
            "Colosse",
        ],
    ),
    (
        Voie::ElfeHaut,
        [
            "Lumière intérieure",
            "Force d'âme",
            "Talent pour la magie",
            "Immortel",
            "Supériorité elfique",
        ],
    ),
    (
        Voie::ElfeSylvain,
        [
            "Lumière des étoiles",
            "Enfant de la forêt",
            "Archer émérite",
            "Flèche sanglante",
            "Supériorité elfique",
        ],
    ),
    (
        Voie::Gnome,
        [
            "Don étrange",
            "Petit pote",
            "Insignifiant",
            "Merveille technologique",
            "Bonne nature",
        ],
    ),
    (
        Voie::Halfelin,
        [
            "Petite taille",
            "Résistance légendaire",
            "Bon pour le moral",
            "Petit veinard",
            "Vif et bien nourri",
        ],
    ),
    (
        Voie::Humain,
        [
            "Diversité",
            "Instinct de survie",
            "Touche-à-tout",
            "Loup parmi les loups",
            "Polyvalence",
        ],
    ),
    (
        Voie::Nain,
        [
            "Habitant des tunnels",
            "Haches et marteaux",
            "Résistance à la magie",
            "Fils du roc",
            "Ténacité",
        ],
    ),
    (
        Voie::Mage,
        [
            "Occultisme",
            "Maîtrise de la magie",
            "Tour de magie",
            "Esprit supérieur",
            "Tempête de mana",
        ],
    ),
];

/// Voies chosen at level 1 besides the voie de peuple (COF.CRE.VOIES).
const PROFIL_VOIE_COUNT: i32 = 2;

/// The rank of the mage family's extra capacité (COF.CRE.MAGE-EXTRA).
const MAGE_EXTRA_RANK: Rank = Rank::new(2);

/// The level of every character at creation (COF.CRE.LEVEL1).
const CREATION_LEVEL: i32 = 1;

/// Checks the current ranks against the voie structure (ADR KER-VOIE), and
/// the ranks chosen at creation against the level-1 choices.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        violations.extend(voie_rank_violations(&voie.id(), &ranks, VOIE_MECHANIC));
    }
    check_slots(character, violations);
    check_level(character, violations);
    check_mage_extra(character, violations);
}

/// COF.CRE.VOIES and COF.CRE.VOIE-DU-MAGE: two chosen voies, one voie de
/// peuple, each from where it may come.
fn check_slots(character: &Character, violations: &mut Vec<Violation>) {
    let profil = character.profil();
    let peuple = character.peuple();
    let non_mage = character
        .family()
        .is_some_and(|family| family != Family::Mages);
    let mut chosen = 0_i32;
    let mut peuple_slot = 0_i32;
    for (voie, _) in character.creation_voies() {
        let foreign = if voie == Voie::Mage {
            peuple_slot += 1;
            if non_mage {
                violations.push(Violation::deviation(
                    VOIE_DU_MAGE,
                    VOIE_MECHANIC,
                    Subject::Voie(voie.id()),
                    Severity::Standard,
                ));
            }
            false
        } else if voie.is_peuple_slot() {
            peuple_slot += 1;
            peuple.is_some_and(|peuple| !peuple.allows_voie(voie))
        } else {
            chosen += 1;
            profil.is_some_and(|profil| voie.profil() != Some(profil))
        };
        if foreign {
            violations.push(Violation::deviation(
                VOIES,
                VOIE_MECHANIC,
                Subject::Voie(voie.id()),
                Severity::Standard,
            ));
        }
    }

    let count_violation = |count: i32, expected: i32, subject| {
        let violation = match count.cmp(&expected) {
            std::cmp::Ordering::Less => Violation::incomplete(VOIES, VOIE_MECHANIC, subject),
            std::cmp::Ordering::Greater => {
                Violation::deviation(VOIES, VOIE_MECHANIC, subject, Severity::Standard)
            }
            std::cmp::Ordering::Equal => return None,
        };
        Some(
            violation
                .with_parameter("voies", count)
                .with_parameter("expected", expected),
        )
    };
    violations.extend(count_violation(
        chosen,
        PROFIL_VOIE_COUNT,
        Subject::Value(PROFIL_VOIES),
    ));
    violations.extend(count_violation(peuple_slot, 1, Subject::Value(PEUPLE_VOIE)));
}

/// Whether `voie` may hold the mage family's extra rank: a voie of the
/// mage's profil or the voie du mage (COF.CRE.MAGE-EXTRA).
fn may_hold_mage_extra(character: &Character, voie: Voie) -> bool {
    character.family() == Some(Family::Mages)
        && (voie == Voie::Mage || voie.profil() == character.profil())
}

/// COF.PRG.LEVEL-REQ: rank 1 only at level 1, rank 2 in a voie that may hold
/// the mage extra.
fn check_level(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.creation_voies() {
        let allowed = if may_hold_mage_extra(character, voie) {
            MAGE_EXTRA_RANK
        } else {
            Rank::new(1)
        };
        if let Some(highest) = ranks.last().copied()
            && highest > allowed
        {
            let mut violation = Violation::deviation(
                LEVEL_REQ,
                VOIE_MECHANIC,
                Subject::Voie(voie.id()),
                Severity::Standard,
            )
            .with_parameter("rank", i32::from(highest.get()))
            .with_parameter("level", CREATION_LEVEL);
            if let Some(required) = level::required_level(highest) {
                violation = violation.with_parameter("required", required);
            }
            violations.push(violation);
        }
    }
}

/// COF.CRE.MAGE-EXTRA: exactly one rank 2 for the mage family, in a voie of
/// its profil or the voie du mage.
fn check_mage_extra(character: &Character, violations: &mut Vec<Violation>) {
    if character.family() != Some(Family::Mages) {
        return;
    }
    let extras = character
        .creation_voies()
        .filter(|(voie, ranks)| {
            may_hold_mage_extra(character, *voie) && ranks.contains(&MAGE_EXTRA_RANK)
        })
        .fold(0_i32, |extras, _| extras + 1);
    let violation = match extras {
        0 => Violation::incomplete(MAGE_EXTRA, VOIE_MECHANIC, Subject::Character),
        1 => return,
        _ => Violation::deviation(
            MAGE_EXTRA,
            VOIE_MECHANIC,
            Subject::Character,
            Severity::Standard,
        ),
    };
    violations.push(violation.with_parameter("extras", extras));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use co_kernel::Category;

    use super::*;
    use crate::Peuple;

    fn voie_violations(character: &Character) -> Vec<Violation> {
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

    /// A humain guerrier with its three rank-1 capacités.
    fn guerrier() -> Character {
        Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(Peuple::Humain)
            .with_rank(Voie::Bouclier, 1)
            .with_rank(Voie::Soldat, 1)
            .with_rank(Voie::Humain, 1)
    }

    /// A gnome magicien with its mage extra capacité.
    fn magicien() -> Character {
        Character::new()
            .with_profil(Profil::Magicien)
            .with_peuple(Peuple::Gnome)
            .with_rank(Voie::MagieDesArcanes, 1)
            .with_rank(Voie::MagieDesArcanes, 2)
            .with_rank(Voie::MagieElementaire, 1)
            .with_rank(Voie::Gnome, 1)
    }

    #[test]
    fn level_one_choices_are_clean() {
        assert_eq!(voie_violations(&guerrier()), []);
        assert_eq!(voie_violations(&magicien()), []);
    }

    #[test]
    fn ability_ids_name_voie_and_rank() {
        assert_eq!(Voie::Rage.ability(Rank::new(1)).as_str(), "cof.rage.1");
        assert_eq!(Voie::MeneurDHomme.id().as_str(), "cof.meneur-d-homme");
        assert_eq!(
            Voie::from_ability_id("cof.meneur-d-homme.3"),
            Some((Voie::MeneurDHomme, Rank::new(3)))
        );
        for bad in ["cof.rage", "cof.rage.x", "cof.unknown.1", ""] {
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
        // Lhagva's and Ionas's capacités (D-02).
        assert_eq!(
            Voie::Pourfendeur.ability_name(Rank::new(1)),
            Some("Réflexes éclair")
        );
        assert_eq!(
            Voie::Rage.ability_name(Rank::new(2)),
            Some("Défier la mort")
        );
        assert_eq!(
            Voie::ElfeHaut.ability_name(Rank::new(1)),
            Some("Lumière intérieure")
        );
        assert_eq!(Voie::Mage.ability_name(Rank::new(1)), Some("Occultisme"));
    }

    #[test]
    fn missing_voies_are_incomplete() {
        let subjects: Vec<_> = voie_violations(&Character::new())
            .into_iter()
            .map(|v| (v.subject().clone(), v.category()))
            .collect();
        assert_eq!(
            subjects,
            [
                (Subject::Value(PROFIL_VOIES), Category::Incomplete),
                (Subject::Value(PEUPLE_VOIE), Category::Incomplete),
            ]
        );
    }

    #[test]
    fn extra_voies_deviate() {
        let character = guerrier()
            .with_rank(Voie::Combat, 1)
            .with_rank(Voie::Mage, 1);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [
                ("COF.CRE.VOIE-DU-MAGE", DEVIATION),
                ("COF.CRE.VOIES", DEVIATION),
                ("COF.CRE.VOIES", DEVIATION),
            ]
        );
        assert_eq!(violations[1].subject(), &Subject::Value(PROFIL_VOIES));
        assert_eq!(violations[2].subject(), &Subject::Value(PEUPLE_VOIE));
    }

    #[test]
    fn voies_from_elsewhere_deviate() {
        let character = Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(Peuple::Nain)
            .with_rank(Voie::Bouclier, 1)
            .with_rank(Voie::Rage, 1)
            .with_rank(Voie::Humain, 1);
        let subjects: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| v.subject().clone())
            .collect();
        assert_eq!(
            subjects,
            [
                Subject::Voie(Voie::Rage.id()),
                Subject::Voie(Voie::Humain.id())
            ]
        );
    }

    #[test]
    fn demi_elfe_takes_a_voie_of_its_upbringing() {
        let character = Character::new()
            .with_profil(Profil::Barde)
            .with_peuple(Peuple::DemiElfe)
            .with_rank(Voie::Escrime, 1)
            .with_rank(Voie::Musicien, 1)
            .with_rank(Voie::ElfeSylvain, 1);
        assert_eq!(voie_violations(&character), []);
    }

    #[test]
    fn mage_replaces_its_voie_de_peuple() {
        let character = magicien()
            .without_rank(Voie::Gnome, 1)
            .with_rank(Voie::Mage, 1);
        assert_eq!(voie_violations(&character), []);
        let both = character.with_rank(Voie::Gnome, 1);
        assert_eq!(
            rules(&voie_violations(&both)),
            [("COF.CRE.VOIES", DEVIATION)]
        );
    }

    #[test]
    fn mage_extra_may_be_in_the_voie_du_mage() {
        let character = magicien()
            .without_rank(Voie::MagieDesArcanes, 2)
            .without_rank(Voie::Gnome, 1)
            .with_rank(Voie::Mage, 1)
            .with_rank(Voie::Mage, 2);
        assert_eq!(voie_violations(&character), []);
    }

    #[test]
    fn mage_extra_stays_in_the_profil_voies() {
        let character = magicien()
            .without_rank(Voie::MagieDesArcanes, 2)
            .without_rank(Voie::MagieElementaire, 1)
            .with_rank(Voie::Rage, 1)
            .with_rank(Voie::Rage, 2);
        let found: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.category()))
            .collect();
        assert_eq!(
            found,
            [
                ("COF.CRE.VOIES".to_owned(), DEVIATION),
                ("COF.PRG.LEVEL-REQ".to_owned(), DEVIATION),
                ("COF.CRE.MAGE-EXTRA".to_owned(), Category::Incomplete),
            ]
        );
    }

    #[test]
    fn missing_mage_extra_is_incomplete() {
        let character = magicien().without_rank(Voie::MagieDesArcanes, 2);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("COF.CRE.MAGE-EXTRA", Category::Incomplete)]
        );
    }

    #[test]
    fn second_mage_extra_deviates() {
        let character = magicien().with_rank(Voie::MagieElementaire, 2);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("COF.CRE.MAGE-EXTRA", DEVIATION)]
        );
    }

    #[test]
    fn rank_above_level_one_deviates() {
        let character = guerrier()
            .with_rank(Voie::Soldat, 2)
            .with_rank(Voie::Humain, 2);
        let subjects: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.subject().clone()))
            .collect();
        assert_eq!(
            subjects,
            [
                (
                    "COF.PRG.LEVEL-REQ".to_owned(),
                    Subject::Voie(Voie::Soldat.id())
                ),
                (
                    "COF.PRG.LEVEL-REQ".to_owned(),
                    Subject::Voie(Voie::Humain.id())
                ),
            ]
        );
        // A mage's voie de peuple does not take the extra rank.
        let mage = magicien().with_rank(Voie::Gnome, 2);
        assert_eq!(
            rules(&voie_violations(&mage)),
            [("COF.PRG.LEVEL-REQ", DEVIATION)]
        );
        let rank_three = magicien().with_rank(Voie::MagieDesArcanes, 3);
        let violations = voie_violations(&rank_three);
        assert_eq!(rules(&violations), [("COF.PRG.LEVEL-REQ", DEVIATION)]);
        assert_eq!(
            violations[0].parameters()[2].value(),
            &co_kernel::ParameterValue::Integer(3)
        );
    }

    #[test]
    fn gap_and_structure_deviate() {
        let character = guerrier().with_rank(Voie::Soldat, 9);
        assert_eq!(
            rules(&voie_violations(&character)),
            [
                ("CO.VOIE.STRUCTURE", DEVIATION),
                ("CO.VOIE.ORDER", DEVIATION),
                ("COF.PRG.LEVEL-REQ", DEVIATION),
            ]
        );
    }

    #[test]
    fn spells_follow_the_catalogue() {
        let spells = |voie: Voie| -> Vec<u8> {
            (1..=5)
                .filter(|rank| voie.is_spell(Rank::new(*rank)))
                .collect()
        };
        assert_eq!(spells(Voie::Air), [1, 2, 3, 4, 5]);
        assert_eq!(spells(Voie::Artefacts), [2, 4]);
        assert_eq!(spells(Voie::OutreTombe), [1, 3, 4, 5]);
        assert_eq!(spells(Voie::Protecteur), [1, 2, 3, 5]);
        assert_eq!(spells(Voie::Mage), [2, 3]);
        assert_eq!(spells(Voie::Seduction), [5]);
        assert_eq!(spells(Voie::Rage), []);
        assert_eq!(spells(Voie::Humain), []);
        // Only the profils with a magic characteristic, the barde's voies and
        // the voie du mage hold spells.
        for voie in Voie::ALL {
            if !spells(voie).is_empty() {
                assert!(
                    voie == Voie::Mage
                        || voie
                            .profil()
                            .is_some_and(|p| p.magic_characteristic().is_some()),
                    "{voie:?}"
                );
            }
        }
    }
}
