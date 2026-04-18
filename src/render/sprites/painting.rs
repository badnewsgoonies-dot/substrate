//! Framed painting: gold rectangular frame (~15% border) with a
//! dark interior "canvas" that has a small subject hint. Reads as
//! "there's art here" without trying to be any specific artwork.

use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};

pub(crate) fn draw_painting(
    frame: &mut Frame, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }

    // Full bounds (the frame)
    let frame_col = rgb(scale(200, fog), scale(160, fog), scale(70, fog));
    let y_start = top_y.max(0.0) as i32;
    let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in y_start..y_end { frame.put(x as usize, y as usize, frame_col); }
    }

    // Canvas interior, inset ~15% on each side
    let canvas_col = rgb(scale(55, fog), scale(45, fog), scale(70, fog));
    let cx_inset = (sprite_w * 0.15) as i32;
    let cy_inset = (sprite_h * 0.15) as i32;
    let cy_start = (top_y as i32 + cy_inset).max(0);
    let cy_end = ((top_y + sprite_h) as i32 - cy_inset).min(HEIGHT as i32);
    for x in (start_x + cx_inset)..(end_x - cx_inset) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in cy_start..cy_end { frame.put(x as usize, y as usize, canvas_col); }
    }

    // Subject hint: a warm horizon band in the upper third of the canvas.
    let horizon_col = rgb(scale(180, fog), scale(110, fog), scale(80, fog));
    let hz = cy_start + ((cy_end - cy_start) as f32 * 0.40) as i32;
    for x in (start_x + cx_inset + 2)..(end_x - cx_inset - 2) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in (hz - 1).max(0)..(hz + 2).min(HEIGHT as i32) {
            frame.put(x as usize, y as usize, horizon_col);
        }
    }
}
