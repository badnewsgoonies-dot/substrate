//! Lamp sprite. Shaft + base + shade, with warm halo band when on.

use crate::kinds::Lamp;
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};
use crate::traits::Switchable;

pub(crate) fn draw_lamp(
    frame: &mut Frame, lamp: &Lamp, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let shaft_col = rgb(scale(80, fog), scale(70, fog), scale(60, fog));
    let shaft_x = (left_x + sprite_w * 0.5) as i32;
    let shaft_y0 = (top_y + sprite_h * 0.35) as i32;
    let shaft_y1 = (top_y + sprite_h * 0.92) as i32;
    if shaft_x >= start_x && shaft_x < end_x && frame.z_buffer[shaft_x as usize] >= dist {
        for y in shaft_y0.max(0)..shaft_y1.min(HEIGHT as i32) {
            frame.put(shaft_x as usize, y as usize, shaft_col);
            if shaft_x + 1 < end_x && (shaft_x + 1) < WIDTH as i32 {
                frame.put((shaft_x + 1) as usize, y as usize, shaft_col);
            }
        }
    }
    let base_col = rgb(scale(70, fog), scale(60, fog), scale(50, fog));
    let base_y0 = (top_y + sprite_h * 0.90) as i32;
    let base_y1 = (top_y + sprite_h).min(HEIGHT as f32) as i32;
    let base_inset = (sprite_w * 0.25) as i32;
    for x in (start_x + base_inset)..(end_x - base_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in base_y0.max(0)..base_y1 { frame.put(x as usize, y as usize, base_col); }
    }
    let shade_on = Switchable::is_on(lamp);
    let shade_rgb = if shade_on { (255u8, 235u8, 180u8) } else { (180u8, 160u8, 130u8) };
    let shade_col = rgb(scale(shade_rgb.0, fog), scale(shade_rgb.1, fog), scale(shade_rgb.2, fog));
    let shade_y0 = (top_y + sprite_h * 0.05) as i32;
    let shade_y1 = (top_y + sprite_h * 0.35) as i32;
    let shade_inset = (sprite_w * 0.10) as i32;
    for x in (start_x + shade_inset)..(end_x - shade_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in shade_y0.max(0)..shade_y1.min(HEIGHT as i32) {
            frame.put(x as usize, y as usize, shade_col);
        }
    }
    if shade_on {
        let halo = rgb(scale(255, fog), scale(245, fog), scale(200, fog));
        let halo_y = (top_y + sprite_h * 0.34) as i32;
        for x in start_x..end_x {
            if frame.z_buffer[x as usize] < dist { continue; }
            if halo_y >= 0 && halo_y < HEIGHT as i32 {
                frame.put(x as usize, halo_y as usize, halo);
            }
        }
    }
}
