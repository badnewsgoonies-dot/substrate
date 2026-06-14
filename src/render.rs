//! Render layer.
//!
//! Layout:
//! - `render.rs` (this file): Frame struct, frame orchestration,
//!   draw_npc, draw_object dispatcher, ambient/crosshair/vignette
//!   post-processing, primitives (rgb, scale, draw_flat), and small
//!   color helpers for kinds that draw via draw_flat.
//! - `render/sprites/*.rs`: one module per kind with a non-trivial
//!   sprite. Adding a new kind's sprite is a new file + one line in
//!   sprites/mod.rs + two arms in the draw_object match below.
//!   Nothing else has to grow.

use crate::body::PlayerBody;
use crate::interact::Target;
use crate::kinds::{LightContribution, Object};
use crate::objects::Objects;
use crate::people::Npc;
use crate::space::{cast_ray, RaySide, Tile, TileGrid, Vec2};
use crate::texture::{sample, Material, Surface};
use crate::time::InGameClock;

mod sprites;

/// The world's art budget: each tile kind -> a procedural material
/// (a few bytes). Walls are textured by sampling this at the ray hit,
/// so there are no stored wall textures anywhere.
fn material_for(tile: Tile) -> Material {
    match tile {
        Tile::BEDROOM => Material { surface: Surface::Wood, seed: 7, tint: (196, 150, 96) },
        Tile::KITCHEN => Material { surface: Surface::Brick, seed: 19, tint: (150, 120, 110) },
        Tile::DOORFRAME => Material { surface: Surface::Plaster, seed: 3, tint: (172, 158, 138) },
        Tile::BATHROOM => Material { surface: Surface::Tile, seed: 11, tint: (210, 222, 230) },
        _ => Material { surface: Surface::Plaster, seed: 1, tint: (180, 180, 180) },
    }
}

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 500;

pub struct Frame {
    pub pixels: Vec<u32>,
    pub z_buffer: Vec<f32>,
}

impl Frame {
    pub fn new() -> Self { Self { pixels: vec![0; WIDTH * HEIGHT], z_buffer: vec![0.0; WIDTH] } }
    #[inline]
    pub fn put(&mut self, x: usize, y: usize, rgb: u32) {
        if x < WIDTH && y < HEIGHT { self.pixels[y * WIDTH + x] = rgb; }
    }
    pub fn fill_column(&mut self, x: usize, y0: i32, y1: i32, rgb: u32) {
        let y0 = y0.max(0) as usize;
        let y1 = (y1.max(0) as usize).min(HEIGHT);
        for y in y0..y1 { self.pixels[y * WIDTH + x] = rgb; }
    }
}
impl Default for Frame { fn default() -> Self { Self::new() } }

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
pub fn scale(c: u8, fog: f32) -> u8 { (c as f32 * fog).clamp(0.0, 255.0) as u8 }

pub fn render_frame(
    frame: &mut Frame, grid: &TileGrid, body: &PlayerBody, npcs: &[Npc],
    objects: &Objects, clock: &InGameClock, fov_tan: f32,
    target: Option<Target>, interact_pulse: f32,
) {
    // Default behavior: materials come from the built-in table.
    render_frame_with(frame, grid, body, npcs, objects, clock, fov_tan,
                      target, interact_pulse, &material_for);
}

/// Same renderer, but the wall material for each tile is supplied by a
/// lookup function. This is the seam where a *scene coordinate* drives
/// the art: pass a closure backed by a parsed `Scene` and the raycaster
/// textures itself from an addressable description instead of hardcode.
pub fn render_frame_with(
    frame: &mut Frame, grid: &TileGrid, body: &PlayerBody, npcs: &[Npc],
    objects: &Objects, clock: &InGameClock, fov_tan: f32,
    target: Option<Target>, interact_pulse: f32,
    material_lookup: &dyn Fn(Tile) -> Material,
) {
    let horizon = (HEIGHT as f32 * 0.5 + body.bob * HEIGHT as f32) as i32;
    let ambient = objects.aggregate_ambient(clock);

    for y in 0..HEIGHT {
        let rgb_val = if (y as i32) < horizon {
            let t = (horizon - y as i32) as f32 / horizon.max(1) as f32;
            rgb((18.0 + t * 16.0) as u8, (14.0 + t * 10.0) as u8, (10.0 + t * 6.0) as u8)
        } else {
            let t = (y as i32 - horizon) as f32 / (HEIGHT as i32 - horizon).max(1) as f32;
            rgb((16.0 + t * 12.0) as u8, (11.0 + t * 8.0) as u8, (7.0 + t * 6.0) as u8)
        };
        for x in 0..WIDTH { frame.pixels[y * WIDTH + x] = rgb_val; }
    }

    let dir = body.forward();
    let plane = Vec2::new(-dir.y, dir.x) * fov_tan;
    for x in 0..WIDTH {
        let camera_x = 2.0 * x as f32 / WIDTH as f32 - 1.0;
        let hit = cast_ray(grid, body.pos, dir, plane, camera_x, 64);
        frame.z_buffer[x] = hit.perp_dist;
        let line_height = (HEIGHT as f32 / hit.perp_dist) as i32;
        let y0 = horizon - line_height / 2;
        let y1 = horizon + line_height / 2;

        // ART FROM MATH: the wall's color comes from a procedural material
        // sampled at the ray-hit coordinate, per pixel — no stored texture.
        let mat = material_lookup(hit.tile);
        // texture u runs along the wall (world units) from the hit point
        let wall_u = match hit.side {
            RaySide::EastWest => hit.hit_x,
            RaySide::NorthSouth => hit.hit_y,
        };
        let side_dim = if hit.side == RaySide::EastWest { 0.72 } else { 1.0 };
        let fog = (1.0 / (1.0 + hit.perp_dist * 0.35)).clamp(0.0, 1.0);
        let denom = (y1 - y0).max(1) as f32;
        let yv0 = y0.max(0);
        let yv1 = y1.min(HEIGHT as i32);
        for y in yv0..yv1 {
            // texture v runs down the wall slice (3 world units tall)
            let wall_v = (y - y0) as f32 / denom * 3.0;
            let (cr, cg, cb) = sample(&mat, wall_u, wall_v);
            let lit = side_dim * fog;
            frame.pixels[y as usize * WIDTH + x] =
                rgb(scale(cr, lit), scale(cg, lit), scale(cb, lit));
        }
    }

    for obj in objects.iter() { draw_object(frame, body, obj, clock, fov_tan, horizon); }
    for npc in npcs { draw_npc(frame, body, npc, fov_tan, horizon); }

    apply_ambient_tint(frame, &ambient);
    draw_crosshair(frame, target, interact_pulse);
    apply_vignette(frame);
}

fn apply_ambient_tint(frame: &mut Frame, ambient: &LightContribution) {
    if ambient.intensity <= 0.0 { return; }
    let tint_r = ambient.rgb.0 as f32 * ambient.intensity * 0.06;
    let tint_g = ambient.rgb.1 as f32 * ambient.intensity * 0.06;
    let tint_b = ambient.rgb.2 as f32 * ambient.intensity * 0.06;
    for p in frame.pixels.iter_mut() {
        let cr = ((*p >> 16) & 0xff) as f32;
        let cg = ((*p >> 8) & 0xff) as f32;
        let cb = (*p & 0xff) as f32;
        *p = rgb(
            (cr + tint_r).min(255.0) as u8,
            (cg + tint_g).min(255.0) as u8,
            (cb + tint_b).min(255.0) as u8,
        );
    }
}

fn draw_crosshair(frame: &mut Frame, target: Option<Target>, interact_pulse: f32) {
    let cx_i = WIDTH / 2;
    let cy_i = HEIGHT / 2;
    let target_strength = if target.is_some() { 0.9 } else { 0.25 };
    let strength = (target_strength + interact_pulse * 0.6).min(1.2);
    let dot_color = (220.0 * strength).min(255.0) as u8;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let x = cx_i as i32 + dx;
            let y = cy_i as i32 + dy;
            if x >= 0 && y >= 0 && (x as usize) < WIDTH && (y as usize) < HEIGHT {
                let idx = y as usize * WIDTH + x as usize;
                let p = frame.pixels[idx];
                let nr = (((p >> 16) & 0xff) as u16 + dot_color as u16).min(255) as u8;
                let ng = (((p >> 8) & 0xff) as u16 + dot_color as u16).min(255) as u8;
                let nb = ((p & 0xff) as u16 + dot_color as u16).min(255) as u8;
                frame.pixels[idx] = rgb(nr, ng, nb);
            }
        }
    }
}

fn apply_vignette(frame: &mut Frame) {
    let cx = WIDTH as f32 * 0.5;
    let cy = HEIGHT as f32 * 0.5;
    let max_r = (cx * cx + cy * cy).sqrt();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt() / max_r;
            let darken = (d.powf(2.0) * 0.55).min(0.75);
            let idx = y * WIDTH + x;
            let p = frame.pixels[idx];
            let cr = ((p >> 16) & 0xff) as f32 * (1.0 - darken);
            let cg = ((p >> 8) & 0xff) as f32 * (1.0 - darken);
            let cb = (p & 0xff) as f32 * (1.0 - darken);
            frame.pixels[idx] = rgb(cr as u8, cg as u8, cb as u8);
        }
    }
}

fn angle_and_dist(body: &PlayerBody, pos: Vec2) -> Option<(f32, f32)> {
    let dx = pos.x - body.pos.x;
    let dy = pos.y - body.pos.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.2 { return None; }
    let world_angle = dy.atan2(dx);
    let mut rel = world_angle - body.angle;
    while rel > std::f32::consts::PI { rel -= 2.0 * std::f32::consts::PI; }
    while rel < -std::f32::consts::PI { rel += 2.0 * std::f32::consts::PI; }
    Some((rel, dist))
}

fn draw_npc(frame: &mut Frame, body: &PlayerBody, npc: &Npc, fov_tan: f32, horizon: i32) {
    let (rel, dist) = match angle_and_dist(body, npc.pos) { Some(v) => v, None => return };
    let half_fov = fov_tan.atan();
    if rel.abs() > half_fov + 0.3 { return; }
    let screen_x = WIDTH as f32 * 0.5 + (rel.tan() / fov_tan) * (WIDTH as f32 * 0.5);
    let sprite_h = (HEIGHT as f32 / dist * 0.95).min(HEIGHT as f32 * 2.0);
    let sprite_w = sprite_h * 0.45;
    let top_y = horizon as f32 - sprite_h * 0.55;
    let left_x = screen_x - sprite_w * 0.5;
    let fog = (1.0 / (1.0 + dist * 0.35)).clamp(0.0, 1.0);
    let skin = rgb(scale(214, fog), scale(186, fog), scale(156, fog));
    let shirt = rgb(scale(92, fog), scale(78, fog), scale(110, fog));
    let hair = rgb(scale(52, fog), scale(36, fog), scale(24, fog));
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let u = (x as f32 - left_x) / sprite_w;
        if !(0.0..=1.0).contains(&u) { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end {
            let v = (y as f32 - top_y) / sprite_h;
            let hx = (u - 0.5) / 0.15;
            let hy = (v - 0.13) / 0.13;
            let in_head = hx * hx + hy * hy <= 1.0;
            let bx = (u - 0.5) / 0.28;
            let by = (v - 0.62) / 0.38;
            let in_body = (bx * bx + by * by <= 1.0) && v > 0.26;
            let color = if in_head && v < 0.10 { hair }
                else if in_head { skin }
                else if in_body { shirt }
                else { continue };
            frame.put(x as usize, y as usize, color);
        }
    }
}

fn draw_object(
    frame: &mut Frame, body: &PlayerBody, obj: &Object,
    _clock: &InGameClock, fov_tan: f32, horizon: i32,
) {
    let (rel, dist) = match angle_and_dist(body, obj.pos()) { Some(v) => v, None => return };
    let half_fov = fov_tan.atan();
    if rel.abs() > half_fov + 0.3 { return; }
    let screen_x = WIDTH as f32 * 0.5 + (rel.tan() / fov_tan) * (WIDTH as f32 * 0.5);
    let (base_h, aspect) = match obj {
        Object::Bed(_) => (0.5, 1.6),
        Object::CoffeeMaker(_) => (0.55, 0.5),
        Object::Book(_) => (0.35, 0.7),
        Object::Window(_) => (1.1, 0.9),
        Object::Fridge(_) => (1.3, 0.55),
        Object::Stove(_) => (0.9, 1.0),
        Object::Sink(_) => (0.5, 1.1),
        Object::Toilet(_) => (0.7, 0.55),
        Object::Shower(_) => (1.25, 0.5),
        Object::Bathtub(_) => (0.45, 1.4),
        Object::BathSink(_) => (0.45, 0.85),
        Object::Lamp(_) => (0.95, 0.4),
        Object::Nightstand(_) => (0.4, 0.6),
        Object::Dresser(_) => (0.75, 1.0),
        Object::Mug(_) => (0.12, 0.55),
        Object::Plant(_) => (0.55, 0.7),
        Object::Painting(_) => (0.6, 1.2),
        Object::Diary(_) => (0.15, 0.75),
    };
    let sprite_h = (HEIGHT as f32 / dist * base_h).min(HEIGHT as f32 * 2.0);
    let sprite_w = sprite_h * aspect;
    let top_y = horizon as f32 - sprite_h * 0.15;
    let left_x = screen_x - sprite_w * 0.5;
    let fog = (1.0 / (1.0 + dist * 0.35)).clamp(0.0, 1.0);

    match obj {
        Object::Fridge(f) => sprites::draw_fridge(frame, f, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Stove(s) => sprites::draw_stove(frame, s, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Sink(s) => sprites::draw_sink(frame, s, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::CoffeeMaker(cm) => sprites::draw_coffee_maker(frame, cm, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Toilet(t) => sprites::draw_toilet(frame, t, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Shower(sh) => sprites::draw_shower(frame, sh, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Bathtub(b) => sprites::draw_bathtub(frame, b, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::BathSink(bs) => sprites::draw_bath_sink(frame, bs, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Lamp(l) => sprites::draw_lamp(frame, l, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Nightstand(_) => draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, (130, 95, 65)),
        Object::Dresser(_) => sprites::draw_dresser(frame, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Bed(b) => draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, bed_color(b)),
        Object::Book(b) => draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, book_color(b)),
        Object::Window(_) => draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, (220, 215, 180)),
        Object::Mug(_) => draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, (235, 225, 210)),
        Object::Plant(_) => sprites::draw_plant(frame, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Painting(_) => sprites::draw_painting(frame, fog, top_y, left_x, sprite_w, sprite_h, dist),
        Object::Diary(d) => draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, diary_color(d)),
    }
}

/// Rectangular fill of `color`, fog-scaled, z-clipped against the
/// frame's depth buffer. Shared by the simple-kind arms in
/// `draw_object` (Bed, Book, Window, Mug, Diary, Nightstand) and by
/// `sprites::draw_coffee_maker` which is itself just a color pick
/// plus a flat fill. `pub(crate)` so sprite modules can call it.
pub(crate) fn draw_flat(
    frame: &mut Frame, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
    color: (u8, u8, u8),
) {
    let (r, g, b) = color;
    let fill = rgb(scale(r, fog), scale(g, fog), scale(b, fog));
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end { frame.put(x as usize, y as usize, fill); }
    }
}

fn bed_color(b: &crate::kinds::Bed) -> (u8, u8, u8) {
    if b.is_unmade() { (140, 110, 90) } else { (180, 150, 130) }
}

fn book_color(b: &crate::kinds::Book) -> (u8, u8, u8) {
    use crate::traits::Openable;
    if b.is_open() { (200, 190, 170) } else { (90, 60, 40) }
}

/// Diary color: closed = pressed-leather brown; open = a softer cream
/// to hint at the exposed pages. Readable as "there is something
/// private here" at a glance.
fn diary_color(d: &crate::kinds::Diary) -> (u8, u8, u8) {
    use crate::traits::Openable;
    if d.is_open() { (215, 200, 170) } else { (75, 50, 35) }
}
