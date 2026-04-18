//! Dresser sprite. Wood body with four horizontal drawer-divider
//! lines and a center-column of drawer pulls.

use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_dresser(
    frame: &mut Frame, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let body_col = rgb(scale(100, fog), scale(70, fog), scale(45, fog));
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end { frame.put(x as usize, y as usize, body_col); }
    }
    let line_col = rgb(scale(60, fog), scale(40, fog), scale(25, fog));
    for i in 1..4 {
        let ly = (top_y + sprite_h * (i as f32 * 0.25)) as i32;
        if ly < 0 || ly >= HEIGHT as i32 { continue; }
        for x in start_x..end_x {
            if frame.z_buffer[x as usize] < dist { continue; }
            frame.put(x as usize, ly as usize, line_col);
        }
    }
    let pull_col = rgb(scale(180, fog), scale(160, fog), scale(120, fog));
    let pull_x = (left_x + sprite_w * 0.5) as i32;
    if pull_x >= start_x && pull_x < end_x && frame.z_buffer[pull_x as usize] >= dist {
        for i in 0..4 {
            let py = (top_y + sprite_h * (0.125 + i as f32 * 0.25)) as i32;
            if py < 0 || py >= HEIGHT as i32 { continue; }
            frame.put(pull_x as usize, py as usize, pull_col);
            if pull_x + 1 < end_x {
                frame.put((pull_x + 1) as usize, py as usize, pull_col);
            }
        }
    }
}
