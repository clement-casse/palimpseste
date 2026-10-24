//! Documents convert to the engines' characters: the book's worked
//! characters, empty documents, and documents that cannot be read.

use co_kernel::{
    Adjustment, Author, Characteristic as C, RuleId, Subject as KernelSubject, Waiver,
};
use palimpseste_document::{
    AbilityOptionEntry, AdjustmentEntry, Characteristic, ContemporainDocument,
    ContemporainEquipment, Document, Engine, Equipment, Error, FantasyDocument, FantasyEquipment,
    ItemEntry, LevelUpEntry, RaisedCharacteristicEntry, Subject, Target, TraitChangeEntry,
    WaiverEntry, XpSpendingEntry,
};

/// Fixture helpers; a test module, so they may unwrap.
#[cfg(test)]
mod helpers {
    use super::Document;

    pub fn fixture(name: &str) -> Document {
        let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    pub fn fantasy(document: &Document) -> co_fantasy::Character {
        match document {
            Document::Fantasy(document) => document.to_character().unwrap(),
            Document::Contemporain(_) => panic!("not a fantasy document"),
        }
    }

    pub fn contemporain(document: &Document) -> co_contemporain::Character {
        match document {
            Document::Contemporain(document) => document.to_character().unwrap(),
            Document::Fantasy(_) => panic!("not a contemporain document"),
        }
    }
}

use helpers::{contemporain, fantasy, fixture};

#[test]
fn lhagva_converts_to_the_engine_character() {
    use co_fantasy::{Armour, Character, Money, Peuple, Profil, Series, Shield, Voie, Weapon};
    let values = [
        (C::For, 3),
        (C::Con, 2),
        (C::Agi, 1),
        (C::Vol, 1),
        (C::Cha, -1),
        (C::Per, 0),
        (C::Int, 0),
    ];
    let expected = values
        .into_iter()
        .fold(
            Character::new()
                .with_profil(Profil::Barbare)
                .with_peuple(Peuple::Humain)
                .with_series(Series::Expert),
            |c, (characteristic, value)| c.with_series_value(characteristic, value),
        )
        .with_peuple_bonus(C::Per)
        .with_rank(Voie::Pourfendeur, 1)
        .with_rank(Voie::Rage, 1)
        .with_rank(Voie::Humain, 1)
        .with_armour(Armour::CuirSimple)
        .with_shield(Shield::GrandBouclier)
        .with_item(Weapon::EpeeLongue, 1)
        .with_item(Weapon::Javelot, 2)
        .with_item(Weapon::Dague, 1)
        .with_item("adventurer's bag", 1)
        .with_purse(Money::pa(7));
    let character = fantasy(&fixture("lhagva"));
    assert_eq!(character, expected);
    assert!(character.report().is_clean());
}

#[test]
fn ionas_converts_to_a_clean_character() {
    let character = fantasy(&fixture("ionas"));
    assert!(character.report().is_clean());
    assert_eq!(
        character
            .value(co_fantasy::Value::VigourPoints)
            .map(|v| v.total()),
        Some(7)
    );
}

#[test]
fn amelia_steele_converts_to_a_clean_character() {
    let character = contemporain(&fixture("amelia-steele"));
    assert!(character.report().is_clean());
    assert_eq!(
        character.age_bracket(),
        Some(co_contemporain::AgeBracket::From31To50)
    );
}

#[test]
fn fixtures_round_trip_through_json() {
    for name in ["lhagva", "ionas", "amelia-steele"] {
        let document = fixture(name);
        let json = serde_json::to_string(&document).unwrap();
        let back: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(back, document, "{name}");
    }
}

#[test]
fn empty_documents_convert_to_new_characters() {
    assert_eq!(
        fantasy(&Document::empty(Engine::Fantasy)),
        co_fantasy::Character::new()
    );
    assert_eq!(
        contemporain(&Document::empty(Engine::Contemporain)),
        co_contemporain::Character::new()
    );
    assert_eq!(Document::empty(Engine::Fantasy).engine(), Engine::Fantasy);
}

#[test]
fn rule_breaking_documents_are_readable() {
    let mut document = FantasyDocument::empty();
    document.voies = vec![palimpseste_document::VoieEntry {
        id: "cof.combat".into(),
        ranks: vec![3, 9],
    }];
    document.series_values.r#for = Some(12);
    let character = document.to_character().unwrap();
    assert!(!character.report().is_clean());
    assert!(Document::Fantasy(document).check().is_ok());
}

#[test]
fn adjustments_and_waivers_convert() {
    let mut document = ContemporainDocument::empty();
    document.adjustments = vec![AdjustmentEntry {
        target: Target::Value("coc.damage.epee".into()),
        amount: 1,
        author: "gm".into(),
        reason: "heirloom".into(),
    }];
    document.waivers = vec![WaiverEntry {
        rule: "COC.CRE.EQUIPMENT".into(),
        subject: Subject::Value("coc.protection.casque".into()),
        author: "gm".into(),
        reason: "biker".into(),
    }];
    let expected = co_contemporain::Character::new()
        .with_adjustment(
            co_contemporain::Target::Value(co_contemporain::Value::Damage(
                co_contemporain::Weapon::Epee,
            )),
            Adjustment::new(1, Author::new("gm"), "heirloom"),
        )
        .with_waiver(Waiver::new(
            RuleId::new("COC.CRE.EQUIPMENT"),
            KernelSubject::Value(co_contemporain::Protection::Casque.id()),
            Author::new("gm"),
            "biker",
        ));
    assert_eq!(document.to_character(), Ok(expected));
}

#[test]
fn unreadable_documents_say_why() {
    // Version 1 is no longer read (ADR APP-DOCUMENT).
    let mut document = FantasyDocument::empty();
    document.version = 1;
    assert_eq!(document.to_character(), Err(Error::UnsupportedVersion(1)));

    let mut document = FantasyDocument::empty();
    document.profil = Some("cof.profil.paladin".into());
    assert_eq!(
        document.to_character(),
        Err(Error::UnknownContent {
            kind: "profil",
            id: "cof.profil.paladin".into()
        })
    );

    let mut document = ContemporainDocument::empty();
    document.items = vec![ItemEntry::Other {
        name: " ".into(),
        quantity: 1,
    }];
    assert_eq!(document.to_character(), Err(Error::Empty("item name")));

    let mut document = ContemporainDocument::empty();
    document.adjustments = vec![AdjustmentEntry {
        target: Target::Characteristic(Characteristic::For),
        amount: 1,
        author: "gm".into(),
        reason: String::new(),
    }];
    assert_eq!(document.to_character(), Err(Error::Empty("reason")));

    let mut document = ContemporainDocument::empty();
    document.adjustments = vec![AdjustmentEntry {
        target: Target::Value("coc.unknown".into()),
        amount: 1,
        author: "gm".into(),
        reason: "test".into(),
    }];
    assert!(matches!(
        document.to_character(),
        Err(Error::UnknownContent { kind: "value", .. })
    ));
}

#[test]
fn json_names_follow_the_typescript_conventions() {
    let json = serde_json::to_value(fixture("lhagva")).unwrap();
    assert_eq!(json["engine"], "fantasy");
    assert_eq!(json["seriesValues"]["FOR"], 3);
    assert_eq!(json["peupleBonus"], "PER");
    assert_eq!(json["items"][3]["kind"], "other");
    let subject = serde_json::to_value(Subject::Characteristic(Characteristic::Agi)).unwrap();
    assert_eq!(
        subject,
        serde_json::json!({"kind": "characteristic", "id": "AGI"})
    );
    assert!(serde_json::from_str::<Document>(r#"{"engine": "fantasy"}"#).is_err());
}

#[test]
fn equipment_in_play_converts_and_says_why_it_cannot() {
    let equipment: Equipment = serde_json::from_str(
        r#"{"engine": "fantasy", "armour": "cof.armour.cuir-renforce",
            "items": [{"kind": "weapon", "id": "cof.weapon.arc-long", "quantity": 1}]}"#,
    )
    .unwrap();
    let Equipment::Fantasy(equipment) = equipment else {
        panic!("fantasy equipment expected");
    };
    let played = fantasy(&fixture("lhagva")).in_play(equipment.to_equipment().unwrap());
    let defence = played.value(co_fantasy::Value::Defence).unwrap();
    assert!(
        defence
            .contributions()
            .iter()
            .any(|c| c.source() == &co_kernel::Source::Value(co_fantasy::Armour::CuirRenforce.id()))
    );

    let unknown = FantasyEquipment {
        shield: Some("cof.shield.pavois".into()),
        ..FantasyEquipment::default()
    };
    assert_eq!(
        unknown.to_equipment(),
        Err(Error::UnknownContent {
            kind: "shield",
            id: "cof.shield.pavois".into()
        })
    );
    let blank = ContemporainEquipment {
        items: vec![ItemEntry::Other {
            name: String::new(),
            quantity: 1,
        }],
        ..ContemporainEquipment::default()
    };
    assert_eq!(blank.to_equipment(), Err(Error::Empty("item name")));
}

#[test]
fn level_ups_and_options_convert() {
    let Document::Fantasy(mut document) = fixture("lhagva") else {
        panic!("fantasy document expected");
    };
    document.level_ups = vec![LevelUpEntry {
        abilities: vec!["cof.pourfendeur.2".into(), "cof.rage.2".into()],
        forgotten: Vec::new(),
        orphan: None,
    }];
    document.ability_options = vec![AbilityOptionEntry {
        ability: "cof.archer.1".into(),
        option: "cof.option.thrown".into(),
    }];
    let character = document.to_character().unwrap();
    assert_eq!(character.level().get(), 2);
    let vigour = character.value(co_fantasy::Value::VigourPoints);
    assert_eq!(vigour.map(|v| v.total()), Some(19));
    let expected = fantasy(&fixture("lhagva"))
        .with_level_up(
            co_fantasy::LevelUp::new()
                .with_rank(co_fantasy::Voie::Pourfendeur, 2)
                .with_rank(co_fantasy::Voie::Rage, 2),
        )
        .with_ability_option(
            co_fantasy::Voie::Archer,
            1,
            co_fantasy::AbilityOption::Thrown,
        );
    assert_eq!(character, expected);

    let forgetting = LevelUpEntry {
        abilities: vec!["cof.combat.1".into()],
        forgotten: vec!["cof.rage.2".into()],
        orphan: Some("cof.orphan.luck-point".into()),
    };
    document.level_ups.push(forgetting);
    let character = document.to_character().unwrap();
    assert_eq!(character.level().get(), 3);
}

#[test]
fn experience_and_acquired_sequels_convert() {
    let Document::Contemporain(mut document) = fixture("amelia-steele") else {
        panic!("contemporain document expected");
    };
    document.experience = 6;
    document.xp_spendings = vec![XpSpendingEntry {
        abilities: vec!["coc.survie.2".into()],
        traits: vec![TraitChangeEntry {
            id: "coc.trait.classe-superieure".into(),
            points: Some(1),
        }],
    }];
    document.raised_characteristics = vec![RaisedCharacteristicEntry {
        voie: "coc.crime".into(),
        characteristic: Characteristic::Cha,
    }];
    document.acquired_sequels = vec!["coc.acquired-sequel.diminue-corps".into()];
    let character = document.to_character().unwrap();
    let expected = contemporain(&fixture("amelia-steele"))
        .with_experience(6)
        .with_xp_spending(
            co_contemporain::XpSpending::new()
                .with_rank(co_contemporain::Voie::Survie, 2)
                .with_trait(co_contemporain::Trait::ClasseSuperieure, 1),
        )
        .with_raised_characteristic(co_contemporain::Voie::Crime, C::Cha)
        .with_acquired_sequel(co_contemporain::Sequel::DiminueCorps);
    assert_eq!(character, expected);

    // A trait without points is given up.
    document.xp_spendings[0].traits[0].points = None;
    let given_up = document.to_character().unwrap();
    assert_ne!(given_up, expected);
}

#[test]
fn unknown_progression_identifiers_are_unreadable() {
    let kind = |result: Result<_, Error>| match result {
        Err(Error::UnknownContent { kind, .. }) => kind,
        other => panic!("unknown content expected, got {other:?}"),
    };
    let mut document = FantasyDocument::empty();
    document.level_ups = vec![LevelUpEntry {
        abilities: vec!["cof.rage.x".into()],
        ..LevelUpEntry::default()
    }];
    assert_eq!(kind(document.to_character().map(|_| ())), "ability");
    document.level_ups = vec![LevelUpEntry {
        orphan: Some("cof.orphan.gold".into()),
        ..LevelUpEntry::default()
    }];
    assert_eq!(kind(document.to_character().map(|_| ())), "orphan exchange");
    document.level_ups = Vec::new();
    document.ability_options = vec![AbilityOptionEntry {
        ability: "cof.archer.1".into(),
        option: "cof.option.unknown".into(),
    }];
    assert_eq!(kind(document.to_character().map(|_| ())), "ability option");

    let mut document = ContemporainDocument::empty();
    document.acquired_sequels = vec!["coc.acquired-sequel.unknown".into()];
    assert_eq!(kind(document.to_character().map(|_| ())), "sequel");
    document.acquired_sequels = Vec::new();
    document.xp_spendings = vec![XpSpendingEntry {
        abilities: vec!["coc.unknown.1".into()],
        traits: Vec::new(),
    }];
    assert_eq!(kind(document.to_character().map(|_| ())), "ability");
}

#[test]
fn progression_json_names_follow_the_typescript_conventions() {
    let mut document = FantasyDocument::empty();
    document.level_ups = vec![LevelUpEntry {
        abilities: vec!["cof.rage.2".into()],
        ..LevelUpEntry::default()
    }];
    let json = serde_json::to_value(Document::Fantasy(document)).unwrap();
    assert_eq!(json["version"], 2);
    assert_eq!(json["levelUps"][0]["abilities"][0], "cof.rage.2");
    let json = serde_json::to_value(Document::Contemporain(ContemporainDocument::empty())).unwrap();
    assert_eq!(json["xpSpendings"], serde_json::json!([]));
    assert_eq!(json["acquiredSequels"], serde_json::json!([]));
}

#[test]
fn fantasy_adjustments_peuple_malus_and_waivers_convert() {
    let waiver = |subject: Subject| WaiverEntry {
        rule: "COF.CNT.VOIE".into(),
        subject,
        author: "gm".into(),
        reason: "house rule".into(),
    };
    let adjustment = |target: Target| AdjustmentEntry {
        target,
        amount: 1,
        author: "gm".into(),
        reason: "blessing".into(),
    };
    let mut document = FantasyDocument::empty();
    document.peuple_bonus = Some(Characteristic::Vol);
    document.peuple_malus = Some(Characteristic::Con);
    document.adjustments = vec![
        adjustment(Target::Characteristic(Characteristic::Per)),
        adjustment(Target::Value("cof.defence".into())),
    ];
    document.waivers = vec![
        waiver(Subject::Ability("cof.rage.2".into())),
        waiver(Subject::Voie("cof.rage".into())),
        waiver(Subject::Characteristic(Characteristic::Int)),
    ];
    let blessing = || Adjustment::new(1, Author::new("gm"), "blessing");
    let house_rule = |subject| {
        Waiver::new(
            RuleId::new("COF.CNT.VOIE"),
            subject,
            Author::new("gm"),
            "house rule",
        )
    };
    let expected = co_fantasy::Character::new()
        .with_peuple_bonus(C::Vol)
        .with_peuple_malus(C::Con)
        .with_adjustment(co_fantasy::Target::Characteristic(C::Per), blessing())
        .with_adjustment(
            co_fantasy::Target::Value(co_fantasy::Value::Defence),
            blessing(),
        )
        .with_waiver(house_rule(KernelSubject::Ability(
            co_kernel::AbilityId::new("cof.rage.2"),
        )))
        .with_waiver(house_rule(KernelSubject::Voie(co_kernel::VoieId::new(
            "cof.rage",
        ))))
        .with_waiver(house_rule(KernelSubject::Characteristic(C::Int)));
    assert_eq!(document.to_character(), Ok(expected));
}

#[test]
fn every_characteristic_and_subject_converts_both_ways() {
    let characteristics = [
        (Characteristic::Agi, C::Agi),
        (Characteristic::Con, C::Con),
        (Characteristic::For, C::For),
        (Characteristic::Per, C::Per),
        (Characteristic::Cha, C::Cha),
        (Characteristic::Int, C::Int),
        (Characteristic::Vol, C::Vol),
    ];
    for (document, kernel) in characteristics {
        assert_eq!(C::from(document), kernel);
        assert_eq!(Characteristic::from(kernel), document);
    }
    let subjects = [
        (Subject::Character, KernelSubject::Character),
        (
            Subject::Characteristic(Characteristic::Cha),
            KernelSubject::Characteristic(C::Cha),
        ),
        (
            Subject::Voie("cof.rage".into()),
            KernelSubject::Voie(co_kernel::VoieId::new("cof.rage")),
        ),
        (
            Subject::Ability("cof.rage.2".into()),
            KernelSubject::Ability(co_kernel::AbilityId::new("cof.rage.2")),
        ),
        (
            Subject::Value("cof.defence".into()),
            KernelSubject::Value(co_kernel::ValueId::new("cof.defence")),
        ),
    ];
    for (document, kernel) in subjects {
        assert_eq!(Subject::from(&kernel), document);
    }
}
