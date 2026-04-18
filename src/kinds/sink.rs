//! Sink: Switchable (on/off) + LiquidSource (water) + TimeDecaySeconds.

use crate::space::Vec2;
use crate::traits::{Liquid, LiquidSource, Switchable, TimeDecaySeconds};

#[derive(Debug, Clone, Copy)]
pub struct Sink {
    pub pos: Vec2,
    on: bool,
    /// Real-time seconds the water has been flowing this session.
    pub seconds_running: f32,
}

impl Sink {
    /// Milliliters per real second when the tap is on.
    pub const FLOW_ML_PER_SEC: u32 = 100;

    pub fn new(pos: Vec2) -> Self {
        Self { pos, on: false, seconds_running: 0.0 }
    }

    /// Water delivered over N whole seconds of running tap.
    pub fn water_delivered_ml(seconds: u32) -> u32 {
        Self::FLOW_ML_PER_SEC.saturating_mul(seconds)
    }
}

impl Switchable for Sink {
    fn is_on(&self) -> bool { self.on }
    fn toggle(&mut self) { self.on = !self.on; }
}

impl LiquidSource for Sink {
    fn produced_liquid(&self) -> Liquid { Liquid::Water }
    fn is_producing(&self) -> bool { self.on }
    fn produce(&mut self, dt: f32) -> (Liquid, u32) {
        if !self.on { return (Liquid::Water, 0); }
        let ml = (Self::FLOW_ML_PER_SEC as f32 * dt) as u32;
        (Liquid::Water, ml)
    }
}

impl TimeDecaySeconds for Sink {
    fn tick_real(&mut self, dt: f32) {
        if self.on {
            self.seconds_running += dt;
        }
    }
}

pub fn dishes_washable_count(water_available_ml: u32, ml_per_dish: u32) -> u32 {     if ml_per_dish == 0 { return 0; }
    water_available_ml / ml_per_dish
 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_off() {
        let s = Sink::new(Vec2::new(0.0, 0.0));
        assert!(!s.is_on());
    }
    #[test]
    fn toggle_turns_on() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        s.toggle();
        assert!(s.is_on());
    }
    #[test]
    fn produces_water_when_on() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        s.toggle();
        let (kind, ml) = s.produce(1.0);
        assert_eq!(kind, Liquid::Water);
        assert_eq!(ml, Sink::FLOW_ML_PER_SEC);
    }
    #[test]
    fn produces_nothing_when_off() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        let (_, ml) = s.produce(1.0);
        assert_eq!(ml, 0);
    }
    #[test]
    fn tick_real_accumulates_only_when_on() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        s.tick_real(1.0);
        assert_eq!(s.seconds_running, 0.0);
        s.toggle();
        s.tick_real(1.0);
        s.tick_real(0.5);
        assert!((s.seconds_running - 1.5).abs() < 1e-4);
    }

    #[test]
    fn sink_water_delivered_ml_scales_with_seconds() {
        assert_eq!(Sink::water_delivered_ml(0), 0);
        assert_eq!(Sink::water_delivered_ml(2), 2 * Sink::water_delivered_ml(1));
    }

    #[test]
    fn test_dishes_washable_count() {
                assert_eq!(dishes_washable_count(1000, 200), 5);
        assert_eq!(dishes_washable_count(50, 200), 0);
        assert_eq!(dishes_washable_count(1000, 0), 0);

    }
}
