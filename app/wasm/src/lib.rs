//! The engines, compiled to WebAssembly for the front end.
//!
//! Each binding is a stateless function over plain data: the front end keeps
//! the choices document, calls the engine and renders what it returns. No
//! Rust object crosses the boundary, so the JavaScript side never frees
//! memory by hand. The types crossing it are serialized with `serde`,
//! described to TypeScript by `tsify` and passed as [`Ts`], which turns a
//! value the front end got wrong into a thrown error instead of a panic.
//!
//! The rules run here, locally (ADR APP-LOCAL-RULES): [`evaluate`]
//! recomputes every value and violation of a document on each call, in
//! microseconds.

mod catalog;
mod sheet;

pub use catalog::{Catalog, catalog};
use palimpseste_document::{Document, Engine, Equipment};
pub use sheet::{Sheet, evaluate, evaluate_in_play};
use tsify::{Ts, Tsify};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

/// The choices document of a character of `engine` with no choice made.
///
/// # Errors
///
/// Throws when `engine` is not an engine name.
#[wasm_bindgen(js_name = emptyDocument)]
pub fn empty_document_js(engine: &Ts<Engine>) -> Result<Ts<Document>, JsError> {
    Ok(Document::empty(engine.to_rust()?).into_ts()?)
}

/// The content `engine` offers at creation, with the identifiers the
/// choices document uses.
///
/// # Errors
///
/// Throws when `engine` is not an engine name.
#[wasm_bindgen(js_name = catalog)]
pub fn catalog_js(engine: &Ts<Engine>) -> Result<Ts<Catalog>, JsError> {
    Ok(catalog(engine.to_rust()?).into_ts()?)
}

/// Evaluates a choices document with its engine: values and breakdowns,
/// violations placed in their creation step, step status.
///
/// # Errors
///
/// Throws when `document` is not a document, or cannot be read (unknown
/// identifier, unsupported version, missing author or reason).
#[wasm_bindgen(js_name = evaluate)]
pub fn evaluate_js(document: &Ts<Document>) -> Result<Ts<Sheet>, JsError> {
    let sheet = evaluate(&document.to_rust()?).map_err(|error| JsError::new(&error.to_string()))?;
    Ok(sheet.into_ts()?)
}

/// Evaluates a choices document with the character in play, wearing and
/// carrying `equipment` (ADR ENG-IN-PLAY): the steps and violations of its
/// creation, its values with the equipment in play, and the findings of the
/// rules that hold in play (`inPlay`).
///
/// # Errors
///
/// Throws when `document` or `equipment` cannot be read, or when the
/// equipment is of another engine.
#[wasm_bindgen(js_name = evaluateInPlay)]
pub fn evaluate_in_play_js(
    document: &Ts<Document>,
    equipment: &Ts<Equipment>,
) -> Result<Ts<Sheet>, JsError> {
    let sheet = evaluate_in_play(&document.to_rust()?, &equipment.to_rust()?)
        .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(sheet.into_ts()?)
}

#[cfg(test)]
mod tests {
    use palimpseste_document::{
        AdjustmentEntry, Characteristic, ContemporainDocument, FantasyDocument, ItemEntry,
        LevelUpEntry, Subject, Target, VoieEntry, WaiverEntry, XpSpendingEntry,
    };

    use super::*;
    use crate::sheet::{Category, Details, ReportedValue, Severity, SourceKind};

    fn fantasy(document: FantasyDocument) -> Sheet {
        evaluate(&Document::Fantasy(document)).unwrap()
    }

    fn contemporain(document: ContemporainDocument) -> Sheet {
        evaluate(&Document::Contemporain(document)).unwrap()
    }

    fn value<'a>(sheet: &'a Sheet, id: &str) -> &'a sheet::ValueLine {
        sheet.values.iter().find(|line| line.id == id).unwrap()
    }

    #[test]
    fn new_characters_of_each_engine_are_incomplete_in_their_first_step() {
        for engine in [Engine::Contemporain, Engine::Fantasy] {
            let sheet = evaluate(&Document::empty(engine)).unwrap();
            assert!(sheet.steps[0].incomplete > 0, "{engine:?}");
            assert!(
                sheet
                    .violations
                    .iter()
                    .all(|violation| violation.category == Category::Incomplete
                        || violation.category == Category::Uncomputable),
                "{engine:?}"
            );
            assert_eq!(sheet.characteristics.len(), 7);
        }
    }

    #[test]
    fn steps_follow_the_book_order() {
        let ids = |engine| {
            evaluate(&Document::empty(engine))
                .unwrap()
                .steps
                .into_iter()
                .map(|step| step.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids(Engine::Fantasy),
            [
                "profil",
                "peuple",
                "characteristics",
                "voies",
                "equipment",
                "derived"
            ]
        );
        assert_eq!(ids(Engine::Contemporain)[0], "domaine-and-age");
    }

    #[test]
    fn values_carry_their_breakdown_and_labels() {
        let mut document = FantasyDocument::empty();
        document.profil = Some("cof.profil.guerrier".into());
        document.peuple = Some("cof.peuple.nain".into());
        document.series_values.con = Some(2);
        document.peuple_bonus = Some(Characteristic::Con);
        document.armour = Some("cof.armour.chemise-de-mailles".into());
        document.items = vec![ItemEntry::Weapon {
            id: "cof.weapon.epee-longue".into(),
            quantity: 1,
        }];
        let sheet = fantasy(document);

        let vigour = value(&sheet, "cof.vigour-points");
        assert_eq!(vigour.total, Some(13));
        assert_eq!(vigour.contributions[0].label, "Combattants");
        assert_eq!(vigour.contributions[1].source, SourceKind::Characteristic);

        let defence = value(&sheet, "cof.defence");
        assert!(
            defence
                .contributions
                .iter()
                .any(|c| c.label == "Chemise de mailles" && c.amount == 4)
        );
        let damage = value(&sheet, "cof.damage.epee-longue.one-handed");
        assert_eq!(damage.dice.as_deref(), Some("1d8"));
        assert_eq!(damage.label, "DM Épée longue (à une main)");
        assert_eq!(
            value(&sheet, "cof.recovery-dice").dice.as_deref(),
            Some("d10")
        );

        let con = &sheet.characteristics[1];
        assert_eq!(con.characteristic, Characteristic::Con);
        assert_eq!(con.total, 3);
        assert_eq!(con.contributions[1].label, "Nain");
        match sheet.details {
            Details::Fantasy(details) => {
                assert_eq!(details.peuple_slot_options, ["cof.nain"]);
                assert_eq!(details.recovery_die.as_deref(), Some("d10"));
            }
            Details::Contemporain(_) => panic!("fantasy details expected"),
        }
    }

    #[test]
    fn violations_carry_step_subject_and_parameters() {
        let mut document = FantasyDocument::empty();
        document.profil = Some("cof.profil.guerrier".into());
        document.voies = vec![VoieEntry {
            id: "cof.combat".into(),
            ranks: vec![1, 2],
        }];
        let sheet = fantasy(document);
        let level = sheet
            .violations
            .iter()
            .find(|v| v.rule == "COF.PRG.LEVEL-REQ")
            .unwrap();
        assert_eq!(level.step, "voies");
        assert_eq!(level.subject, Subject::Voie("cof.combat".into()));
        assert_eq!(level.subject_label.as_deref(), Some("Voie du combat"));
        assert_eq!(
            level.category,
            Category::Deviation {
                severity: Severity::Standard
            }
        );
        assert!(
            level
                .parameters
                .iter()
                .any(|p| p.name == "rank" && p.value == ReportedValue::Integer(2))
        );
        let voies = sheet.steps.iter().find(|s| s.id == "voies").unwrap();
        assert!(voies.deviations >= 1);
    }

    #[test]
    fn waivers_close_deviations_and_unused_ones_are_reported() {
        let waiver = |subject: &str| WaiverEntry {
            rule: "COC.CRE.EQUIPMENT".into(),
            subject: Subject::Value(subject.into()),
            author: "gm".into(),
            reason: "biker".into(),
        };
        let mut document = ContemporainDocument::empty();
        document.protections = vec!["coc.protection.casque".into()];
        document.waivers = vec![
            waiver("coc.protection.casque"),
            waiver("coc.protection.gilet-souple"),
        ];
        let sheet = contemporain(document);
        let equipment = sheet
            .violations
            .iter()
            .find(|v| v.rule == "COC.CRE.EQUIPMENT")
            .unwrap();
        assert!(equipment.waived);
        assert_eq!(equipment.subject_label.as_deref(), Some("Casque"));
        assert_eq!(
            sheet.unused_waivers,
            [waiver("coc.protection.gilet-souple")]
        );
        let step = sheet.steps.iter().find(|s| s.id == "equipment").unwrap();
        assert_eq!(step.deviations, 0);
    }

    #[test]
    fn adjustments_show_in_breakdowns_and_steps() {
        let mut document = ContemporainDocument::empty();
        document.age = Some(20);
        document.adjustments = vec![AdjustmentEntry {
            target: Target::Value("coc.carac-points".into()),
            amount: 2,
            author: "gm".into(),
            reason: "veteran".into(),
        }];
        let sheet = contemporain(document);
        let points = value(&sheet, "coc.carac-points");
        assert_eq!(points.total, Some(12));
        assert_eq!(points.contributions[0].label, "Tranche d'âge");
        assert_eq!(points.contributions[1].label, "Ajustement de gm : veteran");
        let step = sheet
            .steps
            .iter()
            .find(|s| s.id == "characteristics")
            .unwrap();
        assert_eq!(step.notices, 1);
        assert_eq!(
            value(&sheet, "coc.damage.mains-nues").dice.as_deref(),
            Some("1d4")
        );
    }

    #[test]
    fn every_subject_of_a_new_character_has_a_label() {
        for engine in [Engine::Contemporain, Engine::Fantasy] {
            let sheet = evaluate(&Document::empty(engine)).unwrap();
            for violation in sheet.violations {
                if violation.subject != Subject::Character {
                    assert!(violation.subject_label.is_some(), "{violation:?}");
                }
            }
        }
    }

    #[test]
    fn unreadable_documents_are_errors() {
        let mut document = FantasyDocument::empty();
        document.profil = Some("cof.profil.paladin".into());
        assert!(evaluate(&Document::Fantasy(document)).is_err());
    }

    #[test]
    fn in_play_values_follow_the_equipment_and_creation_stays() {
        let mut document = FantasyDocument::empty();
        document.profil = Some("cof.profil.barbare".into());
        document.armour = Some("cof.armour.cuir-simple".into());
        document.items = vec![ItemEntry::Weapon {
            id: "cof.weapon.epee-longue".into(),
            quantity: 1,
        }];
        let equipment = palimpseste_document::FantasyEquipment {
            armour: Some("cof.armour.cotte-de-mailles".into()),
            shield: None,
            items: vec![ItemEntry::Weapon {
                id: "cof.weapon.arc-long".into(),
                quantity: 1,
            }],
        };
        let creation = fantasy(document.clone());
        let sheet =
            evaluate_in_play(&Document::Fantasy(document), &Equipment::Fantasy(equipment)).unwrap();

        let defence = value(&sheet, "cof.defence");
        assert!(
            defence
                .contributions
                .iter()
                .any(|c| c.label == "Cotte de mailles")
        );
        assert!(
            sheet
                .values
                .iter()
                .any(|l| l.id == "cof.damage.arc-long.ranged")
        );
        assert!(
            sheet
                .values
                .iter()
                .all(|l| !l.id.starts_with("cof.damage.epee-longue"))
        );
        // The steps and violations are the creation's: the arc long added in
        // play is no starting equipment matter.
        assert_eq!(sheet.violations, creation.violations);
        assert_eq!(sheet.steps, creation.steps);
        let rules: Vec<_> = sheet.in_play.iter().map(|v| v.rule.as_str()).collect();
        assert!(rules.contains(&"COF.EQP.RANGED"));
        assert!(rules.contains(&"COF.DER.ARMOUR-LIMIT"));
        assert!(!rules.contains(&"COF.CRE.EQUIPMENT"));
        assert_eq!(creation.in_play, []);
    }

    #[test]
    fn in_play_equipment_of_another_engine_is_an_error() {
        let equipment = Equipment::Contemporain(palimpseste_document::ContemporainEquipment {
            protections: Vec::new(),
            items: Vec::new(),
        });
        assert_eq!(
            evaluate_in_play(&Document::empty(Engine::Fantasy), &equipment),
            Err(palimpseste_document::Error::OtherEngine)
        );
    }

    #[test]
    fn catalogs_list_the_content() {
        match catalog(Engine::Fantasy) {
            Catalog::Fantasy(catalog) => {
                assert_eq!(catalog.profils.len(), 14);
                assert_eq!(catalog.voies.len(), 78);
                assert_eq!(catalog.purse_dice, "2d6");
                let guerrier = catalog
                    .profils
                    .iter()
                    .find(|p| p.id == "cof.profil.guerrier")
                    .unwrap();
                assert_eq!(guerrier.family, "cof.family.combattants");
                assert_eq!(guerrier.voies.len(), 5);
                assert_eq!(guerrier.starting_equipment.len(), 1);
            }
            Catalog::Contemporain(_) => panic!("fantasy catalog expected"),
        }
        match catalog(Engine::Contemporain) {
            Catalog::Contemporain(catalog) => {
                assert_eq!(catalog.domaines.len(), 18);
                assert_eq!(catalog.voies.len(), 18);
                assert_eq!(catalog.traits.len(), 32);
                assert_eq!(catalog.health_scale.steps, 20);
                let blesse = &catalog.health_scale.states[2];
                assert_eq!(
                    (blesse.name.as_str(), blesse.step, blesse.malus),
                    ("Blessé", 15, -5)
                );
            }
            Catalog::Fantasy(_) => panic!("contemporain catalog expected"),
        }
    }

    /// A humain guerrier at level 1, with a level-up acquiring `abilities`.
    fn levelled(abilities: &[&str]) -> FantasyDocument {
        let mut document = FantasyDocument::empty();
        document.profil = Some("cof.profil.guerrier".into());
        document.peuple = Some("cof.peuple.humain".into());
        document.voies = ["cof.bouclier", "cof.soldat", "cof.humain"]
            .into_iter()
            .map(|id| VoieEntry {
                id: id.into(),
                ranks: vec![1],
            })
            .collect();
        document.level_ups = vec![LevelUpEntry {
            abilities: abilities.iter().map(|id| (*id).to_owned()).collect(),
            ..LevelUpEntry::default()
        }];
        document
    }

    #[test]
    fn level_ups_have_their_status_and_offer() {
        let sheet = fantasy(levelled(&["cof.bouclier.2"]));
        assert_eq!(sheet.progression.len(), 1);
        let level = &sheet.progression[0];
        assert_eq!((level.number, level.name.as_str()), (2, "Niveau 2"));
        let ids: Vec<_> = level.steps.iter().map(|step| step.id.as_str()).collect();
        assert_eq!(ids, ["abilities", "options", "orientation", "orphan"]);
        // One point left: incomplete in the level's capacités, never in a
        // creation step.
        assert_eq!(level.steps[0].incomplete, 1);
        let points = sheet
            .violations
            .iter()
            .find(|v| v.rule == "COF.PRG.POINTS")
            .unwrap();
        assert_eq!(
            (points.progression, points.step.as_str()),
            (Some(2), "abilities")
        );
        let voies = sheet.steps.iter().find(|s| s.id == "voies").unwrap();
        assert_eq!(voies.incomplete + voies.deviations, 0);

        let Details::Fantasy(details) = sheet.details else {
            panic!("fantasy details expected");
        };
        assert_eq!((details.level, details.evolutive_die.as_str()), (2, "d4"));
        let offer = &details.levels[0];
        assert_eq!((offer.points_available, offer.points_spent), (2, 1));
        assert!(
            offer
                .acquirable
                .iter()
                .any(|a| a.ability == "cof.soldat.2" && a.cost == 1)
        );
        assert!(
            details
                .abilities_held
                .iter()
                .any(|held| held.ability == "cof.bouclier.2" && held.progression == Some(2))
        );
        assert!(
            details
                .abilities_held
                .iter()
                .any(|held| held.ability == "cof.bouclier.1" && held.progression.is_none())
        );
    }

    #[test]
    fn capacites_are_named_by_identifier() {
        let Catalog::Fantasy(content) = catalog(Engine::Fantasy) else {
            panic!("fantasy catalog expected");
        };
        let rage = content
            .abilities
            .iter()
            .find(|entry| entry.id == "cof.rage.2")
            .unwrap();
        assert_eq!(
            (rage.voie.as_str(), rage.rank, rage.label.as_str()),
            ("cof.rage", 2, "Voie de la rage, rang 2")
        );
        assert_eq!(content.abilities.len(), content.voies.len() * 5);
        assert!(
            content
                .ability_options
                .iter()
                .any(|o| o.id == "cof.option.thrown")
        );
        assert_eq!(content.orphan_exchanges.len(), 4);
        // A breakdown names a capacité source by its identifier.
        let mut document = levelled(&["cof.soldat.2", "cof.bouclier.2"]);
        document.voies.push(VoieEntry {
            id: "cof.pourfendeur".into(),
            ranks: vec![1],
        });
        let sheet = fantasy(document);
        let initiative = value(&sheet, "cof.initiative");
        assert!(
            initiative
                .contributions
                .iter()
                .any(|c| c.source_id.as_deref() == Some("cof.pourfendeur.1"))
        );
        let Catalog::Contemporain(content) = catalog(Engine::Contemporain) else {
            panic!("contemporain catalog expected");
        };
        assert_eq!(content.abilities.len(), 90);
        assert_eq!(content.sequels.len(), 3);
        let crime = content.voies.iter().find(|v| v.id == "coc.crime").unwrap();
        assert_eq!(
            crime.rank_five_choice,
            [Characteristic::Per, Characteristic::Cha]
        );
    }

    #[test]
    fn spendings_have_their_status_and_offer() {
        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.crime".into(),
            ranks: vec![1],
        }];
        document.experience = 1;
        document.xp_spendings = vec![XpSpendingEntry {
            abilities: vec!["coc.crime.2".into()],
            traits: Vec::new(),
        }];
        let sheet = contemporain(document);
        assert_eq!(sheet.progression[0].name, "Dépense d'XP 1");
        // Crime 2 costs 2 XP of the 1 earned.
        let cost = sheet
            .violations
            .iter()
            .find(|v| v.rule == "COC.PRG.COST")
            .unwrap();
        assert_eq!((cost.progression, cost.step.as_str()), (Some(1), "ranks"));
        assert_eq!(sheet.progression[0].steps[0].deviations, 1);
        assert_eq!(value(&sheet, "coc.experience").total, Some(-1));
        let Details::Contemporain(details) = sheet.details else {
            panic!("contemporain details expected");
        };
        assert_eq!(details.spendings[0].xp_left, -1);
        assert_eq!(details.spendings[0].acquirable, []);
        assert!(
            details
                .abilities_held
                .iter()
                .any(|held| held.ability == "coc.crime.2" && held.progression == Some(1))
        );
    }
}
