//! Bathtub: Switchable (faucet) + Openable (drain plug) + LiquidContainer + TimeDecaySeconds.
//!
//! Fills when faucet on AND drain closed (plug in). Drains when drain
//! open. The composition is real: the trait accessors all return the
//! same underlying struct, the tub just happens to have both a faucet
//! input and a drain output, gated by two independent state bits.
//!
//! In a future rotation: taking a bath will require the tub full, and
//! getting in will warm the bathroom (Heatable trait).

use crate::space::Vec2;
use crate::traits::{Liquid, LiquidContainer, Openable, Switchable, TimeDecaySeconds};

#[derive(Debug, Clone, Copy)]
pub struct Bathtub {
    pub pos: Vec2,
    faucet_on: bool,
    drain_open: bool,
    water_ml: u32,
}

impl Bathtub {
    pub const CAPACITY_ML: u32 = 200_000; // ~200 L
    pub const FILL_RATE_ML_PER_SEC: f32 = 300.0;
    pub const DRAIN_RATE_ML_PER_SEC: f32 = 500.0;

    pub fn new(pos: Vec2) -> Self {
        Self { pos, faucet_on: false, drain_open: true, water_ml: 0 }
    }

    pub fn water_ml(&self) -> u32 { self.water_ml }
    pub fn fill_fraction(&self) -> f32 {
        (self.water_ml as f32 / Self::CAPACITY_ML as f32).clamp(0.0, 1.0)
    }
    pub fn drain_open(&self) -> bool { self.drain_open }

    /// Time in whole seconds to fill an empty tub at the given flow rate.
    pub fn fill_time_seconds(flow_ml_per_sec: u32) -> u32 {
        if flow_ml_per_sec == 0 { return 0; }
        Self::CAPACITY_ML / flow_ml_per_sec
    }
}

impl Switchable for Bathtub {
    fn is_on(&self) -> bool { self.faucet_on }
    fn toggle(&mut self) { self.faucet_on = !self.faucet_on; }
}

impl Openable for Bathtub {
    fn is_open(&self) -> bool { self.drain_open }
    fn toggle_open(&mut self) { self.drain_open = !self.drain_open; }
}

impl TimeDecaySeconds for Bathtub {
    fn tick_real(&mut self, dt: f32) {
        if self.faucet_on && !self.drain_open {
            let add = (Self::FILL_RATE_ML_PER_SEC * dt) as u32;
            self.water_ml = (self.water_ml + add).min(Self::CAPACITY_ML);
        }
        if self.drain_open && self.water_ml > 0 {
            let remove = (Self::DRAIN_RATE_ML_PER_SEC * dt) as u32;
            self.water_ml = self.water_ml.saturating_sub(remove);
        }
    }
}

impl LiquidContainer for Bathtub {
    fn liquid(&self) -> Option<(Liquid, u32)> {
        if self.water_ml > 0 { Some((Liquid::Water, self.water_ml)) } else { None }
    }
    fn capacity_ml(&self) -> u32 { Self::CAPACITY_ML }
    fn add_liquid(&mut self, kind: Liquid, ml: u32) -> u32 {
        if kind != Liquid::Water { return ml; }
        let room = Self::CAPACITY_ML.saturating_sub(self.water_ml);
        let taken = ml.min(room);
        self.water_ml += taken;
        ml - taken
    }
    fn drain_liquid(&mut self, ml: u32) -> (Liquid, u32) {
        let taken = ml.min(self.water_ml);
        self.water_ml -= taken;
        (Liquid::Water, taken)
    }
}

pub fn drain_time_seconds(water_ml: u32, drain_rate_ml_per_sec: u32) -> u32 {     if drain_rate_ml_per_sec == 0 { return 0; }
    water_ml / drain_rate_ml_per_sec
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty_with_drain_open() {
        let t = Bathtub::new(Vec2::new(0.0, 0.0));
        assert!(!t.is_on());
        assert!(t.is_open());
        assert_eq!(t.water_ml(), 0);
    }

    #[test]
    fn faucet_on_alone_does_not_fill() {
        let mut t = Bathtub::new(Vec2::new(0.0, 0.0));
        t.toggle(); // faucet on, drain still open
        t.tick_real(10.0);
        assert_eq!(t.water_ml(), 0);
    }

    #[test]
    fn faucet_on_plus_drain_closed_fills() {
        let mut t = Bathtub::new(Vec2::new(0.0, 0.0));
        t.toggle(); // faucet on
        t.toggle_open(); // drain closed
        t.tick_real(10.0);
        assert!(t.water_ml() > 0);
    }

    #[test]
    fn drain_open_empties_tub() {
        let mut t = Bathtub::new(Vec2::new(0.0, 0.0));
        t.toggle(); t.toggle_open(); // filling
        t.tick_real(100.0);
        let filled = t.water_ml();
        assert!(filled > 0);
        t.toggle(); // faucet off
        t.toggle_open(); // drain open
        t.tick_real(100.0);
        assert!(t.water_ml() < filled);
    }

    #[test]
    fn capacity_enforced() {
        let mut t = Bathtub::new(Vec2::new(0.0, 0.0));
        let overflow = t.add_liquid(Liquid::Water, Bathtub::CAPACITY_ML * 2);
        assert_eq!(overflow, Bathtub::CAPACITY_ML);
        assert_eq!(t.water_ml(), Bathtub::CAPACITY_ML);
    }

    #[test]
    fn fill_fraction_in_range() {
        let mut t = Bathtub::new(Vec2::new(0.0, 0.0));
        assert_eq!(t.fill_fraction(), 0.0);
        t.add_liquid(Liquid::Water, Bathtub::CAPACITY_ML / 2);
        assert!((t.fill_fraction() - 0.5).abs() < 0.01);
    }

    #[test]
    fn bathtub_fill_time_seconds_zero_flow_returns_zero() {
        assert_eq!(Bathtub::fill_time_seconds(0), 0);
        assert!(Bathtub::fill_time_seconds(1) >= Bathtub::fill_time_seconds(2));
    }

    #[test]
    fn test_drain_time_seconds() {
                assert_eq!(drain_time_seconds(10000, 500), 20);
        assert_eq!(drain_time_seconds(100, 0), 0);
        assert_eq!(drain_time_seconds(0, 500), 0);

    }
}
