//! The documented effects of capacités on the character's values (D-22,
//! D-13), and the options some capacités offer.
//!
//! Each value reads its own table of effects: a capacité, the amount it
//! gives, the equipment it needs and the option it depends on (ADR
//! ENG-COF-CAPACITES). Effects that depend on circumstances (outdoors,
//! against large creatures, while a companion is in sight) are play-time
//! and absent; so are those of capacités whose content the co-rules skill
//! does not give (the weapon categories of Armes de prédilection).

use co_kernel::{
    Characteristic, Characteristics, DerivedValue, Rank, Severity, Source, Subject, Violation,
};

use crate::{
    Armour, Character, Profil, Voie, Weapon, WeaponUse,
    rules::{ABILITY_MECHANIC, CONTENT_VOIE},
};

/// An option a capacité offers, chosen with
/// [`Character::with_ability_option`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbilityOption {
    /// A characteristic: the one raised (Supériorité elfique of the elfe
    /// haut, Combattant héroïque, the lowest characteristic), the one used
    /// in a formula (Peau de pierre: CON in DEF; Charisme héroïque: CHA for
    /// PM; Endurer: CON to PM; Grosse tête: INT for the level-1 PV), or the
    /// one whose tests get a bonus (Tatouages) or a dé bonus (Formation
    /// d'élite).
    Characteristic(Characteristic),
    /// The family's +1 PC exchanged for +1 DR (Éclaireur).
    RecoveryDice,
    /// A heavier armour instead of DEF (Armure lourde: armure de plaques;
    /// Vêtements sacrés: cotte de mailles, for a warrior god).
    Armour,
    /// The thrown weapons variant (Archer émérite).
    Thrown,
}

impl AbilityOption {
    /// Every option, the characteristics first in their kernel order.
    pub const ALL: [Self; 10] = [
        Self::Characteristic(Characteristic::Agi),
        Self::Characteristic(Characteristic::Con),
        Self::Characteristic(Characteristic::For),
        Self::Characteristic(Characteristic::Per),
        Self::Characteristic(Characteristic::Cha),
        Self::Characteristic(Characteristic::Int),
        Self::Characteristic(Characteristic::Vol),
        Self::RecoveryDice,
        Self::Armour,
        Self::Thrown,
    ];

    /// Returns the option whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|option| option.id() == id)
    }

    /// Returns the stable identifier of the option.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Characteristic(Characteristic::Agi) => "cof.option.agi",
            Self::Characteristic(Characteristic::Con) => "cof.option.con",
            Self::Characteristic(Characteristic::For) => "cof.option.for",
            Self::Characteristic(Characteristic::Per) => "cof.option.per",
            Self::Characteristic(Characteristic::Cha) => "cof.option.cha",
            Self::Characteristic(Characteristic::Int) => "cof.option.int",
            Self::Characteristic(Characteristic::Vol) => "cof.option.vol",
            Self::RecoveryDice => "cof.option.recovery-dice",
            Self::Armour => "cof.option.armour",
            Self::Thrown => "cof.option.thrown",
        }
    }

    /// The French display name of the option.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Characteristic(c) => c.abbreviation(),
            Self::RecoveryDice => "1 DR au lieu du PC de la famille",
            Self::Armour => "L'armure plus lourde",
            Self::Thrown => "La variante armes de jet",
        }
    }
}

/// Whether a capacité's option must be chosen, or replaces a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Need {
    /// The capacité has no effect until an option is chosen.
    Required,
    /// Without an option, the capacité keeps its default effect.
    Optional,
}

/// The options the capacité at `rank` in `voie` offers, before reading the
/// character's values: the lowest characteristic is resolved by
/// [`options`].
enum Offer {
    /// These options.
    Options(Need, &'static [AbilityOption]),
    /// One of the lowest characteristics, ties included.
    Lowest,
}

/// Every characteristic, as options.
const ANY_CHARACTERISTIC: [AbilityOption; 7] = {
    let [agi, con, force, per, cha, int, vol, ..] = AbilityOption::ALL;
    [agi, con, force, per, cha, int, vol]
};

/// The capacités offering options, as the co-rules catalogue describes them.
const fn offer(voie: Voie, rank: u8) -> Option<Offer> {
    use AbilityOption::{Armour, Characteristic as C, RecoveryDice, Thrown};
    use Characteristic::{Agi, Cha, Con, For, Int};
    Some(match (voie, rank) {
        // Tatouages' bonus and Formation d'élite's dé bonus to the tests of
        // a characteristic (D-50).
        (Voie::Pagne, 3) | (Voie::Noblesse, 5) => {
            Offer::Options(Need::Required, &ANY_CHARACTERISTIC)
        }
        (Voie::ElfeHaut, 5) => Offer::Options(Need::Required, &[C(Int), C(Cha)]),
        (Voie::CombatADeuxArmes, 4) => Offer::Options(Need::Required, &[C(Agi), C(For)]),
        (Voie::Humain | Voie::Maitrise | Voie::Meditation | Voie::Vent, 5) => Offer::Lowest,
        (Voie::Pagne, 2) | (Voie::Metal, 5) => Offer::Options(Need::Optional, &[C(Con)]),
        (Voie::Seduction, 4) => Offer::Options(Need::Optional, &[C(Cha)]),
        (Voie::Golem, 1) => Offer::Options(Need::Optional, &[C(Int)]),
        (Voie::Resistance, 3) | (Voie::Spiritualite, 1) => {
            Offer::Options(Need::Optional, &[Armour])
        }
        (Voie::Traqueur, 1) => Offer::Options(Need::Optional, &[RecoveryDice]),
        (Voie::Archer, 1) => Offer::Options(Need::Optional, &[Thrown]),
        _ => return None,
    })
}

/// The options the capacité at `rank` in `voie` offers the character; empty
/// when it offers none. The lowest characteristics are read on the starting
/// values, before any capacité raises one.
pub(crate) fn options(character: &Character, voie: Voie, rank: Rank) -> Vec<AbilityOption> {
    match offer(voie, rank.get()) {
        None => Vec::new(),
        Some(Offer::Options(_, options)) => options.to_vec(),
        Some(Offer::Lowest) => {
            let values = character.starting_characteristics();
            let lowest = values.iter().map(|(_, value)| value.get()).min();
            values
                .iter()
                .filter(|(_, value)| Some(value.get()) == lowest)
                .map(|(c, _)| AbilityOption::Characteristic(c))
                .collect()
        }
    }
}

/// Whether the capacité at `rank` in `voie` has no effect until one of its
/// options is chosen (the raises); a capacité with a default keeps it.
pub(crate) fn option_required(voie: Voie, rank: Rank) -> bool {
    matches!(
        offer(voie, rank.get()),
        Some(Offer::Lowest | Offer::Options(Need::Required, _))
    )
}

/// Whether the character holds the capacité at `rank` in `voie`: acquired,
/// or, for the rank-1 capacité of its voie de peuple, kept through the voie
/// du mage that replaced it (COF.CRE.VOIE-DU-MAGE).
pub(crate) fn holds(character: &Character, voie: Voie, rank: Rank) -> bool {
    let kept = rank.get() == 1
        && character.peuple().and_then(crate::Peuple::voie) == Some(voie)
        && character.has_rank(Voie::Mage, Rank::new(1));
    character.has_rank(voie, rank) || kept
}

/// The rank reached in `voie`: the highest rank held (CO.VOIE.RANK-SCALING).
fn rank_reached(character: &Character, voie: Voie) -> u8 {
    character.highest_rank(voie).map_or(0, Rank::get)
}

/// Whether the option `option` is chosen for the capacité at `rank` in
/// `voie`.
pub(crate) fn chose(character: &Character, voie: Voie, rank: u8, option: AbilityOption) -> bool {
    character.ability_option(voie, Rank::new(rank)) == Some(option)
}

/// How much a capacité gives.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Amount {
    /// A flat amount.
    Flat(i32),
    /// The amount of the last step whose rank is reached in the voie
    /// ("+1, +2 at rank 3").
    Steps(&'static [(u8, i32)]),
    /// This much per rank reached in the voie.
    PerRank(i32),
    /// The rank reached in the voie, plus this much.
    RankPlus(i32),
    /// The current value of a characteristic.
    Characteristic(Characteristic),
    /// The current value of a characteristic, at most the rank reached.
    CharacteristicUpToRank(Characteristic),
    /// This much per voie of the profil whose rank (the `u8`) is held.
    PerVoieAt(Profil, u8, i32),
    /// A competence bonus of a voie de profil: 2 + the rank reached, at most
    /// +7 (COF.DER.COMPETENCE).
    Competence,
}

/// What the equipment, or the level, must be for an effect to apply.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Condition {
    /// Whatever the equipment.
    Always,
    /// No armour worn.
    NoArmour,
    /// An armour worn.
    Armour,
    /// An armure de plaques or a plaque complète worn.
    Plate,
    /// A shield carried.
    Shield,
    /// From this level on.
    FromLevel(u8),
}

/// Which option, if any, an effect depends on.
#[derive(Debug, Clone, Copy)]
pub(crate) enum When {
    /// Whatever the option.
    Always,
    /// Only with this option chosen.
    Chosen(AbilityOption),
    /// Only without this option: the default the option replaces.
    NotChosen(AbilityOption),
}

/// A capacité's effect on one value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Effect {
    voie: Voie,
    rank: u8,
    amount: Amount,
    condition: Condition,
    when: When,
}

impl Effect {
    /// An effect of the capacité at `rank` in `voie`, whatever the equipment
    /// and the options.
    pub(crate) const fn new(voie: Voie, rank: u8, amount: Amount) -> Self {
        Self {
            voie,
            rank,
            amount,
            condition: Condition::Always,
            when: When::Always,
        }
    }

    /// A flat amount.
    pub(crate) const fn flat(voie: Voie, rank: u8, amount: i32) -> Self {
        Self::new(voie, rank, Amount::Flat(amount))
    }

    /// The same effect, only with `condition` met.
    pub(crate) const fn only(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    /// The same effect, only `when` the option allows it.
    pub(crate) const fn when(mut self, when: When) -> Self {
        self.when = when;
        self
    }

    /// The voie of the capacité.
    pub(crate) const fn voie(&self) -> Voie {
        self.voie
    }

    /// The rank of the capacité.
    pub(crate) const fn rank(&self) -> u8 {
        self.rank
    }

    /// The amount the effect gives the character, `None` when it does not
    /// apply.
    fn amount(&self, character: &Character, values: &Characteristics) -> Option<i32> {
        if !holds(character, self.voie, Rank::new(self.rank)) {
            return None;
        }
        let applies = match self.condition {
            Condition::Always => true,
            Condition::NoArmour => character.armour().is_none(),
            Condition::Armour => character.armour().is_some(),
            Condition::Plate => matches!(
                character.armour(),
                Some(Armour::ArmureDePlaques | Armour::PlaqueComplete)
            ),
            Condition::Shield => character.shield().is_some(),
            Condition::FromLevel(level) => character.level().get() >= level,
        } && match self.when {
            When::Always => true,
            When::Chosen(option) => chose(character, self.voie, self.rank, option),
            When::NotChosen(option) => !chose(character, self.voie, self.rank, option),
        };
        if !applies {
            return None;
        }
        let rank = rank_reached(character, self.voie);
        let amount = match self.amount {
            Amount::Flat(amount) => amount,
            Amount::Steps(steps) => steps
                .iter()
                .rev()
                .find(|(step, _)| *step <= rank)
                .map_or(0, |(_, amount)| *amount),
            Amount::PerRank(amount) => amount * i32::from(rank),
            Amount::RankPlus(amount) => i32::from(rank) + amount,
            Amount::Characteristic(c) => values[c].get(),
            Amount::CharacteristicUpToRank(c) => values[c].get().min(i32::from(rank)),
            Amount::PerVoieAt(profil, held, amount) => {
                let held = Rank::new(held);
                let voies = profil
                    .voies()
                    .iter()
                    .filter(|voie| character.has_rank(**voie, held))
                    .count();
                amount * i32::try_from(voies).unwrap_or(0)
            }
            Amount::Competence => (i32::from(rank) + 2).min(COMPETENCE_MAX),
        };
        Some(amount)
    }
}

/// The highest competence bonus of a voie de profil (COF.DER.COMPETENCE).
const COMPETENCE_MAX: i32 = 7;

/// Adds every effect of `effects` that applies to the character.
pub(crate) fn apply(
    value: DerivedValue,
    character: &Character,
    values: &Characteristics,
    effects: &[Effect],
) -> DerivedValue {
    effects.iter().fold(value, |value, effect| {
        match effect.amount(character, values) {
            Some(amount) => value.with(
                Source::Ability(effect.voie.ability(Rank::new(effect.rank))),
                amount,
            ),
            None => value,
        }
    })
}

/// Whether any effect of `effects` applies to the character.
pub(crate) fn any(character: &Character, values: &Characteristics, effects: &[Effect]) -> bool {
    effects
        .iter()
        .any(|effect| effect.amount(character, values).is_some())
}

use Amount::{
    Characteristic as Carac, CharacteristicUpToRank, PerRank, PerVoieAt, RankPlus, Steps,
};

/// "+1 per rank 5 held in a voie of `profil`".
const fn per_rank_five(profil: Profil) -> Amount {
    PerVoieAt(profil, 5, 1)
}

/// "+1 at rank 1, +2 at rank 3, +3 at rank 5."
const ONE_TWO_THREE: Amount = Steps(&[(1, 1), (3, 2), (5, 3)]);

/// Effects on PV (COF.DER.PV-L1).
pub(crate) const VIGOUR: &[Effect] = &[
    // Argument de taille.
    Effect::new(Voie::Brute, 1, Carac(Characteristic::For)),
    // Vigueur.
    Effect::new(Voie::Pagne, 1, PerRank(1)),
    // Proche de la nature.
    Effect::flat(Voie::Primitif, 1, 1),
    // Robustesse.
    Effect::new(Voie::Resistance, 1, RankPlus(2)),
    // Maîtrise du ki.
    Effect::new(Voie::Meditation, 3, Carac(Characteristic::Vol)),
];

/// Effects on the number of DR (COF.DER.DR).
pub(crate) const RECOVERY: &[Effect] = &[
    // Éclaireur: +1 DR for the family's +1 PC.
    Effect::flat(Voie::Traqueur, 1, 1).when(When::Chosen(AbilityOption::RecoveryDice)),
];

/// Effects on PC (COF.DER.PC).
pub(crate) const LUCK: &[Effect] = &[
    // Diversité.
    Effect::flat(Voie::Humain, 1, 1),
    // Petit pote.
    Effect::flat(Voie::Gnome, 2, 1),
    // Petit veinard.
    Effect::flat(Voie::Halfelin, 4, 1),
    // Éclaireur: the family's +1 PC exchanged for +1 DR.
    Effect::flat(Voie::Traqueur, 1, -1).when(When::Chosen(AbilityOption::RecoveryDice)),
];

/// Effects on PM, when the character knows a spell (COF.DER.PM).
pub(crate) const MANA: &[Effect] = &[
    // Tour de magie, besides the spell's own.
    Effect::flat(Voie::Mage, 3, 2),
    // Perception héroïque (divination, protecteur).
    Effect::new(Voie::Divination, 4, Carac(Characteristic::Per)),
    Effect::new(Voie::Protecteur, 4, Carac(Characteristic::Per)),
    // Endurer.
    Effect::new(Voie::Metal, 5, Carac(Characteristic::Con)).when(When::Chosen(
        AbilityOption::Characteristic(Characteristic::Con),
    )),
];

/// Effects on Init (COF.DER.INIT).
pub(crate) const INITIATIVE: &[Effect] = &[
    // Réflexes éclair.
    Effect::flat(Voie::Pourfendeur, 1, 3),
    // Murmures dans le vent.
    Effect::flat(Voie::Air, 1, 1),
    // Divination: +1 at rank 3, +1 per rank 5 in an ensorceleur voie.
    Effect::new(Voie::Divination, 1, Steps(&[(1, 1), (3, 2)])),
    Effect::new(Voie::Divination, 1, per_rank_five(Profil::Ensorceleur)),
    // Vitesse du félin.
    Effect::flat(Voie::Fauve, 1, 3),
    // Agile.
    Effect::new(Voie::Deplacement, 1, ONE_TWO_THREE),
    // Souplesse du félin.
    Effect::new(Voie::Aventurier, 3, Steps(&[(3, 2), (5, 3)])),
    // Vivacité.
    Effect::flat(Voie::Combat, 1, 3),
    // Pas du vent.
    Effect::flat(Voie::Vent, 1, 3),
    // Archer émérite, not in its thrown weapons variant.
    Effect::new(Voie::Archer, 1, RankPlus(0)).when(When::NotChosen(AbilityOption::Thrown)),
    // Grâce féline.
    Effect::new(Voie::Saltimbanque, 2, Carac(Characteristic::Cha)),
    // Attaque en finesse.
    Effect::new(Voie::Spadassin, 1, Carac(Characteristic::Agi)),
    // Maîtrise du ki.
    Effect::new(Voie::Meditation, 3, Carac(Characteristic::Vol)),
];

/// Effects on DEF (COF.DER.DEF).
pub(crate) const DEFENCE: &[Effect] = &[
    // Réflexes éclair.
    Effect::new(Voie::Pourfendeur, 1, Steps(&[(1, 1), (5, 2)])),
    // Murmures dans le vent.
    Effect::flat(Voie::Air, 1, 1),
    // Divination: +1 at rank 3, +1 per rank 5 in an ensorceleur voie.
    Effect::new(Voie::Divination, 1, Steps(&[(1, 1), (3, 2)])),
    Effect::new(Voie::Divination, 1, per_rank_five(Profil::Ensorceleur)),
    // Petite taille.
    Effect::flat(Voie::Halfelin, 1, 1),
    // Vitesse du félin.
    Effect::new(Voie::Fauve, 1, ONE_TWO_THREE),
    // Agile.
    Effect::new(Voie::Deplacement, 1, ONE_TWO_THREE),
    // Souplesse du félin.
    Effect::new(Voie::Aventurier, 3, Steps(&[(3, 2), (5, 3)])),
    // Agilité du singe.
    Effect::new(Voie::Maitrise, 1, Steps(&[(1, 2), (4, 3)])),
    // Maîtrise du ki.
    Effect::new(Voie::Meditation, 3, Steps(&[(3, 2), (5, 3)])),
    // Peau de fer.
    Effect::new(Voie::Poing, 2, Steps(&[(2, 2), (5, 3)])),
    // Course du vent.
    Effect::new(Voie::Vent, 2, Steps(&[(2, 1), (5, 2)])),
    // Runes de défense: +1 at rank 3 and at rank 5.
    Effect::new(Voie::Runes, 1, Steps(&[(1, 2), (3, 3), (5, 4)])),
    // Exsangue, -1 in an armour (no bone armour is content).
    Effect::new(Voie::Sang, 3, Steps(&[(3, 2), (5, 3)])),
    Effect::flat(Voie::Sang, 3, -1).only(Condition::Armour),
    // Grâce féline.
    Effect::new(Voie::Saltimbanque, 2, Steps(&[(2, 1), (4, 2)])),
    // Tour de magie.
    Effect::flat(Voie::Mage, 3, 1),
    // Combattant aguerri.
    Effect::flat(Voie::Mercenaire, 3, 1),
    // Combat de masse.
    Effect::flat(Voie::Mercenaire, 5, 1),
    // Dur à cuire.
    Effect::flat(Voie::Resistance, 5, 1),
    // Rempart.
    Effect::flat(Voie::Soldat, 5, 1),
    // Tactiques de meute: +1 per rank 5 in a rôdeur voie.
    Effect::new(Voie::CompagnonAnimal, 5, per_rank_five(Profil::Rodeur)),
    // Peau de pierre, unless CON replaces AGI.
    Effect::new(Voie::Pagne, 2, Steps(&[(2, 1), (4, 2)])).when(When::NotChosen(
        AbilityOption::Characteristic(Characteristic::Con),
    )),
    // Armure lourde, unless the armure de plaques is chosen.
    Effect::flat(Voie::Resistance, 3, 1).when(When::NotChosen(AbilityOption::Armour)),
    // Armure de vent: without armour, or +1 in an armour.
    Effect::new(Voie::Primitif, 2, Steps(&[(2, 2), (5, 3)])).only(Condition::NoArmour),
    Effect::flat(Voie::Primitif, 2, 1).only(Condition::Armour),
    // Vêtements sacrés: without armour, unless the cotte de mailles is chosen.
    Effect::new(Voie::Spiritualite, 1, Steps(&[(1, 2), (3, 3), (5, 4)]))
        .only(Condition::NoArmour)
        .when(When::NotChosen(AbilityOption::Armour)),
    // Dentelles et rapière: CHA without armour, at most the rank.
    Effect::new(
        Voie::Seduction,
        2,
        CharacteristicUpToRank(Characteristic::Cha),
    )
    .only(Condition::NoArmour),
    // Bouclier de la foi, with a shield.
    Effect::new(Voie::GuerreSainte, 2, Steps(&[(2, 1), (5, 2)])).only(Condition::Shield),
    // Défense au bouclier, with a shield.
    Effect::new(Voie::Bouclier, 3, Steps(&[(3, 1), (5, 2)])).only(Condition::Shield),
    // Armure sur mesure: in plate, +1 per rank 5 in a chevalier voie.
    Effect::new(Voie::Guerre, 1, per_rank_five(Profil::Chevalier)).only(Condition::Plate),
];

/// Effects on RD (COF.CNT.EFFECT-SHAPES: damage reduction).
pub(crate) const REDUCTION: &[Effect] = &[
    // Peau d'acier.
    Effect::flat(Voie::Pagne, 5, 3),
    // Fils du roc: RD 2, RD 3 from level 10.
    Effect::flat(Voie::Nain, 4, 2),
    Effect::flat(Voie::Nain, 4, 1).only(Condition::FromLevel(10)),
];

/// The weapons, or the weapon uses, a weapon effect applies to.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Weapons {
    /// Usable at contact.
    Contact,
    /// Shot or thrown.
    Ranged,
    /// Light (equipment table), while no shield is carried.
    Light,
    /// The bows.
    Bows,
    /// The crossbows.
    Crossbows,
    /// The powder weapons.
    Powder,
    /// The axes and the war hammer.
    AxesAndHammers,
    /// Daggers and knives, thrown.
    ThrownDaggers,
    /// Hand-thrown weapons, thrown.
    Thrown,
}

impl Weapons {
    /// Whether `weapon`, in `weapon_use` when the effect reads it, is one of
    /// these weapons.
    fn matches(self, character: &Character, weapon: Weapon, weapon_use: Option<WeaponUse>) -> bool {
        let ranged_use = weapon_use.is_none_or(|weapon_use| weapon_use == WeaponUse::Ranged);
        match self {
            Self::Contact => weapon.is_contact(),
            Self::Ranged => weapon.is_ranged(),
            Self::Light => weapon.is_light() && character.shield().is_none(),
            Self::Bows => matches!(weapon, Weapon::ArcCourt | Weapon::ArcLong),
            Self::Crossbows => weapon.is_crossbow(),
            Self::Powder => weapon.is_powder(),
            Self::AxesAndHammers => matches!(
                weapon,
                Weapon::Hache | Weapon::HacheADeuxMains | Weapon::Hachette | Weapon::Marteau
            ),
            Self::ThrownDaggers => {
                matches!(weapon, Weapon::Dague | Weapon::CouteauxDeLancer) && ranged_use
            }
            Self::Thrown => weapon.is_thrown() && ranged_use,
        }
    }
}

/// A capacité's effect on the values of some weapons.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WeaponEffect {
    weapons: Weapons,
    effect: Effect,
}

impl WeaponEffect {
    const fn new(weapons: Weapons, effect: Effect) -> Self {
        Self { weapons, effect }
    }
}

/// Adds every effect of `effects` on `weapon` (in `weapon_use`, when given)
/// that applies to the character.
pub(crate) fn apply_to_weapon(
    value: DerivedValue,
    character: &Character,
    values: &Characteristics,
    effects: &[WeaponEffect],
    weapon: Weapon,
    weapon_use: Option<WeaponUse>,
) -> DerivedValue {
    let effects: Vec<Effect> = effects
        .iter()
        .filter(|effect| effect.weapons.matches(character, weapon, weapon_use))
        .map(|effect| effect.effect)
        .collect();
    apply(value, character, values, &effects)
}

/// Effects on the critical range, as the lowest natural roll lowered
/// (COF.DER.CRIT-RANGE).
pub(crate) const CRITICAL_RANGE: &[WeaponEffect] = &[
    // Critique brutal.
    WeaponEffect::new(Weapons::Contact, Effect::flat(Voie::DemiOrc, 3, -1)),
    // Briseur d'os.
    WeaponEffect::new(Weapons::Contact, Effect::flat(Voie::Brute, 5, -1)),
    // Écuyer.
    WeaponEffect::new(Weapons::Contact, Effect::flat(Voie::Noblesse, 2, -1)),
    // Tir précis: 19-20, then 18-20 at rank 5.
    WeaponEffect::new(
        Weapons::Ranged,
        Effect::new(Voie::Precision, 3, Steps(&[(3, -1), (5, -2)])),
    ),
    // Frappe chirurgicale.
    WeaponEffect::new(Weapons::Light, Effect::flat(Voie::Spadassin, 3, -2)),
    // Archer émérite (elfe sylvain).
    WeaponEffect::new(Weapons::Bows, Effect::flat(Voie::ElfeSylvain, 3, -1)),
];

/// Effects on the DM added to a weapon's dice (COF.DER.DM).
pub(crate) const DAMAGE: &[WeaponEffect] = &[
    // Haches et marteaux.
    WeaponEffect::new(Weapons::AxesAndHammers, Effect::flat(Voie::Nain, 2, 1)),
    // Doigts agiles.
    WeaponEffect::new(
        Weapons::ThrownDaggers,
        Effect::new(Voie::Roublard, 1, ONE_TWO_THREE),
    ),
    // Poudre puissante: +1 per rank 5 in an arquebusier voie.
    WeaponEffect::new(Weapons::Powder, Effect::flat(Voie::Explosifs, 3, 1)),
    WeaponEffect::new(
        Weapons::Powder,
        Effect::new(Voie::Explosifs, 3, per_rank_five(Profil::Arquebusier)),
    ),
    // Archer émérite: PER to the bows, or FOR to the thrown weapons.
    WeaponEffect::new(
        Weapons::Bows,
        Effect::new(Voie::Archer, 1, Carac(Characteristic::Per))
            .when(When::NotChosen(AbilityOption::Thrown)),
    ),
    WeaponEffect::new(
        Weapons::Thrown,
        Effect::new(Voie::Archer, 1, Carac(Characteristic::For))
            .when(When::Chosen(AbilityOption::Thrown)),
    ),
    // Merveille technologique.
    WeaponEffect::new(
        Weapons::Crossbows,
        Effect::new(Voie::Gnome, 4, Carac(Characteristic::Agi)),
    ),
];

/// Effects on the attack with a weapon (COF.DER.ATTACK), besides the AGI
/// that Précision and Attaque en finesse put in place of FOR.
pub(crate) const WEAPON_ATTACK: &[WeaponEffect] = &[
    // Haches et marteaux.
    WeaponEffect::new(Weapons::AxesAndHammers, Effect::flat(Voie::Nain, 2, 1)),
];

/// The weapons a capacité makes the character master, whatever the profil.
pub(crate) const MASTERY: &[WeaponEffect] = &[
    // Haches et marteaux.
    WeaponEffect::new(Weapons::AxesAndHammers, Effect::flat(Voie::Nain, 2, 1)),
    // Archer émérite (elfe sylvain): the short bow.
    WeaponEffect::new(Weapons::Bows, Effect::flat(Voie::ElfeSylvain, 3, 1)),
    // Merveille technologique.
    WeaponEffect::new(Weapons::Crossbows, Effect::flat(Voie::Gnome, 4, 1)),
];

/// Whether a capacité makes the character master `weapon`.
pub(crate) fn masters(character: &Character, weapon: Weapon) -> bool {
    let values = character.characteristics();
    MASTERY.iter().any(|effect| {
        effect.weapons.matches(character, weapon, None)
            && effect.effect.amount(character, &values).is_some()
    })
}

/// The heaviest armour the capacités allow, if any raises the profil's
/// limit.
pub(crate) fn armour_limit(character: &Character) -> Option<Armour> {
    let values = character.characteristics();
    ARMOUR_LIMITS
        .iter()
        .filter(|(effect, _)| effect.amount(character, &values).is_some())
        .map(|(_, armour)| *armour)
        .max()
}

/// The armour limit a capacité raises the profil's to, when its effect
/// applies (COF.DER.ARMOUR-LIMIT).
pub(crate) const ARMOUR_LIMITS: &[(Effect, Armour)] = &[
    // Briseur d'os.
    (Effect::flat(Voie::Brute, 5, 1), Armour::CotteDeMailles),
    // Armure lourde, chosen instead of DEF.
    (
        Effect::flat(Voie::Resistance, 3, 1).when(When::Chosen(AbilityOption::Armour)),
        Armour::ArmureDePlaques,
    ),
    // Vêtements sacrés, for a warrior god.
    (
        Effect::flat(Voie::Spiritualite, 1, 1).when(When::Chosen(AbilityOption::Armour)),
        Armour::CotteDeMailles,
    ),
];

/// A characteristic a capacité raises by 1.
#[derive(Debug, Clone, Copy)]
enum Raise {
    /// Always this one.
    Fixed(Characteristic),
    /// The one chosen among the capacité's options.
    Chosen,
}

/// The capacités raising characteristics (COF.CNT.EFFECT-SHAPES).
const RAISES: &[(Voie, u8, Raise)] = {
    use Characteristic::{Agi, Cha, Con, For, Int, Per, Vol};
    use Raise::{Chosen, Fixed};
    &[
        // Force héroïque.
        (Voie::Brute, 4, Fixed(For)),
        (Voie::Guerre, 4, Fixed(For)),
        (Voie::Soldat, 4, Fixed(For)),
        // Constitution héroïque.
        (Voie::Mercenaire, 4, Fixed(Con)),
        (Voie::Survie, 4, Fixed(Con)),
        (Voie::Pagne, 4, Fixed(Con)),
        (Voie::Resistance, 4, Fixed(Con)),
        (Voie::Nature, 4, Fixed(Con)),
        // Agilité héroïque.
        (Voie::Pistolero, 4, Fixed(Agi)),
        (Voie::Deplacement, 4, Fixed(Agi)),
        (Voie::Vent, 4, Fixed(Agi)),
        // Charisme héroïque.
        (Voie::Seduction, 4, Fixed(Cha)),
        (Voie::Aventurier, 4, Fixed(Cha)),
        (Voie::Preux, 4, Fixed(Cha)),
        (Voie::Spiritualite, 4, Fixed(Cha)),
        // Perception héroïque.
        (Voie::Traqueur, 4, Fixed(Per)),
        (Voie::Divination, 4, Fixed(Per)),
        (Voie::Protecteur, 4, Fixed(Per)),
        // Volonté héroïque.
        (Voie::Meditation, 4, Fixed(Vol)),
        (Voie::Priere, 4, Fixed(Vol)),
        // Combattant héroïque.
        (Voie::CombatADeuxArmes, 4, Chosen),
        // Endurer, Pacte ténébreux, Ascétisme.
        (Voie::Metal, 5, Fixed(Con)),
        (Voie::SombreMagie, 5, Fixed(Con)),
        (Voie::EnergieVitale, 5, Fixed(Con)),
        // Moment de perfection, Projection mentale, Passe-muraille.
        (Voie::Maitrise, 5, Chosen),
        (Voie::Meditation, 5, Chosen),
        (Voie::Vent, 5, Chosen),
        // Colosse.
        (Voie::DemiOrc, 5, Fixed(For)),
        (Voie::DemiOrc, 5, Fixed(Con)),
        // Supériorité elfique.
        (Voie::ElfeHaut, 5, Fixed(Vol)),
        (Voie::ElfeHaut, 5, Chosen),
        (Voie::ElfeSylvain, 5, Fixed(Agi)),
        (Voie::ElfeSylvain, 5, Fixed(Per)),
        // Bonne nature.
        (Voie::Gnome, 5, Fixed(Con)),
        (Voie::Gnome, 5, Fixed(Cha)),
        // Vif et bien nourri.
        (Voie::Halfelin, 5, Fixed(Agi)),
        (Voie::Halfelin, 5, Fixed(Con)),
        // Polyvalence.
        (Voie::Humain, 5, Chosen),
        (Voie::Humain, 5, Fixed(Vol)),
        // Ténacité.
        (Voie::Nain, 5, Fixed(Con)),
        (Voie::Nain, 5, Fixed(Vol)),
        // Esprit supérieur.
        (Voie::Mage, 4, Fixed(Int)),
        (Voie::Mage, 4, Fixed(Vol)),
    ]
};

/// Adds to `value` the raises of `c` by the capacités the character holds.
pub(crate) fn raise(value: DerivedValue, character: &Character, c: Characteristic) -> DerivedValue {
    RAISES
        .iter()
        .filter(|(voie, rank, _)| holds(character, *voie, Rank::new(*rank)))
        .filter(|(voie, rank, raise)| match raise {
            Raise::Fixed(raised) => *raised == c,
            Raise::Chosen => chose(character, *voie, *rank, AbilityOption::Characteristic(c)),
        })
        .fold(value, |value, (voie, rank, _)| {
            value.with(Source::Ability(voie.ability(Rank::new(*rank))), 1)
        })
}

/// COF.CNT.VOIE: a capacité held whose option is required has one; an
/// option chosen is one the capacité offers.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        for rank in ranks {
            if option_required(voie, rank) && character.ability_option(voie, rank).is_none() {
                violations.push(Violation::incomplete(
                    CONTENT_VOIE,
                    ABILITY_MECHANIC,
                    Subject::Ability(voie.ability(rank)),
                ));
            }
        }
    }
    for (voie, rank, option) in character.ability_options() {
        if !options(character, voie, rank).contains(&option) {
            violations.push(Violation::deviation(
                CONTENT_VOIE,
                ABILITY_MECHANIC,
                Subject::Ability(voie.ability(rank)),
                Severity::Standard,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;
    use crate::{AttackMode, LevelUp, Peuple, Series, Shield, Value};

    fn total(character: &Character, value: Value) -> Option<i32> {
        character.value(value).map(|v| v.total())
    }

    /// A character whose seven values are all 0, so every effect shows alone.
    fn zero() -> Character {
        Characteristic::ALL.into_iter().fold(
            Character::new().with_series(Series::Polyvalent),
            |c, characteristic| c.with_series_value(characteristic, 0),
        )
    }

    /// `voie` acquired from rank 1 to `rank`.
    fn ranks(character: Character, voie: Voie, rank: u8) -> Character {
        (1..=rank).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    fn option_violations(character: &Character) -> Vec<(Subject, Category)> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
            .into_iter()
            .map(|v| (v.subject().clone(), v.category()))
            .collect()
    }

    fn ability(voie: Voie, rank: u8) -> Subject {
        Subject::Ability(voie.ability(Rank::new(rank)))
    }

    #[test]
    fn steps_follow_the_rank_reached() {
        // Agile: +1 DEF and Init, +2 at rank 3, +3 at rank 5.
        let defence = |rank| total(&ranks(zero(), Voie::Deplacement, rank), Value::Defence);
        assert_eq!(defence(1), Some(11));
        assert_eq!(defence(2), Some(11));
        assert_eq!(defence(3), Some(12));
        // Rank 4, Agilité héroïque, raises AGI too.
        assert_eq!(defence(5), Some(10 + 3 + 1));
        let initiative = total(&ranks(zero(), Voie::Deplacement, 3), Value::Initiative);
        assert_eq!(initiative, Some(12));
    }

    #[test]
    fn vigour_reads_ranks_and_characteristics() {
        let guerrier = zero()
            .with_profil(Profil::Guerrier)
            .with_series_value(Characteristic::For, 2);
        // 10 (combattants), Robustesse rank 2 + 2.
        let robust = ranks(guerrier.clone(), Voie::Resistance, 2);
        assert_eq!(total(&robust, Value::VigourPoints), Some(14));
        // Vigueur: +1 per rank; Argument de taille: + FOR.
        let barbare = ranks(guerrier.with_rank(Voie::Brute, 1), Voie::Pagne, 3);
        assert_eq!(total(&barbare, Value::VigourPoints), Some(10 + 3 + 2));
    }

    #[test]
    fn equipment_turns_effects_on_and_off() {
        // Armure de vent: +2 without armour, +1 in an armour.
        let primitif = ranks(zero(), Voie::Primitif, 2);
        assert_eq!(total(&primitif, Value::Defence), Some(12));
        let armoured = primitif.with_armour(Armour::CuirSimple);
        assert_eq!(total(&armoured, Value::Defence), Some(10 + 2 + 1));
        // Défense au bouclier: with a shield only.
        let bouclier = ranks(zero(), Voie::Bouclier, 3);
        assert_eq!(total(&bouclier, Value::Defence), Some(10));
        let shield = bouclier.with_shield(Shield::PetitBouclier);
        assert_eq!(total(&shield, Value::Defence), Some(10 + 1 + 1));
        // In play, the equipment worn decides.
        let in_play = shield.in_play(crate::Equipment::default());
        assert_eq!(in_play.value(Value::Defence).map(|v| v.total()), Some(10));
    }

    #[test]
    fn dentelles_et_rapiere_adds_cha_up_to_the_rank() {
        let seducteur = ranks(
            zero().with_series_value(Characteristic::Cha, 4),
            Voie::Seduction,
            3,
        );
        assert_eq!(total(&seducteur, Value::Defence), Some(13));
        let armoured = seducteur.with_armour(Armour::CuirSimple);
        assert_eq!(total(&armoured, Value::Defence), Some(12));
    }

    #[test]
    fn rank_five_of_a_profil_adds_up() {
        // Divination: +1 DEF, +1 at rank 3, +1 per rank 5 in an ensorceleur voie.
        let devin = ranks(ranks(zero(), Voie::Divination, 5), Voie::Air, 5);
        // Air: +1; Divination: 2 + 2.
        assert_eq!(total(&devin, Value::Defence), Some(10 + 1 + 2 + 2));
    }

    #[test]
    fn damage_reduction_needs_a_capacite() {
        assert_eq!(total(&zero(), Value::DamageReduction), None);
        let nain = zero().with_peuple(Peuple::Nain);
        let fils_du_roc = ranks(nain, Voie::Nain, 4);
        assert_eq!(total(&fils_du_roc, Value::DamageReduction), Some(2));
        let tenth = (0..9).fold(fils_du_roc, |c, _| c.with_level_up(LevelUp::new()));
        assert_eq!(total(&tenth, Value::DamageReduction), Some(3));
    }

    #[test]
    fn characteristics_are_raised_beyond_the_starting_values() {
        let character = ranks(zero(), Voie::Brute, 4);
        assert_eq!(character.characteristics()[Characteristic::For].get(), 1);
        assert_eq!(
            character.starting_characteristics()[Characteristic::For].get(),
            0
        );
        // A raise is no creation choice: a starting +4 raised to +5 is in bounds.
        let strong = character.with_series_value(Characteristic::For, 4);
        assert!(
            strong
                .violations()
                .iter()
                .all(|v| v.rule().as_str() != "COF.CRE.CARAC-BOUNDS")
        );
        // The raise feeds the derived values: contact attack 1 + FOR 5.
        assert_eq!(total(&strong, Value::Attack(AttackMode::Contact)), Some(6));
    }

    #[test]
    fn chosen_raises_need_their_option() {
        let elfe = ranks(zero().with_peuple(Peuple::ElfeHaut), Voie::ElfeHaut, 5);
        assert_eq!(
            option_violations(&elfe),
            [(ability(Voie::ElfeHaut, 5), Category::Incomplete)]
        );
        assert_eq!(elfe.characteristics()[Characteristic::Vol].get(), 1);
        let int = AbilityOption::Characteristic(Characteristic::Int);
        let chosen = elfe.with_ability_option(Voie::ElfeHaut, 5, int);
        assert_eq!(option_violations(&chosen), []);
        assert_eq!(chosen.characteristics()[Characteristic::Int].get(), 1);
        // An option not offered deviates; the Game Master may waive it.
        let agi = AbilityOption::Characteristic(Characteristic::Agi);
        let other = chosen.with_ability_option(Voie::ElfeHaut, 5, agi);
        assert_eq!(
            option_violations(&other),
            [(
                ability(Voie::ElfeHaut, 5),
                Category::Deviation(Severity::Standard)
            )]
        );
        assert_eq!(other.characteristics()[Characteristic::Agi].get(), 1);
    }

    #[test]
    fn lowest_characteristic_options_include_ties() {
        let character = zero()
            .with_series_value(Characteristic::Cha, -1)
            .with_series_value(Characteristic::Int, -1);
        let offered = character.ability_options_offered(Voie::Humain, 5);
        assert_eq!(
            offered,
            [
                AbilityOption::Characteristic(Characteristic::Cha),
                AbilityOption::Characteristic(Characteristic::Int),
            ]
        );
        assert_eq!(character.ability_options_offered(Voie::Rage, 1), []);
    }

    #[test]
    fn optional_options_replace_a_default() {
        // Peau de pierre: +1 DEF, or CON in place of AGI.
        let barbare = ranks(
            zero()
                .with_series_value(Characteristic::Con, 3)
                .with_series_value(Characteristic::Agi, 1),
            Voie::Pagne,
            2,
        );
        assert_eq!(total(&barbare, Value::Defence), Some(10 + 1 + 1));
        let con = AbilityOption::Characteristic(Characteristic::Con);
        let stone = barbare.with_ability_option(Voie::Pagne, 2, con);
        assert_eq!(total(&stone, Value::Defence), Some(10 + 3));
        assert_eq!(option_violations(&stone), []);
        // Éclaireur: the family's +1 PC for +1 DR.
        let rodeur = zero()
            .with_profil(Profil::Rodeur)
            .with_rank(Voie::Traqueur, 1);
        assert_eq!(total(&rodeur, Value::LuckPoints), Some(3));
        let swapped = rodeur.with_ability_option(Voie::Traqueur, 1, AbilityOption::RecoveryDice);
        assert_eq!(total(&swapped, Value::LuckPoints), Some(2));
        assert_eq!(total(&swapped, Value::RecoveryDice), Some(3));
    }

    #[test]
    fn mana_points_count_every_spell() {
        let mage = ranks(
            zero().with_series_value(Characteristic::Vol, 2),
            Voie::MagieDesArcanes,
            3,
        )
        .with_rank(Voie::Mage, 1)
        .with_rank(Voie::Mage, 2)
        .with_rank(Voie::Mage, 3);
        // VOL 2, 3 + 2 spells, Tour de magie +2.
        assert_eq!(total(&mage, Value::ManaPoints), Some(2 + 5 + 2));
        // Charisme héroïque may put CHA in place of VOL.
        let barde = ranks(
            zero().with_series_value(Characteristic::Cha, 3),
            Voie::Seduction,
            5,
        );
        assert_eq!(total(&barde, Value::ManaPoints), Some(1));
        let cha = AbilityOption::Characteristic(Characteristic::Cha);
        let charming = barde.with_ability_option(Voie::Seduction, 4, cha);
        // CHA 3 + 1 (Charisme héroïque) + 1 spell.
        assert_eq!(total(&charming, Value::ManaPoints), Some(5));
    }

    #[test]
    fn weapon_effects_follow_the_weapon() {
        let nain = zero().with_peuple(Peuple::Nain).with_rank(Voie::Nain, 1);
        let haches = nain.clone().with_rank(Voie::Nain, 2);
        let dm = |c: &Character, weapon| total(c, Value::Damage(weapon, WeaponUse::OneHanded));
        assert_eq!(dm(&haches, Weapon::Hache), Some(1));
        assert_eq!(dm(&haches, Weapon::EpeeLongue), Some(0));
        assert!(haches.masters(Weapon::Hache));
        assert!(!nain.masters(Weapon::Hache));
        // Doigts agiles: thrown daggers only.
        let roublard = zero().with_rank(Voie::Roublard, 1);
        assert_eq!(
            total(&roublard, Value::Damage(Weapon::Dague, WeaponUse::Ranged)),
            Some(1)
        );
        assert_eq!(dm(&roublard, Weapon::Dague), Some(0));
    }

    #[test]
    fn critical_ranges_widen_down_to_16() {
        let range = |c: &Character, weapon| total(c, Value::CriticalRange(weapon));
        // Frappe chirurgicale: light weapons, without a shield.
        let spadassin = ranks(zero(), Voie::Spadassin, 3);
        assert_eq!(range(&spadassin, Weapon::Rapiere), Some(17));
        assert_eq!(range(&spadassin, Weapon::EpeeLongue), Some(20));
        let shielded = spadassin.clone().with_shield(Shield::PetitBouclier);
        assert_eq!(range(&shielded, Weapon::Rapiere), Some(19));
        // Critique brutal, Briseur d'os, Écuyer on top: never below 16.
        let brute =
            ranks(ranks(spadassin, Voie::Brute, 5), Voie::Noblesse, 2).with_peuple(Peuple::DemiOrc);
        let brute = ranks(brute, Voie::DemiOrc, 3);
        assert_eq!(range(&brute, Weapon::Rapiere), Some(16));
    }

    #[test]
    fn capacites_raise_the_armour_limit() {
        let barbare = zero()
            .with_profil(Profil::Barbare)
            .with_armour(Armour::CotteDeMailles);
        let limit = |c: &Character| {
            c.violations()
                .iter()
                .any(|v| v.rule().as_str() == "COF.DER.ARMOUR-LIMIT")
        };
        assert!(limit(&barbare));
        assert!(!limit(&ranks(barbare, Voie::Brute, 5)));
        // Armure lourde: the armure de plaques instead of +1 DEF.
        let guerrier = ranks(
            zero()
                .with_profil(Profil::Guerrier)
                .with_armour(Armour::ArmureDePlaques),
            Voie::Resistance,
            3,
        );
        assert!(limit(&guerrier));
        let plate = guerrier.with_ability_option(Voie::Resistance, 3, AbilityOption::Armour);
        assert!(!limit(&plate));
    }

    #[test]
    fn capacites_offer_their_options() {
        use Characteristic::{Agi, Cha, Con, For, Int};
        let c = AbilityOption::Characteristic;
        let character = zero();
        let offered = |voie, rank: u8| character.ability_options_offered(voie, rank);
        assert_eq!(offered(Voie::ElfeHaut, 5), [c(Int), c(Cha)]);
        assert_eq!(offered(Voie::CombatADeuxArmes, 4), [c(Agi), c(For)]);
        assert_eq!(offered(Voie::Pagne, 2), [c(Con)]);
        assert_eq!(offered(Voie::Metal, 5), [c(Con)]);
        assert_eq!(offered(Voie::Seduction, 4), [c(Cha)]);
        assert_eq!(offered(Voie::Golem, 1), [c(Int)]);
        assert_eq!(offered(Voie::Resistance, 3), [AbilityOption::Armour]);
        assert_eq!(offered(Voie::Spiritualite, 1), [AbilityOption::Armour]);
        assert_eq!(offered(Voie::Traqueur, 1), [AbilityOption::RecoveryDice]);
        assert_eq!(offered(Voie::Archer, 1), [AbilityOption::Thrown]);
        // All seven values at 0: every one is among the lowest.
        assert_eq!(offered(Voie::Vent, 5).len(), 7);
    }

    #[test]
    fn combattant_heroique_raises_the_characteristic_chosen() {
        let duellist = ranks(zero(), Voie::CombatADeuxArmes, 4);
        assert_eq!(
            option_violations(&duellist),
            [(ability(Voie::CombatADeuxArmes, 4), Category::Incomplete)]
        );
        let agi = AbilityOption::Characteristic(Characteristic::Agi);
        let agile = duellist.with_ability_option(Voie::CombatADeuxArmes, 4, agi);
        assert_eq!(option_violations(&agile), []);
        assert_eq!(agile.characteristics()[Characteristic::Agi].get(), 1);
        assert_eq!(agile.characteristics()[Characteristic::For].get(), 0);
    }

    #[test]
    fn armure_sur_mesure_counts_in_plate() {
        // Rank 5 of the voie de la guerre, a chevalier voie.
        let chevalier = ranks(zero().with_profil(Profil::Chevalier), Voie::Guerre, 5);
        let source = Source::Ability(Voie::Guerre.ability(Rank::new(1)));
        let fitted = |c: &Character| {
            c.value(Value::Defence).is_some_and(|value| {
                value
                    .contributions()
                    .iter()
                    .any(|contribution| contribution.source() == &source)
            })
        };
        assert!(!fitted(
            &chevalier.clone().with_armour(Armour::CotteDeMailles)
        ));
        assert!(fitted(&chevalier.with_armour(Armour::ArmureDePlaques)));
    }

    #[test]
    fn effects_apply_under_their_condition_and_option() {
        let effect = Effect::new(Voie::Archer, 1, Amount::Flat(2))
            .only(Condition::Shield)
            .when(When::Chosen(AbilityOption::Thrown));
        let archer = zero().with_rank(Voie::Archer, 1);
        let values = archer.characteristics();
        assert_eq!(effect.amount(&archer, &values), None);
        let shielded = archer.with_shield(Shield::PetitBouclier);
        assert_eq!(effect.amount(&shielded, &values), None);
        let thrower = shielded.with_ability_option(Voie::Archer, 1, AbilityOption::Thrown);
        assert_eq!(effect.amount(&thrower, &values), Some(2));
        assert_eq!(
            Effect::flat(Voie::Rage, 1, 1).amount(&thrower, &values),
            None
        );
        let bows = [WeaponEffect::new(Weapons::Bows, effect)];
        let on = |weapon| {
            apply_to_weapon(DerivedValue::new(), &thrower, &values, &bows, weapon, None).total()
        };
        assert_eq!(on(Weapon::ArcCourt), 2);
        assert_eq!(on(Weapon::Dague), 0);
    }

    #[test]
    fn options_are_listed_and_named() {
        for option in AbilityOption::ALL {
            assert_eq!(AbilityOption::from_id(option.id()), Some(option));
            assert_ne!(option.name(), "");
        }
        assert_eq!(AbilityOption::from_id("cof.option.unknown"), None);
        assert_eq!(
            AbilityOption::Characteristic(Characteristic::Int).name(),
            "INT"
        );
        let character = zero();
        assert!(character.ability_option_required(Voie::ElfeHaut, 5));
        assert!(!character.ability_option_required(Voie::Pagne, 2));
        assert!(!character.ability_option_required(Voie::Rage, 1));
    }
}
