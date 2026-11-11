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
    // Voie du mercenaire.
    (
        Voie::Mercenaire,
        1,
        &[
            t(1, "rang + 2 relations de taverne et résister à l'alcool"),
            d("mains nues : 1d4° DM non létaux"),
            d("DM non létaux subis ÷ 2"),
        ],
    ),
    (
        Voie::Mercenaire,
        2,
        &[d(
            "attaque (DM temporaires possibles) qui désarme, renverse ou affaiblit un ennemi de NC < rang",
        )],
    ),
    (
        Voie::Mercenaire,
        3,
        &[
            d("capacité de rang 1 de guerrier, de voleur (cuir) ou de rôdeur (cuir renforcé)"),
            c("+1 DEF"),
        ],
    ),
    (
        Voie::Mercenaire,
        4,
        &[c("CON +1"), die("dé bonus aux tests de CON")],
    ),
    (
        Voie::Mercenaire,
        5,
        &[
            d(
                "combat de 10 créatures ou plus : une action d'attaque ou de mouvement en plus par tour",
            ),
            c("+1 DEF"),
        ],
    ),
    // Voie du pistolero.
    (
        Voie::Pistolero,
        1,
        &[
            d("+5 Init pour tirer avec une arme à poudre chargée en main"),
            d("pas de dé malus pour tirer à la poudre ou à l'arbalète au contact"),
        ],
    ),
    (
        Voie::Pistolero,
        2,
        &[d("après un tir raté : +5 au tir suivant sur la même cible")],
    ),
    (
        Voie::Pistolero,
        3,
        &[d(
            "une pétoire (ou arbalète de poing) dans chaque main : 2 tirs à −2 (sans malus sur la même cible)",
        )],
    ),
    (
        Voie::Pistolero,
        4,
        &[c("AGI +1"), die("dé bonus aux tests d'AGI")],
    ),
    (
        Voie::Pistolero,
        5,
        &[d(
            "tir à la poudre ou à l'arbalète qui dépasse la DEF de 10 ou plus : +2d4° DM",
        )],
    ),
    // Voie de la précision.
    (
        Voie::Precision,
        1,
        &[d(
            "ignore le malus de couvert partiel ; couvert important : −2 seulement",
        )],
    ),
    (
        Voie::Precision,
        2,
        &[d(
            "étudie une cible : au round suivant, tirs contre DEF [10 + AGI de la cible], sans RD ni résistance aux DM",
        )],
    ),
    (
        Voie::Precision,
        3,
        &[c("critique à distance sur 19-20 (18-20 au rang 5)")],
    ),
    (
        Voie::Precision,
        4,
        &[d("tir visé à 10 m ou plus : portée × 2, +2d4° DM")],
    ),
    (
        Voie::Precision,
        5,
        &[d(
            "contre un NC < moitié de son niveau : test d'INT [10 + NC], la cible meurt",
        )],
    ),
    // Voie de l'escrime.
    (
        Voie::Escrime,
        1,
        &[c(
            "arme légère à une main : AGI au lieu de FOR pour toucher (pas aux DM)",
        )],
    ),
    (
        Voie::Escrime,
        2,
        &[d(
            "CHA opposé à la PER : au round suivant, première attaque au contact à + 2 × rang, +2d4° DM si réussie",
        )],
    ),
    (
        Voie::Escrime,
        3,
        &[d(
            "1/combat, INT opposée : désarme, renverse ou aveugle 1 round un ennemi de NC inférieur",
        )],
    ),
    (
        Voie::Escrime,
        4,
        &[d("attaque à l'arme légère : +CHA à l'attaque et aux DM")],
    ),
    (
        Voie::Escrime,
        5,
        &[d(
            "attaque au contact à l'arme légère qui dépasse la DEF de 10 ou plus : +2d4° DM",
        )],
    ),
    // Voie du musicien.
    (
        Voie::Musicien,
        1,
        &[
            d("alliés à portée de voix : +1 à tous les tests pendant CHA min (+2 au rang 5)"),
            t(1, "rang + 2 musique et chant"),
        ],
    ),
    (
        Voie::Musicien,
        2,
        &[d(
            "joue pendant un repos de 30 min : lui et ses alliés à 10 m récupèrent 1d4° PV (2d4° au rang 4)",
        )],
    ),
    (
        Voie::Musicien,
        3,
        &[d("cône de 10 m : [2d4° + CHA] DM (CON : ÷ 2)")],
    ),
    (
        Voie::Musicien,
        4,
        &[d(
            "sphère de silence de 5 m jusqu'à 30 m pendant CHA min ; y lancer un sort : INT 10",
        )],
    ),
    (
        Voie::Musicien,
        5,
        &[d(
            "attaque magique opposée (10 m) : la cible danse [1d4° + CHA] rounds, dé malus en attaque, −5 DEF",
        )],
    ),
    // Voie du saltimbanque.
    (
        Voie::Saltimbanque,
        1,
        &[t(1, "rang + 2 acrobaties, équilibre, saut, escalade")],
    ),
    (
        Voie::Saltimbanque,
        2,
        &[
            c("+CHA à l'Init"),
            c("+1 DEF (+2 au rang 4)"),
            t(1, "rang + 2 danse, mime, jonglerie"),
        ],
    ),
    (
        Voie::Saltimbanque,
        3,
        &[d(
            "1/round, même au contact : lance un couteau de plus ; 10 m ; DM [1d4 + AGI], dé 1d4° au rang 5",
        )],
    ),
    (
        Voie::Saltimbanque,
        4,
        &[d(
            "immunité à la peur, au contrôle mental, à ralenti et immobilisé",
        )],
    ),
    (
        Voie::Saltimbanque,
        5,
        &[d(
            "1/round : esquive une attaque par un test d'attaque à distance contre son résultat ; un critique touche",
        )],
    ),
    // Voie de la séduction.
    (
        Voie::Seduction,
        1,
        &[
            t(1, "rang + 2 séduire, convaincre, mentir"),
            d("1 PC : +[1d4° + CHA] au test d'un allié"),
        ],
    ),
    (
        Voie::Seduction,
        2,
        &[c("sans armure : +CHA à la DEF (au plus le rang)")],
    ),
    (
        Voie::Seduction,
        3,
        &[d(
            "10 min et 1 PC (2 sans langue commune) : un humanoïde de NC 1 devient un ami",
        )],
    ),
    (
        Voie::Seduction,
        4,
        &[
            c("CHA +1"),
            die("dé bonus aux tests de CHA"),
            c("peut calculer les PM avec CHA au lieu de VOL"),
        ],
    ),
    (
        Voie::Seduction,
        5,
        &[d(
            "attaque magique opposée : une créature de niveau inférieur suit une suggestion pendant 1 h",
        )],
    ),
    // Voie du vagabond.
    (
        Voie::Vagabond,
        1,
        &[t(1, "rang + 2 légendes et identifier les objets magiques")],
    ),
    (
        Voie::Vagabond,
        2,
        &[
            t(1, "+1 à tous les tests de compétence (+1/rang 4 de barde)"),
            d("non cumulable avec les autres bonus de compétence, sauf le rang 1 de peuple"),
        ],
    ),
    (
        Voie::Vagabond,
        3,
        &[d(
            "1 PC : un objet courant non listé d'au plus 10 pa ; peut improviser des gadgets",
        )],
    ),
    (
        Voie::Vagabond,
        4,
        &[d(
            "lit, écrit et parle une langue vivante pendant CHA h (un allié : CHA min) ; langues mortes au rang 5",
        )],
    ),
    (
        Voie::Vagabond,
        5,
        &[d(
            "prend l'apparence d'un humanoïde de même taille pendant CHA h ; imiter une personne : test de CHA",
        )],
    ),
    // Voie de l'archer.
    (
        Voie::Archer,
        1,
        &[
            c("arcs : +PER aux DM, + rang à l'Init"),
            c("variante armes de jet : +FOR aux DM, pas d'Init"),
            d("variante armes de jet : portée × 2"),
        ],
    ),
    (
        Voie::Archer,
        2,
        &[d(
            "tire dans la mêlée sans malus ; ne touche jamais un allié",
        )],
    ),
    (
        Voie::Archer,
        3,
        &[d(
            "accepte un dé malus pour +2d4° DM (+3d4° en action limitée)",
        )],
    ),
    (Voie::Archer, 4, &[d("deux attaques à distance à −2")]),
    (
        Voie::Archer,
        5,
        &[d(
            "dé bonus et +1d4° DM, ou un état (aveuglé, affaibli, ralenti, immobilisé) 1 round sur une cible de NC inférieur",
        )],
    ),
    // Voie du compagnon animal.
    (
        Voie::CompagnonAnimal,
        1,
        &[d(
            "un loup obéit à des ordres simples : DEF 12 + rang, PV niveau × 4, Init et attaque magique du rôdeur, DM 1d4 + 2",
        )],
    ),
    (
        Voie::CompagnonAnimal,
        2,
        &[d(
            "au contact ensemble : le loup a un dé bonus en attaque, le rôdeur en pistage et vigilance",
        )],
    ),
    (
        Voie::CompagnonAnimal,
        3,
        &[d(
            "télépathie avec le loup ; lui transfère ses PV pour le soigner",
        )],
    ),
    (
        Voie::CompagnonAnimal,
        4,
        &[d("le loup s'améliore : DEF 18, PV niveau × 5, DM 1d4° + 5")],
    ),
    (
        Voie::CompagnonAnimal,
        5,
        &[
            d("le loup : +1d4° DM sur la cible du rôdeur ; soins doublés"),
            c("+1 DEF au rôdeur et au loup par rang 5 de rôdeur"),
        ],
    ),
    // Voie de la survie.
    (
        Voie::Survie,
        1,
        &[
            t(1, "rang + 2 escalade et survie en milieu sauvage"),
            d("dépense un DR en extérieur : +1d4° PV"),
        ],
    ),
    (
        Voie::Survie,
        2,
        &[d(
            "1/jour : nourriture pour rang personnes et herbes de soin (1d4° PV par rang)",
        )],
    ),
    (
        Voie::Survie,
        3,
        &[d(
            "en extérieur : +1 DEF (+2 au rang 5), 10 m de déplacement gratuit, ignore le terrain difficile naturel",
        )],
    ),
    (
        Voie::Survie,
        4,
        &[c("CON +1"), die("dé bonus aux tests de CON")],
    ),
    (
        Voie::Survie,
        5,
        &[d(
            "1/combat à 0 PV : au tour suivant, récupère [4d4° + CON] PV, +5 DEF 1 round, perd les états non permanents",
        )],
    ),
    // Voie du traqueur.
    (
        Voie::Traqueur,
        1,
        &[
            t(1, "en extérieur : rang + 2 discrétion, vigilance, pistage"),
            c("peut échanger le +1 PC de sa famille contre +1 DR"),
        ],
    ),
    (
        Voie::Traqueur,
        2,
        &[d(
            "attaque au contact rapide : +AGI à l'attaque et aux DM (avec 10 m de déplacement au rang 5)",
        )],
    ),
    (
        Voie::Traqueur,
        3,
        &[
            d("+1d4° DM contre les animaux"),
            d("un type d'ennemi juré de plus par rang 5 de rôdeur"),
        ],
    ),
    (
        Voie::Traqueur,
        4,
        &[c("PER +1"), die("dé bonus aux tests de PER")],
    ),
    (
        Voie::Traqueur,
        5,
        &[d(
            "en extérieur : s'éloigne de 30 m et disparaît sur AGI 10 (15 en terrain découvert)",
        )],
    ),
    // Voie du combat à deux armes.
    (
        Voie::CombatADeuxArmes,
        1,
        &[
            d(
                "1/round, après une attaque ratée de la main principale : attaque gratuite de l'autre main",
            ),
            d("dague, hachette ou épée courte ; autre arme à une main : dé malus"),
        ],
    ),
    (
        Voie::CombatADeuxArmes,
        2,
        &[d(
            "avec deux armes : +1 DEF (+2 au rang 5), doublé sans attaque de la main gauche",
        )],
    ),
    (
        Voie::CombatADeuxArmes,
        3,
        &[
            d(
                "1/round : une attaque gratuite de l'autre main à chaque attaque de la main principale (autre cible : dé malus)",
            ),
            d("remplace le rang 1"),
        ],
    ),
    (
        Voie::CombatADeuxArmes,
        4,
        &[
            c("au choix : AGI +1 ou FOR +1"),
            die("avec AGI +1 : dé bonus aux tests d'AGI"),
            d("avec FOR +1 : la même arme dans chaque main sans dé malus"),
        ],
    ),
    (
        Voie::CombatADeuxArmes,
        5,
        &[d(
            "les deux armes touchent la même cible dans le tour : +1d4° DM à l'une des attaques",
        )],
    ),
    // Voie de l'assassin.
    (
        Voie::Assassin,
        1,
        &[
            t(1, "rang + 2 discrétion, déguisement, dissimuler une arme"),
            d("langage des signes des voleurs"),
            d("dé bonus en attaque contre un ennemi surpris"),
        ],
    ),
    (
        Voie::Assassin,
        2,
        &[d(
            "1/round, ennemi surpris ou de dos, arme légère : +2d4° DM (+1d4°/rang 4 de voleur, max. 7d4°) ; sinon ÷ 2",
        )],
    ),
    (
        Voie::Assassin,
        3,
        &[d(
            "attaque sournoise en action d'attaque contre un ennemi surpris : +2d4° DM",
        )],
    ),
    (
        Voie::Assassin,
        4,
        &[d(
            "1/combat : disparaît jusqu'au tour suivant, réapparaît à 20 m, peut porter une attaque sournoise",
        )],
    ),
    (
        Voie::Assassin,
        5,
        &[d(
            "1/combat : une attaque sournoise est un critique automatique",
        )],
    ),
    // Voie de l'aventurier.
    (
        Voie::Aventurier,
        1,
        &[
            t(1, "rang + 2 bluff, séduction, négociation, marché noir"),
            d("utilise parchemins et baguettes (attaque magique 10 + 2 × rang du sort)"),
        ],
    ),
    (
        Voie::Aventurier,
        2,
        &[d(
            "CHA opposé à l'INT : un humanoïde doit l'attaquer ; riposte gratuite avec attaque sournoise ou +1d4° DM",
        )],
    ),
    (
        Voie::Aventurier,
        3,
        &[
            c("+2 DEF et Init (+3 au rang 5)"),
            d("se relève par une action de mouvement"),
        ],
    ),
    (
        Voie::Aventurier,
        4,
        &[c("CHA +1"), die("dé bonus aux tests de CHA")],
    ),
    (
        Voie::Aventurier,
        5,
        &[
            d(
                "1/combat : un coup immobilise un humanoïde 1d4 rounds (paralysé si NC < moitié de son niveau)",
            ),
            d("contre un immobilisé : attaque sournoise ou +1d4° DM"),
        ],
    ),
    // Voie du déplacement.
    (
        Voie::Deplacement,
        1,
        &[
            t(1, "rang + 2 tests de déplacement"),
            c("+1 DEF et Init (+2 au rang 3, +3 au rang 5)"),
        ],
    ),
    (
        Voie::Deplacement,
        2,
        &[
            d("DM de chute ÷ 2"),
            d("une action de mouvement de plus par combat (deux au rang 5)"),
        ],
    ),
    (
        Voie::Deplacement,
        3,
        &[d(
            "1/round, AGI 15 : franchit un obstacle ou passe derrière un ennemi, puis attaque sournoise ou +1d4° DM",
        )],
    ),
    (
        Voie::Deplacement,
        4,
        &[c("AGI +1"), die("dé bonus aux tests d'AGI")],
    ),
    (
        Voie::Deplacement,
        5,
        &[d(
            "1/round : attaque à distance opposée à l'attaque magique pour éviter un sort qui inflige des DM",
        )],
    ),
    // Voie du roublard.
    (
        Voie::Roublard,
        1,
        &[
            t(1, "rang + 2 crochetage, pièges, vol à la tire, estimation"),
            c("dagues et couteaux lancés : +1 DM (+2 au rang 3, +3 au rang 5)"),
        ],
    ),
    (
        Voie::Roublard,
        2,
        &[
            t(1, "rang + 2 fouille, pièges, passages secrets, embuscades"),
            d("DM des pièges ÷ 2"),
        ],
    ),
    (
        Voie::Roublard,
        3,
        &[d(
            "1/combat : fait le mort (INT 20 pour le voir) ; en se relevant, soigne 1d4° PV et surprend un ennemi au contact",
        )],
    ),
    (
        Voie::Roublard,
        4,
        &[
            die("dé bonus aux tests d'investigation et de contrefaçon"),
            d("1 PC : obtient un indice manqué"),
        ],
    ),
    (
        Voie::Roublard,
        5,
        &[d(
            "3 doses de poison/jour : +2d4° DM et CON ou affaibli, ou endormir par la nourriture",
        )],
    ),
    // Voie du spadassin.
    (
        Voie::Spadassin,
        1,
        &[
            c("+AGI à l'Init"),
            c("arme légère à une main : AGI au lieu de FOR pour toucher (pas aux DM)"),
            t(1, "rang + 2 intimidation"),
        ],
    ),
    (
        Voie::Spadassin,
        2,
        &[d(
            "1/combat : détourne une attaque vers un autre ennemi au contact (pas un critique)",
        )],
    ),
    (
        Voie::Spadassin,
        3,
        &[c(
            "armes légères : critique +2 (18-20), jamais au-delà de 16",
        )],
    ),
    (
        Voie::Spadassin,
        4,
        &[d(
            "chaque round : attaque gratuite de l'autre main à la dague ou à l'épée courte (sans attaque sournoise)",
        )],
    ),
    (
        Voie::Spadassin,
        5,
        &[d(
            "critique à l'arme légère : un état 1 round (chacun 1/combat) ou une attaque sournoise en plus",
        )],
    ),
    // Voie de la brute.
    (
        Voie::Brute,
        1,
        &[
            c("+FOR aux PV max"),
            d(
                "+FOR aux tests de CHA de négociation, persuasion, intimidation, pour lui et ses alliés au contact",
            ),
        ],
    ),
    (
        Voie::Brute,
        2,
        &[
            d("+10 à un test de FOR (pas attaque ni DM) pour 1d4° PV"),
            d("capacités de barbare : limite d'armure relevée à la chemise de mailles"),
        ],
    ),
    (
        Voie::Brute,
        3,
        &[d(
            "attaque au contact : +1d4° DM, ou −3 en attaque pour +2d4° ; peut échanger des dés de DM contre repousser ou renverser",
        )],
    ),
    (
        Voie::Brute,
        4,
        &[c("FOR +1"), die("dé bonus aux tests de FOR")],
    ),
    (
        Voie::Brute,
        5,
        &[
            c("critique au contact +1"),
            d("un critique étourdit 1 round"),
            d("limite d'armure relevée à la cotte de mailles"),
        ],
    ),
    // Voie du pagne.
    (
        Voie::Pagne,
        1,
        &[t(1, "rang + 2 course, saut, escalade"), c("+1 PV par rang")],
    ),
    (
        Voie::Pagne,
        2,
        &[c(
            "CON au lieu d'AGI à la DEF (plafond de l'armure), sinon +1 DEF (+2 au rang 4)",
        )],
    ),
    (
        Voie::Pagne,
        3,
        &[
            t(
                0,
                "+3 aux tests d'une caractéristique choisie (bonus magique)",
            ),
            d("étourdi devient ralenti"),
        ],
    ),
    (
        Voie::Pagne,
        4,
        &[c("CON +1"), die("dé bonus aux tests de CON")],
    ),
    (Voie::Pagne, 5, &[c("RD 3, au moins 1 DM par attaque")]),
    // Voie du pourfendeur.
    (
        Voie::Pourfendeur,
        1,
        &[
            t(1, "rang + 2 aux tests d'AGI pour esquiver"),
            c("+3 Init"),
            c("+1 DEF (+2 au rang 5)"),
        ],
    ),
    (
        Voie::Pourfendeur,
        2,
        &[d(
            "déplacement en ligne droite de 5 à 10 m puis attaque au contact : dé bonus, +1d4° DM",
        )],
    ),
    (
        Voie::Pourfendeur,
        3,
        &[d(
            "met un ennemi à 0 PV au contact : attaque gratuite contre un autre ennemi au contact",
        )],
    ),
    (
        Voie::Pourfendeur,
        4,
        &[d(
            "course de 10 m à travers les ennemis : une attaque à −2 contre chacun",
        )],
    ),
    (
        Voie::Pourfendeur,
        5,
        &[d(
            "1/combat : DM de l'arme automatiques à chaque cible dans un rayon de 5 m",
        )],
    ),
    // Voie du primitif.
    (
        Voie::Primitif,
        1,
        &[
            t(
                1,
                "en extérieur : rang + 2 survie (récupération comprise) et discrétion",
            ),
            c("+1 PV"),
        ],
    ),
    (
        Voie::Primitif,
        2,
        &[
            c("sans armure : +2 DEF (+3 au rang 5) ; en armure : +1 DEF"),
            d("se relève par une action de mouvement"),
        ],
    ),
    (
        Voie::Primitif,
        3,
        &[
            t(1, "rang + 2 repérer pièges et embuscades"),
            d("immunité aux attaques sournoises des créatures de niveau inférieur"),
        ],
    ),
    (
        Voie::Primitif,
        4,
        &[d(
            "1/round : attaque magique opposée pour ignorer un sort (dé malus pour un hybride lanceur de sorts)",
        )],
    ),
    (
        Voie::Primitif,
        5,
        &[d(
            "entre 1 PV et le tiers de ses PV max : récupère 1d4° PV par heure",
        )],
    ),
    // Voie de la rage.
    (
        Voie::Rage,
        1,
        &[
            d(
                "1/combat : les ennemis à 10 m de FOR inférieure ont un dé malus en attaque au contact au tour suivant",
            ),
            t(1, "rang + 2 pour résister à la peur"),
        ],
    ),
    (
        Voie::Rage,
        2,
        &[d(
            "1/combat : CON 10 pour rester à 1 PV au lieu de 0 (automatique en rage)",
        )],
    ),
    (
        Voie::Rage,
        3,
        &[
            d(
                "rage 1/jour (+1/rang 4 de barbare) : +1d4° DM au contact, −2 DEF, dé bonus aux tests de VOL",
            ),
            d("en rage : ni retraite ni attaque à distance ; VOL 15 pour arrêter avant la fin"),
        ],
    ),
    (
        Voie::Rage,
        4,
        &[d(
            "1/combat : ignore les DM d'un critique ; peut entrer en rage par une action gratuite",
        )],
    ),
    (
        Voie::Rage,
        5,
        &[d(
            "consomme deux rages : +2d4° DM, −4 DEF ; VOL 20 pour arrêter avant la fin",
        )],
    ),
    // Voie du cavalier.
    (
        Voie::Cavalier,
        1,
        &[
            d(
                "cheval de guerre : DEF 12 + rang, PV 10 + niveau × 4, Init du chevalier, attaque +5, DM 1d4° + 5",
            ),
            d("monté : 10 m de déplacement en plus autour d'une action"),
            t(1, "rang + 2 équitation et dressage"),
        ],
    ),
    (
        Voie::Cavalier,
        2,
        &[
            d("monté : +1 DM au contact (+2 au rang 5)"),
            d("la monture utilise sa DEF ; monter en selle est gratuit"),
        ],
    ),
    (
        Voie::Cavalier,
        3,
        &[d(
            "charge montée de 10 à 20 m : dé bonus, +1d4° DM ; FOR 20 pour lui barrer la route",
        )],
    ),
    (
        Voie::Cavalier,
        4,
        &[d(
            "peut appeler sa monture d'un autre plan (L), où elle guérit entièrement",
        )],
    ),
    (
        Voie::Cavalier,
        5,
        &[
            d(
                "monture puissante : DEF 20, PV 10 + niveau × 6, DM 2d4° + 5, attaque gratuite 1/round",
            ),
            d("monture volante au niveau 9, avec l'accord du MJ"),
        ],
    ),
    // Voie de la guerre.
    (
        Voie::Guerre,
        1,
        &[
            d("malus d'armure aux tests ÷ 2"),
            c("en armure de plaques : +1 DEF par rang 5 de chevalier"),
        ],
    ),
    (
        Voie::Guerre,
        2,
        &[d(
            "retire la DEF de l'armure (+ bouclier au rang 5) aux DM d'une attaque au contact ; ni étourdi ni renversé par elle",
        )],
    ),
    (
        Voie::Guerre,
        3,
        &[d(
            "attaque au contact : DM ÷ 2 même ratée (pas sur un 1 naturel)",
        )],
    ),
    (
        Voie::Guerre,
        4,
        &[c("FOR +1"), die("dé bonus aux tests de FOR")],
    ),
    (
        Voie::Guerre,
        5,
        &[d(
            "1/combat : coup d'armure automatique, [1d4° + FOR] DM ; cible de FOR inférieure renversée, étourdie ou repoussée",
        )],
    ),
    // Voie du preux.
    (
        Voie::Preux,
        1,
        &[
            d("1/combat : reporte les DM d'une attaque (pas un critique) à la fin du combat"),
            t(1, "rang + 2 haranguer une foule"),
        ],
    ),
    (
        Voie::Preux,
        2,
        &[d(
            "contre les attaques à distance : RD selon l'armure (plaques 3, mailles 2, cuir 1)",
        )],
    ),
    (
        Voie::Preux,
        3,
        &[d(
            "contre un chef ennemi identifiable : +1d4° DM au contact ; il doit l'attaquer (INT 15)",
        )],
    ),
    (
        Voie::Preux,
        4,
        &[c("CHA +1"), die("dé bonus aux tests de CHA")],
    ),
    (
        Voie::Preux,
        5,
        &[d(
            "3 ennemis ou plus l'attaquent au contact dans le round : une action d'attaque en plus",
        )],
    ),
    // Voie du meneur d'hommes.
    (
        Voie::MeneurDHomme,
        1,
        &[
            d("immunité à la peur ; ses alliés ajoutent son CHA contre la peur"),
            t(1, "rang + 2 stratégie et commandement"),
        ],
    ),
    (
        Voie::MeneurDHomme,
        2,
        &[d(
            "1/round : subit une attaque visant un allié au contact, avec sa DEF, DM − rang",
        )],
    ),
    (
        Voie::MeneurDHomme,
        3,
        &[d(
            "1/round : dé bonus à un allié qui attaque un ennemi au contact de lui",
        )],
    ),
    (
        Voie::MeneurDHomme,
        4,
        &[d(
            "1/combat : lui et ses alliés en vue gagnent 10 m, dé bonus et +1d4° DM en attaque",
        )],
    ),
    (
        Voie::MeneurDHomme,
        5,
        &[d(
            "1/round : un allié en vue gagne une action d'attaque ou de mouvement (1/allié/combat)",
        )],
    ),
    // Voie de la noblesse.
    (
        Voie::Noblesse,
        1,
        &[
            d("sait lire et écrire ; une langue de plus"),
            t(1, "rang + 2 histoire, héraldique, géographie, étiquette"),
        ],
    ),
    (
        Voie::Noblesse,
        2,
        &[
            d("un écuyer fidèle (caractéristiques selon le niveau)"),
            c("critique au contact +1"),
            d("après une récupération complète, le groupe soigne +1d4° PV"),
        ],
    ),
    (
        Voie::Noblesse,
        3,
        &[
            t(1, "rang + 2 commander et intimider"),
            d("peut porter la plaque complète (DEF +7) avec toutes les capacités de chevalier"),
        ],
    ),
    (
        Voie::Noblesse,
        4,
        &[d(
            "contre la piétaille (4 créatures semblables ou plus, jamais des cavaliers) : +1d4° DM",
        )],
    ),
    (
        Voie::Noblesse,
        5,
        &[
            d("capacité de rang 1 à 3 d'un profil combattant ou aventurier"),
            die("dé bonus aux tests d'une caractéristique choisie"),
        ],
    ),
    // Voie du bouclier.
    (
        Voie::Bouclier,
        1,
        &[
            d("1/round : +2 DEF à un allié au contact contre une attaque"),
            t(1, "rang + 2 pour éviter la surprise"),
        ],
    ),
    (
        Voie::Bouclier,
        2,
        &[
            d(
                "attaque au contact opposée (AGI possible) : bloque une attaque avant son tour (DM ÷ 2 d'un ennemi énorme)",
            ),
            d("au rang 5 : action gratuite avec dé malus"),
        ],
    ),
    (
        Voie::Bouclier,
        3,
        &[
            c("avec un bouclier : +1 DEF (+2 au rang 5)"),
            d("DM de zone − rang"),
        ],
    ),
    (
        Voie::Bouclier,
        4,
        &[d(
            "au lieu de parer : test opposé à l'attaque magique pour absorber un sort",
        )],
    ),
    (
        Voie::Bouclier,
        5,
        &[d(
            "renvoie un sort absorbé à son lanceur (pas un sort de zone)",
        )],
    ),
    // Voie du combat.
    (
        Voie::Combat,
        1,
        &[
            c("+3 Init"),
            d("+3 pour éviter immobilisé ou renversé"),
            d("une action de mouvement de plus par combat"),
        ],
    ),
    (Voie::Combat, 2, &[d("dé bonus aux manœuvres de combat")]),
    (
        Voie::Combat,
        3,
        &[d(
            "accepte un dé malus pour +2d4° DM au contact (+3d4° en action limitée)",
        )],
    ),
    (Voie::Combat, 4, &[d("deux attaques au contact à −2")]),
    (
        Voie::Combat,
        5,
        &[d(
            "une attaque au contact à −2 contre chaque ennemi au contact",
        )],
    ),
    // Voie du maître d'armes.
    (
        Voie::MaitreDArmes,
        1,
        &[
            d(
                "+1 en attaque avec une catégorie d'armes choisie (épées, haches, masses, lances, armes de jet)",
            ),
            t(1, "rang + 2 expertise des armes, réputation martiale"),
        ],
    ),
    (
        Voie::MaitreDArmes,
        2,
        &[d("armes de prédilection : critique +1")],
    ),
    (
        Voie::MaitreDArmes,
        3,
        &[
            d("armes de prédilection : +1 DM"),
            d("chaque rang 5 de guerrier : une catégorie de plus ou +1 DM (max. +6)"),
        ],
    ),
    (
        Voie::MaitreDArmes,
        4,
        &[d(
            "arme de prédilection : dé bonus et +1d4° DM, ou désarme un ennemi de NC < bonus aux DM",
        )],
    ),
    (
        Voie::MaitreDArmes,
        5,
        &[d(
            "1/round : attaque contre un ennemi au contact qui vient de le rater",
        )],
    ),
    // Voie de la résistance.
    (
        Voie::Resistance,
        1,
        &[
            c("PV max + (rang + 2)"),
            t(1, "rang + 2 pour résister à l'effort, la chaleur, le froid"),
        ],
    ),
    (
        Voie::Resistance,
        2,
        &[
            d("repos court en 10 min (5 au rang 4)"),
            t(1, "+ rang pour résister à étourdi et affaibli"),
        ],
    ),
    (
        Voie::Resistance,
        3,
        &[
            c("+1 DEF, sauf avec l'option armure lourde"),
            d("option : armure de plaques permise avec toutes les capacités de guerrier"),
        ],
    ),
    (
        Voie::Resistance,
        4,
        &[c("CON +1"), die("dé bonus aux tests de CON")],
    ),
    (
        Voie::Resistance,
        5,
        &[
            c("+1 DEF"),
            d("1/combat : agit un round de plus à 0 PV"),
            d("pas de dé malus immobilisé ; peut attaquer étourdi"),
        ],
    ),
    // Voie du soldat.
    (
        Voie::Soldat,
        1,
        &[
            d("1/round : attaque gratuite contre une créature au contact qui s'éloigne"),
            t(1, "rang + 2 contre l'alcool et les privations"),
        ],
    ),
    (
        Voie::Soldat,
        2,
        &[d(
            "1/round, après le jet : 1d4° PV pour +5 à un test de FOR ou de CON",
        )],
    ),
    (
        Voie::Soldat,
        3,
        &[d(
            "1/round : attaque gratuite contre un ennemi au contact qui en attaque un autre",
        )],
    ),
    (
        Voie::Soldat,
        4,
        &[c("FOR +1"), die("dé bonus aux tests de FOR")],
    ),
    (
        Voie::Soldat,
        5,
        &[
            d("le rang 1 vaut contre AGI + 2 ennemis par round ; touché, l'ennemi s'arrête"),
            c("+1 DEF"),
        ],
    ),
    // Voie de l'air.
    (
        Voie::Air,
        1,
        &[
            d("message de 10 mots à une cible connue ou visible (CHA × 100 m), entend la réponse"),
            c("+1 Init, +1 DEF"),
        ],
    ),
    (
        Voie::Air,
        2,
        &[
            d("CHA min : qui l'attaque au contact subit 1d4° DM"),
            d("action d'attaque : décharge à 10 m, [1d4° + CHA] DM"),
        ],
    ),
    (
        Voie::Air,
        3,
        &[d(
            "déplace un objet ou une cible consentante de 50 kg/rang à 20 m pendant CHA min ; peut la lâcher sur une cible surprise",
        )],
    ),
    (
        Voie::Air,
        4,
        &[d("ligne de 10 m : [4d4° + CHA] DM (AGI : ÷ 2)")],
    ),
    (
        Voie::Air,
        5,
        &[d(
            "intangible CHA min : traverse les murs, immunité aux DM physiques ; ni attaque ni sort",
        )],
    ),
    // Voie de la divination.
    (
        Voie::Divination,
        1,
        &[
            d(
                "attaque magique opposée : révèle l'identité publique d'une créature de NC inférieur",
            ),
            c("+1 Init et +1 DEF (+1 au rang 3, +1 par rang 5 d'ensorceleur)"),
        ],
    ),
    (
        Voie::Divination,
        2,
        &[d(
            "CHA min : détecte les créatures invisibles ou cachées à 20 m",
        )],
    ),
    (
        Voie::Divination,
        3,
        &[d(
            "CHA rounds : voit et entend un lieu connu, ou derrière une porte touchée",
        )],
    ),
    (
        Voie::Divination,
        4,
        &[
            c("PER +1"),
            die("dé bonus aux tests de PER"),
            c("+PER aux PM"),
        ],
    ),
    (
        Voie::Divination,
        5,
        &[d(
            "1/combat, pendant un round : +10 en attaque, DEF et PER, DM subis ÷ 2, agit quand il veut",
        )],
    ),
    // Voie de l'envoûteur.
    (
        Voie::Envouteur,
        1,
        &[
            d(
                "attaque magique opposée (20 m) : la cible obéit à un ordre de 2 ou 3 mots au tour suivant",
            ),
            t(1, "rang + 2 persuasion et séduction"),
        ],
    ),
    (
        Voie::Envouteur,
        2,
        &[d(
            "1/combat : [1d4° + CHA] créatures de faible NC dans une zone de 10 m dorment CHA min (NC max. relevé aux rangs 4 et 5)",
        )],
    ),
    (
        Voie::Envouteur,
        3,
        &[d(
            "attaque magique opposée : cible confuse CHA rounds (VOL pour en sortir)",
        )],
    ),
    (
        Voie::Envouteur,
        4,
        &[d(
            "attaque magique opposée : un humanoïde de niveau inférieur agit en vieil ami jusqu'à ce qu'on l'attaque",
        )],
    ),
    (
        Voie::Envouteur,
        5,
        &[d(
            "attaque magique opposée : contrôle une cible de niveau inférieur CHA min (une seule action si trop haut)",
        )],
    ),
    // Voie des illusions.
    (
        Voie::Illusions,
        1,
        &[
            d(
                "illusion fixe visuelle et sonore (2 m de côté par rang, 50 m) pendant CHA min ; animée au rang 4",
            ),
            t(1, "rang + 2 pour tromper"),
        ],
    ),
    (
        Voie::Illusions,
        2,
        &[d(
            "[1d4 + CHA] rounds : une attaque qui le touche est annulée sur 5-6 au d6",
        )],
    ),
    (
        Voie::Illusions,
        3,
        &[d(
            "attaque illusoire : [3d4° + CHA] à une cible ou [2d4° + CHA] à rang cibles ; un test de PER l'annule",
        )],
    ),
    (
        Voie::Illusions,
        4,
        &[d(
            "prend l'apparence d'une créature vue, de même taille, pendant CHA min",
        )],
    ),
    (
        Voie::Illusions,
        5,
        &[d(
            "attaque magique opposée : un humanoïde vivant tombe à 0 PV (de niveau égal ou supérieur : étourdi 1 round)",
        )],
    ),
    // Voie de l'invocation.
    (
        Voie::Invocation,
        1,
        &[d(
            "attaque magique contre la DEF à 20 m : [1d4° + CHA] DM ; cible de faible NC renversée sauf FOR 10",
        )],
    ),
    (
        Voie::Invocation,
        2,
        &[d(
            "force invisible qui fait des tâches simples à 20 m pendant CHA min (FOR = CHA)",
        )],
    ),
    (
        Voie::Invocation,
        3,
        &[d(
            "lame d'énergie rang rounds : attaque magique gratuite chaque round à 20 m, [1d4° + CHA] DM",
        )],
    ),
    (
        Voie::Invocation,
        4,
        &[d("se téléporte avec CHA alliés jusqu'à 60 m, en vue")],
    ),
    (
        Voie::Invocation,
        5,
        &[d(
            "mur invisible indestructible ou dôme de 3 m pendant CHA min",
        )],
    ),
    // Voie des artefacts.
    (
        Voie::Artefacts,
        1,
        &[
            c("bâton : [1d4° + INT] DM magiques"),
            d(
                "au rang 3 : attaque au contact en action limitée avec l'attaque magique, [2d4° + INT] DM",
            ),
        ],
    ),
    (
        Voie::Artefacts,
        2,
        &[
            d(
                "ouvre une porte fermée (attaque magique contre la serrure) ou la scelle INT min (+5 solidité et RD)",
            ),
            d("au rang 4 : scellé permanent avec une gemme de 100 pa"),
        ],
    ),
    (
        Voie::Artefacts,
        3,
        &[d(
            "sac de 50 kg par rang ; 1/heure, des objets temporaires d'au plus 25 pa",
        )],
    ),
    (
        Voie::Artefacts,
        4,
        &[d(
            "onde de choc de 10 m : [3d4° + INT] DM automatiques ; FOR ou renversé",
        )],
    ),
    (
        Voie::Artefacts,
        5,
        &[d(
            "appareil : Téléportation, Interruption du temps, Forme éthérée, Prescience, 1/jour chacun (échoue sur 1-2)",
        )],
    ),
    // Voie des élixirs.
    (
        Voie::Elixirs,
        1,
        &[
            d("soigne 1d4° PV et donne un dé bonus aux 3 tests suivants dans les 30 min"),
            t(1, "rang + 2 alchimie et identifier les potions"),
        ],
    ),
    (
        Voie::Elixirs,
        2,
        &[d(
            "lancé à 10 m, explosion de 3 m : 2d4° DM (3d4° au rang 4, 4d4° au rang 5) ; AGI : ÷ 2",
        )],
    ),
    (
        Voie::Elixirs,
        3,
        &[d(
            "après 1 min : soigne [2d4° + INT] PV ou guérit un poison",
        )],
    ),
    (
        Voie::Elixirs,
        4,
        &[d(
            "apprend INT élixirs (max. 4) : Forme gazeuse, Maîtrise des éléments, Chute ralentie, Masque mortuaire",
        )],
    ),
    (
        Voie::Elixirs,
        5,
        &[d(
            "apprend INT élixirs (max. 4, chacun en vaut deux) : Invisibilité, Vol, Accélération, Masque du prédateur",
        )],
    ),
    // Voie du métal.
    (
        Voie::Metal,
        1,
        &[
            d("bâton ou marteau enflammé INT min : +2 DM de feu (+1/rang 4 de forgesort)"),
            t(1, "rang + 2 forge"),
        ],
    ),
    (
        Voie::Metal,
        2,
        &[d(
            "attaque magique opposée : chauffe un objet de métal porté ; arme : −2 en attaque et 1 DM/round ; armure : 1d4° DM/round",
        )],
    ),
    (
        Voie::Metal,
        3,
        &[d(
            "INT min : +2 DEF contre les armes de métal ; DM des projectiles à pointe de métal ÷ 2",
        )],
    ),
    (
        Voie::Metal,
        4,
        &[d(
            "attaque magique opposée : tord un objet de métal (arme hors d'usage, armure : dé malus) ; 3d4° DM à une structure",
        )],
    ),
    (
        Voie::Metal,
        5,
        &[
            d("DM de feu ÷ 2"),
            c("CON +1"),
            die("dé bonus aux tests de CON"),
            c("option : +CON aux PM"),
        ],
    ),
    // Voie du golem.
    (
        Voie::Golem,
        1,
        &[
            d("tests d'INT à la place des tests de FOR"),
            c("option : INT au lieu de CON pour les PV du niveau 1"),
            t(1, "rang + 2 bricolage et sciences"),
        ],
    ),
    (
        Voie::Golem,
        2,
        &[d(
            "golem serviteur : DEF 10 + rang, PV niveau × 5, Init et attaque magique du forgesort, DM 1d4° + 1",
        )],
    ),
    (
        Voie::Golem,
        3,
        &[d(
            "1/round : le golem subit une attaque à la place d'un personnage au contact",
        )],
    ),
    (
        Voie::Golem,
        4,
        &[d(
            "change le golem en figurine (RD 10) ; il revient par une action de mouvement",
        )],
    ),
    (
        Voie::Golem,
        5,
        &[
            d("une amélioration du golem, +1/rang 5 de forgesort"),
            d(
                "au choix : armure, félin, baliste, taille, vol, cerveau, puissance, arme à deux mains",
            ),
        ],
    ),
    // Voie des runes.
    (
        Voie::Runes,
        1,
        &[
            c("+2 DEF (+1 au rang 3 et au rang 5)"),
            d("peut être placée sur le golem"),
        ],
    ),
    (
        Voie::Runes,
        2,
        &[d(
            "arme enchantée 24 h : 1/combat, DM maximaux (hors dés bonus)",
        )],
    ),
    (
        Voie::Runes,
        3,
        &[d(
            "armure enchantée 24 h : 1/jour, ignore les DM d'une attaque (un critique inflige ses DM)",
        )],
    ),
    (
        Voie::Runes,
        4,
        &[d(
            "bijou enchanté 24 h : 1/combat, un d20 de plus pour une attaque ou un test de caractéristique",
        )],
    ),
    (
        Voie::Runes,
        5,
        &[d(
            "rituel de 10 min (3 PM, concentration) : zone de 10 m gardée 12 h, alarme ou [3d4° + INT] DM élémentaires",
        )],
    ),
    // Voie de la magie des arcanes.
    (
        Voie::MagieDesArcanes,
        1,
        &[d(
            "1d4° DM automatiques à 30 m, le dé explose une fois ; +1 DM/rang 4 de magicien (max. INT)",
        )],
    ),
    (
        Voie::MagieDesArcanes,
        2,
        &[d(
            "INT min : par action de mouvement, monte de 5 m ou descend de 10 m",
        )],
    ),
    (
        Voie::MagieDesArcanes,
        3,
        &[d(
            "gazeux 1 min : passe par les fissures, immunité aux armes ordinaires",
        )],
    ),
    (
        Voie::MagieDesArcanes,
        4,
        &[d(
            "[1d4° + INT] rounds : une action de mouvement de plus par round, ou à la place +3 DEF ou −1 PM",
        )],
    ),
    (
        Voie::MagieDesArcanes,
        5,
        &[d(
            "rayon à 20 m : [5d4° + INT] DM ; une créature à 0 PV est désintégrée",
        )],
    ),
    // Voie de la magie destructrice.
    (
        Voie::MagieDestructrice,
        1,
        &[d(
            "jusqu'à 3 cibles adjacentes : [1d4° + INT] DM (2d4° au rang 4) ; AGI : ÷ 2",
        )],
    ),
    (
        Voie::MagieDestructrice,
        2,
        &[d(
            "attaque magique opposée : −2 aux tests de FOR, à l'attaque au contact et aux DM jusqu'à la fin du combat",
        )],
    ),
    (
        Voie::MagieDestructrice,
        3,
        &[d(
            "attaque magique à 30 m : [3d4° + INT] DM, puis 1d6 par round jusqu'à un 1 ou un 2",
        )],
    ),
    (
        Voie::MagieDestructrice,
        4,
        &[d(
            "explosion à 30 m, rayon 5 m : [4d4° + INT] DM ; AGI : ÷ 2",
        )],
    ),
    (
        Voie::MagieDestructrice,
        5,
        &[d(
            "une attaque magique contre chaque cible choisie à 10 m : [2d4° + INT] DM",
        )],
    ),
    // Voie de la magie élémentaire.
    (
        Voie::MagieElementaire,
        1,
        &[d(
            "attaque magique opposée : 1d4° DM/round pendant INT rounds (pas contre qui ne respire pas)",
        )],
    ),
    (
        Voie::MagieElementaire,
        2,
        &[d(
            "INT min : DM élémentaires subis − (rang + 2) ; peut changer l'élément d'un sort",
        )],
    ),
    (
        Voie::MagieElementaire,
        3,
        &[d(
            "INT min : arme +1d4° DM élémentaires ; les sorts de cet élément coûtent 1 PM de moins",
        )],
    ),
    (
        Voie::MagieElementaire,
        4,
        &[d(
            "respire sous l'eau 10 min, avec un compagnon de plus par point d'INT",
        )],
    ),
    (
        Voie::MagieElementaire,
        5,
        &[d(
            "INT min : −5 à tous les DM subis, jusqu'à niveau × 3 DM absorbés",
        )],
    ),
    // Voie de la magie protectrice.
    (
        Voie::MagieProtectrice,
        1,
        &[d(
            "INT min : +3 DEF (+4 au rang 3, +1/rang 5 de magicien ou de la voie du mage) ; compte comme une armure, jamais avec une",
        )],
    ),
    (
        Voie::MagieProtectrice,
        2,
        &[d(
            "jusqu'à INT cibles à 10 m chutent sans DM, même hors de son tour",
        )],
    ),
    (
        Voie::MagieProtectrice,
        3,
        &[d("[1d4° + INT] rounds : DM d'armes subis et infligés ÷ 2")],
    ),
    (
        Voie::MagieProtectrice,
        4,
        &[d(
            "cercle de 2 m pour INT personnes : test opposé pour annuler un sort ; invoqués et morts-vivants : dé malus en attaque",
        )],
    ),
    (
        Voie::MagieProtectrice,
        5,
        &[d(
            "INT rounds hors du temps pour utiliser sorts et objets sur lui-même",
        )],
    ),
    // Voie de la magie universelle.
    (
        Voie::MagieUniverselle,
        1,
        &[
            d("un objet éclaire à 10 m pendant INT h"),
            d("1/combat : aveugle 1 round une créature de faible NC"),
        ],
    ),
    (
        Voie::MagieUniverselle,
        2,
        &[d(
            "familier (petit animal) qui partage ses sens ; en vue : +2 Init et DEF ; s'il tombe : 1d4° PV de contrecoup",
        )],
    ),
    (
        Voie::MagieUniverselle,
        3,
        &[d(
            "invisible [1d4° + INT] min, jusqu'à ce qu'il attaque ; sur un allié au rang 5 (L)",
        )],
    ),
    (
        Voie::MagieUniverselle,
        4,
        &[d("vole [2d4° + INT] min à sa vitesse au sol")],
    ),
    (
        Voie::MagieUniverselle,
        5,
        &[d(
            "1/jour : se téléporte jusqu'à niveau × INT km ; avec des alliés au niveau 10",
        )],
    ),
    // Voie du démon.
    (
        Voie::Demon,
        1,
        &[d(
            "attaque magique opposée : dé malus au prochain test de la cible (3 tests en L) ; 1/combat par cible",
        )],
    ),
    (
        Voie::Demon,
        2,
        &[d(
            "INT min : dé bonus aux tests de CHA ; toucher qui draine [1d4° + INT] PV et le soigne d'autant",
        )],
    ),
    (
        Voie::Demon,
        3,
        &[
            d("sacrifie 1d4° PV : + INT à un d20 ou à la DEF"),
            d("ajoute la VOL à ses DR"),
        ],
    ),
    (
        Voie::Demon,
        4,
        &[d(
            "forme de démon ailé INT min : dé bonus en attaque au contact, +5 DEF et tests physiques, 2 griffes [1d4° + INT], vol",
        )],
    ),
    (
        Voie::Demon,
        5,
        &[d(
            "sacrifie 1d4° PV : invoque un démon ailé (DEF 18, PV niveau × 5, DM 2d4° + 5) pendant INT min",
        )],
    ),
    // Voie de la mort.
    (
        Voie::Mort,
        1,
        &[d(
            "1/round : un humanoïde qui meurt à 20 m le soigne de NC PV (ou 1 PM au rang 3 si NC > 4)",
        )],
    ),
    (
        Voie::Mort,
        2,
        &[d(
            "INT min : compte comme non-vivant (ignoré des morts-vivants, DM de froid ÷ 2, ne peut être soigné)",
        )],
    ),
    (
        Voie::Mort,
        3,
        &[d(
            "attaque magique opposée à 30 m : [2d4° + INT] DM, dont il se soigne",
        )],
    ),
    (
        Voie::Mort,
        4,
        &[d(
            "attaque magique opposée : la cible fuit INT rounds ; en L, toutes les créatures au contact",
        )],
    ),
    (
        Voie::Mort,
        5,
        &[d(
            "attaque magique opposée à 20 m : [5d4° + INT] DM (÷ 2 si ratée) ; 1/combat par cible",
        )],
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
    /// changing a value (a characteristic in place of another) or a weapon's
    /// attack, DM or dice from the ranks below it.
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
        // A total without the other capacités' terms: a lower rank reaching
        // a higher amount (Doigts agiles at rank 5) is not this capacité.
        // A spell's 1 PM is the rule's, not the capacité's.
        let total = |c: &Character, value| {
            c.value(value).map(|line| {
                line.contributions()
                    .iter()
                    .filter(|t| match t.source() {
                        Source::Ability(_) => ability(value, t).as_deref() == Some(id.as_str()),
                        _ => true,
                    })
                    .map(Contribution::amount)
                    .sum::<i32>()
            })
        };
        // PM appear with the first spell (ADR ENG-COF-PM): only a total on
        // both sides compares.
        let changed = |value| match (total(&held, value), total(&below, value)) {
            (Some(held), Some(below)) => held != below,
            _ => false,
        };
        let differs = Value::FIXED.into_iter().any(changed)
            || WEAPONS.iter().any(|w| {
                WeaponUse::ALL.iter().any(|u| {
                    held.damage_dice(*w, *u) != below.damage_dice(*w, *u)
                        || [Value::WeaponAttack(*w, *u), Value::Damage(*w, *u)]
                            .into_iter()
                            .any(changed)
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
