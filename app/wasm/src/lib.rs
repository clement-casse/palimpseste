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
mod roll;
mod sheet;

pub use catalog::{Catalog, catalog};
use palimpseste_document::{Document, Engine, Equipment};
pub use roll::{
    Mode, NaturalRoll, Play, RollBonusDie, RollKind, RollLine, RollLineKind, RollNote, RollOffer,
    RollOptionLine, RollOutcome, RollRequest, RolledDie, roll, roll_offer,
};
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

/// What the roll `roll` offers the character of `document` in play (ADR
/// ENG-ROLLS): its base, the options to pick, the dice.
///
/// # Errors
///
/// Throws when `document` or `play` cannot be read, when `roll` names no
/// roll, or when the equipment is of another engine.
#[wasm_bindgen(js_name = rollOffer)]
pub fn roll_offer_js(
    document: &Ts<Document>,
    play: &Ts<Play>,
    roll: &str,
) -> Result<Ts<RollOffer>, JsError> {
    let offer = roll_offer(&document.to_rust()?, &play.to_rust()?, roll)
        .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(offer.into_ts()?)
}

#[wasm_bindgen]
extern "C" {
    /// Fills `bytes` with the browser's random values.
    #[wasm_bindgen(js_namespace = crypto, js_name = getRandomValues)]
    fn random_values(bytes: &mut [u8]);
}

/// Rolls `request` for the character of `document` in play: the faces are
/// drawn here, from the browser's random values, and the engine resolves
/// the roll from them (ADR APP-ROLLS).
///
/// # Errors
///
/// Throws when `document`, `play` or `request` cannot be read, when the
/// request names no roll or an option not offered, or when the equipment
/// is of another engine.
#[wasm_bindgen(js_name = roll)]
pub fn roll_js(
    document: &Ts<Document>,
    play: &Ts<Play>,
    request: &Ts<RollRequest>,
) -> Result<Ts<RollOutcome>, JsError> {
    let mut random = || {
        let mut byte = [0_u8];
        random_values(&mut byte);
        byte[0]
    };
    let outcome = roll(
        &document.to_rust()?,
        &play.to_rust()?,
        &request.to_rust()?,
        &mut random,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(outcome.into_ts()?)
}

#[cfg(test)]
mod tests {
    use palimpseste_document::{
        AbilityOptionEntry, AdjustmentEntry, Characteristic, ContemporainDocument, FantasyDocument,
        ItemEntry, LevelUpEntry, SpecialityEntry, Subject, Target, VoieEntry, WaiverEntry,
        XpSpendingEntry,
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
        document.specialities = vec![SpecialityEntry {
            ability: "cof.humain.1".into(),
            speciality: "citadin".into(),
        }];
        document.level_ups = vec![LevelUpEntry {
            abilities: abilities.iter().map(|id| (*id).to_owned()).collect(),
            ..LevelUpEntry::default()
        }];
        document
    }

    #[test]
    fn levels_gained_show_in_the_vigour_breakdown() {
        let sheet = fantasy(levelled(&[]));
        let vigour = value(&sheet, "cof.vigour-points");
        // Level 2, CON 0: 2 x 5 (combattants) + CON, then 5 + CON.
        let gained = vigour.contributions.last().unwrap();
        assert_eq!(
            (gained.label.as_str(), gained.amount),
            ("Niveaux gagnés", 5)
        );
        assert_eq!(vigour.total, Some(15));
    }

    #[test]
    fn capacites_held_carry_their_test_bonuses_and_weapons_their_attack() {
        let mut document = levelled(&[]);
        document.voies.push(VoieEntry {
            id: "cof.brute".into(),
            ranks: vec![1, 2, 3, 4],
        });
        document.items = vec![ItemEntry::Weapon {
            id: "cof.weapon.hache".into(),
            quantity: 1,
        }];
        let sheet = fantasy(document);
        let Details::Fantasy(details) = &sheet.details else {
            panic!("fantasy details expected")
        };
        let held = |id: &str| {
            details
                .abilities_held
                .iter()
                .find(|h| h.ability == id)
                .unwrap()
        };
        let shield = held("cof.bouclier.1");
        assert_eq!(shield.test_bonuses.len(), 1);
        assert_eq!(shield.test_bonuses[0].value.total, Some(3));
        assert_eq!(shield.bonus_die, None);
        let heroic = held("cof.brute.4").bonus_die.as_ref().unwrap();
        assert_eq!(heroic.characteristics, [Characteristic::For]);
        assert_eq!(
            (heroic.scope.as_str(), heroic.on_speciality),
            ("tous les tests", false)
        );
        assert_eq!(shield.test_bonuses[0].scope, "éviter la surprise");
        assert!(value(&sheet, "cof.attack.hache.one-handed").total.is_some());
        assert_eq!(
            value(&sheet, "cof.damage.hache.one-handed").dice.as_deref(),
            Some("1d8")
        );

        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.crime".into(),
            ranks: vec![1],
        }];
        let sheet = contemporain(document);
        let Details::Contemporain(details) = &sheet.details else {
            panic!("contemporain details expected")
        };
        let pickpocket = &details.abilities_held[0];
        assert_eq!(pickpocket.test_bonuses[0].value.total, Some(5));
        assert_eq!(
            pickpocket.test_bonuses[0].characteristics,
            [Characteristic::Agi, Characteristic::Cha]
        );
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
            (
                rage.voie.as_str(),
                rage.rank,
                rage.name.as_str(),
                rage.label.as_str()
            ),
            (
                "cof.rage",
                2,
                "Défier la mort",
                "Défier la mort (Voie de la rage, rang 2)"
            )
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

    #[test]
    fn contemporain_in_play_values_follow_the_equipment() {
        let mut document = ContemporainDocument::empty();
        document.items = vec![
            ItemEntry::Weapon {
                id: "coc.weapon.couteau".into(),
                quantity: 1,
            },
            ItemEntry::Other {
                name: "Lampe".into(),
                quantity: 1,
            },
        ];
        let equipment = palimpseste_document::ContemporainEquipment {
            protections: vec!["coc.protection.gilet-souple".into()],
            items: vec![ItemEntry::Weapon {
                id: "coc.weapon.couteau".into(),
                quantity: 0,
            }],
        };
        let creation = contemporain(document.clone());
        let sheet = evaluate_in_play(
            &Document::Contemporain(document),
            &Equipment::Contemporain(equipment),
        )
        .unwrap();
        let reduction = |sheet: &Sheet| value(sheet, "coc.damage-reduction").total;
        assert!(reduction(&sheet) > reduction(&creation));
        assert!(creation.values.iter().any(|l| l.id == "coc.damage.couteau"));
        // A weapon put down in play is no longer listed; bare hands stay.
        assert!(sheet.values.iter().all(|l| l.id != "coc.damage.couteau"));
        assert!(sheet.values.iter().any(|l| l.id == "coc.damage.mains-nues"));
        assert_eq!(sheet.violations, creation.violations);
        assert_eq!(sheet.in_play, []);
    }

    #[test]
    fn every_category_and_subject_kind_is_reported() {
        let mut document = ContemporainDocument::empty();
        document.age = Some(8);
        document.carac_points.agi = -1;
        document.carac_points.con = -1;
        document.carac_points.r#for = 10;
        let sheet = contemporain(document);
        let find = |rule: &str| sheet.violations.iter().find(|v| v.rule == rule).unwrap();
        let bracket = find("COC.CRE.AGE-BRACKET");
        assert_eq!(bracket.category, Category::Uncomputable);
        let limits = find("COC.CRE.CARAC-LIMITS");
        assert_eq!(
            (&limits.subject, limits.subject_label.as_deref()),
            (&Subject::Characteristic(Characteristic::For), Some("FOR"))
        );
        assert_eq!(
            limits.category,
            Category::Deviation {
                severity: Severity::Absolute
            }
        );
        let lowered = find("COC.CRE.CARAC-LOWERED");
        assert_eq!(
            (&lowered.subject, &lowered.subject_label),
            (&Subject::Character, &None)
        );
        assert_eq!(
            lowered.category,
            Category::Deviation {
                severity: Severity::Recommended
            }
        );

        // A capacité whose option is required, without one, is named.
        let mut document = FantasyDocument::empty();
        document.voies = vec![VoieEntry {
            id: "cof.combat-a-deux-armes".into(),
            ranks: vec![1, 2, 3, 4],
        }];
        let sheet = fantasy(document);
        let option = sheet
            .violations
            .iter()
            .find(|v| v.rule == "COF.CNT.VOIE")
            .unwrap();
        assert_eq!(
            (&option.subject, option.subject_label.as_deref()),
            (
                &Subject::Ability("cof.combat-a-deux-armes.4".into()),
                Some("Combattant héroïque (Voie du combat à deux armes, rang 4)")
            )
        );
    }

    #[test]
    fn capacites_offering_options_are_listed() {
        let mut document = FantasyDocument::empty();
        document.voies = vec![VoieEntry {
            id: "cof.archer".into(),
            ranks: vec![1],
        }];
        let Details::Fantasy(details) = fantasy(document).details else {
            panic!("fantasy details expected");
        };
        assert_eq!(
            details.ability_options,
            [sheet::AbilityOptionLine {
                ability: "cof.archer.1".into(),
                offered: vec!["cof.option.thrown".into()],
                required: false,
                progression: None,
            }]
        );
    }

    #[test]
    fn capacites_carry_their_actions_spell_costs_and_specialities() {
        let Catalog::Fantasy(fantasy) = catalog(Engine::Fantasy) else {
            panic!("fantasy catalog expected");
        };
        let entry = |id: &str| fantasy.abilities.iter().find(|a| a.id == id).unwrap();
        let lightning = entry("cof.air.4");
        assert_eq!(lightning.spell_cost, Some(4));
        assert_eq!(
            lightning
                .actions
                .iter()
                .map(|a| (a.abbreviation.as_str(), a.name.as_str()))
                .collect::<Vec<_>>(),
            [("A", "Action d'attaque")]
        );
        let tunnels = &entry("cof.nain.1").effects;
        assert_eq!(
            tunnels
                .iter()
                .map(|e| (e.text.as_str(), e.computed))
                .collect::<Vec<_>>(),
            [
                ("vision dans le noir 30 m", false),
                (
                    "+3 pierre, architecture, mines, passages secrets, pièges muraux",
                    false
                )
            ]
        );
        let links: Vec<_> = tunnels.iter().map(|e| e.bonus.clone()).collect();
        assert_eq!(links, [None, Some(catalog::EffectBonus::Test { index: 1 })]);
        let tattoos = &entry("cof.pagne.3").effects;
        assert_eq!(tattoos[0].bonus, Some(catalog::EffectBonus::AllTests));
        let superior = &entry("cof.mage.4").effects;
        assert_eq!(superior[1].bonus, Some(catalog::EffectBonus::Die));
        let brute = entry("cof.brute.1");
        assert_eq!((brute.actions.len(), brute.spell_cost), (0, None));
        assert_eq!(entry("cof.humain.1").speciality.as_deref(), Some("origine"));
        let Catalog::Contemporain(contemporain) = catalog(Engine::Contemporain) else {
            panic!("contemporain catalog expected");
        };
        let arts = contemporain
            .abilities
            .iter()
            .find(|a| a.id == "coc.arts-et-artisanats.1")
            .unwrap();
        assert_eq!(
            (
                arts.speciality.as_deref(),
                arts.actions.len(),
                arts.spell_cost
            ),
            (Some("discipline"), 0, None)
        );
        let links: Vec<_> = arts
            .effects
            .iter()
            .map(|e| (e.computed, e.bonus.clone()))
            .collect();
        assert_eq!(
            links,
            [(false, Some(catalog::EffectBonus::Test { index: 1 }))]
        );
    }

    #[test]
    fn held_capacites_show_their_speciality_and_its_bonus() {
        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.arts-et-artisanats".into(),
            ranks: vec![1],
        }];
        document.specialities = vec![SpecialityEntry {
            ability: "coc.arts-et-artisanats.1".into(),
            speciality: "Peinture".into(),
        }];
        let Details::Contemporain(details) = contemporain(document).details else {
            panic!("contemporain details expected");
        };
        let [held] = details.abilities_held.as_slice() else {
            panic!("one capacité expected");
        };
        assert_eq!(held.speciality.as_deref(), Some("Peinture"));
        let [bonus] = held.test_bonuses.as_slice() else {
            panic!("one bonus expected");
        };
        assert_eq!(
            (bonus.scope.as_str(), bonus.on_speciality),
            ("culture et pratique de la discipline", true)
        );
    }

    #[test]
    fn a_human_mage_is_asked_the_origin_of_the_diversity_the_voie_du_mage_keeps() {
        let mut document = FantasyDocument::empty();
        document.peuple = Some("cof.peuple.humain".into());
        document.profil = Some("cof.profil.ensorceleur".into());
        document.voies = vec![VoieEntry {
            id: "cof.mage".into(),
            ranks: vec![1],
        }];
        let held = |document: FantasyDocument| {
            let Details::Fantasy(details) = fantasy(document).details else {
                panic!("fantasy details expected");
            };
            details.abilities_held
        };
        let abilities = held(document.clone());
        let ids = abilities
            .iter()
            .map(|h| h.ability.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["cof.humain.1", "cof.mage.1"]);
        let kept = abilities
            .iter()
            .map(|h| h.kept_through.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(kept, [Some("cof.mage.1"), None]);
        assert_eq!(abilities[0].progression, None);
        document.specialities = vec![SpecialityEntry {
            ability: "cof.humain.1".into(),
            speciality: "citadin".into(),
        }];
        assert_eq!(held(document)[0].speciality.as_deref(), Some("citadin"));
    }

    #[test]
    fn values_list_their_situational_modifiers() {
        let mut document = FantasyDocument::empty();
        document.voies = vec![VoieEntry {
            id: "cof.traqueur".into(),
            ranks: vec![1, 2, 3],
        }];
        document.items = vec![ItemEntry::Weapon {
            id: "cof.weapon.epee-longue".into(),
            quantity: 1,
        }];
        let sheet = fantasy(document);
        let damage = value(&sheet, "cof.damage.epee-longue.one-handed");
        let [hunter] = damage.situational.as_slice() else {
            panic!("one situational modifier expected");
        };
        assert_eq!(
            (
                hunter.situation.as_str(),
                hunter.ability.as_str(),
                hunter.amount,
                hunter.dice.as_deref()
            ),
            ("contre un animal", "cof.traqueur.3", None, Some("1d4"))
        );
        assert_eq!(value(&sheet, "cof.defence").situational, []);

        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.crime".into(),
            ranks: vec![1, 2],
        }];
        let sheet = contemporain(document);
        let strength = sheet
            .characteristics
            .iter()
            .find(|line| line.characteristic == Characteristic::For)
            .unwrap();
        let [note] = strength.situational.as_slice() else {
            panic!("one note expected");
        };
        assert_eq!(
            (note.situation.as_str(), note.amount),
            ("à la place du CHA pour contraindre", None)
        );
    }

    #[test]
    fn the_effects_with_a_formula_are_written_for_the_character() {
        let mut document = FantasyDocument::empty();
        document.voies = vec![VoieEntry {
            id: "cof.illusions".into(),
            ranks: vec![1, 2],
        }];
        let sheet = fantasy(document);
        let mirage = sheet
            .written_effects
            .iter()
            .find(|w| w.ability == "cof.illusions.1")
            .unwrap();
        assert!(mirage.effects[0].contains(&sheet::EffectPartLine::Value {
            value: "4 m de côté".into(),
            formula: "2 m de côté par rang".into(),
        }));
        // Only the capacités with a formula are written.
        assert!(
            sheet
                .written_effects
                .iter()
                .all(|w| w.ability != "cof.nain.1")
        );
    }

    #[test]
    fn a_bonus_die_names_the_capacite_it_goes_to() {
        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.vehicules".into(),
            ranks: vec![1, 2, 3, 4, 5],
        }];
        let die = |document: &ContemporainDocument| {
            let Details::Contemporain(details) = contemporain(document.clone()).details else {
                panic!("contemporain details expected");
            };
            let five = details
                .rank_five
                .iter()
                .find(|l| l.voie == "coc.vehicules")
                .cloned();
            let held = details
                .abilities_held
                .into_iter()
                .find(|h| h.ability == "coc.vehicules.5");
            (five.unwrap(), held.and_then(|h| h.bonus_die).unwrap())
        };
        let (five, line) = die(&document);
        assert_eq!(five.bonus_die_choice.len(), 5);
        assert_eq!(five.bonus_die_choice[0], "coc.vehicules.1");
        assert!(line.on_chosen_ability);
        assert_eq!((line.target, line.target_scope), (None, None));

        document.specialities = vec![SpecialityEntry {
            ability: "coc.vehicules.2".into(),
            speciality: "moto".into(),
        }];
        document.bonus_die_targets = vec![palimpseste_document::BonusDieTargetEntry {
            voie: "coc.vehicules".into(),
            target: "coc.vehicules.2".into(),
        }];
        let (_, line) = die(&document);
        assert_eq!(line.target.as_deref(), Some("coc.vehicules.2"));
        assert_eq!(
            line.target_scope.as_deref(),
            Some("moto — conduite du véhicule")
        );

        // Cerveau: the tests of the capacité chosen, without speciality.
        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.crime".into(),
            ranks: vec![1, 2, 3, 4, 5],
        }];
        document.bonus_die_targets = vec![palimpseste_document::BonusDieTargetEntry {
            voie: "coc.crime".into(),
            target: "coc.crime.3".into(),
        }];
        let Details::Contemporain(details) = contemporain(document).details else {
            panic!("contemporain details expected");
        };
        let held = details
            .abilities_held
            .into_iter()
            .find(|h| h.ability == "coc.crime.5");
        let line = held.and_then(|h| h.bonus_die).unwrap();
        assert_eq!(
            line.target_scope.as_deref(),
            Some("serrures, coffres, systèmes de sécurité")
        );
    }

    #[test]
    fn held_capacites_carry_the_choice_made() {
        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.crime".into(),
            ranks: vec![1, 2, 3, 4, 5],
        }];
        document.raised_characteristics = vec![palimpseste_document::RaisedCharacteristicEntry {
            voie: "coc.crime".into(),
            characteristic: Characteristic::Cha,
        }];
        let Details::Contemporain(details) = contemporain(document).details else {
            panic!("contemporain details expected");
        };
        let choice = |held: &[sheet::HeldAbility], id: &str| {
            held.iter()
                .find(|h| h.ability == id)
                .and_then(|h| h.choice.clone())
        };
        assert_eq!(
            choice(&details.abilities_held, "coc.crime.5").as_deref(),
            Some("CHA")
        );
        assert_eq!(choice(&details.abilities_held, "coc.crime.4"), None);

        let mut document = FantasyDocument::empty();
        document.voies = vec![VoieEntry {
            id: "cof.combat-a-deux-armes".into(),
            ranks: vec![1, 2, 3, 4],
        }];
        // Two entries for one capacité: the last wins, as in the engine.
        document.ability_options = vec![
            AbilityOptionEntry {
                ability: "cof.combat-a-deux-armes.4".into(),
                option: "cof.option.agi".into(),
            },
            AbilityOptionEntry {
                ability: "cof.combat-a-deux-armes.4".into(),
                option: "cof.option.for".into(),
            },
        ];
        let Details::Fantasy(details) = fantasy(document).details else {
            panic!("fantasy details expected");
        };
        assert_eq!(
            choice(&details.abilities_held, "cof.combat-a-deux-armes.4").as_deref(),
            Some("FOR")
        );
        let Catalog::Fantasy(content) = catalog(Engine::Fantasy) else {
            panic!("fantasy catalog expected");
        };
        let heroic = content
            .abilities
            .iter()
            .find(|a| a.id == "cof.combat-a-deux-armes.4")
            .unwrap();
        assert!(heroic.effects[0].choice);
    }

    #[test]
    fn rank_five_capacites_say_what_they_raise() {
        let mut document = ContemporainDocument::empty();
        document.voies = vec![VoieEntry {
            id: "coc.crime".into(),
            ranks: vec![1, 2, 3, 4],
        }];
        document.experience = 5;
        document.xp_spendings = vec![XpSpendingEntry {
            abilities: vec!["coc.crime.5".into()],
            traits: Vec::new(),
        }];
        let Details::Contemporain(details) = contemporain(document).details else {
            panic!("contemporain details expected");
        };
        let [line] = details.rank_five.as_slice() else {
            panic!("one rank 5 capacité expected");
        };
        assert_eq!(
            (line.voie.as_str(), line.choice.as_slice(), line.progression),
            (
                "coc.crime",
                [Characteristic::Per, Characteristic::Cha].as_slice(),
                Some(1)
            )
        );
    }

    #[test]
    fn a_spending_offers_the_class_rise() {
        let mut document = ContemporainDocument::empty();
        document.traits = vec![palimpseste_document::TraitEntry {
            id: "coc.trait.classe-inferieure".into(),
            points: 2,
        }];
        document.experience = 4;
        document.xp_spendings = vec![XpSpendingEntry {
            abilities: Vec::new(),
            traits: Vec::new(),
        }];
        let Details::Contemporain(details) = contemporain(document).details else {
            panic!("contemporain details expected");
        };
        assert_eq!(
            details.spendings[0].class_rise,
            Some(sheet::ClassRise {
                id: "coc.trait.classe-inferieure".into(),
                points: Some(1),
                cost: 4,
            })
        );
    }

    /// Random bytes giving the `faces` wanted on dice of at most 20 faces.
    fn faces(faces: &[u8]) -> impl FnMut() -> u8 {
        let mut bytes = faces
            .iter()
            .map(|face| face - 1)
            .collect::<Vec<_>>()
            .into_iter();
        move || bytes.next().unwrap()
    }

    #[test]
    fn the_sheet_names_the_rolls_of_its_lines() {
        let mut document = FantasyDocument::empty();
        document.items = vec![ItemEntry::Weapon {
            id: "cof.weapon.epee-longue".into(),
            quantity: 1,
        }];
        let sheet = fantasy(document);
        assert_eq!(sheet.characteristics[0].roll, "cof.roll.test.agi");
        assert_eq!(
            value(&sheet, "cof.attack.contact").roll.as_deref(),
            Some("cof.roll.attack.contact")
        );
        let damage = value(&sheet, "cof.damage.epee-longue.one-handed");
        assert_eq!(
            damage.roll.as_deref(),
            Some("cof.roll.damage.epee-longue.one-handed")
        );
        assert_eq!(
            damage.attack_roll.as_deref(),
            Some("cof.roll.attack.epee-longue.one-handed")
        );
        assert_eq!(value(&sheet, "cof.defence").roll, None);

        let sheet = contemporain(ContemporainDocument::empty());
        assert_eq!(sheet.characteristics[2].roll, "coc.roll.test.for");
        assert_eq!(
            value(&sheet, "coc.attack.unarmed").roll.as_deref(),
            Some("coc.roll.attack.unarmed")
        );
        let damage = value(&sheet, "coc.damage.mains-nues");
        assert_eq!(damage.roll.as_deref(), Some("coc.roll.damage.mains-nues"));
        assert_eq!(
            damage.attack_roll.as_deref(),
            Some("coc.roll.weapon-attack.mains-nues")
        );
    }

    #[test]
    fn a_test_offers_its_bonuses_and_resolves_line_by_line() {
        let mut document = FantasyDocument::empty();
        document.voies = vec![VoieEntry {
            id: "cof.spadassin".into(),
            ranks: vec![1],
        }];
        let document = Document::Fantasy(document);
        let play = Play::default();
        let offer = roll_offer(&document, &play, "cof.roll.test.cha").unwrap();
        assert_eq!(offer.name, "Test de CHA");
        assert_eq!(offer.dice, "1d20");
        assert_eq!(offer.helper_bonus, Some(2));
        assert_eq!(
            offer.helpers_label.as_deref(),
            Some("Alliés ayant réussi leur aide")
        );
        let [bonus] = offer.options.as_slice() else {
            panic!("one option expected")
        };
        assert_eq!(bonus.label, "intimidation");
        assert!(bonus.origin.ends_with("rang 1)"), "{}", bonus.origin);
        assert_eq!((bonus.amount, bonus.picked), (Some(3), false));
        // Intimidation: usually CHA, offered to every test (D-60).
        assert!(bonus.featured);
        let agility = roll_offer(&document, &play, "cof.roll.test.agi").unwrap();
        let [aside] = agility.options.as_slice() else {
            panic!("one option expected")
        };
        assert!(!aside.featured);

        let request = |options: Vec<u32>| RollRequest {
            roll: "cof.roll.test.cha".into(),
            options,
            mode: Mode::Bonus,
            modifier: -1,
            helpers: 2,
            critical: false,
        };
        let plain = roll(&document, &play, &request(vec![]), &mut faces(&[7, 20])).unwrap();
        assert_eq!(
            plain
                .dice
                .iter()
                .map(|d| (d.face, d.kept))
                .collect::<Vec<_>>(),
            [(7, false), (20, true)]
        );
        assert_eq!(plain.natural, Some(NaturalRoll::Twenty));
        assert_eq!(plain.total, 20 + offer.base + 4 - 1);
        assert_eq!(plain.unpicked, offer.options);
        assert_eq!(
            plain
                .lines
                .iter()
                .map(|l| l.label.as_str())
                .collect::<Vec<_>>(),
            [
                "1d20",
                "Valeur de série",
                "Coopération : 2 alliés",
                "Modificateur du MJ"
            ]
        );

        let picked = roll(&document, &play, &request(vec![0]), &mut faces(&[7, 7])).unwrap();
        assert_eq!(picked.total, 7 + offer.base + 3 + 4 - 1);
        assert_eq!(picked.unpicked, []);
        // Of two equal dice, one is kept.
        assert_eq!(picked.dice.iter().filter(|d| d.kept).count(), 1);
        let line = picked
            .lines
            .iter()
            .find(|l| l.label == "intimidation")
            .unwrap();
        assert_eq!((line.kind, line.amount), (RollLineKind::Option, 3));

        assert!(roll(&document, &play, &request(vec![4]), &mut faces(&[7, 7])).is_err());
        assert!(roll_offer(&document, &play, "cof.roll.test.none").is_err());
        assert!(roll_offer(&document, &play, "coc.roll.test.cha").is_err());
    }

    #[test]
    fn an_attack_leads_to_its_damage_doubled_on_a_critical() {
        let mut document = FantasyDocument::empty();
        document.items = vec![ItemEntry::Weapon {
            id: "cof.weapon.rapiere".into(),
            quantity: 1,
        }];
        let document = Document::Fantasy(document);
        let play = Play::default();
        let attack = "cof.roll.attack.rapiere.one-handed";
        let offer = roll_offer(&document, &play, attack).unwrap();
        assert_eq!(offer.critical_from, Some(19));
        assert_eq!(offer.helper_bonus, None);
        let damage = offer.damage_roll.clone().unwrap();
        let request = |roll: &str, critical| RollRequest {
            roll: roll.into(),
            options: vec![],
            mode: Mode::Plain,
            modifier: 0,
            helpers: 0,
            critical,
        };
        let hit = roll(&document, &play, &request(attack, false), &mut faces(&[19])).unwrap();
        assert!(hit.critical);
        assert_eq!(hit.natural, None);
        assert_eq!(hit.damage_roll.as_deref(), Some(damage.as_str()));

        let offer = roll_offer(&document, &play, &damage).unwrap();
        assert_eq!(offer.critical.as_deref(), Some("DM doublés"));
        let plain = roll(&document, &play, &request(&damage, false), &mut faces(&[4])).unwrap();
        let doubled = roll(&document, &play, &request(&damage, true), &mut faces(&[4])).unwrap();
        assert_eq!(doubled.total, plain.total * 2);
        assert!(doubled.critical);
        assert_eq!(
            doubled.lines.last().map(|l| l.label.as_str()),
            Some("Critique : DM doublés")
        );
    }

    #[test]
    fn a_coc2_roll_follows_the_health_and_the_equipment_in_play() {
        let document = Document::Contemporain(ContemporainDocument::empty());
        let play = Play {
            equipment: None,
            damage: 10,
        };
        let offer = roll_offer(&document, &play, "coc.roll.test.con").unwrap();
        assert_eq!(offer.helpers_label.as_deref(), Some("Alliés aidant"));
        let [health] = offer.options.as_slice() else {
            panic!("one option expected")
        };
        assert_eq!(
            (
                health.label.as_str(),
                health.origin.as_str(),
                health.amount,
                health.picked
            ),
            ("Affaibli", "État de santé", Some(-3), true)
        );
        let mental = roll_offer(&document, &play, "coc.roll.test.int").unwrap();
        assert!(!mental.options[0].picked);
        assert_eq!(
            roll_offer(&document, &Play::default(), "coc.roll.test.con")
                .unwrap()
                .options,
            []
        );

        let damage = roll_offer(&document, &play, "coc.roll.damage.mains-nues").unwrap();
        assert_eq!(damage.critical.as_deref(), Some("bonus critique +4"));
        assert_eq!(damage.dice, "1d4");
        let request = RollRequest {
            roll: "coc.roll.damage.mains-nues".into(),
            options: vec![],
            mode: Mode::Plain,
            modifier: 0,
            helpers: 0,
            critical: true,
        };
        let outcome = roll(&document, &play, &request, &mut faces(&[3])).unwrap();
        assert_eq!(outcome.total, 3 + damage.base + 4);
        assert_eq!(
            outcome.dice,
            [RolledDie {
                faces: 4,
                face: 3,
                kept: true
            }]
        );

        // An equipment of another engine is an error.
        let other = Play {
            equipment: Some(Equipment::Fantasy(palimpseste_document::FantasyEquipment {
                armour: None,
                shield: None,
                items: vec![],
            })),
            damage: 0,
        };
        assert!(roll_offer(&document, &other, "coc.roll.test.con").is_err());
        // The AGI test takes the malus of the protections worn in play.
        let worn = Play {
            equipment: Some(Equipment::Contemporain(
                palimpseste_document::ContemporainEquipment {
                    protections: vec!["coc.protection.gilet-rigide".into()],
                    items: vec![],
                },
            )),
            damage: 0,
        };
        let agility = roll_offer(&document, &worn, "coc.roll.test.agi").unwrap();
        assert!(agility.base < 0);
        assert_eq!(agility.contributions.len(), 2);
    }

    fn request(roll: &str, options: Vec<u32>, mode: Mode) -> RollRequest {
        RollRequest {
            roll: roll.into(),
            options,
            mode,
            modifier: 0,
            helpers: 0,
            critical: false,
        }
    }

    fn voie(id: &str, ranks: &[u8]) -> VoieEntry {
        VoieEntry {
            id: id.into(),
            ranks: ranks.to_vec(),
        }
    }

    #[test]
    fn a_heroic_characteristic_starts_a_roll_with_a_bonus_die_and_a_malus_die_can_be_asked() {
        let mut document = FantasyDocument::empty();
        document.voies = vec![voie("cof.brute", &[1, 2, 3, 4])];
        let document = Document::Fantasy(document);
        let play = Play::default();
        let offer = roll_offer(&document, &play, "cof.roll.test.for").unwrap();
        assert_eq!(offer.mode, Mode::Bonus);
        assert_eq!(offer.dice, "2d20");
        let [die] = offer.bonus_dice.as_slice() else {
            panic!("one dé bonus expected")
        };
        assert!(die.origin.contains("Force héroïque"), "{}", die.origin);
        assert!(die.granted && die.featured);
        assert_eq!(
            roll_offer(&document, &play, "cof.roll.test.agi")
                .unwrap()
                .mode,
            Mode::Plain
        );

        // A dé malus keeps the lower die: a natural 1, then one ally.
        let mut malus = request("cof.roll.test.for", vec![], Mode::Malus);
        malus.helpers = 1;
        let outcome = roll(&document, &play, &malus, &mut faces(&[1, 9])).unwrap();
        assert_eq!(outcome.natural, Some(NaturalRoll::One));
        assert_eq!(outcome.mode, Mode::Malus);
        assert!(
            outcome
                .lines
                .iter()
                .any(|l| l.label == "Coopération : 1 allié")
        );
    }

    #[test]
    fn competence_bonuses_count_once_per_source_and_stop_at_the_cap() {
        let mut document = FantasyDocument::empty();
        document.profil = Some("cof.profil.voleur".into());
        document.peuple = Some("cof.peuple.halfelin".into());
        document.voies = vec![voie("cof.spadassin", &[1]), voie("cof.explosifs", &[1, 2])];
        let plain = Document::Fantasy(document.clone());
        let play = Play::default();
        let offer = roll_offer(&plain, &play, "cof.roll.test.int").unwrap();
        assert_eq!(offer.options.len(), 2);
        // Two voies de profil: the higher counts, the other is set aside.
        let outcome = roll(
            &plain,
            &play,
            &request("cof.roll.test.int", vec![0, 1], Mode::Plain),
            &mut faces(&[10]),
        )
        .unwrap();
        let [aside] = outcome.set_aside.as_slice() else {
            panic!("one bonus set aside expected")
        };
        assert_eq!(aside.amount, Some(3));
        assert_eq!(outcome.total, 10 + offer.base + 4);

        // A voie de peuple adds to a voie de profil, up to +15.
        document.voies = vec![voie("cof.spadassin", &[1]), voie("cof.halfelin", &[1])];
        document.adjustments = vec![AdjustmentEntry {
            target: Target::Value("cof.test-bonus.spadassin.1.1".into()),
            amount: 12,
            author: "gm".into(),
            reason: "a legend".into(),
        }];
        let document = Document::Fantasy(document);
        let offer = roll_offer(&document, &play, "cof.roll.test.cha").unwrap();
        let picked = (0..u32::try_from(offer.options.len()).unwrap()).collect::<Vec<_>>();
        let outcome = roll(
            &document,
            &play,
            &request("cof.roll.test.cha", picked, Mode::Plain),
            &mut faces(&[10]),
        )
        .unwrap();
        let cap = outcome
            .lines
            .iter()
            .find(|l| l.kind == RollLineKind::Cap)
            .expect("a line takes back what exceeds +15");
        assert_eq!(cap.label, "Bonus de compétence plafonnés à +15");
        assert_eq!(cap.amount, -3);
    }

    #[test]
    fn a_coc2_roll_names_its_bonuses_notes_and_dice_and_draws_each_line() {
        let mut document = ContemporainDocument::empty();
        document.voies = vec![
            voie("coc.crime", &[1, 2]),
            voie("coc.investigation", &[1, 2]),
        ];
        document.specialities = vec![palimpseste_document::SpecialityEntry {
            ability: "coc.investigation.2".into(),
            speciality: "généalogie".into(),
        }];
        let document = Document::Contemporain(document);
        let play = Play {
            equipment: None,
            damage: 10,
        };

        // A test bonus to some AGI tests, picked, with the help of one ally.
        let offer = roll_offer(&document, &play, "coc.roll.test.agi").unwrap();
        let bonus = offer
            .options
            .iter()
            .position(|o| o.label.contains("vol"))
            .expect("the pickpocket's bonus is offered");
        let mut asked = request(
            "coc.roll.test.agi",
            vec![u32::try_from(bonus).unwrap()],
            Mode::Plain,
        );
        asked.helpers = 1;
        asked.modifier = 2;
        let outcome = roll(&document, &play, &asked, &mut faces(&[10])).unwrap();
        let labels = outcome
            .lines
            .iter()
            .map(|l| l.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"Coopération : 1 allié"));
        assert!(labels.contains(&"Modificateur du MJ"));
        let line = outcome
            .lines
            .iter()
            .find(|l| l.kind == RollLineKind::Option)
            .unwrap();
        assert!(
            line.origin
                .as_deref()
                .is_some_and(|origin| origin.contains("Pickpocket"))
        );

        // What the player left unpicked is recalled with the result.
        let outcome = roll(
            &document,
            &play,
            &request("coc.roll.test.int", vec![], Mode::Plain),
            &mut faces(&[10]),
        )
        .unwrap();
        assert!(outcome.unpicked.iter().any(|o| o.label == "Affaibli"));
        assert_eq!(outcome.set_aside, []);
        assert!(
            roll(
                &document,
                &play,
                &request("coc.roll.test.int", vec![99], Mode::Plain),
                &mut faces(&[10]),
            )
            .is_err()
        );

        // A note without an amount, and the dé bonus of a speciality.
        let strength = roll_offer(&document, &play, "coc.roll.test.for").unwrap();
        let [note] = strength.notes.as_slice() else {
            panic!("one note expected")
        };
        assert!(note.origin.contains("Gangster"), "{}", note.origin);
        let intelligence = roll_offer(&document, &play, "coc.roll.test.int").unwrap();
        let [die] = intelligence.bonus_dice.as_slice() else {
            panic!("one dé bonus expected")
        };
        assert_eq!(die.scope, "généalogie (spécialité d’investigation)");
        assert!(die.origin.starts_with("Spécialiste"), "{}", die.origin);
        // A speciality the engine cannot tell: put forward, not granted.
        assert!(die.featured && !die.granted);
    }

    #[test]
    fn a_coc2_attack_leads_to_its_damage_roll() {
        let document = Document::Contemporain(ContemporainDocument::empty());
        let play = Play::default();
        let attack = roll_offer(&document, &play, "coc.roll.attack.unarmed").unwrap();
        assert_eq!((attack.kind, attack.damage_roll), (RollKind::Attack, None));
        let weapon = roll_offer(&document, &play, "coc.roll.weapon-attack.mains-nues").unwrap();
        assert_eq!(weapon.kind, RollKind::Attack);
        assert_eq!(
            weapon.damage_roll.as_deref(),
            Some("coc.roll.damage.mains-nues")
        );
        let outcome = roll(
            &document,
            &play,
            &request("coc.roll.weapon-attack.mains-nues", vec![], Mode::Plain),
            &mut faces(&[20]),
        )
        .unwrap();
        assert!(outcome.critical);
        assert_eq!(
            outcome.damage_roll.as_deref(),
            Some("coc.roll.damage.mains-nues")
        );
    }

    #[test]
    fn an_equipment_of_the_other_engine_is_refused_for_a_coc2_character_and_a_cof2_one() {
        let document = Document::Fantasy(FantasyDocument::empty());
        let other = Play {
            equipment: Some(Equipment::Contemporain(
                palimpseste_document::ContemporainEquipment {
                    protections: vec![],
                    items: vec![],
                },
            )),
            damage: 0,
        };
        assert!(roll_offer(&document, &other, "cof.roll.test.for").is_err());
    }

    #[test]
    fn the_state_of_health_is_given_for_each_step_filled() {
        let sheet = contemporain(ContemporainDocument::empty());
        let Details::Contemporain(details) = &sheet.details else {
            panic!("COC2 details expected")
        };
        let states = &details.health_states;
        assert_eq!(states.len(), 21);
        assert_eq!(states[4], None);
        assert_eq!(states[5].as_deref(), Some("coc.health-state.contusionne"));
        assert_eq!(states[20].as_deref(), Some("coc.health-state.mourant"));
    }
}
