//! BathSink: same shape as kitchen Sink but smaller flow, rendered
//! smaller. Kept as a separate kind because sprite sizing + future
//! differentiation (hot/cold handles, different liquid profile) make
//! it worth distinct identity even though the trait impls are parallel.

use crate::space::Vec2;
use crate::traits::{Liquid, LiquidSource, Switchable, TimeDecaySeconds};

#[derive(Debug, Clone, Copy)]
pub struct BathSink {
    pub pos: Vec2,
    on: bool,
    pub seconds_running: f32,
}

impl BathSink {
    pub const FLOW_ML_PER_SEC: u32 = 80;

    pub fn new(pos: Vec2) -> Self {
        Self { pos, on: false, seconds_running: 0.0 }
    }

    pub fn water_delivered_ml(seconds: u32) -> u32 {
        Self::FLOW_ML_PER_SEC.saturating_mul(seconds)
    }
}

impl Switchable for BathSink {
    fn is_on(&self) -> bool { self.on }
    fn toggle(&mut self) { self.on = !self.on; }
}

impl LiquidSource for BathSink {
    fn produced_liquid(&self) -> Liquid { Liquid::Water }
    fn is_producing(&self) -> bool { self.on }
    fn produce(&mut self, dt: f32) -> (Liquid, u32) {
        if !self.on { return (Liquid::Water, 0); }
        let ml = (Self::FLOW_ML_PER_SEC as f32 * dt) as u32;
        (Liquid::Water, ml)
    }
}

impl TimeDecaySeconds for BathSink {
    fn tick_real(&mut self, dt: f32) {
        if self.on {
            self.seconds_running += dt;
        }
    }
}

pub fn teeth_brush_water_ml(seconds_brushing: u32, flow_ml_per_sec: u32) -> u32 {     seconds_brushing.saturating_mul(flow_ml_per_sec)
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_off_and_produces_nothing() {
        let mut s = BathSink::new(Vec2::new(0.0, 0.0));
        assert!(!s.is_on());
        let (_, ml) = s.produce(1.0);
        assert_eq!(ml, 0);
    }

    #[test]
    fn toggle_enables_flow() {
        let mut s = BathSink::new(Vec2::new(0.0, 0.0));
        s.toggle();
        let (kind, ml) = s.produce(1.0);
        assert_eq!(kind, Liquid::Water);
        assert_eq!(ml, BathSink::FLOW_ML_PER_SEC);
    }

    #[test]
    fn smaller_flow_than_kitchen() {
        use crate::kinds::Sink;
        assert!(BathSink::FLOW_ML_PER_SEC < Sink::FLOW_ML_PER_SEC);
    }

    #[test]
    fn smaller_flow_than_shower() {
        use crate::kinds::Shower;
        assert!(BathSink::FLOW_ML_PER_SEC < Shower::FLOW_ML_PER_SEC);
    }

    #[test]
    fn bath_sink_water_delivered_ml_scales_with_seconds() {
        assert_eq!(BathSink::water_delivered_ml(0), 0);
        assert_eq!(BathSink::water_delivered_ml(2), 2 * BathSink::water_delivered_ml(1));
    }

    #[test]
    fn test_teeth_brush_water_ml() {
                assert_eq!(teeth_brush_water_ml(120, 80), 9600);
        assert_eq!(teeth_brush_water_ml(0, 80), 0);

    }
}
