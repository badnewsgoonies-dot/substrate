//! Toilet: Cyclable (Idle/Flushing/Refilling) + TimeDecaySeconds + LiquidContainer (tank).
//!
//! Press Use to flush when tank is full. 3 real seconds of Flushing, then
//! auto-refill back up. Press during Flushing/Refilling is a no-op.

use crate::space::Vec2;
use crate::traits::{Cyclable, Liquid, LiquidContainer, TimeDecaySeconds};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToiletState {
    Idle,
    Flushing,
    Refilling,
}

#[derive(Debug, Clone, Copy)]
pub struct Toilet {
    pub pos: Vec2,
    pub state: ToiletState,
    tank_ml: u32,
    state_timer_s: f32,
}

impl Toilet {
    pub const TANK_CAPACITY_ML: u32 = 6000;
    pub const FLUSH_DURATION_SEC: f32 = 3.0;
    pub const FLUSH_VOLUME_ML: u32 = 5000;
    pub const REFILL_RATE_ML_PER_SEC: f32 = 1500.0;

    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            state: ToiletState::Idle,
            tank_ml: Self::TANK_CAPACITY_ML,
            state_timer_s: 0.0,
        }
    }

    pub fn tank_ml(&self) -> u32 { self.tank_ml }
    pub fn tank_fraction(&self) -> f32 {
        self.tank_ml as f32 / Self::TANK_CAPACITY_ML as f32
    }

    /// Total water (ml) consumed by N flushes. Saturates on overflow.
    pub fn total_flush_water_ml(flushes: u32) -> u32 {
        Self::FLUSH_VOLUME_ML.saturating_mul(flushes)
    }
}

impl Cyclable for Toilet {
    fn state_index(&self) -> usize {
        match self.state {
            ToiletState::Idle => 0,
            ToiletState::Flushing => 1,
            ToiletState::Refilling => 2,
        }
    }
    fn state_count(&self) -> usize { 3 }
    fn cycle(&mut self) {
        // Only Idle -> Flushing is user-triggered, and only if tank is full.
        if self.state == ToiletState::Idle && self.tank_ml >= Self::FLUSH_VOLUME_ML {
            self.state = ToiletState::Flushing;
            self.state_timer_s = 0.0;
        }
    }
}

impl TimeDecaySeconds for Toilet {
    fn tick_real(&mut self, dt: f32) {
        match self.state {
            ToiletState::Idle => {}
            ToiletState::Flushing => {
                self.state_timer_s += dt;
                if self.state_timer_s >= Self::FLUSH_DURATION_SEC {
                    self.tank_ml = self.tank_ml.saturating_sub(Self::FLUSH_VOLUME_ML);
                    self.state = ToiletState::Refilling;
                    self.state_timer_s = 0.0;
                }
            }
            ToiletState::Refilling => {
                let add = (Self::REFILL_RATE_ML_PER_SEC * dt) as u32;
                self.tank_ml = (self.tank_ml + add).min(Self::TANK_CAPACITY_ML);
                if self.tank_ml >= Self::TANK_CAPACITY_ML {
                    self.state = ToiletState::Idle;
                }
            }
        }
    }
}

impl LiquidContainer for Toilet {
    fn liquid(&self) -> Option<(Liquid, u32)> {
        if self.tank_ml > 0 { Some((Liquid::Water, self.tank_ml)) } else { None }
    }
    fn capacity_ml(&self) -> u32 { Self::TANK_CAPACITY_ML }
    fn add_liquid(&mut self, kind: Liquid, ml: u32) -> u32 {
        if kind != Liquid::Water { return ml; }
        let room = Self::TANK_CAPACITY_ML.saturating_sub(self.tank_ml);
        let taken = ml.min(room);
        self.tank_ml += taken;
        ml - taken
    }
    fn drain_liquid(&mut self, ml: u32) -> (Liquid, u32) {
        let taken = ml.min(self.tank_ml);
        self.tank_ml -= taken;
        (Liquid::Water, taken)
    }
}

pub fn seconds_to_refill(empty_ml: u32, refill_rate_ml_per_sec: u32) -> u32 {     if refill_rate_ml_per_sec == 0 { return 0; }
    empty_ml / refill_rate_ml_per_sec
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_idle_with_full_tank() {
        let t = Toilet::new(Vec2::new(0.0, 0.0));
        assert_eq!(t.state, ToiletState::Idle);
        assert_eq!(t.tank_ml(), Toilet::TANK_CAPACITY_ML);
    }

    #[test]
    fn flush_cycles_through_states() {
        let mut t = Toilet::new(Vec2::new(0.0, 0.0));
        t.cycle();
        assert_eq!(t.state, ToiletState::Flushing);
        t.tick_real(Toilet::FLUSH_DURATION_SEC + 0.1);
        assert_eq!(t.state, ToiletState::Refilling);
        assert!(t.tank_ml() < Toilet::TANK_CAPACITY_ML);
        t.tick_real(10.0);
        assert_eq!(t.state, ToiletState::Idle);
        assert_eq!(t.tank_ml(), Toilet::TANK_CAPACITY_ML);
    }

    #[test]
    fn cycle_during_flush_is_noop() {
        let mut t = Toilet::new(Vec2::new(0.0, 0.0));
        t.cycle();
        t.cycle();
        assert_eq!(t.state, ToiletState::Flushing);
    }

    #[test]
    fn flush_requires_full_tank() {
        let mut t = Toilet::new(Vec2::new(0.0, 0.0));
        t.drain_liquid(Toilet::TANK_CAPACITY_ML);
        t.cycle();
        assert_eq!(t.state, ToiletState::Idle);
    }

    #[test]
    fn add_water_stops_at_capacity() {
        let mut t = Toilet::new(Vec2::new(0.0, 0.0));
        let overflow = t.add_liquid(Liquid::Water, 1000);
        assert_eq!(overflow, 1000);
    }

    #[test]
    fn reject_non_water() {
        let mut t = Toilet::new(Vec2::new(0.0, 0.0));
        let overflow = t.add_liquid(Liquid::Coffee, 500);
        assert_eq!(overflow, 500);
    }

    #[test]
    fn toilet_total_flush_water_ml_saturates_on_overflow() {
        assert_eq!(Toilet::total_flush_water_ml(0), 0);
        assert_eq!(Toilet::total_flush_water_ml(u32::MAX), u32::MAX);
    }

    #[test]
    fn test_seconds_to_refill() {
                assert_eq!(seconds_to_refill(6000, 1500), 4);
        assert_eq!(seconds_to_refill(100, 0), 0);

    }
}
