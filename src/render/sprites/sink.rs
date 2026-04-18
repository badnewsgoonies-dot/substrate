//! Kitchen sink sprite. Cabinet + basin + faucet, animated water
//! stream when on.

use crate::kinds::Sink;
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};
use crate::traits::Switchable;

pub(crate) fn draw_sink(
    frame: &mut Frame, sk: &Sink, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let cabinet = rgb(scale(150, fog), scale(120, fog), scale(90, fog));
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    let cab_y0 = (top_y + sprite_h * 0.30) as i32;
    let cab_y1 = (top_y + sprite_h).min(HEIGHT as f32) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in cab_y0.max(0)..cab_y1 { frame.put(x as usize, y as usize, cabinet); }
    }
    let basin = rgb(scale(110, fog), scale(112, fog), scale(120, fog));
    let basin_y0 = top_y.max(0.0) as i32;
    let basin_y1 = (top_y + sprite_h * 0.30) as i32;
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in basin_y0..basin_y1 { frame.put(x as usize, y as usize, basin); }
    }
    let faucet = rgb(scale(180, fog), scale(180, fog), scale(180, fog));
    let fx = (left_x + sprite_w * 0.5) as i32;
    let fy0 = (top_y + sprite_h * 0.02) as i32;
    let fy1 = (top_y + sprite_h * 0.22) as i32;
    if fx >= start_x && fx < end_x && frame.z_buffer[fx as usize] >= dist {
        for y in fy0.max(0)..fy1.min(HEIGHT as i32) {
            frame.put(fx as usize, y as usize, faucet);
            if fx + 1 < end_x { frame.put((fx + 1) as usize, y as usize, faucet); }
        }
    }
    if sk.is_on() {
        let water_a = rgb(scale(130, fog), scale(180, fog), scale(220, fog));
        let water_b = rgb(scale(170, fog), scale(210, fog), scale(240, fog));
        let t = sk.seconds_running;
        let stream_x = fx + 1;
        let stream_y0 = fy1;
        let stream_y1 = (top_y + sprite_h * 0.30) as i32;
        if stream_x >= start_x && stream_x < end_x && frame.z_buffer[stream_x as usize] >= dist {
            for y in stream_y0.max(0)..stream_y1.min(HEIGHT as i32) {
                let phase = ((y as f32) + t * 40.0) as i32;
                let col = if (phase & 3) < 2 { water_a } else { water_b };
                frame.put(stream_x as usize, y as usize, col);
            }
        }
    }
}
