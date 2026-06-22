//! Sprite generation — the substrate authoring a piece of the renderer.
//!
//! A sprite is a stack of coloured RECTANGLES in normalized billboard
//! coords (u,v in 0..1) — expressive enough for a real chair (back, seat,
//! legs), and the SAME representation the browser raycaster renders from
//! the scene coordinate. So one sprite representation drives both the
//! compile-time Rust draw code (generated + golden-tested here) and the
//! runtime browser sprites. `kindgen`'s spec->source->golden discipline,
//! applied to rendering.

/// One rectangle of the billboard, in normalized (u,v) coords, filled rgb.
/// Painted in order; later rects draw over earlier ones.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub u0: f32,
    pub u1: f32,
    pub v0: f32,
    pub v1: f32,
    pub rgb: (u8, u8, u8),
}

/// A declarative sprite: name + the Rust type it draws + a stack of rects.
#[derive(Debug, Clone)]
pub struct SpriteSpec {
    pub name: &'static str,
    pub kind_type: &'static str,
    pub doc: &'static str,
    pub rects: &'static [Rect],
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
    s.push_str("    let x0 = left_x.max(0.0) as i32;\n");
    s.push_str("    let x1 = (left_x + sprite_w).min(WIDTH as f32) as i32;\n");
    s.push_str("    if x0 >= x1 { return; }\n");
    for (i, r) in spec.rects.iter().enumerate() {
        s.push_str(&format!("    let c{i} = rgb(scale({}, fog), scale({}, fog), scale({}, fog));\n",
            r.rgb.0, r.rgb.1, r.rgb.2));
        s.push_str(&format!("    let r{i}x0 = (left_x + sprite_w * {:?}) as i32;\n", r.u0));
        s.push_str(&format!("    let r{i}x1 = (left_x + sprite_w * {:?}) as i32;\n", r.u1));
        s.push_str(&format!("    let r{i}y0 = (top_y + sprite_h * {:?}) as i32;\n", r.v0));
        s.push_str(&format!("    let r{i}y1 = (top_y + sprite_h * {:?}).min(HEIGHT as f32) as i32;\n", r.v1));
        s.push_str(&format!("    for x in r{i}x0.max(x0)..r{i}x1.min(x1) {{\n"));
        s.push_str("        if frame.z_buffer[x as usize] < dist { continue; }\n");
        s.push_str(&format!("        for y in r{i}y0.max(0)..r{i}y1 {{ frame.put(x as usize, y as usize, c{i}); }}\n"));
        s.push_str("    }\n");
    }
    s.push_str("}\n");
    s
}

/// The chair sprite — the SAME rects the browser scene-coordinate carries.
pub const CHAIR_SPEC: SpriteSpec = SpriteSpec {
    name: "gen_chair",
    kind_type: "Chair",
    doc: "Chair sprite (generated from a SpriteSpec; rects shared with the browser).",
    rects: &[
        Rect { u0: 0.18, u1: 0.40, v0: 0.08, v1: 0.62, rgb: (150, 96, 40) },
        Rect { u0: 0.18, u1: 0.86, v0: 0.50, v1: 0.62, rgb: (165, 108, 48) },
        Rect { u0: 0.20, u1: 0.30, v0: 0.62, v1: 0.96, rgb: (120, 80, 40) },
        Rect { u0: 0.74, u1: 0.84, v0: 0.62, v1: 0.96, rgb: (120, 80, 40) },
    ],
};

/// Serialize a spec to the scene-coordinate `spr` line — the shared format
/// the Python substrate emits and the browser parses. This is the bridge:
/// the generator and the runtime sprites speak the same representation.
pub fn spec_to_scene_line(spec: &SpriteSpec, scene_kind: &str) -> String {
    let mut s = format!("spr {scene_kind}");
    for r in spec.rects {
        s.push_str(&format!(" {:?}:{:?}:{:?}:{:?}:{},{},{}",
            r.u0, r.u1, r.v0, r.v1, r.rgb.0, r.rgb.1, r.rgb.2));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_sprite_is_wellformed() {
        let src = emit_sprite_source(&CHAIR_SPEC);
        assert!(src.contains("pub(crate) fn draw_gen_chair("));
        assert!(src.contains("frame.z_buffer[x as usize] < dist"));
        assert!(src.contains("let c0 ="));
        assert!(src.contains("let c3 ="));   // four rects
    }

    #[test]
    fn scene_line_matches_browser_format() {
        // the spec serializes to the same `spr` representation the browser reads
        let line = spec_to_scene_line(&CHAIR_SPEC, "chair");
        assert!(line.starts_with("spr chair "));
        assert!(line.contains("0.18:0.4:0.08:0.62:150,96,40"));
    }

    // GOLDEN: the generator reproduces the on-disk generated sprite byte-exact.
    #[test]
    fn golden_matches_on_disk() {
        assert_eq!(emit_sprite_source(&CHAIR_SPEC),
                   include_str!("render/sprites/gen_chair.rs"));
    }
}
