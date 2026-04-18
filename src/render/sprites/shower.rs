//! Shower sprite. Tiled back wall + pipe + shower head, animated
//! droplet stream when on.

use crate::kinds::Shower;
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};
use crate::traits::Switchable;

pub(crate) fn draw_shower(
    frame: &mut Frame, sh: &Shower, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let tile = rgb(scale(200, fog), scale(208, fog), scale(215, fog));
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end { frame.put(x as usize, y as usize, tile); }
    }
    let pipe = rgb(scale(170, fog), scale(175, fog), scale(180, fog));
    let px = (left_x + sprite_w * 0.7) as i32;
    if px >= start_x && px < end_x && frame.z_buffer[px as usize] >= dist {
        let y0 = top_y.max(0.0) as i32;
        let y1 = (top_y + sprite_h * 0.28).min(HEIGHT as f32) as i32;
        for y in y0..y1 {
            frame.put(px as usize, y as usize, pipe);
            if px + 1 < end_x { frame.put((px + 1) as usize, y as usize, pipe); }
        }
    }
    let head = rgb(scale(190, fog), scale(192, fog), scale(195, fog));
    let head_y0 = (top_y + sprite_h * 0.25) as i32;
    let head_y1 = (top_y + sprite_h * 0.32) as i32;
    let head_x0 = (left_x + sprite_w * 0.40) as i32;
    let head_x1 = (left_x + sprite_w * 0.98) as i32;
    for x in head_x0.max(start_x)..head_x1.min(end_x) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in head_y0.max(0)..head_y1.min(HEIGHT as i32) { frame.put(x as usize, y as usize, head); }
    }
    if sh.is_on() {
        let water_a = rgb(scale(130, fog), scale(180, fog), scale(220, fog));
        let water_b = rgb(scale(170, fog), scale(210, fog), scale(240, fog));
        let t = sh.seconds_running;
        let stream_x0 = (left_x + sprite_w * 0.50) as i32;
        let stream_x1 = (left_x + sprite_w * 0.90) as i32;
        let stream_y0 = (top_y + sprite_h * 0.33) as i32;
        let stream_y1 = (top_y + sprite_h * 0.95) as i32;
        for x in stream_x0.max(start_x)..stream_x1.min(end_x) {
            if frame.z_buffer[x as usize] < dist { continue; }
            for y in stream_y0.max(0)..stream_y1.min(HEIGHT as i32) {
                let phase = ((y as f32) + t * 60.0 + (x as f32) * 0.7) as i32;
                if (phase & 5) < 2 {
                    let col = if (phase & 7) < 4 { water_a } else { water_b };
                    frame.put(x as usize, y as usize, col);
                }
            }
        }
    }
}
