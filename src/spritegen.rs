//! Sprite generation — the substrate authoring a piece of the renderer.
//!
//! The measurement (the three M's over the harvested toolbox) flagged the
//! `draw_*` sprite family as a tight cluster. They turned out to be
//! hand-tuned siblings, not template instances — so rather than retrofit
//! a generator onto hand-authored code, this generates NEW sprites from a
//! clean declarative `SpriteSpec`, the same spec->source->golden-test
//! discipline `kindgen` uses for kinds. A generated sprite is real engine
//! code: it compiles, it draws, and its source is byte-exact reproducible
//! from its spec (proven by `emit_sprite_source(&SPEC) == include_str!`).

/// One horizontal band of the billboard, as a fraction of sprite height,
/// filled with an (r,g,b) colour. Bands are painted top-to-bottom.
#[derive(Debug, Clone, Copy)]
pub struct Band {
    pub v0: f32,
    pub v1: f32,
    pub rgb: (u8, u8, u8),
    /// horizontal inset as a fraction of sprite width (0.0 = full width)
    pub inset: f32,
}

/// A declarative sprite: a name + a stack of coloured bands. This is the
/// whole "art recipe" for a billboard object — a few bytes, like a kind spec.
#[derive(Debug, Clone)]
pub struct SpriteSpec {
    pub name: &'static str,
    pub kind_type: &'static str, // the Rust type it draws (e.g. "Crate")
    pub doc: &'static str,
    pub bands: &'static [Band],
}

/// Emit a complete, compileable `draw_<name>` sprite module from the spec.
pub fn emit_sprite_source(spec: &SpriteSpec) -> String {
    let mut s = String::new();
    s.push_str(&format!("//! {}\n\n", spec.doc));
    s.push_str(&format!("use crate::kinds::{};\n", spec.kind_type));
    s.push_str("use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};\n\n");
    s.push_str(&format!("pub(crate) fn draw_{}(\n", spec.name));
    s.push_str(&format!("    frame: &mut Frame, _obj: &{}, fog: f32,\n", spec.kind_type));
    s.push_str("    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,\n");
    s.push_str(") {\n");
    s.push_str("    let start_x = left_x.max(0.0) as i32;\n");
    s.push_str("    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;\n");
    s.push_str("    if start_x >= end_x { return; }\n");
    for (i, b) in spec.bands.iter().enumerate() {
        s.push_str(&format!("    let band{i} = rgb(scale({}, fog), scale({}, fog), scale({}, fog));\n",
            b.rgb.0, b.rgb.1, b.rgb.2));
        s.push_str(&format!("    let b{i}_y0 = (top_y + sprite_h * {:?}) as i32;\n", b.v0));
        s.push_str(&format!("    let b{i}_y1 = (top_y + sprite_h * {:?}).min(HEIGHT as f32) as i32;\n", b.v1));
        if b.inset > 0.0 {
            s.push_str(&format!("    let b{i}_inset = (sprite_w * {:?}) as i32;\n", b.inset));
            s.push_str(&format!("    for x in (start_x + b{i}_inset)..(end_x - b{i}_inset) {{\n"));
        } else {
            s.push_str(&format!("    for x in start_x..end_x {{\n"));
        }
        s.push_str("        if frame.z_buffer[x as usize] < dist { continue; }\n");
        s.push_str(&format!("        for y in b{i}_y0.max(0)..b{i}_y1 {{ frame.put(x as usize, y as usize, band{i}); }}\n"));
        s.push_str("    }\n");
    }
    s.push_str("}\n");
    s
}

/// A generated crate sprite spec (used by the game).
pub const CRATE_SPEC: SpriteSpec = SpriteSpec {
    name: "gen_crate",
    kind_type: "Plant",
    doc: "Wooden crate sprite (generated from a SpriteSpec).",
    bands: &[
        Band { v0: 0.10, v1: 0.30, rgb: (120, 92, 56), inset: 0.05 },
        Band { v0: 0.30, v1: 0.95, rgb: (146, 110, 70), inset: 0.0 },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_sprite_is_wellformed() {
        let src = emit_sprite_source(&CRATE_SPEC);
        assert!(src.contains("pub(crate) fn draw_gen_crate("));
        assert!(src.contains("use crate::kinds::Plant;"));
        assert!(src.contains("frame.z_buffer[x as usize] < dist"));
        // two bands -> two band colours
        assert!(src.contains("let band0 ="));
        assert!(src.contains("let band1 ="));
    }

    #[test]
    fn spec_changes_propagate_to_source() {
        let spec = SpriteSpec {
            name: "widget", kind_type: "Plant", doc: "d",
            bands: &[Band { v0: 0.0, v1: 1.0, rgb: (1, 2, 3), inset: 0.0 }],
        };
        let src = emit_sprite_source(&spec);
        assert!(src.contains("pub(crate) fn draw_widget("));
        assert!(src.contains("scale(1, fog), scale(2, fog), scale(3, fog)"));
    }

    // GOLDEN TEST: the generator reproduces the on-disk generated sprite
    // byte-exact. This is the proof the substrate authored it.
    #[test]
    fn golden_matches_on_disk() {
        assert_eq!(emit_sprite_source(&CRATE_SPEC),
                   include_str!("render/sprites/gen_crate.rs"));
    }
}
