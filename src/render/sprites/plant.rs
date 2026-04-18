//! Potted plant sprite. Pot rectangle at the base + elliptical
//! canopy with a diagonal highlight.

use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_plant(
    frame: &mut Frame, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let pot_col = rgb(scale(130, fog), scale(80, fog), scale(55, fog));
    let pot_y0 = (top_y + sprite_h * 0.72) as i32;
    let pot_y1 = (top_y + sprite_h).min(HEIGHT as f32) as i32;
    let pot_inset = (sprite_w * 0.12) as i32;
    for x in (start_x + pot_inset)..(end_x - pot_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in pot_y0.max(0)..pot_y1 { frame.put(x as usize, y as usize, pot_col); }
    }
    let canopy = rgb(scale(80, fog), scale(140, fog), scale(60, fog));
    let highlight = rgb(scale(120, fog), scale(180, fog), scale(90, fog));
    let cx = (left_x + sprite_w * 0.5) as f32;
    let cy = (top_y + sprite_h * 0.38) as f32;
    let rx = sprite_w * 0.45;
    let ry = sprite_h * 0.38;
    let y_start = top_y.max(0.0) as i32;
    let y_end = (top_y + sprite_h * 0.75) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let dx = (x as f32 - cx) / rx;
        for y in y_start.max(0)..y_end.min(HEIGHT as i32) {
            let dy = (y as f32 - cy) / ry;
            let r2 = dx * dx + dy * dy;
            if r2 > 1.0 { continue; }
            let col = if dx < -0.2 && dy < -0.2 && r2 < 0.7 { highlight } else { canopy };
            frame.put(x as usize, y as usize, col);
        }
    }
}
