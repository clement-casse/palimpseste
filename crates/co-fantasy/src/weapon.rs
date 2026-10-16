//! Weapons (COF.EQP.MELEE, COF.EQP.RANGED), their damage and critical range
//! (COF.DER.DM, COF.DER.CRIT-RANGE), and the checks on the weapons carried.

use co_kernel::{
    Characteristic, Characteristics, DerivedValue, Dice, Rank, Severity, Source, Subject, ValueId,
    Violation,
};

use crate::{
    AttackMode, Character, Voie, derived,
    rules::{CRIT_RANGE, PEUPLES, RANGED, WEAPON_MECHANIC},
};

/// A weapon of the contact and ranged weapon tables.
///
/// Unarmed attacks are not a weapon. Weapons the tables do not list are
/// free-text items (D-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weapon {
    /// Bâton.
    Baton,
    /// Bâton ferré.
    BatonFerre,
    /// Dague, at contact or thrown.
    Dague,
    /// Épée à deux mains.
    EpeeADeuxMains,
    /// Épée bâtarde.
    EpeeBatarde,
    /// Épée courte.
    EpeeCourte,
    /// Épée longue.
    EpeeLongue,
    /// Épieu, at contact or thrown.
    Epieu,
    /// Fléau.
    Fleau,
    /// Fléau à deux mains.
    FleauADeuxMains,
    /// Gourdin.
    Gourdin,
    /// Hache.
    Hache,
    /// Hache à deux mains.
    HacheADeuxMains,
    /// Lance, at contact or thrown.
    Lance,
    /// Lance de cavalerie.
    LanceDeCavalerie,
    /// Marteau.
    Marteau,
    /// Masse.
    Masse,
    /// Pique.
    Pique,
    /// Rapière.
    Rapiere,
    /// Stylet.
    Stylet,
    /// Vivelame.
    Vivelame,
    /// Arbalète de poing.
    ArbaleteDePoing,
    /// Arbalète légère.
    ArbaleteLegere,
    /// Arbalète lourde.
    ArbaleteLourde,
    /// Arc court.
    ArcCourt,
    /// Arc long.
    ArcLong,
    /// Couteaux de lancer, thrown or at contact.
    CouteauxDeLancer,
    /// Fronde.
    Fronde,
    /// Hachette.
    Hachette,
    /// Javelot.
    Javelot,
    /// Lance-pierre.
    LancePierre,
    /// Pétoire.
    Petoire,
    /// Mousquet.
    Mousquet,
}

/// How a weapon is used: its damage dice depend on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WeaponUse {
    /// At contact, in one hand.
    OneHanded,
    /// At contact, in two hands.
    TwoHanded,
    /// Shot or thrown.
    Ranged,
}

impl WeaponUse {
    /// Every use, in declaration order.
    pub const ALL: [Self; 3] = [Self::OneHanded, Self::TwoHanded, Self::Ranged];

    /// The French display name of the use.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OneHanded => "À une main",
            Self::TwoHanded => "À deux mains",
            Self::Ranged => "À distance",
        }
    }

    /// The attack value this use rolls against (COF.DER.ATTACK).
    #[must_use]
    pub const fn attack_mode(self) -> AttackMode {
        match self {
            Self::OneHanded | Self::TwoHanded => AttackMode::Contact,
            Self::Ranged => AttackMode::Ranged,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::OneHanded => "one-handed",
            Self::TwoHanded => "two-handed",
            Self::Ranged => "ranged",
        }
    }
}

/// Damage dice of a use, as (count, faces).
type Roll = Option<(u8, u8)>;

/// One row of the weapon tables.
struct Profile {
    one_handed: Roll,
    two_handed: Roll,
    ranged: Roll,
    /// A ranged weapon held in two hands.
    ranged_two_hands: bool,
    /// Contondant.
    blunt: bool,
    /// The lowest natural roll of a critical success.
    critical: i32,
}

impl Profile {
    /// A row with no use yet, contondant or not, critical on 20.
    const NONE: Self = Self {
        one_handed: None,
        two_handed: None,
        ranged: None,
        ranged_two_hands: false,
        blunt: false,
        critical: 20,
    };

    const fn one_handed(self, count: u8, faces: u8) -> Self {
        Self {
            one_handed: Some((count, faces)),
            ..self
        }
    }

    const fn two_handed(self, count: u8, faces: u8) -> Self {
        Self {
            two_handed: Some((count, faces)),
            ..self
        }
    }

    const fn ranged(self, count: u8, faces: u8) -> Self {
        Self {
            ranged: Some((count, faces)),
            ..self
        }
    }

    /// Shot with both hands.
    const fn in_two_hands(self) -> Self {
        Self {
            ranged_two_hands: true,
            ..self
        }
    }

    const fn blunt(self) -> Self {
        Self {
            blunt: true,
            ..self
        }
    }

    const fn critical(self, critical: i32) -> Self {
        Self { critical, ..self }
    }
}

impl Weapon {
    /// Every weapon, in table order: contact weapons, then ranged weapons.
    pub const ALL: [Self; 33] = [
        Self::Baton,
        Self::BatonFerre,
        Self::Dague,
        Self::EpeeADeuxMains,
        Self::EpeeBatarde,
        Self::EpeeCourte,
        Self::EpeeLongue,
        Self::Epieu,
        Self::Fleau,
        Self::FleauADeuxMains,
        Self::Gourdin,
        Self::Hache,
        Self::HacheADeuxMains,
        Self::Lance,
        Self::LanceDeCavalerie,
        Self::Marteau,
        Self::Masse,
        Self::Pique,
        Self::Rapiere,
        Self::Stylet,
        Self::Vivelame,
        Self::ArbaleteDePoing,
        Self::ArbaleteLegere,
        Self::ArbaleteLourde,
        Self::ArcCourt,
        Self::ArcLong,
        Self::CouteauxDeLancer,
        Self::Fronde,
        Self::Hachette,
        Self::Javelot,
        Self::LancePierre,
        Self::Petoire,
        Self::Mousquet,
    ];

    /// Returns the weapon whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|weapon| weapon.id().as_str() == id)
    }

    /// The French display name of the weapon.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Baton => "Bâton",
            Self::BatonFerre => "Bâton ferré",
            Self::Dague => "Dague",
            Self::EpeeADeuxMains => "Épée à deux mains",
            Self::EpeeBatarde => "Épée bâtarde",
            Self::EpeeCourte => "Épée courte",
            Self::EpeeLongue => "Épée longue",
            Self::Epieu => "Épieu",
            Self::Fleau => "Fléau",
            Self::FleauADeuxMains => "Fléau à deux mains",
            Self::Gourdin => "Gourdin",
            Self::Hache => "Hache",
            Self::HacheADeuxMains => "Hache à deux mains",
            Self::Lance => "Lance",
            Self::LanceDeCavalerie => "Lance de cavalerie",
            Self::Marteau => "Marteau",
            Self::Masse => "Masse",
            Self::Pique => "Pique",
            Self::Rapiere => "Rapière",
            Self::Stylet => "Stylet",
            Self::Vivelame => "Vivelame",
            Self::ArbaleteDePoing => "Arbalète de poing",
            Self::ArbaleteLegere => "Arbalète légère",
            Self::ArbaleteLourde => "Arbalète lourde",
            Self::ArcCourt => "Arc court",
            Self::ArcLong => "Arc long",
            Self::CouteauxDeLancer => "Couteaux de lancer",
            Self::Fronde => "Fronde",
            Self::Hachette => "Hachette",
            Self::Javelot => "Javelot",
            Self::LancePierre => "Lance-pierre",
            Self::Petoire => "Pétoire",
            Self::Mousquet => "Mousquet",
        }
    }

    /// Returns the kernel identifier of the weapon.
    #[must_use]
    pub fn id(self) -> ValueId {
        ValueId::new(format!("cof.weapon.{}", self.slug()))
    }

    /// The damage dice of the weapon in `weapon_use`, `None` when the weapon
    /// cannot be used that way.
    #[must_use]
    pub fn dice(self, weapon_use: WeaponUse) -> Option<Dice> {
        let profile = self.profile();
        let (count, faces) = match weapon_use {
            WeaponUse::OneHanded => profile.one_handed,
            WeaponUse::TwoHanded => profile.two_handed,
            WeaponUse::Ranged => profile.ranged,
        }?;
        Some(Dice::new((0..count).map(|_| faces)))
    }

    /// Usable at contact.
    pub(crate) const fn is_contact(self) -> bool {
        let profile = self.profile();
        profile.one_handed.is_some() || profile.two_handed.is_some()
    }

    /// Usable at contact in one hand.
    pub(crate) const fn is_one_handed_contact(self) -> bool {
        self.profile().one_handed.is_some()
    }

    /// Shot or thrown.
    pub(crate) const fn is_ranged(self) -> bool {
        self.profile().ranged.is_some()
    }

    /// Usable in one hand, at contact or at range.
    pub(crate) const fn is_one_handed(self) -> bool {
        let profile = self.profile();
        profile.one_handed.is_some() || (profile.ranged.is_some() && !profile.ranged_two_hands)
    }

    /// Contondant.
    pub(crate) const fn is_blunt(self) -> bool {
        self.profile().blunt
    }

    /// A powder weapon.
    pub(crate) const fn is_powder(self) -> bool {
        matches!(self, Self::Petoire | Self::Mousquet)
    }

    /// An arbalète.
    pub(crate) const fn is_crossbow(self) -> bool {
        matches!(
            self,
            Self::ArbaleteDePoing | Self::ArbaleteLegere | Self::ArbaleteLourde
        )
    }

    /// The highest damage of the weapon, whatever its use.
    pub(crate) fn highest_damage(self) -> i32 {
        WeaponUse::ALL
            .into_iter()
            .filter_map(|weapon_use| self.dice(weapon_use))
            .map(|dice| dice.highest())
            .max()
            .unwrap_or(0)
    }

    /// The highest damage of the weapon's lightest use.
    fn lightest_use_damage(self) -> i32 {
        WeaponUse::ALL
            .into_iter()
            .filter_map(|weapon_use| self.dice(weapon_use))
            .map(|dice| dice.highest())
            .min()
            .unwrap_or(0)
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Baton => "baton",
            Self::BatonFerre => "baton-ferre",
            Self::Dague => "dague",
            Self::EpeeADeuxMains => "epee-a-deux-mains",
            Self::EpeeBatarde => "epee-batarde",
            Self::EpeeCourte => "epee-courte",
            Self::EpeeLongue => "epee-longue",
            Self::Epieu => "epieu",
            Self::Fleau => "fleau",
            Self::FleauADeuxMains => "fleau-a-deux-mains",
            Self::Gourdin => "gourdin",
            Self::Hache => "hache",
            Self::HacheADeuxMains => "hache-a-deux-mains",
            Self::Lance => "lance",
            Self::LanceDeCavalerie => "lance-de-cavalerie",
            Self::Marteau => "marteau",
            Self::Masse => "masse",
            Self::Pique => "pique",
            Self::Rapiere => "rapiere",
            Self::Stylet => "stylet",
            Self::Vivelame => "vivelame",
            Self::ArbaleteDePoing => "arbalete-de-poing",
            Self::ArbaleteLegere => "arbalete-legere",
            Self::ArbaleteLourde => "arbalete-lourde",
            Self::ArcCourt => "arc-court",
            Self::ArcLong => "arc-long",
            Self::CouteauxDeLancer => "couteaux-de-lancer",
            Self::Fronde => "fronde",
            Self::Hachette => "hachette",
            Self::Javelot => "javelot",
            Self::LancePierre => "lance-pierre",
            Self::Petoire => "petoire",
            Self::Mousquet => "mousquet",
        }
    }

    /// The row of the weapon tables. A weapon listed in both tables (dague,
    /// épieu, lance, couteaux de lancer) has a contact and a ranged use.
    const fn profile(self) -> Profile {
        let row = Profile::NONE;
        match self {
            Self::Baton => row.two_handed(1, 4).blunt(),
            Self::BatonFerre => row.two_handed(1, 6).blunt(),
            Self::Dague => row.one_handed(1, 4).ranged(1, 4),
            Self::EpeeADeuxMains | Self::HacheADeuxMains => row.two_handed(2, 6),
            Self::EpeeBatarde => row.one_handed(1, 8).two_handed(1, 12),
            Self::EpeeCourte => row.one_handed(1, 6),
            Self::EpeeLongue | Self::Hache => row.one_handed(1, 8),
            Self::Epieu | Self::Lance => row.one_handed(1, 6).two_handed(1, 10).ranged(1, 6),
            Self::Fleau | Self::Marteau | Self::Masse => row.one_handed(1, 6).blunt(),
            Self::FleauADeuxMains => row.two_handed(1, 10).blunt(),
            Self::Gourdin => row.one_handed(1, 4).blunt(),
            Self::LanceDeCavalerie => row.one_handed(2, 6),
            Self::Pique => row.two_handed(1, 10),
            Self::Rapiere => row.one_handed(1, 6).critical(19),
            Self::Stylet => row.one_handed(1, 3),
            Self::Vivelame => row.two_handed(1, 10).critical(19),
            Self::ArbaleteDePoing | Self::Hachette | Self::Javelot => row.ranged(1, 6),
            Self::ArbaleteLegere => row.ranged(2, 4).in_two_hands(),
            Self::ArbaleteLourde | Self::Mousquet => row.ranged(2, 6).in_two_hands(),
            Self::ArcCourt => row.ranged(1, 6).in_two_hands(),
            Self::ArcLong => row.ranged(1, 8).in_two_hands(),
            Self::CouteauxDeLancer => row.one_handed(1, 3).ranged(1, 4),
            Self::Fronde => row.ranged(1, 4).blunt(),
            Self::LancePierre => row.ranged(1, 3).blunt(),
            Self::Petoire => row.ranged(1, 10),
        }
    }
}

/// The identifier of the damage of `weapon` in `weapon_use`.
pub(crate) fn damage_id(weapon: Weapon, weapon_use: WeaponUse) -> ValueId {
    ValueId::new(format!(
        "cof.damage.{}.{}",
        weapon.slug(),
        weapon_use.slug()
    ))
}

/// The identifier of the critical range of `weapon`.
pub(crate) fn critical_range_id(weapon: Weapon) -> ValueId {
    ValueId::new(format!("cof.critical-range.{}", weapon.slug()))
}

/// The DM added to the dice (COF.DER.DM): FOR at contact, except for the
/// stylet; nothing at range. `None` when the weapon cannot be used that way.
pub(crate) fn damage(
    values: &Characteristics,
    weapon: Weapon,
    weapon_use: WeaponUse,
) -> Option<DerivedValue> {
    weapon.dice(weapon_use)?;
    let value = DerivedValue::new();
    Some(
        if weapon_use.attack_mode() == AttackMode::Contact && weapon != Weapon::Stylet {
            value.with_characteristic(values, Characteristic::For)
        } else {
            value
        },
    )
}

/// The lowest natural roll of a critical success (COF.DER.CRIT-RANGE): 20,
/// widened by the weapon.
pub(crate) fn critical_range(weapon: Weapon) -> DerivedValue {
    let value = DerivedValue::new().with(Source::Rule(CRIT_RANGE), 20);
    let widening = weapon.profile().critical - 20;
    if widening == 0 {
        value
    } else {
        value.with(Source::Value(weapon.id()), widening)
    }
}

/// The highest damage a halfelin's weapon may deal (Petite taille, D-29).
const SMALL_DAMAGE_MAX: i32 = 10;

/// Checks the weapons carried: the arc long needs FOR +1 (COF.EQP.RANGED),
/// a halfelin cannot wield large weapons (D-29).
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let strength = character.characteristics()[Characteristic::For].get();
    let small = derived::has_peuple_ability(character, Voie::Halfelin);
    for weapon in character.weapons() {
        let subject = || Subject::Value(weapon.id());
        if weapon == Weapon::ArcLong && strength < 1 {
            violations.push(
                Violation::deviation(RANGED, WEAPON_MECHANIC, subject(), Severity::Standard)
                    .with_parameter("value", strength)
                    .with_parameter("min", 1),
            );
        }
        let too_large = matches!(weapon, Weapon::ArcLong | Weapon::ArbaleteLourde)
            || weapon.lightest_use_damage() > SMALL_DAMAGE_MAX;
        if small && too_large {
            violations.push(
                Violation::deviation(PEUPLES, WEAPON_MECHANIC, subject(), Severity::Standard)
                    .with_parameter("ability", Voie::Halfelin.ability(Rank::new(1)).as_str()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;
    use crate::{Peuple, Value};

    fn weapon_violations(character: &Character) -> Vec<(String, Subject)> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.subject().clone()))
            .collect()
    }

    #[test]
    fn dice_follow_the_use() {
        assert_eq!(
            Weapon::EpeeBatarde.dice(WeaponUse::OneHanded),
            Some(Dice::new([8]))
        );
        assert_eq!(
            Weapon::EpeeBatarde.dice(WeaponUse::TwoHanded),
            Some(Dice::new([12]))
        );
        assert_eq!(Weapon::EpeeBatarde.dice(WeaponUse::Ranged), None);
        assert_eq!(Weapon::Dague.dice(WeaponUse::Ranged), Some(Dice::new([4])));
        assert_eq!(
            Weapon::ArbaleteLegere.dice(WeaponUse::Ranged),
            Some(Dice::new([4, 4]))
        );
        assert_eq!(Weapon::BatonFerre.dice(WeaponUse::OneHanded), None);
    }

    #[test]
    fn hands_and_kinds() {
        assert!(Weapon::Dague.is_one_handed());
        assert!(Weapon::Javelot.is_one_handed());
        assert!(!Weapon::ArcCourt.is_one_handed());
        assert!(!Weapon::BatonFerre.is_one_handed());
        assert!(Weapon::Lance.is_contact() && Weapon::Lance.is_ranged());
        assert!(!Weapon::Hachette.is_contact());
        assert_eq!(Weapon::Epieu.highest_damage(), 10);
        assert_eq!(Weapon::EpeeBatarde.lightest_use_damage(), 8);
    }

    #[test]
    fn contact_damage_adds_strength() {
        let values = Characteristics::from_fn(|c| match c {
            Characteristic::For => 3.into(),
            _ => 0.into(),
        });
        let total = |weapon, weapon_use| damage(&values, weapon, weapon_use).map(|v| v.total());
        assert_eq!(total(Weapon::EpeeLongue, WeaponUse::OneHanded), Some(3));
        assert_eq!(total(Weapon::Dague, WeaponUse::Ranged), Some(0));
        assert_eq!(total(Weapon::Stylet, WeaponUse::OneHanded), Some(0));
        assert_eq!(total(Weapon::EpeeLongue, WeaponUse::TwoHanded), None);
    }

    #[test]
    fn rapiere_and_vivelame_widen_the_critical_range() {
        assert_eq!(critical_range(Weapon::EpeeLongue).total(), 20);
        assert_eq!(critical_range(Weapon::Rapiere).total(), 19);
        assert_eq!(critical_range(Weapon::Vivelame).total(), 19);
        let character = Character::new();
        assert_eq!(
            character
                .value(Value::CriticalRange(Weapon::Rapiere))
                .map(|v| v.total()),
            Some(19)
        );
    }

    #[test]
    fn arc_long_needs_strength() {
        let weak = Character::new().with_item(Weapon::ArcLong, 1);
        assert_eq!(
            weapon_violations(&weak),
            [(
                "COF.EQP.RANGED".to_owned(),
                Subject::Value(Weapon::ArcLong.id())
            )]
        );
        let strong = weak.with_series_value(Characteristic::For, 1);
        assert_eq!(weapon_violations(&strong), []);
    }

    #[test]
    fn halfelin_cannot_wield_large_weapons() {
        let character = Character::new()
            .with_peuple(Peuple::Halfelin)
            .with_series_value(Characteristic::For, 1)
            .with_rank(Voie::Halfelin, 1)
            .with_item(Weapon::EpeeBatarde, 1)
            .with_item(Weapon::EpeeADeuxMains, 1)
            .with_item(Weapon::ArcLong, 1)
            .with_item(Weapon::Dague, 1);
        let subjects: Vec<_> = weapon_violations(&character)
            .into_iter()
            .filter(|(rule, _)| rule == PEUPLES.as_str())
            .map(|(_, subject)| subject)
            .collect();
        assert_eq!(
            subjects,
            [
                Subject::Value(Weapon::EpeeADeuxMains.id()),
                Subject::Value(Weapon::ArcLong.id()),
            ]
        );
        let category = character
            .violations()
            .into_iter()
            .find(|v| v.rule() == &PEUPLES)
            .map(|v| v.category());
        assert_eq!(category, Some(Category::Deviation(Severity::Standard)));
        let without = character
            .without_rank(Voie::Halfelin, 1)
            .with_series_value(Characteristic::For, 2);
        assert_eq!(weapon_violations(&without), []);
    }
}
