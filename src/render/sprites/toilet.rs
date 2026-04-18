//! Toilet sprite. Bowl + tank + seat, plus a brief water-shimmer
//! band while flushing.

use crate::kinds::{Toilet, ToiletState};
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_toilet(
    frame: &mut Frame, tl: &Toilet, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let bowl = rgb(scale(235, fog), scale(238, fog), scale(235, fog));
    let bowl_y0 = (top_y + sprite_h * 0.40) as i32;
    let bowl_y1 = (top_y + sprite_h).min(HEIGHT as f32) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in bowl_y0.max(0)..bowl_y1 { frame.put(x as usize, y as usize, bowl); }
    }
    let tank = rgb(scale(220, fog), scale(222, fog), scale(220, fog));
    let tank_inset = (sprite_w * 0.15) as i32;
    let tank_y0 = top_y.max(0.0) as i32;
    let tank_y1 = (top_y + sprite_h * 0.40) as i32;
    for x in (start_x + tank_inset)..(end_x - tank_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in tank_y0..tank_y1.min(HEIGHT as i32) { frame.put(x as usize, y as usize, tank); }
    }
    if tl.state == ToiletState::Flushing {
        let shimmer = rgb(scale(140, fog), scale(180, fog), scale(220, fog));
        let flush_y = (top_y + sprite_h * 0.20) as i32;
        for x in (start_x + tank_inset)..(end_x - tank_inset) {
            if frame.z_buffer[x as usize] < dist { continue; }
            if flush_y >= 0 && flush_y < HEIGHT as i32 {
                frame.put(x as usize, flush_y as usize, shimmer);
            }
        }
    }
    let seat = rgb(scale(200, fog), scale(205, fog), scale(200, fog));
    let seat_y = (top_y + sprite_h * 0.42) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        if seat_y >= 0 && seat_y < HEIGHT as i32 {
            frame.put(x as usize, seat_y as usize, seat);
        }
    }
}
