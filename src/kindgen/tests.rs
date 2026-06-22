//! Meta-tests for the factory. Cover: (a) emitter output matches
//! on-disk sources byte-exact, (b) emitter parameterizes correctly
//! across names and capacities, (c) integration fragments are
//! literally present in every target file, (d) the universe manifest
//! bijects with ALL_KIND_TAGS, and (e) generated/hand-crafted entries
//! are consistent with their origin declarations.

use super::*;
use crate::kinds::ALL_KIND_TAGS;

// ---------- Emitter: output matches on-disk sources ----------

#[test]
fn mug_spec_emits_source_matching_disk() {
    assert_eq!(emit_kind_source(&MUG_SPEC), include_str!("../kinds/mug.rs"));
}

#[test]
fn plant_spec_emits_source_matching_disk() {
    assert_eq!(emit_kind_source(&PLANT_SPEC), include_str!("../kinds/plant.rs"));
}

#[test]
fn painting_spec_emits_source_matching_disk() {
    assert_eq!(emit_kind_source(&PAINTING_SPEC), include_str!("../kinds/painting.rs"));
}

#[test]
fn chair_spec_emits_source_matching_disk() {
    assert_eq!(emit_kind_source(&CHAIR_SPEC), include_str!("../kinds/chair.rs"));
}

#[test]
fn diary_spec_emits_source_matching_disk() {
    assert_eq!(emit_kind_source(&DIARY_SPEC), include_str!("../kinds/diary.rs"));
}

// ---------- Emitter: shape + parameterization ----------

#[test]
fn emits_pub_struct_with_spec_name() {
    assert!(emit_kind_source(&MUG_SPEC).contains("pub struct Mug "));
}

#[test]
fn emits_capacity_const_from_spec() {
    assert!(emit_kind_source(&MUG_SPEC).contains("pub const CAPACITY_ML: u32 = 250;"));
}

#[test]
fn emits_liquid_container_impl() {
    let s = emit_kind_source(&MUG_SPEC);
    assert!(s.contains("impl LiquidContainer for Mug"));
    assert!(s.contains("fn add_liquid"));
    assert!(s.contains("fn drain_liquid"));
}

#[test]
fn emits_test_module_with_expected_tests() {
    let s = emit_kind_source(&MUG_SPEC);
    assert!(s.contains("#[cfg(test)]"));
    assert!(s.contains("fn starts_empty"));
    assert!(s.contains("fn add_water_bounded_by_capacity"));
    assert!(s.contains("fn reject_non_water"));
    assert!(s.contains("fn drain_removes_water"));
}

#[test]
fn parameterizes_name_across_all_sites() {
    let spec = KindSpec {
        name: "Teapot",
        doc: "Teapot: 750ml porcelain kettle.",
        archetype: Archetype::SimpleLiquidContainer { capacity_ml: 750 },
    };
    let src = emit_kind_source(&spec);
    assert!(src.contains("pub struct Teapot"));
    assert!(src.contains("impl LiquidContainer for Teapot"));
    assert!(src.contains("let m = Teapot::new"));
    assert!(src.contains("Teapot::CAPACITY_ML"));
    assert!(!src.contains("Mug"));
}

#[test]
fn parameterizes_capacity() {
    let spec = KindSpec {
        name: "Vase",
        doc: "Vase: 1200ml.",
        archetype: Archetype::SimpleLiquidContainer { capacity_ml: 1200 },
    };
    assert!(emit_kind_source(&spec).contains("pub const CAPACITY_ML: u32 = 1200;"));
}

#[test]
fn faucet_archetype_emits_three_trait_impls() {
    let spec = KindSpec {
        name: "UtilityTap", doc: "90 ml/s.",
        archetype: Archetype::SwitchableFaucet { flow_ml_per_sec: 90 },
    };
    let s = emit_kind_source(&spec);
    assert!(s.contains("impl Switchable for UtilityTap"));
    assert!(s.contains("impl LiquidSource for UtilityTap"));
    assert!(s.contains("impl TimeDecaySeconds for UtilityTap"));
    assert!(s.contains("pub const FLOW_ML_PER_SEC: u32 = 90;"));
}

#[test]
fn faucet_archetype_has_on_off_state_fields() {
    let spec = KindSpec {
        name: "GardenHose", doc: "200 ml/s.",
        archetype: Archetype::SwitchableFaucet { flow_ml_per_sec: 200 },
    };
    let s = emit_kind_source(&spec);
    assert!(s.contains("on: bool,"));
    assert!(s.contains("pub seconds_running: f32,"));
}

#[test]
fn decorative_archetype_emits_minimal_shape() {
    let spec = KindSpec {
        name: "Painting", doc: "Framed art.",
        archetype: Archetype::DecorativeItem,
    };
    let s = emit_kind_source(&spec);
    assert!(s.contains("pub struct Painting"));
    assert!(s.contains("pub pos: Vec2,"));
    assert!(s.contains("fn stores_pos()"));
    assert!(!s.contains("impl Switchable"));
    assert!(!s.contains("impl LiquidContainer"));
    assert!(!s.contains("CAPACITY_ML"));
}

#[test]
fn openable_archetype_emits_minimal_shape() {
    let spec = KindSpec {
        name: "JewelryBox", doc: "A small wooden jewelry box.",
        archetype: Archetype::SimpleOpenable,
    };
    let s = emit_kind_source(&spec);
    assert!(s.contains("pub struct JewelryBox"));
    assert!(s.contains("pub pos: Vec2,"));
    assert!(s.contains("open: bool,"));
    assert!(s.contains("impl Openable for JewelryBox"));
    assert!(s.contains("fn is_open(&self) -> bool"));
    assert!(s.contains("fn toggle_open(&mut self)"));
    assert!(s.contains("fn starts_closed()"));
    assert!(s.contains("fn toggle_opens_and_closes()"));
    assert!(!s.contains("impl LiquidContainer"));
    assert!(!s.contains("CAPACITY_ML"));
}

#[test]
fn generator_output_is_deterministic() {
    assert_eq!(emit_kind_source(&MUG_SPEC), emit_kind_source(&MUG_SPEC));
    assert_eq!(emit_kind_source(&DIARY_SPEC), emit_kind_source(&DIARY_SPEC));
}

#[test]
fn emission_ends_with_newline() {
    assert!(emit_kind_source(&MUG_SPEC).ends_with('\n'));
    assert!(emit_kind_source(&DIARY_SPEC).ends_with('\n'));
}

// ---------- Integration patch shape ----------

#[test]
fn mug_integration_patch_has_expected_shape() {
    let p = emit_integration_patch(&MUG_SPEC);
    assert_eq!(p.mod_decl, "pub mod mug;");
    assert_eq!(p.use_decl, "pub use mug::Mug;");
    assert_eq!(p.tag_variant, "    Mug,");
    assert_eq!(p.obj_variant, "    Mug(Mug),");
    assert_eq!(p.pos_arm, "            Object::Mug(x) => x.pos,");
    assert_eq!(p.kind_tag_arm, "            Object::Mug(_) => ObjectKindTag::Mug,");
    assert_eq!(p.use_action_arm, "            Object::Mug(_) => {}");
    assert_eq!(p.liquid_container_arm.as_deref(), Some("            Object::Mug(x) => Some(x),"));
    assert_eq!(p.wildcard_kind_tag, "| ObjectKindTag::Mug");
}

#[test]
fn plant_integration_patch_has_expected_shape() {
    let p = emit_integration_patch(&PLANT_SPEC);
    assert_eq!(p.mod_decl, "pub mod plant;");
    assert_eq!(p.use_decl, "pub use plant::Plant;");
    assert_eq!(p.obj_variant, "    Plant(Plant),");
    assert!(p.liquid_container_arm.is_none());
    assert_eq!(p.wildcard_kind_tag, "| ObjectKindTag::Plant");
}

#[test]
fn painting_integration_patch_has_expected_shape() {
    let p = emit_integration_patch(&PAINTING_SPEC);
    assert_eq!(p.mod_decl, "pub mod painting;");
    assert_eq!(p.use_decl, "pub use painting::Painting;");
    assert_eq!(p.obj_variant, "    Painting(Painting),");
    assert!(p.liquid_container_arm.is_none());
    assert_eq!(p.wildcard_kind_tag, "| ObjectKindTag::Painting");
}

#[test]
fn diary_integration_patch_has_expected_shape() {
    let p = emit_integration_patch(&DIARY_SPEC);
    assert_eq!(p.mod_decl, "pub mod diary;");
    assert_eq!(p.use_decl, "pub use diary::Diary;");
    assert_eq!(p.obj_variant, "    Diary(Diary),");
    assert_eq!(p.use_action_arm, "            Object::Diary(x) => x.toggle_open(),");
    assert!(p.liquid_container_arm.is_none());
    assert_eq!(p.wildcard_kind_tag, "| ObjectKindTag::Diary");
}

#[test]
fn faucet_archetype_has_no_liquid_container_arm() {
    let spec = KindSpec {
        name: "TestTap", doc: "Test.",
        archetype: Archetype::SwitchableFaucet { flow_ml_per_sec: 50 },
    };
    assert!(emit_integration_patch(&spec).liquid_container_arm.is_none());
}

#[test]
fn decorative_archetype_has_no_liquid_container_arm() {
    assert!(emit_integration_patch(&PLANT_SPEC).liquid_container_arm.is_none());
}

#[test]
fn openable_archetype_has_no_liquid_container_arm() {
    assert!(emit_integration_patch(&DIARY_SPEC).liquid_container_arm.is_none());
}

#[test]
fn faucet_archetype_use_action_arm_toggles() {
    let spec = KindSpec {
        name: "Spigot", doc: "A spigot.",
        archetype: Archetype::SwitchableFaucet { flow_ml_per_sec: 10 },
    };
    assert_eq!(
        emit_integration_patch(&spec).use_action_arm,
        "            Object::Spigot(x) => x.toggle(),"
    );
}

#[test]
fn openable_archetype_use_action_arm_toggles_open() {
    assert_eq!(
        emit_integration_patch(&DIARY_SPEC).use_action_arm,
        "            Object::Diary(x) => x.toggle_open(),"
    );
}

#[test]
fn snake_case_converts_camel_case_module_name() {
    let spec = KindSpec {
        name: "CokeCan", doc: "355ml.",
        archetype: Archetype::SimpleLiquidContainer { capacity_ml: 355 },
    };
    let p = emit_integration_patch(&spec);
    assert_eq!(p.mod_decl, "pub mod coke_can;");
    assert_eq!(p.use_decl, "pub use coke_can::CokeCan;");
}

#[test]
fn single_letter_name_stays_lowercase() {
    let spec = KindSpec {
        name: "X", doc: "X.",
        archetype: Archetype::SimpleLiquidContainer { capacity_ml: 1 },
    };
    assert_eq!(emit_integration_patch(&spec).mod_decl, "pub mod x;");
}

// ---------- Integration: fragments literally present in on-disk files ----------

#[test]
fn mug_patch_fragments_appear_in_actual_kinds_mod_rs() {
    let p = emit_integration_patch(&MUG_SPEC);
    let m = include_str!("../kinds/mod.rs");
    assert!(m.contains(&p.mod_decl));
    assert!(m.contains(&p.use_decl));
    assert!(m.contains(&p.tag_variant));
    assert!(m.contains(&p.obj_variant));
    assert!(m.contains(&p.pos_arm));
    assert!(m.contains(&p.kind_tag_arm));
    assert!(m.contains(&p.use_action_arm));
    assert!(m.contains(p.liquid_container_arm.as_deref().unwrap()));
}

#[test]
fn mug_patch_fragments_appear_in_actual_mind_rs() {
    let p = emit_integration_patch(&MUG_SPEC);
    let m = include_str!("../mind.rs");
    assert!(m.contains(&p.mind_variant));
    assert!(m.contains(&p.mind_from_tag_arm));
}

#[test]
fn mug_wildcard_fragment_appears_in_ai_and_interact() {
    let p = emit_integration_patch(&MUG_SPEC);
    assert!(include_str!("../ai.rs").contains(&p.wildcard_kind_tag));
    assert!(include_str!("../interact.rs").contains(&p.wildcard_kind_tag));
}

#[test]
fn all_registered_specs_have_integrated_fragments() {
    let mod_rs = include_str!("../kinds/mod.rs");
    let mind_rs = include_str!("../mind.rs");
    let ai_rs = include_str!("../ai.rs");
    let interact_rs = include_str!("../interact.rs");
    for spec in generated_specs() {
        let p = emit_integration_patch(spec);
        let n = spec.name;
        assert!(mod_rs.contains(&p.mod_decl),       "{n}: missing mod_decl");
        assert!(mod_rs.contains(&p.use_decl),       "{n}: missing use_decl");
        assert!(mod_rs.contains(&p.tag_variant),    "{n}: missing tag_variant");
        assert!(mod_rs.contains(&p.obj_variant),    "{n}: missing obj_variant");
        assert!(mod_rs.contains(&p.pos_arm),        "{n}: missing pos_arm");
        assert!(mod_rs.contains(&p.kind_tag_arm),   "{n}: missing kind_tag_arm");
        assert!(mod_rs.contains(&p.use_action_arm), "{n}: missing use_action_arm");
        if let Some(lc) = &p.liquid_container_arm {
            assert!(mod_rs.contains(lc), "{n}: missing liquid_container_arm");
        }
        assert!(mind_rs.contains(&p.mind_variant),       "{n}: missing mind_variant");
        assert!(mind_rs.contains(&p.mind_from_tag_arm),  "{n}: missing mind_from_tag_arm");
        assert!(ai_rs.contains(&p.wildcard_kind_tag),    "{n}: missing from ai.rs");
        assert!(interact_rs.contains(&p.wildcard_kind_tag), "{n}: missing from interact.rs");
    }
}

#[test]
fn registry_contains_all_current_specs() {
    let names: Vec<&str> = generated_specs().iter().map(|s| s.name).collect();
    assert!(names.contains(&"Mug"));
    assert!(names.contains(&"Plant"));
    assert!(names.contains(&"Painting"));
    assert!(names.contains(&"Chair"));
    assert!(names.contains(&"Diary"));
}

// ---------- Universe manifest completeness + consistency ----------

#[test]
fn universe_manifest_covers_every_kind_tag() {
    for tag in ALL_KIND_TAGS {
        let tn = tag.name();
        assert!(
            manifest_entry_for(tn).is_some(),
            "No manifest entry for kind {tn}"
        );
    }
}

#[test]
fn universe_manifest_size_matches_all_kind_tags() {
    assert_eq!(UNIVERSE_MANIFEST.len(), ALL_KIND_TAGS.len());
}

#[test]
fn no_duplicate_tag_names_in_manifest() {
    use std::collections::HashSet;
    let names: HashSet<&str> = UNIVERSE_MANIFEST.iter().map(|e| e.tag_name).collect();
    assert_eq!(names.len(), UNIVERSE_MANIFEST.len());
}

#[test]
fn every_manifest_tag_name_matches_a_kind_tag() {
    let valid: std::collections::HashSet<&str> =
        ALL_KIND_TAGS.iter().map(|t| t.name()).collect();
    for e in UNIVERSE_MANIFEST {
        assert!(
            valid.contains(e.tag_name),
            "manifest has '{}' but no such ObjectKindTag",
            e.tag_name
        );
    }
}

#[test]
fn every_generated_spec_has_manifest_entry_as_generated() {
    for spec in generated_specs() {
        let entry = manifest_entry_for(spec.name)
            .unwrap_or_else(|| panic!("spec {} missing from manifest", spec.name));
        match &entry.origin {
            KindOrigin::Generated(s) => assert!(
                std::ptr::eq(*s, *spec),
                "manifest entry for {} points at a different spec", spec.name
            ),
            KindOrigin::HandCrafted { .. } => panic!(
                "{} is in GENERATED_SPECS but manifest'd as HandCrafted", spec.name
            ),
        }
    }
}

#[test]
fn hand_crafted_kinds_declare_at_least_one_archetype() {
    for entry in UNIVERSE_MANIFEST {
        if let KindOrigin::HandCrafted { archetype_conformance } = &entry.origin {
            assert!(
                !archetype_conformance.is_empty(),
                "{} is HandCrafted but declares no archetype conformance",
                entry.tag_name
            );
        }
    }
}

#[test]
fn universe_has_exactly_nineteen_kinds() {
    assert_eq!(UNIVERSE_MANIFEST.len(), 19);
}

#[test]
fn sink_bathsink_shower_all_declare_faucet_conformance() {
    for name in ["Sink", "BathSink", "Shower"] {
        let entry = manifest_entry_for(name).unwrap();
        match &entry.origin {
            KindOrigin::HandCrafted { archetype_conformance } => {
                assert!(archetype_conformance.contains(&"SwitchableFaucet"),
                    "{name} should declare SwitchableFaucet conformance");
            }
            _ => panic!("{name} unexpectedly Generated"),
        }
    }
}
