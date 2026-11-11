//! What each capacité does, as short effects in our own words, each
//! computed by the engine or only described (ADR ENG-ABILITY-EFFECTS).

use co_kernel::{AbilityEffect, Bonus, Rank};

use crate::Voie;

/// The effects of the capacité at `rank` in `voie`, empty when not written
/// yet or outside ranks 1 to 5.
pub(crate) fn of(voie: Voie, rank: Rank) -> &'static [AbilityEffect] {
    EFFECTS
        .iter()
        .find(|(v, r, _)| *v == voie && *r == rank.get())
        .map_or(&[], |(_, _, effects)| effects)
}

/// An effect the engine computes.
const fn c(text: &'static str) -> AbilityEffect {
    AbilityEffect::computed(text)
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
            c("FOR ou AGI +1"),
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
            c("AGI ou PER +1"),
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
            c("FOR ou PER +1"),
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
            d("1/séance : + rang à un test"),
        ],
    ),
    (
        Voie::ArtsEtArtisanats,
        4,
        &[
            t(1, "+5 dans une quatrième discipline"),
            d(
                "reconnu de ses fans (INT 15 région, 20 pays, 25 monde) : + rang à ses tests de CHA pour obtenir de l'aide",
            ),
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
            c("FOR, AGI ou CON +1"),
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
            c("FOR ou AGI +1"),
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
            c("PER ou CHA +1"),
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
            d("1/scène : + rang à la prochaine action des alliés, ou − rang à celle des ennemis"),
        ],
    ),
    (
        Voie::Discours,
        5,
        &[
            die("dé bonus dans une approche"),
            d("CHA 15 : captive (+5 au prochain test social ou attitude améliorée d'un cran)"),
            c("CHA ou INT +1"),
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
