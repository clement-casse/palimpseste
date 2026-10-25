//! The capacités' modifiers that hold only in a situation (ADR
//! ENG-SITUATIONAL): listed beside the values they modify, never added to
//! them.

use co_kernel::{AbilityId, Characteristic, Rank};

use crate::{AttackMode, Character, Target, Value, Voie};

/// A situation a capacité's modifier needs, named in our own words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Situation {
    /// Against contact weapons.
    AgainstContactWeapons,
    /// Against ranged attacks.
    AgainstRanged,
    /// Against explosions.
    AgainstExplosives,
    /// Against unarmed attacks.
    AgainstUnarmed,
    /// Parrying unarmed.
    ParryingUnarmed,
    /// In a moving vehicle.
    InMovingVehicle,
    /// A combat manoeuvre, at -2 instead of -5.
    CombatManoeuvre,
    /// Automatic fire, at -2 instead of -5, using half a magazine.
    AutomaticFire,
    /// FOR tested in place of CHA to coerce.
    Coercion,
    /// INT tested in place of a physical test of the same difficulty.
    PhysicalTest,
}

impl Situation {
    /// Every situation, in declaration order.
    pub const ALL: [Self; 10] = [
        Self::AgainstContactWeapons,
        Self::AgainstRanged,
        Self::AgainstExplosives,
        Self::AgainstUnarmed,
        Self::ParryingUnarmed,
        Self::InMovingVehicle,
        Self::CombatManoeuvre,
        Self::AutomaticFire,
        Self::Coercion,
        Self::PhysicalTest,
    ];

    /// The French name, such as `contre les armes de contact`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AgainstContactWeapons => "contre les armes de contact",
            Self::AgainstRanged => "contre les attaques à distance",
            Self::AgainstExplosives => "contre les explosions",
            Self::AgainstUnarmed => "contre les attaques à mains nues",
            Self::ParryingUnarmed => "en parant à mains nues",
            Self::InMovingVehicle => "dans un véhicule en mouvement",
            Self::CombatManoeuvre => "manœuvre à −2 au lieu de −5",
            Self::AutomaticFire => "tir automatique à −2 au lieu de −5, demi-chargeur",
            Self::Coercion => "à la place du CHA pour contraindre",
            Self::PhysicalTest => "à la place d’un test physique de même difficulté",
        }
    }
}

/// A capacité's modifier to a value or a characteristic, holding only in
/// its situation: an amount, or none when the situation says it all (a
/// manoeuvre's malus, a characteristic tested in place of another).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Situational {
    situation: Situation,
    ability: AbilityId,
    amount: Option<i32>,
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

    /// The amount added in the situation, `None` for a note.
    #[must_use]
    pub const fn amount(&self) -> Option<i32> {
        self.amount
    }
}

/// A row of the table: what it modifies, in which situation, the capacité
/// and the amount it gives.
type Row = (Target, Situation, Voie, u8, Option<i32>);

const DEFENCE: Target = Target::Value(Value::Defence);

/// The situational modifiers of the capacités, in voie order.
const ROWS: &[Row] = &[
    // Entraîné, Tacticien.
    (
        DEFENCE,
        Situation::AgainstContactWeapons,
        Voie::ArmesDeContact,
        2,
        Some(2),
    ),
    (
        Target::Value(Value::Attack(AttackMode::ContactWeapon)),
        Situation::CombatManoeuvre,
        Voie::ArmesDeContact,
        3,
        None,
    ),
    // Entraîné.
    (
        DEFENCE,
        Situation::AgainstRanged,
        Voie::ArmesADistance,
        2,
        Some(2),
    ),
    // Mitrailleur.
    (
        DEFENCE,
        Situation::AgainstExplosives,
        Voie::ArmesLourdes,
        2,
        Some(2),
    ),
    (
        Target::Value(Value::Attack(AttackMode::HeavyWeapon)),
        Situation::AutomaticFire,
        Voie::ArmesLourdes,
        2,
        None,
    ),
    // Entraîné, Tacticien, Disciple.
    (
        DEFENCE,
        Situation::AgainstUnarmed,
        Voie::CombatAMainsNues,
        2,
        Some(2),
    ),
    (
        Target::Value(Value::Attack(AttackMode::Unarmed)),
        Situation::CombatManoeuvre,
        Voie::CombatAMainsNues,
        3,
        None,
    ),
    (
        Target::Value(Value::DamageReduction),
        Situation::ParryingUnarmed,
        Voie::CombatAMainsNues,
        4,
        Some(1),
    ),
    // Gangster.
    (
        Target::Characteristic(Characteristic::For),
        Situation::Coercion,
        Voie::Crime,
        2,
        None,
    ),
    // Esprit pratique.
    (
        Target::Characteristic(Characteristic::Int),
        Situation::PhysicalTest,
        Voie::Sciences,
        3,
        None,
    ),
    // Pilote émérite.
    (
        DEFENCE,
        Situation::InMovingVehicle,
        Voie::Vehicules,
        4,
        Some(2),
    ),
];

/// The situational modifiers of the capacités the character holds to
/// `target`, in voie order.
pub(crate) fn of(character: &Character, target: Target) -> Vec<Situational> {
    ROWS.iter()
        .filter(|(applies, ..)| *applies == target)
        .filter(|(_, _, voie, rank, _)| character.has_rank(*voie, Rank::new(*rank)))
        .map(|(_, situation, voie, rank, amount)| Situational {
            situation: *situation,
            ability: voie.ability(Rank::new(*rank)),
            amount: *amount,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listed(character: &Character, target: Target) -> Vec<(Situation, Option<i32>)> {
        of(character, target)
            .iter()
            .map(|s| (s.situation(), s.amount()))
            .collect()
    }

    #[test]
    fn situations_are_named() {
        assert!(Situation::ALL.iter().all(|s| !s.name().is_empty()));
        assert_eq!(
            Situation::AgainstRanged.name(),
            "contre les attaques à distance"
        );
    }

    #[test]
    fn defence_lists_each_capacite_held_and_keeps_its_total() {
        let character = Character::new()
            .with_rank(Voie::ArmesDeContact, 1)
            .with_rank(Voie::ArmesDeContact, 2);
        assert_eq!(
            listed(&character, DEFENCE),
            [(Situation::AgainstContactWeapons, Some(2))]
        );
        let total = |c: &Character| c.value(Value::Defence).map(|v| v.total());
        assert_eq!(
            total(&character),
            total(&Character::new().with_rank(Voie::ArmesDeContact, 1))
        );
        assert_eq!(listed(&Character::new(), DEFENCE), []);
    }

    #[test]
    fn notes_have_no_amount() {
        let character = Character::new().with_rank(Voie::Crime, 2);
        let [note] = of(&character, Target::Characteristic(Characteristic::For))
            .try_into()
            .unwrap();
        assert_eq!(
            (note.situation(), note.amount()),
            (Situation::Coercion, None)
        );
        assert_eq!(note.ability(), &Voie::Crime.ability(Rank::new(2)));
        assert_eq!(
            listed(&character, Target::Characteristic(Characteristic::Cha)),
            []
        );
    }

    #[test]
    fn the_in_play_character_lists_the_same() {
        let character = Character::new().with_rank(Voie::Vehicules, 4);
        let played = character.in_play(crate::Equipment::default());
        assert_eq!(played.situational(DEFENCE), character.situational(DEFENCE));
        assert_eq!(
            listed(&character, DEFENCE),
            [(Situation::InMovingVehicle, Some(2))]
        );
    }
}
