//! Fridge sprite. Shell + handle when closed, interior reveal with
//! per-slot food items when open.

use crate::kinds::{FoodItem, Fridge};
use crate::render::{rgb, scale, Frame, HEIGHT, WIDTH};
use crate::traits::Openable;

pub(crate) fn draw_fridge(
    frame: &mut Frame, fr: &Fridge, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let (shell_r, shell_g, shell_b) = if fr.is_open() { (200u8, 200u8, 195u8) } else { (220u8, 220u8, 214u8) };
    let shell = rgb(scale(shell_r, fog), scale(shell_g, fog), scale(shell_b, fog));
    let start_x = left_x.max(0.0) as i32;
    let end_x = (left_x + sprite_w).min(WIDTH as f32) as i32;
    if start_x >= end_x { return; }
    for x in start_x..end_x {
        if frame.z_buffer[x as usize] < dist { continue; }
        let y_start = top_y.max(0.0) as i32;
        let y_end = (top_y + sprite_h).min(HEIGHT as f32) as i32;
        for y in y_start..y_end { frame.put(x as usize, y as usize, shell); }
    }
    if !fr.is_open() {
        let handle_col = rgb(scale(120, fog), scale(120, fog), scale(115, fog));
        let hx = (left_x + sprite_w * 0.85) as i32;
        if hx >= start_x && hx < end_x && frame.z_buffer[hx as usize] >= dist {
            let y_start = (top_y + sprite_h * 0.15) as i32;
            let y_end = (top_y + sprite_h * 0.85) as i32;
            for y in y_start.max(0)..y_end.min(HEIGHT as i32) {
                frame.put(hx as usize, y as usize, handle_col);
            }
        }
        return;
    }
    let interior = rgb(scale(30, fog), scale(30, fog), scale(34, fog));
    let inset_px = (sprite_w * 0.12).max(2.0);
    let int_left = (left_x + inset_px) as i32;
    let int_right = (left_x + sprite_w - inset_px) as i32;
    let int_top = (top_y + sprite_h * 0.08) as i32;
    let int_bottom = (top_y + sprite_h * 0.92) as i32;
    for x in int_left.max(start_x)..int_right.min(end_x) {
        if frame.z_buffer[x as usize] < dist { continue; }
        for y in int_top.max(0)..int_bottom.min(HEIGHT as i32) {
            frame.put(x as usize, y as usize, interior);
        }
    }
    let slot_h = ((int_bottom - int_top) as f32) / 3.0;
    for (i, slot) in fr.slots.iter().enumerate() {
        let y0 = int_top as f32 + slot_h * i as f32 + slot_h * 0.15;
        let y1 = int_top as f32 + slot_h * (i as f32 + 1.0) - slot_h * 0.15;
        let item = match slot.item { Some(it) => it, None => continue };
        let (br, bg, bb) = match item {
            FoodItem::Milk => (230u8, 230u8, 235u8),
            FoodItem::Eggs => (220u8, 200u8, 150u8),
            FoodItem::Leftovers => (170u8, 110u8, 70u8),
        };
        let full = item.starting_minutes().max(1) as f32;
        let freshness = (slot.minutes_remaining as f32 / full).clamp(0.0, 1.0);
        let (r, g, b) = if slot.minutes_remaining == 0 { (70u8, 80u8, 50u8) } else {
            let t = freshness;
            let gr = 120.0;
            let mix = |c: u8| ((c as f32) * t + gr * (1.0 - t)) as u8;
            (mix(br), mix(bg), mix(bb))
        };
        let color = rgb(scale(r, fog), scale(g, fog), scale(b, fog));
        let block_left = int_left as f32 + (int_right - int_left) as f32 * 0.2;
        let block_right = int_right as f32 - (int_right - int_left) as f32 * 0.2;
        for x in (block_left as i32).max(start_x)..(block_right as i32).min(end_x) {
            if frame.z_buffer[x as usize] < dist { continue; }
            for y in (y0 as i32).max(0)..(y1 as i32).min(HEIGHT as i32) {
                frame.put(x as usize, y as usize, color);
            }
        }
    }
}
