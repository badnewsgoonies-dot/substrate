//! Body primitive — player embodiment: position, angle, posture, plus
//! the Needs state that the appliance interactions feed back into.
//!
//! Kept as its own module (not a `kind`) because the body isn't an object
//! in the world; it's the player's embodiment. Separate concern.

use crate::needs::Needs;
use crate::space::{cast_ray, RayHit, TileGrid, Vec2};

pub const BOB_RAD_PER_WALK_UNIT: f32 = 6.0;
pub const BOB_AMPLITUDE: f32 = 0.018;
pub const BOB_DECAY_PER_SEC: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Posture {
    Standing,
    Sitting,
    Lying,
}

pub struct PlayerBody {
    pub pos: Vec2,
    pub angle: f32,
    pub posture: Posture,
    pub bob: f32,
    bob_phase: f32,
    pub needs: Needs,
}

impl PlayerBody {
    pub fn new(pos: Vec2, angle: f32) -> Self {
        Self {
            pos, angle,
            posture: Posture::Standing,
            bob: 0.0, bob_phase: 0.0,
            needs: Needs::new(),
        }
    }
    pub fn forward(&self) -> Vec2 { Vec2::from_angle(self.angle) }
    pub fn right(&self) -> Vec2 { Vec2::from_angle(self.angle + std::f32::consts::FRAC_PI_2) }

    pub fn walk(&mut self, grid: &TileGrid, forward_amt: f32, right_amt: f32, pad: f32, dt: f32) {
        let delta = self.forward() * forward_amt + self.right() * right_amt;
        let moving = delta.x.abs() > 1e-4 || delta.y.abs() > 1e-4;
        let try_x = self.pos.x + delta.x;
        let probe_x = try_x + delta.x.signum() * pad;
        if !grid.is_solid_at(probe_x, self.pos.y) { self.pos.x = try_x; }
        let try_y = self.pos.y + delta.y;
        let probe_y = try_y + delta.y.signum() * pad;
        if !grid.is_solid_at(self.pos.x, probe_y) { self.pos.y = try_y; }
        if moving {
            let speed = delta.length() / dt.max(1e-4);
            self.bob_phase += speed * BOB_RAD_PER_WALK_UNIT * dt;
            self.bob = self.bob_phase.sin() * BOB_AMPLITUDE;
        } else {
            self.bob *= (-BOB_DECAY_PER_SEC * dt).exp();
        }
    }

    pub fn turn(&mut self, delta_angle: f32) { self.angle += delta_angle; }

    pub fn crosshair_ray(&self, grid: &TileGrid) -> RayHit {
        let dir = self.forward();
        let plane = Vec2::new(-dir.y, dir.x) * 0.5;
        cast_ray(grid, self.pos, dir, plane, 0.0, 64)
    }

    /// Advance needs forward by the given number of in-game minutes.
    /// Called once per frame from the main loop, once per tick_time
    /// transition in the simulation, or explicitly by tests.
    pub fn tick_needs(&mut self, minutes: u32) {
        self.needs.apply_minute_decay(minutes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forward_east_when_angle_zero() {
        let body = PlayerBody::new(Vec2::new(1.0, 1.0), 0.0);
        let f = body.forward();
        assert!((f.x - 1.0).abs() < 1e-5);
        assert!(f.y.abs() < 1e-5);
    }
    #[test]
    fn right_is_perpendicular_to_forward() {
        let body = PlayerBody::new(Vec2::new(1.0, 1.0), 0.3);
        let f = body.forward();
        let r = body.right();
        assert!((f.x * r.x + f.y * r.y).abs() < 1e-5);
    }
    #[test]
    fn walks_into_open_space() {
        let grid = TileGrid::from_ascii(&["11111","1...1","1...1","1...1","11111"]);
        let mut body = PlayerBody::new(Vec2::new(2.0, 2.0), 0.0);
        body.walk(&grid, 0.3, 0.0, 0.15, 1.0 / 60.0);
        assert!(body.pos.x > 2.0);
    }
    #[test]
    fn blocked_by_wall() {
        let grid = TileGrid::from_ascii(&["11111","1...1","1...1","1...1","11111"]);
        let mut body = PlayerBody::new(Vec2::new(3.5, 2.0), 0.0);
        let before = body.pos.x;
        body.walk(&grid, 1.0, 0.0, 0.15, 1.0 / 60.0);
        assert!(body.pos.x < 4.0);
        assert!(body.pos.x >= before);
    }
    #[test]
    fn bob_frequency_independent_of_dt() {
        let grid = TileGrid::from_ascii(&["11111","1...1","1...1","1...1","11111"]);
        let mut a = PlayerBody::new(Vec2::new(2.0, 2.0), 0.0);
        for _ in 0..60 { a.walk(&grid, 0.01, 0.0, 0.0, 1.0 / 60.0); }
        let mut b = PlayerBody::new(Vec2::new(2.0, 2.0), 0.0);
        for _ in 0..30 { b.walk(&grid, 0.02, 0.0, 0.0, 1.0 / 30.0); }
        let phase_diff = (a.bob_phase - b.bob_phase).abs();
        assert!(phase_diff < 1e-3, "phase differs by {}", phase_diff);
    }

    #[test]
    fn new_body_has_fully_satisfied_needs() {
        let body = PlayerBody::new(Vec2::new(0.0, 0.0), 0.0);
        assert_eq!(body.needs, Needs::new());
    }

    #[test]
    fn tick_needs_decays_hunger() {
        let mut body = PlayerBody::new(Vec2::new(0.0, 0.0), 0.0);
        body.tick_needs(100);
        assert!(body.needs.hunger < crate::needs::NEED_MAX);
    }
}
