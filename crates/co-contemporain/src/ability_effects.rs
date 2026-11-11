//! What each capacité does, as short effects in our own words, each
//! computed by the engine or only described (ADR ENG-ABILITY-EFFECTS).

use Characteristic::Cha;
use co_kernel::{AbilityEffect, Bonus, Characteristic, EffectPart, Measure, Rank, Term};

use crate::{Character, Voie};

/// The effects of the capacité at `rank` in `voie`, empty outside ranks 1
/// to 5.
pub(crate) fn of(voie: Voie, rank: Rank) -> &'static [AbilityEffect] {
    EFFECTS
        .iter()
        .find(|(v, r, _)| *v == voie && *r == rank.get())
        .map_or(&[], |(_, _, effects)| effects)
}

/// A rank-reached term, written `formula`.
const fn rank(formula: &'static str) -> Term {
    Term::new(Measure::Rank, formula)
}

/// A characteristic term, written `formula`.
const fn carac(characteristic: Characteristic, formula: &'static str) -> Term {
    Term::new(Measure::Characteristic(characteristic), formula)
}

/// The effects of the capacité at `rank` in `voie` written for
/// `character`: each formula's value, the rank reached at least `rank`.
/// COC2 has no level, nor counts of voies.
pub(crate) fn written(
    character: &Character,
    voie: Voie,
    rank: Rank,
) -> Vec<(AbilityEffect, Vec<EffectPart>)> {
    let reached = character
        .highest_rank(voie)
        .map_or(0, Rank::get)
        .max(rank.get());
    let value_of = |measure: Measure| match measure {
        Measure::Rank => Some(i32::from(reached)),
        Measure::Characteristic(c) => Some(character.characteristic(c).total()),
        Measure::Level | Measure::VoiesAt { .. } => None,
    };
    of(voie, rank)
        .iter()
        .map(|effect| (*effect, effect.parts(value_of)))
        .collect()
}

/// An effect the engine computes.
const fn c(text: &'static str) -> AbilityEffect {
    AbilityEffect::computed(text)
}

/// A computed effect depending on the choice recorded for its capacité.
const fn ch(text: &'static str) -> AbilityEffect {
    AbilityEffect::chosen(text)
}

/// An effect the engine only describes.
const fn d(text: &'static str) -> AbilityEffect {
    AbilityEffect::described(text)
}

/// A described effect giving the capacité's test bonus numbered `n`.
const fn t(n: u8, text: &'static str) -> AbilityEffect {
    AbilityEffect::giving(Bonus::Test(n), text)
}

/// A described effect giving the capacité's dé bonus.
const fn die(text: &'static str) -> AbilityEffect {
    AbilityEffect::giving(Bonus::Die, text)
}

/// The effects of the capacités, in voie and rank order.
const EFFECTS: &[(Voie, u8, &[AbilityEffect])] = &[
    // Voie des armes de contact.
    (
        Voie::ArmesDeContact,
        1,
        &[
            c("+1 ATC par rang avec une arme de contact"),
            t(
                1,
                "+1 par rang aux tests de VOL contre le choc d'un combat au contact",
            ),
        ],
    ),
    (
        Voie::ArmesDeContact,
        2,
        &[
            d(
                "choix permanent : pas de malus avec les armes improvisées, ou une arme de prédilection à +2",
            ),
            d("+2 DEF contre les armes de contact"),
        ],
    ),
    (
        Voie::ArmesDeContact,
        3,
        &[
            d("chaque tour : transfère jusqu'à 3 points entre Init, DEF et ATC"),
            d("manœuvres à −2 au lieu de −5"),
        ],
    ),
    (
        Voie::ArmesDeContact,
        4,
        &[
            c("critique sur 18-20"),
            c("+AGI aux DM et au bonus critique"),
        ],
    ),
    (
        Voie::ArmesDeContact,
        5,
        &[
            d("dé bonus avec l'arme de prédilection"),
            d("ignore une condition défavorable"),
            ch("FOR ou AGI +1"),
        ],
    ),
    // Voie des armes à distance.
    (
        Voie::ArmesADistance,
        1,
        &[
            c("+1 ATD par rang avec une arme à distance"),
            t(
                1,
                "+1 par rang aux tests de VOL contre le choc d'une fusillade",
            ),
        ],
    ),
    (
        Voie::ArmesADistance,
        2,
        &[
            d("une arme de prédilection à +2"),
            d("+2 DEF contre les attaques à distance"),
        ],
    ),
    (
        Voie::ArmesADistance,
        3,
        &[
            d("début du tour : transfère jusqu'à 3 points entre Init, DEF et ATD"),
            d("ignore une condition défavorable"),
        ],
    ),
    (
        Voie::ArmesADistance,
        4,
        &[
            c("critique sur 18-20"),
            c("+PER aux DM et au bonus critique"),
        ],
    ),
    (
        Voie::ArmesADistance,
        5,
        &[
            d("dé bonus avec l'arme de prédilection"),
            d("ignore une deuxième condition défavorable"),
            ch("AGI ou PER +1"),
        ],
    ),
    // Voie des armes lourdes.
    (
        Voie::ArmesLourdes,
        1,
        &[
            c("+1 ATD par rang avec une arme lourde"),
            t(
                1,
                "+1 par rang aux tests de VOL contre le choc des armes lourdes",
            ),
        ],
    ),
    (
        Voie::ArmesLourdes,
        2,
        &[
            d("une arme de prédilection à +2"),
            d("+2 DEF contre les explosions"),
            d("tir automatique à −2 au lieu de −5, avec un demi-chargeur"),
        ],
    ),
    (
        Voie::ArmesLourdes,
        3,
        &[
            d("chaque tour : transfère jusqu'à 3 points entre Init, DEF et ATD"),
            d("ignore une condition défavorable"),
        ],
    ),
    (
        Voie::ArmesLourdes,
        4,
        &[
            c("armes lourdes : +1d6 DM"),
            d("recharger est une action de mouvement"),
        ],
    ),
    (
        Voie::ArmesLourdes,
        5,
        &[
            d("dé bonus avec l'arme de prédilection"),
            d("ignore une deuxième condition défavorable"),
            ch("FOR ou PER +1"),
        ],
    ),
    // Voie des arts et artisanats.
    (
        Voie::ArtsEtArtisanats,
        1,
        &[t(
            1,
            "+5 en culture (INT) et en pratique (CHA ou AGI) d'une discipline",
        )],
    ),
    (
        Voie::ArtsEtArtisanats,
        2,
        &[t(
            1,
            "+5 dans une deuxième discipline, et pour reproduire une œuvre de ses disciplines",
        )],
    ),
    (
        Voie::ArtsEtArtisanats,
        3,
        &[
            t(1, "+5 dans une troisième discipline"),
            d("1/séance : {} à un test").with(&[rank("+ rang").signed()]),
        ],
    ),
    (
        Voie::ArtsEtArtisanats,
        4,
        &[
            t(1, "+5 dans une quatrième discipline"),
            d("reconnu de ses fans (INT 15 région, 20 pays, 25 monde) : {} à ses tests de CHA pour obtenir de l'aide").with(&[rank("+ rang").signed()]),
        ],
    ),
    (
        Voie::ArtsEtArtisanats,
        5,
        &[
            die("dé bonus en connaissance et en pratique d'une discipline choisie"),
            c("CHA +1"),
            d("difficulté de notoriété −5"),
        ],
    ),
    // Voie de l'athlétisme.
    (
        Voie::Athletisme,
        1,
        &[
            t(
                1,
                "+1 par rang aux tests d'athlétisme (FOR, AGI, CON), hors combat",
            ),
            c("CON +1"),
        ],
    ),
    (
        Voie::Athletisme,
        2,
        &[t(1, "+2 dans une discipline sportive")],
    ),
    (
        Voie::Athletisme,
        3,
        &[
            d("après avoir subi 2 DM : +2 à un test de FOR, AGI ou CON, une attaque ou la DEF"),
            d("ignore un nouvel état de santé jusqu'à la fin du tour suivant"),
            t(1, "+2 dans une deuxième discipline sportive"),
        ],
    ),
    (
        Voie::Athletisme,
        4,
        &[t(1, "+2 dans une troisième discipline sportive")],
    ),
    (
        Voie::Athletisme,
        5,
        &[
            die("dé bonus dans une discipline sportive choisie"),
            d("le +2 du rang 3 passe à +5"),
            ch("FOR, AGI ou CON +1"),
        ],
    ),
    // Voie du combat à mains nues.
    (
        Voie::CombatAMainsNues,
        1,
        &[
            c("+1 ATC par rang à mains nues"),
            t(
                1,
                "+1 par rang aux tests de VOL contre le choc d'une bagarre",
            ),
        ],
    ),
    (
        Voie::CombatAMainsNues,
        2,
        &[
            d("un art martial choisi"),
            d("+2 DEF contre les attaques à mains nues"),
            c("mains nues : 1d6 + FOR DM, bonus critique 6"),
        ],
    ),
    (
        Voie::CombatAMainsNues,
        3,
        &[
            d("début du tour : transfère jusqu'à 3 points entre Init, DEF et ATC"),
            d("manœuvres à −2 au lieu de −5"),
        ],
    ),
    (
        Voie::CombatAMainsNues,
        4,
        &[
            c("critique sur 18-20"),
            d("RD 1 en parant à mains nues ou en frappant une structure"),
            c("+AGI aux DM et au bonus critique"),
        ],
    ),
    (
        Voie::CombatAMainsNues,
        5,
        &[
            d("dé bonus avec son art martial"),
            d("ignore une condition défavorable"),
            ch("FOR ou AGI +1"),
        ],
    ),
    // Voie des corporations.
    (
        Voie::Corporations,
        1,
        &[
            d("membre d'une corporation"),
            t(1, "+1 par rang aux tests de CHA avec ses membres"),
        ],
    ),
    (
        Voie::Corporations,
        2,
        &[t(
            1,
            "+5 aux tests d'INT pour obtenir une information accessible à son organisation",
        )],
    ),
    (
        Voie::Corporations,
        3,
        &[
            d("une deuxième corporation"),
            t(
                1,
                "+5 aux tests de CHA pour obtenir aide ou moyens par l'autorité",
            ),
        ],
    ),
    (
        Voie::Corporations,
        4,
        &[t(
            1,
            "+5 aux tests de CHA pour pousser une direction à l'illégalité ou à couvrir",
        )],
    ),
    (
        Voie::Corporations,
        5,
        &[
            d("une troisième corporation"),
            die("dé bonus à un test des rangs 2 à 4"),
            c("CHA +1, INT +1"),
        ],
    ),
    // Voie du crime.
    (
        Voie::Crime,
        1,
        &[t(
            1,
            "+5 en AGI pour voler, tricher, escamoter ; +5 en CHA opposé pour distraire, mentir, bluffer",
        )],
    ),
    (
        Voie::Crime,
        2,
        &[
            d("FOR au lieu de CHA pour contraindre"),
            c("+2 Init"),
            d("1/séance : fuite surprise (ennemis surpris 1 round)"),
        ],
    ),
    (
        Voie::Crime,
        3,
        &[t(
            1,
            "+5 pour forcer serrures et coffres, analyser un système de sécurité",
        )],
    ),
    (
        Voie::Crime,
        4,
        &[t(
            1,
            "+5 pour se faire passer pour un autre (+ le bonus des arts au rang 1 pour un acteur)",
        )],
    ),
    (
        Voie::Crime,
        5,
        &[
            die("dé bonus à un test des rangs 1, 3 ou 4"),
            d("PER 10 : repère une personne louche ou une faille exploitable"),
            ch("PER ou CHA +1"),
        ],
    ),
    // Voie du discours.
    (
        Voie::Discours,
        1,
        &[t(1, "+5 aux tests sociaux d'une approche")],
    ),
    (Voie::Discours, 2, &[t(1, "+5 dans une deuxième approche")]),
    (
        Voie::Discours,
        3,
        &[
            t(1, "+5 dans une troisième approche"),
            d("CHA opposé : pousse quelqu'un à perdre son sang-froid ou à attaquer"),
        ],
    ),
    (
        Voie::Discours,
        4,
        &[
            t(1, "+5 dans la quatrième approche"),
            d("1/scène : {} à la prochaine action des alliés, ou {} à celle des ennemis").with(&[rank("+ rang").signed(), rank("− rang").times(-1)]),
        ],
    ),
    (
        Voie::Discours,
        5,
        &[
            die("dé bonus dans une approche"),
            d("CHA 15 : captive (+5 au prochain test social ou attitude améliorée d'un cran)"),
            ch("CHA ou INT +1"),
        ],
    ),
    // Voie de la furtivité.
    (
        Voie::Furtivite,
        1,
        &[t(
            1,
            "+5 en AGI pour se déplacer en silence ou agir discrètement",
        )],
    ),
    (
        Voie::Furtivite,
        2,
        &[
            t(1, "+5 aux tests de PER des cinq sens"),
            d("PER 15 pour éviter la surprise ou un piège"),
        ],
    ),
    (
        Voie::Furtivite,
        3,
        &[d(
            "−5 à qui veut se souvenir de lui, le trouver, le filer ou le repérer (un compagnon au rang 4, deux au rang 5)",
        )],
    ),
    (
        Voie::Furtivite,
        4,
        &[
            t(
                1,
                "+5 en PER pour trouver une cachette, une faille de sécurité, un accès discret",
            ),
            d("réussi : dé bonus au test d'AGI qui suit"),
        ],
    ),
    (
        Voie::Furtivite,
        5,
        &[
            die("dé bonus à un test des rangs 1 à 4"),
            d("une planque secrète (5 semaines d'autonomie)"),
            ch("PER ou AGI +1"),
        ],
    ),
    // Voie de l'investigation.
    (
        Voie::Investigation,
        1,
        &[t(
            1,
            "+1 par rang pour trouver un indice (PER) ou remarquer un détail (INT) en fouillant",
        )],
    ),
    (
        Voie::Investigation,
        2,
        &[die("dé bonus aux tests d'INT d'une spécialité")],
    ),
    (
        Voie::Investigation,
        3,
        &[
            d("fouilles en 1d6 au lieu de 2d6"),
            d("le MJ signale un indice manqué : nouveau test de PER ou d'INT 15"),
        ],
    ),
    (
        Voie::Investigation,
        4,
        &[
            d("mémoire parfaite sur demande"),
            die("dé bonus dans une deuxième spécialité"),
        ],
    ),
    (
        Voie::Investigation,
        5,
        &[d("durée de fouille réduite d'un cran"), ch("INT ou PER +1")],
    ),
    // Voie des langues.
    (
        Voie::Langues,
        1,
        &[d("parle, lit et écrit {}").with(&[rank("une langue de plus par rang").unit(" langue(s) de plus")])],
    ),
    (
        Voie::Langues,
        2,
        &[t(
            1,
            "+5 en INT sur la géographie, l'histoire et la culture d'un pays dont il parle la langue",
        )],
    ),
    (
        Voie::Langues,
        3,
        &[t(
            1,
            "+5 en INT pour se servir d'une langue inconnue d'une famille qu'il parle",
        )],
    ),
    (
        Voie::Langues,
        4,
        &[t(
            1,
            "+5 en INT pour déchiffrer écritures et codes inconnus, et en mathématiques",
        )],
    ),
    (
        Voie::Langues,
        5,
        &[die("dé bonus aux tests des rangs 2 à 4"), c("INT +1")],
    ),
    // Voie de la médecine.
    (
        Voie::Medecine,
        1,
        &[
            t(
                1,
                "+5 en INT pour les premiers soins et stabiliser un mourant",
            ),
            t(
                2,
                "+1 par rang aux tests de VOL contre le choc des scènes médicales",
            ),
        ],
    ),
    (
        Voie::Medecine,
        2,
        &[
            t(
                1,
                "+5 en INT dans une spécialité médicale et en culture médicale",
            ),
            c("INT au lieu de FOR ou d'AGI aux DM, si elle est plus haute"),
        ],
    ),
    (
        Voie::Medecine,
        3,
        &[
            d("1d6 h et INT 15 : annule une séquelle physique de moins de 24 h"),
            t(1, "+5 dans une deuxième spécialité médicale"),
        ],
    ),
    (
        Voie::Medecine,
        4,
        &[
            d("pas de malus pour de mauvaises conditions ou des soins improvisés"),
            t(1, "+5 avec une médecine traditionnelle"),
        ],
    ),
    (
        Voie::Medecine,
        5,
        &[
            t(1, "+5 aux tests sociaux sur ses spécialités"),
            die("dé bonus en INT dans une spécialité choisie des rangs 2 ou 4"),
            c("INT +1"),
        ],
    ),
    // Voie de la psychologie.
    (
        Voie::Psychologie,
        1,
        &[t(
            1,
            "+5 en PER pour lire l'état émotionnel et les attitudes",
        )],
    ),
    (
        Voie::Psychologie,
        2,
        &[d(
            "2 min d'échange, PER opposée (avec le +5) : détecte un mensonge ou une dissimulation ; échec : 1 essai/24 h par cible",
        )],
    ),
    (
        Voie::Psychologie,
        3,
        &[
            t(1, "+2 en VOL contre les événements traumatisants"),
            t(
                2,
                "+5 en VOL opposée pour calmer une personne choquée ou violente",
            ),
        ],
    ),
    (
        Voie::Psychologie,
        4,
        &[d(
            "PER opposée en conversation, ou INT 15 sur dossier : un indice et la meilleure approche sociale",
        )],
    ),
    (
        Voie::Psychologie,
        5,
        &[
            die("dé bonus aux tests de PER de psychologie"),
            ch("PER ou VOL +1"),
        ],
    ),
    // Voie de la technologie.
    (
        Voie::Technologie,
        1,
        &[t(
            1,
            "+1 par rang en INT pour comprendre, entretenir, réparer ou saboter un système",
        )],
    ),
    (
        Voie::Technologie,
        2,
        &[t(1, "+5 dans une spécialité technique")],
    ),
    (
        Voie::Technologie,
        3,
        &[
            d(
                "1/séance : +5 au test de CHA 10 pour trouver un fournisseur ; 1/aventure : matériel lourd (3d6 h)",
            ),
            t(1, "+5 dans une deuxième spécialité technique"),
        ],
    ),
    (
        Voie::Technologie,
        4,
        &[
            d(
                "1/séance, après 1d6 × 10 min : +2 aux performances d'un objet (véhicule, programme, attaque ou DM d'une arme)",
            ),
            t(1, "+5 dans une troisième spécialité technique"),
        ],
    ),
    (
        Voie::Technologie,
        5,
        &[
            d("pas de pénalité sans les outils adaptés ; temps ÷ 2 avec"),
            die("dé bonus dans une spécialité choisie"),
            c("INT +1"),
        ],
    ),
    // Voie des sciences.
    (
        Voie::Sciences,
        1,
        &[
            t(1, "+5 en INT dans une discipline"),
            t(
                2,
                "+1 par rang aux tests de CHA avec ses pairs (s'ajoute à corporations rang 1 dans le même milieu)",
            ),
        ],
    ),
    (
        Voie::Sciences,
        2,
        &[
            d("accès aux ressources professionnelles"),
            t(1, "+5 dans une deuxième discipline du même groupe"),
        ],
    ),
    (
        Voie::Sciences,
        3,
        &[
            d("test d'INT à la place d'un test physique de même difficulté"),
            t(1, "+5 dans une troisième discipline (tout groupe)"),
        ],
    ),
    (
        Voie::Sciences,
        4,
        &[
            d("le bonus de CHA du rang 1 vaut aussi pour lever des fonds et face aux médias"),
            t(1, "+5 dans une quatrième discipline"),
        ],
    ),
    (
        Voie::Sciences,
        5,
        &[
            t(1, "+5 aux tests sociaux sur ses spécialités"),
            die("dé bonus dans une discipline choisie"),
            c("INT +1"),
        ],
    ),
    // Voie de la survie.
    (
        Voie::Survie,
        1,
        &[t(
            1,
            "+1 par rang en INT sur les coutumes locales, les moyens de voyage, passer pour un local",
        )],
    ),
    (
        Voie::Survie,
        2,
        &[t(
            1,
            "+5 aux tests de survie et d'équipement dans un milieu hostile choisi",
        )],
    ),
    (
        Voie::Survie,
        3,
        &[
            d("4 h de sommeil lui suffisent"),
            t(
                1,
                "+1 par rang en CON contre la fatigue, la faim, la maladie, le poison",
            ),
            t(2, "+5 dans un deuxième milieu"),
        ],
    ),
    (
        Voie::Survie,
        4,
        &[
            d("jusqu'à {} : +5 à leurs tests de survie").with(&[carac(Cha, "[5 + CHA] compagnons").plus(5).unit(" compagnons")]),
            t(1, "+5 dans un troisième milieu"),
        ],
    ),
    (
        Voie::Survie,
        5,
        &[
            die("dé bonus en connaissance ou en survie dans un milieu choisi"),
            c("CON +1"),
        ],
    ),
    // Voie des véhicules.
    (
        Voie::Vehicules,
        1,
        &[
            t(1, "+2 pour piloter un véhicule choisi à un seul pilote"),
            t(
                2,
                "+1 par rang à l'entretien courant (s'ajoute à technologie rang 1)",
            ),
        ],
    ),
    (
        Voie::Vehicules,
        2,
        &[
            t(1, "+2 avec un deuxième véhicule"),
            d("le premier véhicule passe à +5"),
            d("une manœuvre périlleuse sans élargir la zone d'échec (+2 seulement pour deux)"),
        ],
    ),
    (
        Voie::Vehicules,
        3,
        &[
            t(1, "+2 avec un troisième véhicule"),
            d("ignore une condition défavorable de poursuite"),
            d("les occupants subissent 1d6 DM de moins en accident"),
        ],
    ),
    (
        Voie::Vehicules,
        4,
        &[
            t(1, "+2 avec un quatrième véhicule (avec équipage possible)"),
            d("le deuxième véhicule passe à +5"),
            d("deux manœuvres périlleuses librement"),
            d("en mouvement : +2 DEF au véhicule et aux occupants"),
        ],
    ),
    (
        Voie::Vehicules,
        5,
        &[
            t(1, "+2 avec un cinquième véhicule"),
            d("le troisième véhicule passe à +5 ; équipage +2"),
            die("dé bonus avec un véhicule choisi"),
            c("AGI +1"),
        ],
    ),
];

#[cfg(test)]
mod tests {
    use co_kernel::{Application, Characteristic, Contribution, Source};

    use super::*;
    use crate::{Character, TestBonus, Value, Weapon, test_bonus};

    /// The values of the sheet: the fixed ones and each weapon's.
    fn values() -> Vec<Value> {
        let mut values: Vec<Value> = Value::FIXED.to_vec();
        for weapon in Weapon::ALL {
            values.extend([
                Value::Damage(weapon),
                Value::CriticalBonus(weapon),
                Value::CriticalRange(weapon),
            ]);
        }
        values
    }

    /// The ranks 1 to `rank` of `voie` for an adult with distinct values,
    /// a rank 5 raising the first characteristic it offers.
    fn ranks(voie: Voie, rank: u8) -> Character {
        let points = [
            (Characteristic::Agi, 3),
            (Characteristic::Con, 1),
            (Characteristic::For, 0),
            (Characteristic::Per, 2),
            (Characteristic::Cha, 1),
            (Characteristic::Int, 2),
            (Characteristic::Vol, -1),
        ];
        let adult = points.into_iter().fold(
            Character::new().with_age(30),
            |c, (characteristic, points)| c.with_carac_points(characteristic, points),
        );
        let held = (1..=rank).fold(adult, |c, r| c.with_rank(voie, r));
        match voie.rank_five_choice().first() {
            Some(raised) if rank == 5 => held.with_raised_characteristic(voie, *raised),
            _ => held,
        }
    }

    /// Whether the engine counts the capacité at `rank` in `voie` in a total:
    /// named by a characteristic or a value, or changing a value's total,
    /// without the other capacités' terms, or a weapon's dice from the ranks
    /// below it.
    fn computed(voie: Voie, rank: u8) -> bool {
        let id = voie.ability(Rank::new(rank)).as_str().to_owned();
        let held = ranks(voie, rank);
        let below = ranks(voie, rank - 1);
        let own = |c: &&Contribution| match c.source() {
            Source::Ability(other) => other.as_str() == id,
            _ => true,
        };
        let named = |c: &Character| {
            Characteristic::ALL.iter().any(|ch| {
                c.characteristic(*ch)
                    .contributions()
                    .iter()
                    .any(|t| t.source() == &Source::Ability(voie.ability(Rank::new(rank))))
            }) || values().iter().any(|value| {
                c.value(*value).is_some_and(|line| {
                    line.contributions()
                        .iter()
                        .any(|t| t.source() == &Source::Ability(voie.ability(Rank::new(rank))))
                })
            })
        };
        let total = |c: &Character, value| {
            c.value(value).map(|line| {
                line.contributions()
                    .iter()
                    .filter(own)
                    .map(Contribution::amount)
                    .sum::<i32>()
            })
        };
        let changed =
            values()
                .into_iter()
                .any(|value| match (total(&held, value), total(&below, value)) {
                    (Some(held), Some(below)) => held != below,
                    _ => false,
                });
        let dice = Weapon::ALL
            .iter()
            .any(|w| held.damage_dice(*w) != below.damage_dice(*w));
        named(&held) || changed || dice
    }

    #[test]
    fn effects_are_short_and_in_voie_and_rank_order() {
        assert!(EFFECTS.iter().all(|(_, _, effects)| !effects.is_empty()));
        assert!(
            EFFECTS
                .iter()
                .flat_map(|(_, _, effects)| effects.iter())
                .all(|e| e.text().chars().count() <= 120 && !e.text().is_empty())
        );
        let keys: Vec<_> = EFFECTS.iter().map(|(v, r, _)| (*v, *r)).collect();
        let mut unique = keys.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(keys.len(), unique.len());
        assert_eq!(Voie::Crime.ability_effects(Rank::new(2)).len(), 3);
        assert_eq!(Voie::Crime.ability_effects(Rank::new(6)), []);
    }

    #[test]
    fn effects_depending_on_a_choice_are_those_of_the_rank_five_choices() {
        let chosen: Vec<_> = EFFECTS
            .iter()
            .filter(|(_, _, effects)| effects.iter().any(|e| e.choice()))
            .map(|(voie, rank, _)| (*voie, *rank))
            .collect();
        let expected: Vec<_> = Voie::ALL
            .into_iter()
            .filter(|voie| !voie.rank_five_choice().is_empty())
            .map(|voie| (voie, 5))
            .collect();
        assert_eq!(chosen, expected);
    }

    #[test]
    fn every_term_is_one_coc2_writes() {
        // COC2 has no level, nor counts of voies: such a term would never be
        // written.
        let unwritten: Vec<_> = EFFECTS
            .iter()
            .flat_map(|(_, _, effects)| effects.iter())
            .flat_map(|effect| effect.terms().iter())
            .filter(|term| matches!(term.measure(), Measure::Level | Measure::VoiesAt { .. }))
            .map(|term| term.formula())
            .collect();
        assert_eq!(unwritten, Vec::<&str>::new());
    }

    #[test]
    fn every_slot_has_its_term() {
        let wrong: Vec<_> = EFFECTS
            .iter()
            .flat_map(|(_, _, effects)| effects.iter())
            .filter(|e| {
                e.template().matches("{}").count() != e.terms().len() || e.text().contains("{}")
            })
            .map(|e| e.template())
            .collect();
        assert_eq!(wrong, Vec::<&str>::new());
    }

    #[test]
    fn formulas_are_written_for_the_character() {
        let linguist = ranks(Voie::Langues, 3);
        let bilingual = linguist.written_effects(Voie::Langues, 1);
        assert_eq!(
            bilingual[0].1,
            [
                EffectPart::Text("parle, lit et écrit ".to_owned()),
                EffectPart::Value {
                    value: "3 langue(s) de plus".to_owned(),
                    formula: "une langue de plus par rang"
                }
            ]
        );
        let guide = ranks(Voie::Survie, 4).written_effects(Voie::Survie, 4);
        assert!(
            matches!(&guide[0].1[1], EffectPart::Value { value, .. } if value == "6 compagnons")
        );
    }

    #[test]
    fn every_capacite_has_its_effects() {
        let missing: Vec<_> = Voie::ALL
            .into_iter()
            .flat_map(|voie| (1..=5).map(move |rank| (voie, rank)))
            .filter(|(voie, rank)| voie.ability_effects(Rank::new(*rank)).is_empty())
            .collect();
        assert_eq!(missing, []);
        assert_eq!(EFFECTS.len(), Voie::ALL.len() * 5);
    }

    #[test]
    fn a_capacite_with_a_computed_effect_is_in_a_total() {
        let wrong: Vec<_> = EFFECTS
            .iter()
            .filter(|(voie, rank, effects)| {
                let claims = effects
                    .iter()
                    .any(|e| e.application() == Application::Computed);
                claims != computed(*voie, *rank)
            })
            .map(|(voie, rank, _)| (*voie, *rank))
            .collect();
        assert_eq!(wrong, []);
    }

    #[test]
    fn every_bonus_of_a_capacite_has_its_effect() {
        let wrong: Vec<_> = EFFECTS
            .iter()
            .filter(|(voie, rank, effects)| {
                let mut linked: Vec<u8> = effects
                    .iter()
                    .filter_map(|e| match e.bonus() {
                        Some(Bonus::Test(n)) => Some(n),
                        _ => None,
                    })
                    .collect();
                linked.sort_unstable();
                let bonuses: Vec<u8> = TestBonus::all()
                    .filter(|b| (b.voie(), b.rank().get()) == (*voie, *rank))
                    .map(TestBonus::index)
                    .collect();
                let die = effects.iter().any(|e| e.bonus() == Some(Bonus::Die));
                linked != bonuses || die != test_bonus::gives_bonus_die(*voie, *rank)
            })
            .map(|(voie, rank, _)| (*voie, *rank))
            .collect();
        assert_eq!(wrong, []);
    }
}
