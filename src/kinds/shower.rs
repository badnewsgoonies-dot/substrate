//! Shower: Switchable + LiquidSource (hot water) + TimeDecaySeconds.
//!
//! Like the kitchen sink but vertical, higher flow rate, and the water
//! emitted is hot. Hot-water consumption will feed a virtual water
//! heater in a future rotation; for now it just flows.

use crate::space::Vec2;
use crate::traits::{Liquid, LiquidSource, Switchable, TimeDecaySeconds};

#[derive(Debug, Clone, Copy)]
pub struct Shower {
    pub pos: Vec2,
    on: bool,
    pub seconds_running: f32,
}

impl Shower {
    pub const FLOW_ML_PER_SEC: u32 = 150;

    pub fn new(pos: Vec2) -> Self {
        Self { pos, on: false, seconds_running: 0.0 }
    }

    pub fn water_delivered_ml(seconds: u32) -> u32 {
        Self::FLOW_ML_PER_SEC.saturating_mul(seconds)
    }
}

impl Switchable for Shower {
    fn is_on(&self) -> bool { self.on }
    fn toggle(&mut self) { self.on = !self.on; }
}

impl LiquidSource for Shower {
    fn produced_liquid(&self) -> Liquid { Liquid::Water }
    fn is_producing(&self) -> bool { self.on }
    fn produce(&mut self, dt: f32) -> (Liquid, u32) {
        if !self.on { return (Liquid::Water, 0); }
        let ml = (Self::FLOW_ML_PER_SEC as f32 * dt) as u32;
        (Liquid::Water, ml)
    }
}

impl TimeDecaySeconds for Shower {
    fn tick_real(&mut self, dt: f32) {
        if self.on {
            self.seconds_running += dt;
        }
    }
}

pub fn hot_water_remaining_ml(tank_ml: u32, used_ml: u32) -> u32 {     tank_ml.saturating_sub(used_ml)
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_off() {
        let s = Shower::new(Vec2::new(0.0, 0.0));
        assert!(!s.is_on());
    }

    #[test]
    fn toggle_turns_on() {
        let mut s = Shower::new(Vec2::new(0.0, 0.0));
        s.toggle();
        assert!(s.is_on());
    }

    #[test]
    fn produces_when_on() {
        let mut s = Shower::new(Vec2::new(0.0, 0.0));
        s.toggle();
        let (kind, ml) = s.produce(1.0);
        assert_eq!(kind, Liquid::Water);
        assert_eq!(ml, Shower::FLOW_ML_PER_SEC);
    }

    #[test]
    fn no_flow_when_off() {
        let mut s = Shower::new(Vec2::new(0.0, 0.0));
        let (_, ml) = s.produce(1.0);
        assert_eq!(ml, 0);
    }

    #[test]
    fn tick_accumulates_only_when_on() {
        let mut s = Shower::new(Vec2::new(0.0, 0.0));
        s.tick_real(2.0);
        assert_eq!(s.seconds_running, 0.0);
        s.toggle();
        s.tick_real(1.5);
        assert!((s.seconds_running - 1.5).abs() < 1e-4);
    }

    #[test]
    fn higher_flow_than_kitchen_sink() {
        use crate::kinds::Sink;
        assert!(Shower::FLOW_ML_PER_SEC > Sink::FLOW_ML_PER_SEC);
    }

    #[test]
    fn shower_water_delivered_ml_scales_with_seconds() {
        assert_eq!(Shower::water_delivered_ml(0), 0);
        assert_eq!(Shower::water_delivered_ml(2), 2 * Shower::water_delivered_ml(1));
    }

    #[test]
    fn test_hot_water_remaining_ml() {
                assert_eq!(hot_water_remaining_ml(50000, 15000), 35000);
        assert_eq!(hot_water_remaining_ml(1000, 9999), 0);

    }
}
