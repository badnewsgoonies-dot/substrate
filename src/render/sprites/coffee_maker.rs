//! Coffee maker sprite. Just a brew-state-dependent color fed
//! through `draw_flat`. No geometry of its own.

use crate::kinds::{BrewState, CoffeeMaker};
use crate::render::{draw_flat, Frame};

pub(crate) fn draw_coffee_maker(
    frame: &mut Frame, cm: &CoffeeMaker, fog: f32,
    top_y: f32, left_x: f32, sprite_w: f32, sprite_h: f32, dist: f32,
) {
    let col = match cm.brew_state() {
        BrewState::Idle => if cm.output_ml() > 0 { (100, 70, 50) } else { (80, 80, 80) },
        BrewState::Brewing => (140, 90, 60),
        BrewState::Done => (100, 70, 50),
    };
    draw_flat(frame, fog, top_y, left_x, sprite_w, sprite_h, dist, col);
}
