//! People primitive. Every Npc now carries its own Needs, so the AI
//! layer can make decisions based on what the NPC is feeling.

use crate::needs::Needs;
use crate::routine::{activity_target, Activity, Schedule};
use crate::space::{TileGrid, Vec2};
use crate::time::InGameClock;

pub const NPC_WALK_SPEED: f32 = 1.0;
pub const NPC_ARRIVE_RADIUS: f32 = 0.25;

pub struct Npc {
    pub name: &'static str,
    pub pos: Vec2,
    pub facing: f32,
    pub schedule: Schedule,
    pub current_activity: Activity,
    pub needs: Needs,
}

impl Npc {
    pub fn new(name: &'static str, pos: Vec2, facing: f32) -> Self {
        Self {
            name, pos, facing,
            schedule: Schedule::new(vec![]),
            current_activity: Activity::Out,
            needs: Needs::new(),
        }
    }

    pub fn with_schedule(name: &'static str, pos: Vec2, facing: f32, schedule: Schedule) -> Self {
        let current_activity = schedule.activity_at(&InGameClock::at_hour(0));
        Self { name, pos, facing, schedule, current_activity, needs: Needs::new() }
    }

    /// Walk toward a target position, clamped by walls. Shared by the
    /// routine-based tick() and the needs-driven ai::ai_tick_npc().
    pub fn walk_toward(&mut self, grid: &TileGrid, target: Vec2, dt: f32) -> f32 {
        let dx = target.x - self.pos.x;
        let dy = target.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < NPC_ARRIVE_RADIUS { return dist; }
        let step = (NPC_WALK_SPEED * dt).min(dist);
        let nx = self.pos.x + dx / dist * step;
        let ny = self.pos.y + dy / dist * step;
        if !grid.is_solid_at(nx, self.pos.y) { self.pos.x = nx; }
        if !grid.is_solid_at(self.pos.x, ny) { self.pos.y = ny; }
        self.facing = dy.atan2(dx);
        ((target.x - self.pos.x).powi(2) + (target.y - self.pos.y).powi(2)).sqrt()
    }

    pub fn tick(&mut self, clock: &InGameClock, grid: &TileGrid, dt: f32) {
        self.current_activity = self.schedule.activity_at(clock);
        let target = match activity_target(self.current_activity) {
            Some(t) => t,
            None => return,
        };
        self.walk_toward(grid, target, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn simple_grid() -> TileGrid {
        TileGrid::from_ascii(&[
            "1111111111111111","1..............2","1..............2",
            "1..............2","1..............2","1..............2",
            "1..............2","1..............2","2..............2",
            "2..............2","2..............2","2..............2",
            "2..............2","2..............2","2..............2",
            "2222222222222222",
        ])
    }
    #[test]
    fn tick_idles_when_out() {
        let sch = Schedule::roommate_default();
        let mut n = Npc::with_schedule("roommate", Vec2::new(3.0, 10.5), 0.0, sch);
        let grid = simple_grid();
        let clock = InGameClock::at_hour(12);
        let before = n.pos;
        n.tick(&clock, &grid, 0.1);
        assert_eq!(n.pos, before);
    }

    #[test]
    fn new_npc_has_full_needs() {
        let n = Npc::new("bob", Vec2::new(0.0, 0.0), 0.0);
        assert_eq!(n.needs, Needs::new());
    }

    #[test]
    fn walk_toward_closes_distance() {
        let grid = simple_grid();
        let mut n = Npc::new("bob", Vec2::new(2.0, 2.0), 0.0);
        let target = Vec2::new(5.0, 2.0);
        let d_before = ((target.x - n.pos.x).powi(2) + (target.y - n.pos.y).powi(2)).sqrt();
        let d_after = n.walk_toward(&grid, target, 0.2);
        assert!(d_after < d_before);
    }

    #[test]
    fn walk_toward_stops_at_arrive_radius() {
        let grid = simple_grid();
        let mut n = Npc::new("bob", Vec2::new(4.9, 5.0), 0.0);
        let target = Vec2::new(5.0, 5.0);
        let before = n.pos;
        let d = n.walk_toward(&grid, target, 1.0);
        assert!(d < NPC_ARRIVE_RADIUS);
        assert_eq!(n.pos, before); // no movement when already within radius
    }
}
