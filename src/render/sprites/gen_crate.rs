//! Wooden crate sprite (generated from a SpriteSpec).

use crate::kinds::Plant;
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_gen_crate(
    frame: &mut Frame, _obj: &Plant, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let band0 = rgb(scale(120, fog), scale(92, fog), scale(56, fog));
    let b0_y0 = (top_y + sprite_h * 0.1) as i32;
    let b0_y1 = (top_y + sprite_h * 0.3).min(HEIGHT as f32) as i32;
    let b0_inset = (sprite_w * 0.05) as i32;
    for x in (start_x + b0_inset)..(end_x - b0_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in b0_y0.max(0)..b0_y1 { frame.put(x as usize, y as usize, band0); }
    }
    let band1 = rgb(scale(146, fog), scale(110, fog), scale(70, fog));
    let b1_y0 = (top_y + sprite_h * 0.3) as i32;
    let b1_y1 = (top_y + sprite_h * 0.95).min(HEIGHT as f32) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in b1_y0.max(0)..b1_y1 { frame.put(x as usize, y as usize, band1); }
    }
}
