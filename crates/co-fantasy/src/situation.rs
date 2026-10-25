//! The capacités' modifiers that hold only in a situation (ADR
//! ENG-SITUATIONAL): listed beside the values they modify, never added to
//! them.

use co_kernel::{AbilityId, Dice, Rank};

use crate::{
    Armour, AttackMode, Character, Target, Value, Voie,
    ability::{Amount, Effect},
};

/// A situation a capacité's modifier needs, named in our own words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Situation {
    /// Against a creature larger than the character.
    AgainstLarger,
    /// Outdoors.
    Outdoors,
    /// With a weapon in each hand.
    TwoWeapons,
    /// With a weapon in each hand, giving up the off-hand attacks.
    TwoWeaponsNoOffHand,
    /// While the familiar is in sight.
    FamiliarInSight,
    /// In difficult terrain.
    DifficultTerrain,
    /// With a loaded powder weapon in hand.
    LoadedPowderWeapon,
    /// Mounted.
    Mounted,
    /// Against ranged attacks.
    AgainstRanged,
    /// Against a medium humanoid.
    AgainstMediumHumanoid,
    /// Against an animal.
    AgainstAnimal,
    /// Against the enemy leader.
    AgainstLeader,
    /// Against piétaille.
    AgainstPietaille,
}

impl Situation {
    /// Every situation, in declaration order.
    pub const ALL: [Self; 13] = [
        Self::AgainstLarger,
        Self::Outdoors,
        Self::TwoWeapons,
        Self::TwoWeaponsNoOffHand,
        Self::FamiliarInSight,
        Self::DifficultTerrain,
        Self::LoadedPowderWeapon,
        Self::Mounted,
        Self::AgainstRanged,
        Self::AgainstMediumHumanoid,
        Self::AgainstAnimal,
        Self::AgainstLeader,
        Self::AgainstPietaille,
    ];

    /// The French name, such as `contre plus grand que lui`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AgainstLarger => "contre plus grand que lui",
            Self::Outdoors => "en extérieur",
            Self::TwoWeapons => "à deux armes",
            Self::TwoWeaponsNoOffHand => "à deux armes, sans attaque de la main gauche",
            Self::FamiliarInSight => "familier en vue",
            Self::DifficultTerrain => "en terrain difficile",
            Self::LoadedPowderWeapon => "arme à poudre chargée en main",
            Self::Mounted => "monté",
            Self::AgainstRanged => "contre les attaques à distance",
            Self::AgainstMediumHumanoid => "contre un humanoïde de taille moyenne",
            Self::AgainstAnimal => "contre un animal",
            Self::AgainstLeader => "contre le chef ennemi",
            Self::AgainstPietaille => "contre la piétaille",
        }
    }
}

/// A capacité's modifier to a value, holding only in its situation: an
/// amount, or dice added to a weapon's DM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Situational {
    situation: Situation,
    ability: AbilityId,
    amount: Option<i32>,
    dice: Option<Dice>,
}

impl Situational {
    /// The situation it needs.
    #[must_use]
    pub const fn situation(&self) -> Situation {
        self.situation
    }

    /// The capacité giving it.
    #[must_use]
    pub const fn ability(&self) -> &AbilityId {
        &self.ability
    }

    /// The amount added in the situation, `None` for dice.
    #[must_use]
    pub const fn amount(&self) -> Option<i32> {
        self.amount
    }

    /// The dice added to the DM in the situation, `None` for an amount.
    #[must_use]
    pub const fn dice(&self) -> Option<&Dice> {
        self.dice.as_ref()
    }
}

/// The values a row modifies.
#[derive(Debug, Clone, Copy)]
enum Applies {
    /// One value.
    Value(Value),
    /// Every attack value and the attack with every weapon.
    EveryAttack,
    /// The DM of every weapon used at contact.
    ContactDamage,
    /// The DM of every weapon.
    EveryDamage,
}

impl Applies {
    fn matches(self, target: Target) -> bool {
        let Target::Value(value) = target else {
            return false;
        };
        match self {
            Self::Value(other) => other == value,
            Self::EveryAttack => matches!(value, Value::Attack(_) | Value::WeaponAttack(..)),
            Self::ContactDamage => matches!(
                value,
                Value::Damage(_, weapon_use) if weapon_use.attack_mode() == AttackMode::Contact
            ),
            Self::EveryDamage => matches!(value, Value::Damage(..)),
        }
    }
}

/// What a row gives once its capacité is held.
#[derive(Debug, Clone, Copy)]
enum Gain {
    /// The effect's amount.
    Amount,
    /// Piqûres d'insectes' RD by the armour worn (D-52).
    ArmourReduction,
    /// This many dés évolutifs.
    EvolutiveDice(u8),
}

/// A row of the table: what it modifies, in which situation, the capacité
/// with its amount, and what it gives.
type Row = (Applies, Situation, Effect, Gain);

use Applies::{ContactDamage, EveryAttack, EveryDamage};
use Gain::{ArmourReduction, EvolutiveDice};
use Situation::{
    AgainstAnimal, AgainstLarger, AgainstLeader, AgainstMediumHumanoid, AgainstPietaille,
    AgainstRanged, DifficultTerrain, FamiliarInSight, LoadedPowderWeapon, Mounted, Outdoors,
    TwoWeapons, TwoWeaponsNoOffHand,
};

const DEFENCE: Applies = Applies::Value(Value::Defence);
const INITIATIVE: Applies = Applies::Value(Value::Initiative);
const REDUCTION: Applies = Applies::Value(Value::DamageReduction);

/// The situational modifiers of the capacités, in voie order; a "+a (+b at
/// rank 5)" is the total reached (D-33).
const ROWS: &[Row] = &[
    // Insignifiant.
    (
        DEFENCE,
        AgainstLarger,
        Effect::new(Voie::Gnome, 3, Amount::Steps(&[(3, 2), (5, 3)])),
        Gain::Amount,
    ),
    // Loup parmi les loups.
    (
        EveryDamage,
        AgainstMediumHumanoid,
        Effect::flat(Voie::Humain, 4, 0),
        EvolutiveDice(1),
    ),
    // Plus vite que son ombre.
    (
        INITIATIVE,
        LoadedPowderWeapon,
        Effect::flat(Voie::Pistolero, 1, 5),
        Gain::Amount,
    ),
    // Grand pas.
    (
        DEFENCE,
        Outdoors,
        Effect::new(Voie::Survie, 3, Amount::Steps(&[(3, 1), (5, 2)])),
        Gain::Amount,
    ),
    // Chasseur émérite.
    (
        EveryDamage,
        AgainstAnimal,
        Effect::flat(Voie::Traqueur, 3, 0),
        EvolutiveDice(1),
    ),
    // Parade croisée, doubled without off-hand attacks (D-53).
    (
        DEFENCE,
        TwoWeapons,
        Effect::new(Voie::CombatADeuxArmes, 2, Amount::Steps(&[(2, 1), (5, 2)])),
        Gain::Amount,
    ),
    (
        DEFENCE,
        TwoWeaponsNoOffHand,
        Effect::new(Voie::CombatADeuxArmes, 2, Amount::Steps(&[(2, 2), (5, 4)])),
        Gain::Amount,
    ),
    // Cavalier émérite.
    (
        ContactDamage,
        Mounted,
        Effect::new(Voie::Cavalier, 2, Amount::Steps(&[(2, 1), (5, 2)])),
        Gain::Amount,
    ),
    // Piqûres d'insectes.
    (
        REDUCTION,
        AgainstRanged,
        Effect::flat(Voie::Preux, 2, 0),
        ArmourReduction,
    ),
    // Laissez-le-moi.
    (
        ContactDamage,
        AgainstLeader,
        Effect::flat(Voie::Preux, 3, 0),
        EvolutiveDice(1),
    ),
    // Massacrer la piétaille.
    (
        EveryDamage,
        AgainstPietaille,
        Effect::flat(Voie::Noblesse, 4, 0),
        EvolutiveDice(1),
    ),
    // Petit compagnon.
    (
        DEFENCE,
        FamiliarInSight,
        Effect::flat(Voie::Animaux, 2, 2),
        Gain::Amount,
    ),
    // Terrains difficiles: DEF, Init and every attack (D-54).
    (
        DEFENCE,
        DifficultTerrain,
        Effect::flat(Voie::Nature, 2, 1),
        Gain::Amount,
    ),
    (
        INITIATIVE,
        DifficultTerrain,
        Effect::flat(Voie::Nature, 2, 3),
        Gain::Amount,
    ),
    (
        EveryAttack,
        DifficultTerrain,
        Effect::flat(Voie::Nature, 2, 1),
        Gain::Amount,
    ),
];

/// Piqûres d'insectes' RD with `armour` (D-52): leather 1, mail 2, plate 3.
const fn armour_reduction(armour: Armour) -> Option<i32> {
    match armour {
        Armour::TissusMatelasses => None,
        Armour::CuirSimple | Armour::CuirRenforce => Some(1),
        Armour::ChemiseDeMailles | Armour::CotteDeMailles => Some(2),
        Armour::ArmureDePlaques | Armour::PlaqueComplete => Some(3),
    }
}

/// The situational modifiers of the capacités the character holds to
/// `target`, in voie order.
pub(crate) fn of(character: &Character, target: Target) -> Vec<Situational> {
    let values = character.characteristics();
    ROWS.iter()
        .filter(|(applies, ..)| applies.matches(target))
        .filter_map(|(_, situation, effect, gain)| {
            let amount = effect.amount(character, &values)?;
            let (amount, dice) = match gain {
                Gain::Amount => (Some(amount), None),
                ArmourReduction => (Some(armour_reduction(character.armour()?)?), None),
                EvolutiveDice(count) => {
                    let die = character.level().evolutive_die();
                    (None, Some(Dice::new((0..*count).map(|_| die))))
                }
            };
            Some(Situational {
                situation: *situation,
                ability: effect.voie().ability(Rank::new(effect.rank())),
                amount,
                dice,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LevelUp, Weapon, WeaponUse};

    fn ranks(character: Character, voie: Voie, highest: u8) -> Character {
        (1..=highest).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    fn listed(character: &Character, value: Value) -> Vec<(Situation, Option<i32>)> {
        of(character, Target::Value(value))
            .iter()
            .map(|s| (s.situation(), s.amount()))
            .collect()
    }

    #[test]
    fn situations_are_named() {
        assert!(Situation::ALL.iter().all(|s| !s.name().is_empty()));
        assert_eq!(Situation::AgainstLarger.name(), "contre plus grand que lui");
    }

    #[test]
    fn modifiers_follow_the_rank_reached_and_stay_out_of_the_totals() {
        let gnome = ranks(Character::new(), Voie::Gnome, 3);
        assert_eq!(
            listed(&gnome, Value::Defence),
            [(Situation::AgainstLarger, Some(2))]
        );
        let gnome = ranks(Character::new(), Voie::Gnome, 5);
        assert_eq!(
            listed(&gnome, Value::Defence),
            [(Situation::AgainstLarger, Some(3))]
        );
        // Insignifiant adds nothing to DEF itself.
        let total = |c: &Character| c.value(Value::Defence).map(|v| v.total());
        assert_eq!(
            total(&ranks(Character::new(), Voie::Gnome, 3)),
            total(&ranks(Character::new(), Voie::Gnome, 2))
        );
        assert_eq!(listed(&Character::new(), Value::Defence), []);
    }

    #[test]
    fn one_capacite_may_modify_several_values() {
        let druid = ranks(Character::new(), Voie::Nature, 2);
        assert_eq!(
            listed(&druid, Value::Initiative),
            [(Situation::DifficultTerrain, Some(3))]
        );
        assert_eq!(
            listed(&druid, Value::Attack(AttackMode::Magic)),
            [(Situation::DifficultTerrain, Some(1))]
        );
        assert_eq!(
            listed(
                &druid,
                Value::WeaponAttack(Weapon::Baton, WeaponUse::TwoHanded)
            ),
            [(Situation::DifficultTerrain, Some(1))]
        );
        assert_eq!(
            of(
                &druid,
                Target::Characteristic(co_kernel::Characteristic::Agi)
            ),
            []
        );
    }

    #[test]
    fn evolutive_dice_follow_the_level() {
        let damage = Value::Damage(Weapon::EpeeLongue, WeaponUse::OneHanded);
        let hunter = ranks(Character::new(), Voie::Traqueur, 3);
        let [bonus] = of(&hunter, Target::Value(damage)).try_into().unwrap();
        assert_eq!(bonus.situation(), Situation::AgainstAnimal);
        assert_eq!(
            (bonus.amount(), bonus.dice()),
            (None, Some(&Dice::new([4])))
        );
        assert_eq!(bonus.ability(), &Voie::Traqueur.ability(Rank::new(3)));
        let levelled = (0..5).fold(hunter, |c, _| c.with_level_up(LevelUp::new()));
        let [bonus] = of(&levelled, Target::Value(damage)).try_into().unwrap();
        assert_eq!(bonus.dice(), Some(&Dice::new([6])));
    }

    #[test]
    fn contact_damage_leaves_ranged_weapons_alone() {
        let rider = ranks(Character::new(), Voie::Cavalier, 2);
        let contact = Value::Damage(Weapon::EpeeLongue, WeaponUse::OneHanded);
        let ranged = Value::Damage(Weapon::ArcCourt, WeaponUse::Ranged);
        assert_eq!(listed(&rider, contact), [(Situation::Mounted, Some(1))]);
        assert_eq!(listed(&rider, ranged), []);
    }

    #[test]
    fn piqures_d_insectes_read_the_armour_worn() {
        let knight = ranks(Character::new(), Voie::Preux, 2);
        assert_eq!(listed(&knight, Value::DamageReduction), []);
        let mail = knight.clone().with_armour(Armour::CotteDeMailles);
        assert_eq!(
            listed(&mail, Value::DamageReduction),
            [(Situation::AgainstRanged, Some(2))]
        );
        let padded = knight.clone().with_armour(Armour::TissusMatelasses);
        assert_eq!(listed(&padded, Value::DamageReduction), []);
        // In play, the armour worn then.
        let played = padded.in_play(crate::Equipment::new().with_armour(Armour::PlaqueComplete));
        let reductions: Vec<_> = played
            .situational(Target::Value(Value::DamageReduction))
            .iter()
            .map(Situational::amount)
            .collect();
        assert_eq!(reductions, [Some(3)]);
    }

    #[test]
    fn parade_croisee_lists_both_situations() {
        let fighter = ranks(Character::new(), Voie::CombatADeuxArmes, 5);
        assert_eq!(
            listed(&fighter, Value::Defence),
            [
                (Situation::TwoWeapons, Some(2)),
                (Situation::TwoWeaponsNoOffHand, Some(4))
            ]
        );
    }
}
