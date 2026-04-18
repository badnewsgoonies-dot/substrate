//! Mug: a single-chamber ceramic drinking vessel holding up to 250ml.

use crate::space::Vec2;
use crate::traits::{Liquid, LiquidContainer};

#[derive(Debug, Clone, Copy)]
pub struct Mug {
    pub pos: Vec2,
    water_ml: u32,
}

impl Mug {
    pub const CAPACITY_ML: u32 = 250;

    pub fn new(pos: Vec2) -> Self {
        Self { pos, water_ml: 0 }
    }

    pub fn water_ml(&self) -> u32 { self.water_ml }

    pub fn fill_fraction(&self) -> f32 {
        (self.water_ml as f32 / Self::CAPACITY_ML as f32).clamp(0.0, 1.0)
    }
}

impl LiquidContainer for Mug {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let m = Mug::new(Vec2::new(0.0, 0.0));
        assert_eq!(m.water_ml(), 0);
        assert_eq!(m.fill_fraction(), 0.0);
    }

    #[test]
    fn add_water_bounded_by_capacity() {
        let mut m = Mug::new(Vec2::new(0.0, 0.0));
        let overflow = m.add_liquid(Liquid::Water, Mug::CAPACITY_ML * 2);
        assert_eq!(overflow, Mug::CAPACITY_ML);
        assert_eq!(m.water_ml(), Mug::CAPACITY_ML);
    }

    #[test]
    fn reject_non_water() {
        let mut m = Mug::new(Vec2::new(0.0, 0.0));
        let overflow = m.add_liquid(Liquid::Coffee, 100);
        assert_eq!(overflow, 100);
        assert_eq!(m.water_ml(), 0);
    }

    #[test]
    fn drain_removes_water() {
        let mut m = Mug::new(Vec2::new(0.0, 0.0));
        m.add_liquid(Liquid::Water, 200);
        let (kind, ml) = m.drain_liquid(50);
        assert_eq!(kind, Liquid::Water);
        assert_eq!(ml, 50);
        assert_eq!(m.water_ml(), 150);
    }
}
