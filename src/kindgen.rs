//! kindgen: generate kind source files from declarative specs.
//!
//! A `KindSpec` is a tiny declarative descriptor; `emit_kind_source`
//! returns a complete Rust module, and `emit_integration_patch` returns
//! string fragments to wire the kind into Object/ObjectKindTag/Mind and
//! match statements that dispatch over them.
//!
//! Every spec in `GENERATED_SPECS` is driven through a meta-test
//! (`all_registered_specs_have_integrated_fragments`) that iterates the
//! registry and asserts every fragment is literally present in the
//! real on-disk kinds/mod.rs + mind.rs + ai.rs + interact.rs.
//!
//! Every kind in the universe — generated OR hand-crafted — is listed
//! in `UNIVERSE_MANIFEST`. The `universe_manifest_covers_every_kind_tag`
//! meta-test asserts the manifest is bijective with `ALL_KIND_TAGS`.
//! No kind can silently exist outside the factory's knowledge.
//!
//! Module layout:
//! - `kindgen.rs` (this file): public types, static registries, dispatch
//! - `kindgen/emit.rs`: private archetype-specific emission functions
//! - `kindgen/tests.rs`: all meta-tests (`#[cfg(test)]` only)
//!
//! Adding a new archetype is a three-part, cheap edit: add a variant
//! here + one `emit::*` function + one golden test. No single file
//! balloons.

mod emit;
#[cfg(test)]
mod tests;

// ---------- Specs ----------

pub struct KindSpec {
    pub name: &'static str,
    pub doc: &'static str,
    pub archetype: Archetype,
}

pub enum Archetype {
    /// A vessel that holds up to N ml of water. Pos + one capacity constant.
    SimpleLiquidContainer { capacity_ml: u32 },
    /// An on/off faucet that produces water at a fixed flow rate.
    SwitchableFaucet { flow_ml_per_sec: u32 },
    /// A world-decorating object with only position state.
    DecorativeItem,
    /// An object with an open/closed toggle via `Openable`.
    SimpleOpenable,
}

pub static MUG_SPEC: KindSpec = KindSpec {
    name: "Mug",
    doc: "Mug: a single-chamber ceramic drinking vessel holding up to 250ml.",
    archetype: Archetype::SimpleLiquidContainer { capacity_ml: 250 },
};

pub static PLANT_SPEC: KindSpec = KindSpec {
    name: "Plant",
    doc: "Plant: a decorative potted houseplant.",
    archetype: Archetype::DecorativeItem,
};

pub static PAINTING_SPEC: KindSpec = KindSpec {
    name: "Painting",
    doc: "Painting: a framed piece of art on the wall.",
    archetype: Archetype::DecorativeItem,
};

pub static DIARY_SPEC: KindSpec = KindSpec {
    name: "Diary",
    doc: "Diary: a private journal with a pressed-leather cover.",
    archetype: Archetype::SimpleOpenable,
};

pub static GENERATED_SPECS: &[&KindSpec] =
    &[&MUG_SPEC, &PLANT_SPEC, &PAINTING_SPEC, &DIARY_SPEC];

pub fn generated_specs() -> &'static [&'static KindSpec] { GENERATED_SPECS }

// ---------- Universe manifest ----------

pub enum KindOrigin {
    Generated(&'static KindSpec),
    HandCrafted { archetype_conformance: &'static [&'static str] },
}

pub struct UniverseEntry {
    pub tag_name: &'static str,
    pub origin: KindOrigin,
}

pub static UNIVERSE_MANIFEST: &[UniverseEntry] = &[
    UniverseEntry { tag_name: "Bed",         origin: KindOrigin::HandCrafted { archetype_conformance: &["StatefulToggle"] } },
    UniverseEntry { tag_name: "Book",        origin: KindOrigin::HandCrafted { archetype_conformance: &["SimpleOpenable"] } },
    UniverseEntry { tag_name: "CoffeeMaker", origin: KindOrigin::HandCrafted { archetype_conformance: &["Cyclable", "LiquidContainer", "Powered", "TimeDecaySeconds"] } },
    UniverseEntry { tag_name: "Window",      origin: KindOrigin::HandCrafted { archetype_conformance: &["Switchable", "LightEmitter"] } },
    UniverseEntry { tag_name: "Fridge",      origin: KindOrigin::HandCrafted { archetype_conformance: &["Openable", "Container", "Powered", "TimeDecayMinutes"] } },
    UniverseEntry { tag_name: "Stove",       origin: KindOrigin::HandCrafted { archetype_conformance: &["Powered", "LightEmitter", "TimeDecayMinutes"] } },
    UniverseEntry { tag_name: "Sink",        origin: KindOrigin::HandCrafted { archetype_conformance: &["SwitchableFaucet"] } },
    UniverseEntry { tag_name: "Toilet",      origin: KindOrigin::HandCrafted { archetype_conformance: &["Cyclable", "LiquidContainer", "TimeDecaySeconds"] } },
    UniverseEntry { tag_name: "Shower",      origin: KindOrigin::HandCrafted { archetype_conformance: &["SwitchableFaucet"] } },
    UniverseEntry { tag_name: "Bathtub",     origin: KindOrigin::HandCrafted { archetype_conformance: &["Openable", "Switchable", "LiquidContainer", "TimeDecaySeconds"] } },
    UniverseEntry { tag_name: "BathSink",    origin: KindOrigin::HandCrafted { archetype_conformance: &["SwitchableFaucet"] } },
    UniverseEntry { tag_name: "Lamp",        origin: KindOrigin::HandCrafted { archetype_conformance: &["Switchable", "LightEmitter", "Powered"] } },
    UniverseEntry { tag_name: "Nightstand",  origin: KindOrigin::HandCrafted { archetype_conformance: &["Container"] } },
    UniverseEntry { tag_name: "Dresser",     origin: KindOrigin::HandCrafted { archetype_conformance: &["Container"] } },
    UniverseEntry { tag_name: "Mug",         origin: KindOrigin::Generated(&MUG_SPEC) },
    UniverseEntry { tag_name: "Plant",       origin: KindOrigin::Generated(&PLANT_SPEC) },
    UniverseEntry { tag_name: "Painting",    origin: KindOrigin::Generated(&PAINTING_SPEC) },
    UniverseEntry { tag_name: "Diary",       origin: KindOrigin::Generated(&DIARY_SPEC) },
];

pub fn universe_manifest() -> &'static [UniverseEntry] { UNIVERSE_MANIFEST }

pub fn manifest_entry_for(tag_name: &str) -> Option<&'static UniverseEntry> {
    UNIVERSE_MANIFEST.iter().find(|e| e.tag_name == tag_name)
}

// ---------- Code emission ----------

pub struct IntegrationPatch {
    pub mod_decl: String,
    pub use_decl: String,
    pub tag_variant: String,
    pub obj_variant: String,
    pub pos_arm: String,
    pub kind_tag_arm: String,
    pub use_action_arm: String,
    pub liquid_container_arm: Option<String>,
    pub mind_variant: String,
    pub mind_from_tag_arm: String,
    pub wildcard_kind_tag: String,
}

pub fn emit_kind_source(spec: &KindSpec) -> String {
    match &spec.archetype {
        Archetype::SimpleLiquidContainer { capacity_ml } =>
            emit::simple_liquid_container(spec.name, spec.doc, *capacity_ml),
        Archetype::SwitchableFaucet { flow_ml_per_sec } =>
            emit::switchable_faucet(spec.name, spec.doc, *flow_ml_per_sec),
        Archetype::DecorativeItem =>
            emit::decorative_item(spec.name, spec.doc),
        Archetype::SimpleOpenable =>
            emit::simple_openable(spec.name, spec.doc),
    }
}

pub fn emit_integration_patch(spec: &KindSpec) -> IntegrationPatch {
    let name = spec.name;
    let module_name = snake_case(name);
    let liquid_container_arm = match &spec.archetype {
        Archetype::SimpleLiquidContainer { .. } =>
            Some(format!("            Object::{name}(x) => Some(x),")),
        Archetype::SwitchableFaucet { .. }
        | Archetype::DecorativeItem
        | Archetype::SimpleOpenable => None,
    };
    let use_action_arm = match &spec.archetype {
        Archetype::SimpleLiquidContainer { .. } | Archetype::DecorativeItem =>
            format!("            Object::{name}(_) => {{}}"),
        Archetype::SwitchableFaucet { .. } =>
            format!("            Object::{name}(x) => x.toggle(),"),
        Archetype::SimpleOpenable =>
            format!("            Object::{name}(x) => x.toggle_open(),"),
    };
    IntegrationPatch {
        mod_decl: format!("pub mod {module_name};"),
        use_decl: format!("pub use {module_name}::{name};"),
        tag_variant: format!("    {name},"),
        obj_variant: format!("    {name}({name}),"),
        pos_arm: format!("            Object::{name}(x) => x.pos,"),
        kind_tag_arm: format!("            Object::{name}(_) => ObjectKindTag::{name},"),
        use_action_arm,
        liquid_container_arm,
        mind_variant: format!("    {name},"),
        mind_from_tag_arm: format!("            ObjectKindTag::{name} => Self::{name},"),
        wildcard_kind_tag: format!("| ObjectKindTag::{name}"),
    }
}

fn snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.char_indices() {
        if c.is_uppercase() && i > 0 { out.push('_'); }
        out.push(c.to_ascii_lowercase());
    }
    out
}
