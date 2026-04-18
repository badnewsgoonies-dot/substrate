//! Archetype-specific emission. Each function returns a complete,
//! compileable Rust module body for one kind. Private to the parent
//! `kindgen` module — outside callers go through
//! `kindgen::emit_kind_source`.

pub(super) fn simple_liquid_container(name: &str, doc: &str, capacity_ml: u32) -> String {
    let mut s = String::new();
    s.push_str(&format!("//! {doc}\n\n"));
    s.push_str("use crate::space::Vec2;\n");
    s.push_str("use crate::traits::{Liquid, LiquidContainer};\n\n");
    s.push_str("#[derive(Debug, Clone, Copy)]\n");
    s.push_str(&format!("pub struct {name} {{\n    pub pos: Vec2,\n    water_ml: u32,\n}}\n\n"));
    s.push_str(&format!("impl {name} {{\n"));
    s.push_str(&format!("    pub const CAPACITY_ML: u32 = {capacity_ml};\n\n"));
    s.push_str("    pub fn new(pos: Vec2) -> Self {\n        Self { pos, water_ml: 0 }\n    }\n\n");
    s.push_str("    pub fn water_ml(&self) -> u32 { self.water_ml }\n\n");
    s.push_str("    pub fn fill_fraction(&self) -> f32 {\n");
    s.push_str("        (self.water_ml as f32 / Self::CAPACITY_ML as f32).clamp(0.0, 1.0)\n");
    s.push_str("    }\n}\n\n");
    s.push_str(&format!("impl LiquidContainer for {name} {{\n"));
    s.push_str("    fn liquid(&self) -> Option<(Liquid, u32)> {\n");
    s.push_str("        if self.water_ml > 0 { Some((Liquid::Water, self.water_ml)) } else { None }\n    }\n");
    s.push_str("    fn capacity_ml(&self) -> u32 { Self::CAPACITY_ML }\n");
    s.push_str("    fn add_liquid(&mut self, kind: Liquid, ml: u32) -> u32 {\n");
    s.push_str("        if kind != Liquid::Water { return ml; }\n");
    s.push_str("        let room = Self::CAPACITY_ML.saturating_sub(self.water_ml);\n");
    s.push_str("        let taken = ml.min(room);\n        self.water_ml += taken;\n        ml - taken\n    }\n");
    s.push_str("    fn drain_liquid(&mut self, ml: u32) -> (Liquid, u32) {\n");
    s.push_str("        let taken = ml.min(self.water_ml);\n        self.water_ml -= taken;\n        (Liquid::Water, taken)\n    }\n}\n\n");
    s.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
    s.push_str("    #[test]\n    fn starts_empty() {\n");
    s.push_str(&format!("        let m = {name}::new(Vec2::new(0.0, 0.0));\n"));
    s.push_str("        assert_eq!(m.water_ml(), 0);\n        assert_eq!(m.fill_fraction(), 0.0);\n    }\n\n");
    s.push_str("    #[test]\n    fn add_water_bounded_by_capacity() {\n");
    s.push_str(&format!("        let mut m = {name}::new(Vec2::new(0.0, 0.0));\n"));
    s.push_str(&format!("        let overflow = m.add_liquid(Liquid::Water, {name}::CAPACITY_ML * 2);\n"));
    s.push_str(&format!("        assert_eq!(overflow, {name}::CAPACITY_ML);\n"));
    s.push_str(&format!("        assert_eq!(m.water_ml(), {name}::CAPACITY_ML);\n    }}\n\n"));
    s.push_str("    #[test]\n    fn reject_non_water() {\n");
    s.push_str(&format!("        let mut m = {name}::new(Vec2::new(0.0, 0.0));\n"));
    s.push_str("        let overflow = m.add_liquid(Liquid::Coffee, 100);\n");
    s.push_str("        assert_eq!(overflow, 100);\n        assert_eq!(m.water_ml(), 0);\n    }\n\n");
    s.push_str("    #[test]\n    fn drain_removes_water() {\n");
    s.push_str(&format!("        let mut m = {name}::new(Vec2::new(0.0, 0.0));\n"));
    s.push_str("        m.add_liquid(Liquid::Water, 200);\n");
    s.push_str("        let (kind, ml) = m.drain_liquid(50);\n");
    s.push_str("        assert_eq!(kind, Liquid::Water);\n        assert_eq!(ml, 50);\n        assert_eq!(m.water_ml(), 150);\n    }\n}\n");
    s
}

pub(super) fn switchable_faucet(name: &str, doc: &str, flow_ml_per_sec: u32) -> String {
    let mut s = String::new();
    s.push_str(&format!("//! {doc}\n\n"));
    s.push_str("use crate::space::Vec2;\n");
    s.push_str("use crate::traits::{Liquid, LiquidSource, Switchable, TimeDecaySeconds};\n\n");
    s.push_str("#[derive(Debug, Clone, Copy)]\n");
    s.push_str(&format!("pub struct {name} {{\n    pub pos: Vec2,\n    on: bool,\n    pub seconds_running: f32,\n}}\n\n"));
    s.push_str(&format!("impl {name} {{\n"));
    s.push_str(&format!("    pub const FLOW_ML_PER_SEC: u32 = {flow_ml_per_sec};\n\n"));
    s.push_str("    pub fn new(pos: Vec2) -> Self {\n        Self { pos, on: false, seconds_running: 0.0 }\n    }\n}\n\n");
    s.push_str(&format!("impl Switchable for {name} {{\n"));
    s.push_str("    fn is_on(&self) -> bool { self.on }\n    fn toggle(&mut self) { self.on = !self.on; }\n}\n\n");
    s.push_str(&format!("impl LiquidSource for {name} {{\n"));
    s.push_str("    fn produced_liquid(&self) -> Liquid { Liquid::Water }\n");
    s.push_str("    fn is_producing(&self) -> bool { self.on }\n");
    s.push_str("    fn produce(&mut self, dt: f32) -> (Liquid, u32) {\n");
    s.push_str("        if !self.on { return (Liquid::Water, 0); }\n");
    s.push_str("        let ml = (Self::FLOW_ML_PER_SEC as f32 * dt) as u32;\n        (Liquid::Water, ml)\n    }\n}\n\n");
    s.push_str(&format!("impl TimeDecaySeconds for {name} {{\n"));
    s.push_str("    fn tick_real(&mut self, dt: f32) {\n        if self.on { self.seconds_running += dt; }\n    }\n}\n");
    s
}

pub(super) fn decorative_item(name: &str, doc: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("//! {doc}\n\n"));
    s.push_str("use crate::space::Vec2;\n\n");
    s.push_str("#[derive(Debug, Clone, Copy)]\n");
    s.push_str(&format!("pub struct {name} {{\n    pub pos: Vec2,\n}}\n\n"));
    s.push_str(&format!("impl {name} {{\n    pub fn new(pos: Vec2) -> Self {{\n        Self {{ pos }}\n    }}\n}}\n\n"));
    s.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
    s.push_str("    #[test]\n    fn stores_pos() {\n");
    s.push_str(&format!("        let x = {name}::new(Vec2::new(3.0, 4.0));\n"));
    s.push_str("        assert_eq!(x.pos, Vec2::new(3.0, 4.0));\n    }\n}\n");
    s
}

pub(super) fn simple_openable(name: &str, doc: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("//! {doc}\n\n"));
    s.push_str("use crate::space::Vec2;\n");
    s.push_str("use crate::traits::Openable;\n\n");
    s.push_str("#[derive(Debug, Clone, Copy)]\n");
    s.push_str(&format!("pub struct {name} {{\n    pub pos: Vec2,\n    open: bool,\n}}\n\n"));
    s.push_str(&format!("impl {name} {{\n    pub fn new(pos: Vec2) -> Self {{\n        Self {{ pos, open: false }}\n    }}\n}}\n\n"));
    s.push_str(&format!("impl Openable for {name} {{\n    fn is_open(&self) -> bool {{ self.open }}\n    fn toggle_open(&mut self) {{ self.open = !self.open; }}\n}}\n\n"));
    s.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
    s.push_str("    #[test]\n    fn starts_closed() {\n");
    s.push_str(&format!("        let x = {name}::new(Vec2::new(0.0, 0.0));\n"));
    s.push_str("        assert!(!x.is_open());\n    }\n\n");
    s.push_str("    #[test]\n    fn toggle_opens_and_closes() {\n");
    s.push_str(&format!("        let mut x = {name}::new(Vec2::new(0.0, 0.0));\n"));
    s.push_str("        x.toggle_open();\n        assert!(x.is_open());\n        x.toggle_open();\n        assert!(!x.is_open());\n    }\n}\n");
    s
}
