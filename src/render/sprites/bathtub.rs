//! Bathtub sprite. Shell + inset basin + water fill proportional to
//! fill_fraction + faucet nub.

use crate::kinds::Bathtub;
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_bathtub(
    frame: &mut Frame, bt: &Bathtub, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let body_col = rgb(scale(230, fog), scale(232, fog), scale(230, fog));
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end { frame.put(x as usize, y as usize, body_col); }
    }
    let basin = rgb(scale(245, fog), scale(248, fog), scale(248, fog));
    let inset = (sprite_w * 0.06) as i32;
    let basin_y0 = (top_y + sprite_h * 0.18) as i32;
    let basin_y1 = (top_y + sprite_h * 0.90) as i32;
    for x in (start_x + inset)..(end_x - inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in basin_y0.max(0)..basin_y1.min(HEIGHT as i32) {
            frame.put(x as usize, y as usize, basin);
        }
    }
    let fill = bt.fill_fraction();
    if fill > 0.01 {
        let water_col = rgb(scale(100, fog), scale(160, fog), scale(205, fog));
        let water_top = basin_y1 - ((basin_y1 - basin_y0) as f32 * fill) as i32;
        for x in (start_x + inset)..(end_x - inset) {
            if frame.z_buffer[x as usize] < dist { continue; }
            for y in water_top.max(basin_y0)..basin_y1.min(HEIGHT as i32) {
                frame.put(x as usize, y as usize, water_col);
            }
        }
    }
    let faucet = rgb(scale(160, fog), scale(165, fog), scale(170, fog));
    let fx = start_x + (inset / 2).max(1);
    if fx >= start_x && fx < end_x && frame.z_buffer[fx as usize] >= dist {
        let fy0 = (top_y + sprite_h * 0.10) as i32;
        let fy1 = (top_y + sprite_h * 0.20) as i32;
        for y in fy0.max(0)..fy1.min(HEIGHT as i32) {
            frame.put(fx as usize, y as usize, faucet);
            if fx + 1 < end_x { frame.put((fx + 1) as usize, y as usize, faucet); }
        }
    }
}
