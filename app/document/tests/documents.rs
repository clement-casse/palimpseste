//! Documents convert to the engines' characters: the book's worked
//! characters, empty documents, and documents that cannot be read.

use co_kernel::{
    Adjustment, Author, Characteristic as C, RuleId, Subject as KernelSubject, Waiver,
};
use palimpseste_document::{
    AdjustmentEntry, Characteristic, ContemporainDocument, Document, DocumentError, Engine,
    FantasyDocument, ItemEntry, Subject, Target, WaiverEntry,
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
    let mut document = FantasyDocument::empty();
    document.version = 2;
    assert_eq!(
        document.to_character(),
        Err(DocumentError::UnsupportedVersion(2))
    );

    let mut document = FantasyDocument::empty();
    document.profil = Some("cof.profil.paladin".into());
    assert_eq!(
        document.to_character(),
        Err(DocumentError::UnknownContent {
            kind: "profil",
            id: "cof.profil.paladin".into()
        })
    );

    let mut document = ContemporainDocument::empty();
    document.items = vec![ItemEntry::Other {
        name: " ".into(),
        quantity: 1,
    }];
    assert_eq!(
        document.to_character(),
        Err(DocumentError::Empty("item name"))
    );

    let mut document = ContemporainDocument::empty();
    document.adjustments = vec![AdjustmentEntry {
        target: Target::Characteristic(Characteristic::For),
        amount: 1,
        author: "gm".into(),
        reason: String::new(),
    }];
    assert_eq!(document.to_character(), Err(DocumentError::Empty("reason")));

    let mut document = ContemporainDocument::empty();
    document.adjustments = vec![AdjustmentEntry {
        target: Target::Value("coc.unknown".into()),
        amount: 1,
        author: "gm".into(),
        reason: "test".into(),
    }];
    assert!(matches!(
        document.to_character(),
        Err(DocumentError::UnknownContent { kind: "value", .. })
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
