//! Chair sprite (generated from a SpriteSpec; rects shared with the browser).

use crate::kinds::Plant;
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_gen_chair(
    frame: &mut Frame, _obj: &Plant, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let x0 = left_x.max(0.0) as i32;
    let x1 = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if x0 >= x1 { return; }
    let c0 = rgb(scale(150, fog), scale(96, fog), scale(40, fog));
    let r0x0 = (left_x + sprite_w * 0.18) as i32;
    let r0x1 = (left_x + sprite_w * 0.4) as i32;
    let r0y0 = (top_y + sprite_h * 0.08) as i32;
    let r0y1 = (top_y + sprite_h * 0.62).min(HEIGHT as f32) as i32;
    for x in r0x0.max(x0)..r0x1.min(x1) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in r0y0.max(0)..r0y1 { frame.put(x as usize, y as usize, c0); }
    }
    let c1 = rgb(scale(165, fog), scale(108, fog), scale(48, fog));
    let r1x0 = (left_x + sprite_w * 0.18) as i32;
    let r1x1 = (left_x + sprite_w * 0.86) as i32;
    let r1y0 = (top_y + sprite_h * 0.5) as i32;
    let r1y1 = (top_y + sprite_h * 0.62).min(HEIGHT as f32) as i32;
    for x in r1x0.max(x0)..r1x1.min(x1) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in r1y0.max(0)..r1y1 { frame.put(x as usize, y as usize, c1); }
    }
    let c2 = rgb(scale(120, fog), scale(80, fog), scale(40, fog));
    let r2x0 = (left_x + sprite_w * 0.2) as i32;
    let r2x1 = (left_x + sprite_w * 0.3) as i32;
    let r2y0 = (top_y + sprite_h * 0.62) as i32;
    let r2y1 = (top_y + sprite_h * 0.96).min(HEIGHT as f32) as i32;
    for x in r2x0.max(x0)..r2x1.min(x1) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in r2y0.max(0)..r2y1 { frame.put(x as usize, y as usize, c2); }
    }
    let c3 = rgb(scale(120, fog), scale(80, fog), scale(40, fog));
    let r3x0 = (left_x + sprite_w * 0.74) as i32;
    let r3x1 = (left_x + sprite_w * 0.84) as i32;
    let r3y0 = (top_y + sprite_h * 0.62) as i32;
    let r3y1 = (top_y + sprite_h * 0.96).min(HEIGHT as f32) as i32;
    for x in r3x0.max(x0)..r3x1.min(x1) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in r3y0.max(0)..r3y1 { frame.put(x as usize, y as usize, c3); }
    }
}
