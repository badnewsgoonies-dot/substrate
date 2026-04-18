//! Stove sprite. Body + cooktop band + 4 burners colored by level/warmth,
//! oven door with handle below.

use crate::kinds::{BurnerLevel, Stove};
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_stove(
    frame: &mut Frame, st: &Stove, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let body_col = rgb(scale(150, fog), scale(148, fog), scale(152, fog));
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end { frame.put(x as usize, y as usize, body_col); }
    }
    let cooktop = rgb(scale(35, fog), scale(32, fog), scale(32, fog));
    let top_band_y0 = (top_y + sprite_h * 0.06) as i32;
    let top_band_y1 = (top_y + sprite_h * 0.46) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in top_band_y0.max(0)..top_band_y1.min(HEIGHT as i32) {
            frame.put(x as usize, y as usize, cooktop);
        }
    }
    for (i, b) in st.burners.burners.iter().enumerate() {
        let col_i = (i as u32) & 1;
        let row_i = ((i as u32) >> 1) & 1;
        let bx0 = left_x + sprite_w * (0.15 + col_i as f32 * 0.45);
        let bx1 = bx0 + sprite_w * 0.30;
        let by0 = top_y + sprite_h * (0.12 + row_i as f32 * 0.18);
        let by1 = by0 + sprite_h * 0.14;
        let (br, bg, bb) = if b.burned { (60u8, 55u8, 50u8) } else {
            match b.level {
                BurnerLevel::Off => if b.warmth > 0 {
                    let t = (b.warmth as f32 / 120.0).clamp(0.0, 1.0);
                    ((80.0 + t * 80.0) as u8, (40.0 + t * 20.0) as u8, 35u8)
                } else { (50u8, 45u8, 45u8) },
                BurnerLevel::Low => (200u8, 90u8, 40u8),
                BurnerLevel::High => (250u8, 170u8, 60u8),
            }
        };
        let col = rgb(scale(br, fog), scale(bg, fog), scale(bb, fog));
        for x in (bx0 as i32).max(start_x)..(bx1 as i32).min(end_x) {
            if frame.z_buffer[x as usize] < dist { continue; }
            for y in (by0 as i32).max(0)..(by1 as i32).min(HEIGHT as i32) {
                frame.put(x as usize, y as usize, col);
            }
        }
    }
    let door = rgb(scale(60, fog), scale(58, fog), scale(62, fog));
    let door_y0 = (top_y + sprite_h * 0.52) as i32;
    let door_y1 = (top_y + sprite_h * 0.95) as i32;
    let door_inset = (sprite_w * 0.08) as i32;
    for x in (start_x + door_inset)..(end_x - door_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in door_y0.max(0)..door_y1.min(HEIGHT as i32) {
            frame.put(x as usize, y as usize, door);
        }
    }
    let handle = rgb(scale(180, fog), scale(180, fog), scale(175, fog));
    let handle_y = (top_y + sprite_h * 0.58) as i32;
    for x in (start_x + door_inset + 2)..(end_x - door_inset - 2) {
        if frame.z_buffer[x as usize] < dist { continue; }
        if handle_y >= 0 && handle_y < HEIGHT as i32 {
            frame.put(x as usize, handle_y as usize, handle);
            if handle_y + 1 < HEIGHT as i32 { frame.put(x as usize, (handle_y + 1) as usize, handle); }
        }
    }
}
