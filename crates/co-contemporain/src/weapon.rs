//! Weapons (COC.EQP.MELEE, COC.EQP.RANGED) and their damage, bonus critique
//! and critical range (COC.DER.DM-BONUS, COC.DER.CRIT-BONUS,
//! COC.DER.CRIT-RANGE).

use co_kernel::{Characteristic, Characteristics, DerivedValue, Dice, Rank, Source, ValueId};

use crate::{AttackMode, Character, Value, Voie, rules::CRIT_RANGE};

/// A row of the weapon tables; a row covers every weapon it names.
///
/// Weapons the tables do not give a category or damage for (dynamite, mine,
/// tear gas grenade) are free text items (D-26, D-30).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weapon {
    /// Mains nues: always at hand, never carried (D-12).
    MainsNues,
    /// Fouet.
    Fouet,
    /// Couteau, poignard, baïonnette.
    Couteau,
    /// Batte de base-ball, matraque, tonfa.
    Batte,
    /// Marteau, maillet, nunchaku.
    Marteau,
    /// Hachette, hache de jet.
    Hachette,
    /// Machette.
    Machette,
    /// Fleuret, rapière, canne-épée.
    Fleuret,
    /// Épée, sabre, katana, hache.
    Epee,
    /// Hache, grand maillet, épée à deux mains.
    HacheADeuxMains,
    /// Hallebarde, pique.
    Hallebarde,
    /// Taser.
    Taser,
    /// Shuriken, couteau de lancer.
    Shuriken,
    /// Sagaie, javelot.
    Javelot,
    /// Arbalète.
    Arbalete,
    /// Arc court.
    ArcCourt,
    /// Arc long.
    ArcLong,
    /// Pistolet léger.
    PistoletLeger,
    /// Pistolet moyen (cal. 38).
    PistoletMoyen,
    /// Pistolet lourd (cal. 45).
    PistoletLourd,
    /// Fusil de chasse.
    FusilDeChasse,
    /// Carabine à répétition.
    CarabineARepetition,
    /// Fusil à pompe.
    FusilAPompe,
    /// Fusil de précision.
    FusilDePrecision,
    /// Fusil d'assaut.
    FusilDAssaut,
    /// Pistolet mitrailleur.
    PistoletMitrailleur,
    /// Mitrailleuse.
    Mitrailleuse,
    /// Mitrailleuse lourde, mounted.
    MitrailleuseLourde,
    /// Lance-flammes.
    LanceFlammes,
    /// Lance-grenade.
    LanceGrenade,
    /// Lance-roquette, bazooka, mortier.
    LanceRoquette,
    /// Canon léger (75 mm).
    CanonLeger,
    /// Grenade à fragmentation.
    GrenadeFragmentation,
    /// Grenade incendiaire: its first damage.
    GrenadeIncendiaire,
}

/// One row of the weapon tables.
struct Profile {
    mode: AttackMode,
    count: u8,
    faces: u8,
    /// Added to the dice: the "+1" of 1d6+1.
    flat: i32,
    /// An area weapon: no bonus critique.
    area: bool,
}

impl Profile {
    const fn new(mode: AttackMode, count: u8, faces: u8) -> Self {
        Self {
            mode,
            count,
            faces,
            flat: 0,
            area: false,
        }
    }

    const fn plus(self, flat: i32) -> Self {
        Self { flat, ..self }
    }

    const fn area(self) -> Self {
        Self { area: true, ..self }
    }
}

/// The rank of the weapon voies whose capacité widens the critical range and
/// adds a characteristic to the DM and bonus critique.
const MASTERY_RANK: u8 = 4;

/// The widening of the critical range by the fusil de précision and by the
/// rank-4 capacités.
const CRITICAL_WIDENING: i32 = -2;

impl Weapon {
    /// Every row of the weapon tables, in table order.
    pub const ALL: [Self; 34] = [
        Self::MainsNues,
        Self::Fouet,
        Self::Couteau,
        Self::Batte,
        Self::Marteau,
        Self::Hachette,
        Self::Machette,
        Self::Fleuret,
        Self::Epee,
        Self::HacheADeuxMains,
        Self::Hallebarde,
        Self::Taser,
        Self::Shuriken,
        Self::Javelot,
        Self::Arbalete,
        Self::ArcCourt,
        Self::ArcLong,
        Self::PistoletLeger,
        Self::PistoletMoyen,
        Self::PistoletLourd,
        Self::FusilDeChasse,
        Self::CarabineARepetition,
        Self::FusilAPompe,
        Self::FusilDePrecision,
        Self::FusilDAssaut,
        Self::PistoletMitrailleur,
        Self::Mitrailleuse,
        Self::MitrailleuseLourde,
        Self::LanceFlammes,
        Self::LanceGrenade,
        Self::LanceRoquette,
        Self::CanonLeger,
        Self::GrenadeFragmentation,
        Self::GrenadeIncendiaire,
    ];

    /// Returns the weapon whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|weapon| weapon.id().as_str() == id)
    }

    /// The French display name of the weapon row.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MainsNues => "Mains nues",
            Self::Fouet => "Fouet",
            Self::Couteau => "Couteau, poignard, baïonnette",
            Self::Batte => "Batte de base-ball, matraque, tonfa",
            Self::Marteau => "Marteau, maillet, nunchaku",
            Self::Hachette => "Hachette, hache de jet",
            Self::Machette => "Machette",
            Self::Fleuret => "Fleuret, rapière, canne-épée",
            Self::Epee => "Épée, sabre, katana, hache",
            Self::HacheADeuxMains => "Hache, grand maillet, épée à deux mains",
            Self::Hallebarde => "Hallebarde, pique",
            Self::Taser => "Taser",
            Self::Shuriken => "Shuriken, couteau de lancer",
            Self::Javelot => "Sagaie, javelot",
            Self::Arbalete => "Arbalète",
            Self::ArcCourt => "Arc court",
            Self::ArcLong => "Arc long",
            Self::PistoletLeger => "Pistolet léger",
            Self::PistoletMoyen => "Pistolet moyen (cal. 38)",
            Self::PistoletLourd => "Pistolet lourd (cal. 45)",
            Self::FusilDeChasse => "Fusil de chasse",
            Self::CarabineARepetition => "Carabine à répétition",
            Self::FusilAPompe => "Fusil à pompe",
            Self::FusilDePrecision => "Fusil de précision",
            Self::FusilDAssaut => "Fusil d'assaut",
            Self::PistoletMitrailleur => "Pistolet mitrailleur",
            Self::Mitrailleuse => "Mitrailleuse",
            Self::MitrailleuseLourde => "Mitrailleuse lourde",
            Self::LanceFlammes => "Lance-flammes",
            Self::LanceGrenade => "Lance-grenade",
            Self::LanceRoquette => "Lance-roquette, bazooka, mortier",
            Self::CanonLeger => "Canon léger (75 mm)",
            Self::GrenadeFragmentation => "Grenade à fragmentation",
            Self::GrenadeIncendiaire => "Grenade incendiaire",
        }
    }

    /// Returns the kernel identifier of the weapon.
    #[must_use]
    pub fn id(self) -> ValueId {
        ValueId::new(format!("coc.weapon.{}", self.slug()))
    }

    /// The attack value the weapon rolls against: ATC or ATD of its
    /// category.
    #[must_use]
    pub const fn attack_mode(self) -> AttackMode {
        self.profile().mode
    }

    /// The damage dice of the table, before any capacité.
    pub(crate) fn dice(self) -> Dice {
        let profile = self.profile();
        Dice::new((0..profile.count).map(|_| profile.faces))
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::MainsNues => "mains-nues",
            Self::Fouet => "fouet",
            Self::Couteau => "couteau",
            Self::Batte => "batte",
            Self::Marteau => "marteau",
            Self::Hachette => "hachette",
            Self::Machette => "machette",
            Self::Fleuret => "fleuret",
            Self::Epee => "epee",
            Self::HacheADeuxMains => "hache-a-deux-mains",
            Self::Hallebarde => "hallebarde",
            Self::Taser => "taser",
            Self::Shuriken => "shuriken",
            Self::Javelot => "javelot",
            Self::Arbalete => "arbalete",
            Self::ArcCourt => "arc-court",
            Self::ArcLong => "arc-long",
            Self::PistoletLeger => "pistolet-leger",
            Self::PistoletMoyen => "pistolet-moyen",
            Self::PistoletLourd => "pistolet-lourd",
            Self::FusilDeChasse => "fusil-de-chasse",
            Self::CarabineARepetition => "carabine-a-repetition",
            Self::FusilAPompe => "fusil-a-pompe",
            Self::FusilDePrecision => "fusil-de-precision",
            Self::FusilDAssaut => "fusil-d-assaut",
            Self::PistoletMitrailleur => "pistolet-mitrailleur",
            Self::Mitrailleuse => "mitrailleuse",
            Self::MitrailleuseLourde => "mitrailleuse-lourde",
            Self::LanceFlammes => "lance-flammes",
            Self::LanceGrenade => "lance-grenade",
            Self::LanceRoquette => "lance-roquette",
            Self::CanonLeger => "canon-leger",
            Self::GrenadeFragmentation => "grenade-fragmentation",
            Self::GrenadeIncendiaire => "grenade-incendiaire",
        }
    }

    const fn profile(self) -> Profile {
        use AttackMode::{ContactWeapon, HeavyWeapon, RangedWeapon, Unarmed};
        match self {
            Self::MainsNues => Profile::new(Unarmed, 1, 4),
            Self::Fouet => Profile::new(ContactWeapon, 1, 4),
            Self::Couteau | Self::Batte | Self::Hachette => Profile::new(ContactWeapon, 1, 6),
            Self::Marteau | Self::Machette => Profile::new(ContactWeapon, 1, 6).plus(1),
            Self::Fleuret => Profile::new(ContactWeapon, 1, 8),
            Self::Epee => Profile::new(ContactWeapon, 1, 8).plus(1),
            Self::HacheADeuxMains | Self::Hallebarde => Profile::new(ContactWeapon, 1, 10),
            Self::Taser => Profile::new(RangedWeapon, 1, 4),
            Self::Shuriken => Profile::new(RangedWeapon, 1, 6),
            Self::Javelot | Self::ArcCourt | Self::PistoletLeger => {
                Profile::new(RangedWeapon, 1, 8)
            }
            Self::ArcLong | Self::PistoletMoyen | Self::PistoletMitrailleur => {
                Profile::new(RangedWeapon, 1, 8).plus(1)
            }
            Self::Arbalete
            | Self::PistoletLourd
            | Self::FusilDeChasse
            | Self::CarabineARepetition => Profile::new(RangedWeapon, 1, 10),
            Self::FusilAPompe | Self::FusilDePrecision => Profile::new(RangedWeapon, 1, 10).plus(1),
            Self::FusilDAssaut => Profile::new(RangedWeapon, 1, 12),
            Self::Mitrailleuse => Profile::new(HeavyWeapon, 1, 12),
            Self::MitrailleuseLourde => Profile::new(HeavyWeapon, 2, 6).plus(1),
            Self::LanceFlammes => Profile::new(HeavyWeapon, 2, 6),
            Self::LanceGrenade => Profile::new(HeavyWeapon, 2, 6).plus(2),
            Self::LanceRoquette | Self::GrenadeFragmentation => {
                Profile::new(HeavyWeapon, 4, 6).area()
            }
            Self::CanonLeger => Profile::new(HeavyWeapon, 6, 6).area(),
            Self::GrenadeIncendiaire => Profile::new(HeavyWeapon, 2, 6).area(),
        }
    }
}

/// The identifier of a weapon value, such as `coc.damage.couteau`.
pub(crate) fn value_id(value: &str, weapon: Weapon) -> ValueId {
    ValueId::new(format!("coc.{value}.{}", weapon.slug()))
}

/// The voie whose rank-4 capacité adds a characteristic to the weapon's DM
/// and bonus critique and widens its critical range, with that
/// characteristic (COC.CNT.VOIES).
const fn mastery(weapon: Weapon) -> Option<(Voie, Characteristic)> {
    match weapon.attack_mode() {
        AttackMode::ContactWeapon => Some((Voie::ArmesDeContact, Characteristic::Agi)),
        AttackMode::RangedWeapon => Some((Voie::ArmesADistance, Characteristic::Per)),
        AttackMode::Unarmed | AttackMode::HeavyWeapon => None,
    }
}

/// Adds the characteristic of the rank-4 weapon capacité, when acquired.
fn with_mastery(
    value: DerivedValue,
    character: &Character,
    values: &Characteristics,
    weapon: Weapon,
) -> DerivedValue {
    let rank = Rank::new(MASTERY_RANK);
    match mastery(weapon) {
        Some((voie, characteristic)) if character.has_rank(voie, rank) => value.with(
            Source::Ability(voie.ability(rank)),
            values[characteristic].get(),
        ),
        _ => value,
    }
}

/// The damage dice: the table's, and 1d6 unarmed with the rank-2 capacité
/// of the combat à mains nues.
pub(crate) fn damage_dice(character: &Character, weapon: Weapon) -> Dice {
    if weapon == Weapon::MainsNues && character.has_rank(Voie::CombatAMainsNues, Rank::new(2)) {
        Dice::new([6])
    } else {
        weapon.dice()
    }
}

/// The DM added to the dice: the weapon's own, the contact DM bonus for
/// unarmed and contact attacks (COC.DER.DM-BONUS), and the rank-4
/// capacité's characteristic.
pub(crate) fn damage(
    character: &Character,
    values: &Characteristics,
    weapon: Weapon,
) -> DerivedValue {
    let flat = weapon.profile().flat;
    let mut value = DerivedValue::new();
    if flat != 0 {
        value = value.with(Source::Value(weapon.id()), flat);
    }
    if matches!(
        weapon.attack_mode(),
        AttackMode::Unarmed | AttackMode::ContactWeapon
    ) && let Some(bonus) = character.value(Value::ContactDamageBonus)
    {
        value.extend(bonus.contributions().iter().cloned());
    }
    with_mastery(value, character, values, weapon)
}

/// The bonus critique (COC.DER.CRIT-BONUS): the highest result of the dice
/// and the weapon's own DM, with the rank-4 capacité's characteristic.
/// `None` for an area weapon.
pub(crate) fn critical_bonus(
    character: &Character,
    values: &Characteristics,
    weapon: Weapon,
) -> Option<DerivedValue> {
    let profile = weapon.profile();
    if profile.area {
        return None;
    }
    let highest = damage_dice(character, weapon).highest() + profile.flat;
    let value = DerivedValue::new().with(Source::Value(weapon.id()), highest);
    Some(with_mastery(value, character, values, weapon))
}

/// The lowest natural roll of a critical success (COC.DER.CRIT-RANGE): 20,
/// widened by 2 by the fusil de précision and by the rank-4 capacité.
pub(crate) fn critical_range(character: &Character, weapon: Weapon) -> DerivedValue {
    let mut value = DerivedValue::new().with(Source::Rule(CRIT_RANGE), 20);
    if weapon == Weapon::FusilDePrecision {
        value = value.with(Source::Value(weapon.id()), CRITICAL_WIDENING);
    }
    let rank = Rank::new(MASTERY_RANK);
    match mastery(weapon) {
        Some((voie, _)) if character.has_rank(voie, rank) => {
            value.with(Source::Ability(voie.ability(rank)), CRITICAL_WIDENING)
        }
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::Characteristic;

    use super::*;
    use crate::Trait;

    fn total(character: &Character, value: Value) -> Option<i32> {
        character.value(value).map(|v| v.total())
    }

    fn ranks(character: Character, voie: Voie, highest: u8) -> Character {
        (1..=highest).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    #[test]
    fn bonus_critique_is_the_highest_damage() {
        let character = Character::new();
        for (weapon, expected) in [
            (Weapon::MainsNues, Some(4)),
            (Weapon::Couteau, Some(6)),
            (Weapon::Marteau, Some(7)),
            (Weapon::FusilAPompe, Some(11)),
            (Weapon::MitrailleuseLourde, Some(13)),
            (Weapon::LanceGrenade, Some(14)),
            (Weapon::LanceFlammes, Some(12)),
            (Weapon::LanceRoquette, None),
            (Weapon::GrenadeFragmentation, None),
        ] {
            assert_eq!(
                total(&character, Value::CriticalBonus(weapon)),
                expected,
                "{weapon:?}"
            );
        }
    }

    #[test]
    fn contact_damage_adds_the_contact_bonus() {
        let character = Character::new()
            .with_carac_points(Characteristic::For, 2)
            .with_trait(Trait::ForceDeLaNature, 2);
        assert_eq!(
            total(&character, Value::Damage(Weapon::Epee)),
            Some(1 + 2 + 1)
        );
        assert_eq!(total(&character, Value::Damage(Weapon::MainsNues)), Some(3));
        assert_eq!(
            total(&character, Value::Damage(Weapon::PistoletMoyen)),
            Some(1)
        );
        assert_eq!(total(&character, Value::Damage(Weapon::Arbalete)), Some(0));
    }

    #[test]
    fn unarmed_dice_grow_with_combat_a_mains_nues() {
        let character = Character::new();
        assert_eq!(character.damage_dice(Weapon::MainsNues), Dice::new([4]));
        let boxer = ranks(character, Voie::CombatAMainsNues, 2);
        assert_eq!(boxer.damage_dice(Weapon::MainsNues), Dice::new([6]));
        assert_eq!(
            total(&boxer, Value::CriticalBonus(Weapon::MainsNues)),
            Some(6)
        );
        assert_eq!(boxer.damage_dice(Weapon::LanceFlammes), Dice::new([6, 6]));
    }

    #[test]
    fn rank_four_capacites_add_to_damage_and_critical() {
        let character = Character::new()
            .with_carac_points(Characteristic::Agi, 2)
            .with_carac_points(Characteristic::Per, 3);
        let fencer = ranks(character.clone(), Voie::ArmesDeContact, 4);
        assert_eq!(total(&fencer, Value::Damage(Weapon::Fleuret)), Some(2));
        assert_eq!(
            total(&fencer, Value::CriticalBonus(Weapon::Fleuret)),
            Some(8 + 2)
        );
        assert_eq!(
            total(&fencer, Value::CriticalRange(Weapon::Fleuret)),
            Some(18)
        );
        assert_eq!(
            total(&fencer, Value::CriticalRange(Weapon::MainsNues)),
            Some(20)
        );
        let sniper = ranks(character, Voie::ArmesADistance, 4);
        assert_eq!(
            total(&sniper, Value::Damage(Weapon::FusilDePrecision)),
            Some(1 + 3)
        );
        assert_eq!(
            total(&sniper, Value::CriticalBonus(Weapon::FusilDePrecision)),
            Some(11 + 3)
        );
        assert_eq!(
            total(&sniper, Value::CriticalRange(Weapon::FusilDePrecision)),
            Some(16)
        );
        assert_eq!(
            total(
                &Character::new(),
                Value::CriticalRange(Weapon::FusilDePrecision)
            ),
            Some(18)
        );
        let below = ranks(Character::new(), Voie::ArmesADistance, 3);
        assert_eq!(
            total(&below, Value::CriticalRange(Weapon::PistoletLeger)),
            Some(20)
        );
    }

    #[test]
    fn weapons_attack_with_their_category() {
        assert_eq!(Weapon::MainsNues.attack_mode(), AttackMode::Unarmed);
        assert_eq!(Weapon::Couteau.attack_mode(), AttackMode::ContactWeapon);
        assert_eq!(
            Weapon::PistoletLeger.attack_mode(),
            AttackMode::RangedWeapon
        );
        assert_eq!(
            Weapon::GrenadeIncendiaire.attack_mode(),
            AttackMode::HeavyWeapon
        );
        assert_eq!(Weapon::Couteau.id().as_str(), "coc.weapon.couteau");
    }
}
