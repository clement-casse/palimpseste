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

/// A described effect giving the capacité's test bonus numbered `n`, 0 for
/// all of them.
const fn t(n: u8, text: &'static str) -> AbilityEffect {
    AbilityEffect::giving(Bonus::Test(n), text)
}

/// A described effect giving the capacité's dé bonus.
const fn die(text: &'static str) -> AbilityEffect {
    AbilityEffect::giving(Bonus::Die, text)
}

/// The effects of the capacités, in voie and rank order.
const EFFECTS: &[(Voie, u8, &[AbilityEffect])] = &[
    // Voie de l'artilleur.
    (
        Voie::Artilleur,
        1,
        &[
            t(1, "rang + 2 réparer ou comprendre un mécanisme"),
            d("dé bonus en attaque avec un engin de siège"),
        ],
    ),
    (
        Voie::Artilleur,
        2,
        &[
            d("jusqu'à 2 armes à chargeur de [2 + INT] tirs (+1/rang 3 d'arquebusier)"),
            d("recharge : L par tir"),
        ],
    ),
    (
        Voie::Artilleur,
        3,
        &[
            d("surveille une zone de 20 m : attaque à distance sur qui y bouge"),
            d("touchée : DM × 2 ou la cible s'arrête"),
        ],
    ),
    (
        Voie::Artilleur,
        4,
        &[
            c("armes à poudre : dé de DM doublé"),
            d("2 tirs par recharge"),
            d("critique : dé triplé"),
        ],
    ),
    (
        Voie::Artilleur,
        5,
        &[
            d("canon portatif : attaque avec dé bonus, [5d4° + INT] DM, 100 m"),
            d("recharge en 2 rounds"),
        ],
    ),
    // Voie des explosifs.
    (
        Voie::Explosifs,
        1,
        &[
            d("cône de 10 m : une attaque contre chaque cible, DM ÷ 2"),
            t(1, "rang + 2 artificier"),
        ],
    ),
    (
        Voie::Explosifs,
        2,
        &[
            d("charge posée en 3 rounds : [3d4° + INT] DM à une structure, RD ÷ 2"),
            d("rang charges/jour (avec rangs 4-5)"),
        ],
    ),
    (
        Voie::Explosifs,
        3,
        &[
            c("armes à poudre : +1 DM (+1/rang 5 d'arquebusier)"),
            d("portée +10 m"),
            d("tirs magiques"),
        ],
    ),
    (
        Voie::Explosifs,
        4,
        &[d(
            "piège en 1 min, rayon 5 m : [5d4° + INT] DM de feu (AGI 15 : ÷ 2)",
        )],
    ),
    (
        Voie::Explosifs,
        5,
        &[
            d("bombe lancée à 20 m, rayon 5 m : [4d4° + INT] DM (AGI 10 : ÷ 2)"),
            d("aveugle ceux qui ratent"),
        ],
    ),
    // Voie du demi-orc.
    (
        Voie::DemiOrc,
        1,
        &[
            t(1, "+3 intimidation"),
            d("voit dans le noir total comme en pénombre (30 m)"),
        ],
    ),
    (
        Voie::DemiOrc,
        2,
        &[d(
            "gagne une capacité de rang 1 d'une voie de barbare ou de guerrier",
        )],
    ),
    (
        Voie::DemiOrc,
        3,
        &[
            c("critique au contact sur 19-20"),
            d("+1d4° DM sur un critique"),
        ],
    ),
    (
        Voie::DemiOrc,
        4,
        &[d(
            "attaque au contact : saignement de 1d4° DM/round jusqu'aux soins ; non cumulable",
        )],
    ),
    (Voie::DemiOrc, 5, &[c("FOR +1, CON +1")]),
    // Voie de l'elfe haut.
    (
        Voie::ElfeHaut,
        1,
        &[
            d("nuit étoilée = pénombre"),
            t(1, "+3 tests d'érudition (INT) et d'art (CHA)"),
        ],
    ),
    (
        Voie::ElfeHaut,
        2,
        &[
            d("immunité à la peur et au sommeil magique"),
            t(1, "+ rang pour résister aux sorts"),
        ],
    ),
    (
        Voie::ElfeHaut,
        3,
        &[d(
            "capacité de rang 1 de magicien ou d'ensorceleur (en armure, sans DEF) ou de rang 2 (sans armure)",
        )],
    ),
    (
        Voie::ElfeHaut,
        4,
        &[
            d("repos, nourriture et boisson ÷ 2"),
            d("immunité aux poisons et maladies"),
        ],
    ),
    (Voie::ElfeHaut, 5, &[c("VOL +1 ; INT ou CHA +1")]),
    // Voie de l'elfe sylvain.
    (
        Voie::ElfeSylvain,
        1,
        &[
            d("nuit étoilée = pénombre"),
            t(1, "+3 survie en forêt (escalade, discrétion, chasse)"),
        ],
    ),
    (
        Voie::ElfeSylvain,
        2,
        &[d(
            "capacité de rang 1 de druide ou de rôdeur, utilisable jusqu'au cuir renforcé",
        )],
    ),
    (
        Voie::ElfeSylvain,
        3,
        &[
            c("arcs : critique +1"),
            d("+1d4° DM sur un critique"),
            d("maîtrise l'arc court"),
        ],
    ),
    (
        Voie::ElfeSylvain,
        4,
        &[d(
            "attaque à distance : saignement de 1d4° DM/round jusqu'aux soins",
        )],
    ),
    (Voie::ElfeSylvain, 5, &[c("AGI +1, PER +1")]),
    // Voie du gnome.
    (
        Voie::Gnome,
        1,
        &[
            t(1, "+3 tests d'INT scientifiques"),
            d("capacité de rang 1 d'ensorceleur (1/jour en armure)"),
            d("vision dans le noir 10 m"),
        ],
    ),
    (
        Voie::Gnome,
        2,
        &[t(1, "+3 tests sociaux sauf intimidation"), c("+1 PC")],
    ),
    (
        Voie::Gnome,
        3,
        &[d("+2 DEF contre plus grand (+3 au rang 5)")],
    ),
    (
        Voie::Gnome,
        4,
        &[
            d("maîtrise arbalètes (et armes à poudre si permises)"),
            c("+AGI à leurs DM"),
        ],
    ),
    (Voie::Gnome, 5, &[c("CON +1, CHA +1")]),
    // Voie du halfelin.
    (
        Voie::Halfelin,
        1,
        &[
            c("+1 DEF"),
            t(1, "+3 discrétion et vol"),
            d(
                "armes : à une main jusqu'à 1d6 DM, à deux mains jusqu'à 1d10, rien au-delà ; ni arc long ni arbalète lourde",
            ),
        ],
    ),
    (Voie::Halfelin, 2, &[t(1, "+ rang pour résister aux sorts")]),
    (
        Voie::Halfelin,
        3,
        &[d("bon repas (4/jour max, 3 h d'écart) : soigne 1d4° PV")],
    ),
    (
        Voie::Halfelin,
        4,
        &[
            c("+1 PC"),
            d("1/combat, esquive une attaque (pas un critique)"),
        ],
    ),
    (Voie::Halfelin, 5, &[c("AGI +1, CON +1")]),
    // Voie de l'humain.
    (
        Voie::Humain,
        1,
        &[
            t(
                1,
                "+3 aux deux domaines de son origine (l'un remplaçable par un milieu ou un métier, jamais le combat)",
            ),
            c("+1 PC"),
        ],
    ),
    (
        Voie::Humain,
        2,
        &[d(
            "1/combat : DM ÷ 2 d'une attaque qui le mettrait à 0 PV, puis +2 DEF jusqu'à la fin du combat",
        )],
    ),
    (
        Voie::Humain,
        3,
        &[d(
            "capacité de rang 1 ou 2 de tout profil ; limite d'armure si rang 2 ou bonus de DEF",
        )],
    ),
    (
        Voie::Humain,
        4,
        &[d(
            "1/round : +1d4° DM contre un humanoïde de taille moyenne (DM initiaux)",
        )],
    ),
    (
        Voie::Humain,
        5,
        &[c("caractéristique la plus basse +1 ; VOL +1")],
    ),
    // Voie du nain.
    (
        Voie::Nain,
        1,
        &[
            d("vision dans le noir 30 m"),
            t(
                1,
                "+3 pierre, architecture, mines, passages secrets, pièges muraux",
            ),
        ],
    ),
    (
        Voie::Nain,
        2,
        &[
            c("haches et marteaux de guerre : +1 attaque et DM"),
            d("maîtrise les haches et marteaux de guerre"),
        ],
    ),
    (
        Voie::Nain,
        3,
        &[d(
            "1/jour, ignore un sort qui le cible (pas de zone, pas d'un NC ≥ 2 × son niveau)",
        )],
    ),
    (
        Voie::Nain,
        4,
        &[c(
            "RD 2 (3 au niveau 10), min. 1 DM par attaque ; cumulable",
        )],
    ),
    (Voie::Nain, 5, &[c("CON +1, VOL +1")]),
    // Voie du mage.
    (
        Voie::Mage,
        1,
        &[
            d("garde la capacité de rang 1 de son peuple"),
            t(1, "rang + 2 aux connaissances magiques"),
        ],
    ),
    (
        Voie::Mage,
        2,
        &[
            d("détecte la magie à 10 m, l'identifie (INT 10 + rang)"),
            d("dissipe un sort non permanent (attaque magique opposée)"),
        ],
    ),
    (
        Voie::Mage,
        3,
        &[
            d("un tour mineur gratuit/round (10 m, sans PM ni DM, rien exigeant une carac > 0)"),
            c("+1 DEF"),
            c("+2 PM"),
        ],
    ),
    (
        Voie::Mage,
        4,
        &[c("INT +1, VOL +1"), die("dé bonus aux tests d'INT")],
    ),
    (
        Voie::Mage,
        5,
        &[d("+1 PM (cible unique) ou +3 PM (zone) : +1d4° DM au sort")],
    ),
];

#[cfg(test)]
mod tests {
    use co_kernel::{Application, Characteristic, Contribution, Source};

    use super::*;
    use crate::{
        Armour, Character, LevelUp, Profil, Series, Shield, TestBonus, Value, Weapon, WeaponUse,
        test_bonus,
    };

    /// One weapon of each kind a capacité singles out: light, heavy, axe,
    /// hammer, staff, thrown, bow, crossbow, powder.
    const WEAPONS: [Weapon; 12] = [
        Weapon::EpeeLongue,
        Weapon::Rapiere,
        Weapon::Dague,
        Weapon::EpeeADeuxMains,
        Weapon::Hache,
        Weapon::Marteau,
        Weapon::Baton,
        Weapon::Javelot,
        Weapon::ArcCourt,
        Weapon::ArcLong,
        Weapon::ArbaleteLegere,
        Weapon::Petoire,
    ];

    /// The capacité a contribution names, unless it is a spell's 1 PM: that
    /// is the rule of D-23, not the capacité's own effect.
    fn ability(value: Value, contribution: &Contribution) -> Option<String> {
        let Source::Ability(id) = contribution.source() else {
            return None;
        };
        let spell = Voie::from_ability_id(id.as_str()).is_some_and(|(v, r)| v.is_spell(r));
        let spell_pm = value == Value::ManaPoints && contribution.amount() == 1 && spell;
        (!spell_pm).then(|| id.as_str().to_owned())
    }

    /// Whether the capacité `id` is in a total of the character's sheet: a
    /// characteristic or a value. Test bonuses, dé bonus and situational
    /// modifiers are in none.
    fn in_a_total(character: &Character, id: &str) -> bool {
        let named = |value: Value, line: &co_kernel::DerivedValue| {
            line.contributions()
                .iter()
                .any(|c| ability(value, c).as_deref() == Some(id))
        };
        if Characteristic::ALL
            .iter()
            .any(|c| named(Value::Initiative, &character.characteristic(*c)))
        {
            return true;
        }
        let mut values: Vec<Value> = Value::FIXED.to_vec();
        for weapon in WEAPONS {
            values.push(Value::CriticalRange(weapon));
            for weapon_use in WeaponUse::ALL {
                values.push(Value::WeaponAttack(weapon, weapon_use));
                values.push(Value::Damage(weapon, weapon_use));
            }
        }
        values.iter().any(|value| {
            character
                .value(*value)
                .is_some_and(|line| named(*value, &line))
        })
    }

    /// The ranks 1 to `rank` of `voie`, each with its first option when it
    /// needs one or when `optional`, for a magicien with distinct values: a profil gives the
    /// family values (PV, PC), and distinct values show a characteristic
    /// used in place of another.
    fn ranks(voie: Voie, rank: u8, optional: bool) -> Character {
        let values = [
            (Characteristic::Agi, 3),
            (Characteristic::Con, 1),
            (Characteristic::For, 0),
            (Characteristic::Per, 2),
            (Characteristic::Cha, 1),
            (Characteristic::Int, 2),
            (Characteristic::Vol, -1),
        ];
        let mage = values.into_iter().fold(
            Character::new()
                .with_profil(Profil::Magicien)
                .with_series(Series::Polyvalent),
            |c, (characteristic, value)| c.with_series_value(characteristic, value),
        );
        (1..=rank).fold(mage, |c, r| {
            let c = c.with_rank(voie, r);
            // A capacité with a default keeps it unless `optional`.
            match c.ability_options_offered(voie, r).first() {
                Some(option) if optional || c.ability_option_required(voie, r) => {
                    c.with_ability_option(voie, r, *option)
                }
                _ => c,
            }
        })
    }

    /// Whether the engine counts the capacité at `rank` in `voie` in a total,
    /// with its default or its option: named by a characteristic or a value,
    /// without armour or in plate with a shield, at level 1 or 10; or
    /// changing a weapon's attack or dice from the ranks below it.
    fn computed(voie: Voie, rank: u8) -> bool {
        [false, true]
            .into_iter()
            .any(|optional| computed_with(voie, rank, optional))
    }

    /// [`computed`], the optional options chosen or not.
    fn computed_with(voie: Voie, rank: u8, optional: bool) -> bool {
        let id = voie.ability(Rank::new(rank)).as_str().to_owned();
        let held = ranks(voie, rank, optional);
        let below = ranks(voie, rank - 1, optional);
        let total = |c: &Character, value| c.value(value).map(|v| v.total());
        let differs = WEAPONS.iter().any(|w| {
            WeaponUse::ALL.iter().any(|u| {
                held.damage_dice(*w, *u) != below.damage_dice(*w, *u)
                    || total(&held, Value::WeaponAttack(*w, *u))
                        != total(&below, Value::WeaponAttack(*w, *u))
            })
        });
        let plated = |c: &Character| {
            c.clone()
                .with_armour(Armour::PlaqueComplete)
                .with_shield(Shield::GrandBouclier)
        };
        differs || in_a_total(&held, &id) || in_a_total(&plated(&held), &id) || {
            let senior = (0..9).fold(held.clone(), |c, _| c.with_level_up(LevelUp::new()));
            in_a_total(&senior, &id) || in_a_total(&plated(&senior), &id)
        }
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
        assert_eq!(Voie::Nain.ability_effects(Rank::new(1)).len(), 2);
        assert_eq!(Voie::Nain.ability_effects(Rank::new(6)), []);
    }

    #[test]
    fn a_capacite_with_a_computed_effect_is_in_a_total() {
        let wrong = |rows: &[(Voie, u8, &[AbilityEffect])]| -> Vec<(Voie, u8)> {
            rows.iter()
                .filter(|(voie, rank, effects)| {
                    let claims = effects
                        .iter()
                        .any(|e| e.application() == Application::Computed);
                    claims != computed(*voie, *rank)
                })
                .map(|(voie, rank, _)| (*voie, *rank))
                .collect()
        };
        // Slow in a debug build: the rows are checked in parallel.
        let wrong: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = EFFECTS
                .chunks(EFFECTS.len().div_ceil(8))
                .map(|rows| scope.spawn(move || wrong(rows)))
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().unwrap_or_default())
                .collect()
        });
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
                let mut bonuses: Vec<u8> = TestBonus::all()
                    .filter(|b| (b.voie(), b.rank().get()) == (*voie, *rank))
                    .map(TestBonus::index)
                    .collect();
                bonuses.dedup();
                if linked == [0] {
                    linked.clone_from(&bonuses);
                }
                linked.sort_unstable();
                let die = effects.iter().any(|e| e.bonus() == Some(Bonus::Die));
                linked != bonuses || die != test_bonus::gives_bonus_die(*voie, *rank)
            })
            .map(|(voie, rank, _)| (*voie, *rank))
            .collect();
        assert_eq!(wrong, []);
    }
}
