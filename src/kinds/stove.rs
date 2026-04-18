//! Stove: Powered + TimeDecayMinutes + (per-burner) LightEmitter.
//!
//! Press cycles one burner at a time via an internal cursor. The Stove
//! as a whole is Powered (any-burner-on), and each burner contributes
//! light when at Low/High/burned-warm.

use crate::kinds::LightContribution;
use crate::space::Vec2;
use crate::traits::{Powered, TimeDecayMinutes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnerLevel { Off, Low, High }

impl BurnerLevel {
    pub const fn heat_per_minute(self) -> u32 {
        match self {
            BurnerLevel::Off => 0,
            BurnerLevel::Low => 2,
            BurnerLevel::High => 6,
        }
    }
    pub fn cycle(self) -> Self {
        match self {
            BurnerLevel::Off => BurnerLevel::Low,
            BurnerLevel::Low => BurnerLevel::High,
            BurnerLevel::High => BurnerLevel::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Burner {
    pub level: BurnerLevel,
    pub warmth: u32,
    pub burned: bool,
}

impl Burner {
    pub const WARMTH_COOKED: u32 = 60;
    pub const WARMTH_BURNED: u32 = 140;
    pub const fn off() -> Self { Self { level: BurnerLevel::Off, warmth: 0, burned: false } }
    pub fn tick_minutes(&mut self, dm: u32) {
        if self.burned { return; }
        let add = self.level.heat_per_minute().saturating_mul(dm);
        self.warmth = self.warmth.saturating_add(add);
        if self.warmth >= Self::WARMTH_BURNED { self.burned = true; }
        if self.level == BurnerLevel::Off {
            self.warmth = self.warmth.saturating_sub(dm.saturating_mul(3));
        }
    }
    pub fn is_cooked(&self) -> bool { !self.burned && self.warmth >= Self::WARMTH_COOKED }

    /// Pure predicate: would a burner with this warmth reading be cooked?
    /// Does not consider the burned flag — just the warmth threshold.
    pub fn warmth_at_or_above_cooked(warmth: u32) -> bool {
        warmth >= Self::WARMTH_COOKED
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoveBurners { pub burners: [Burner; 4] }

impl StoveBurners {
    pub const fn off() -> Self {
        Self { burners: [Burner::off(), Burner::off(), Burner::off(), Burner::off()] }
    }
    pub fn any_on(&self) -> bool {
        self.burners.iter().any(|b| b.level != BurnerLevel::Off)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Stove {
    pub pos: Vec2,
    pub burners: StoveBurners,
    pub cursor: u8,
}

impl Stove {
    pub const POWER_PER_ACTIVE_BURNER_W: u32 = 1500;

    pub fn new(pos: Vec2) -> Self {
        Self { pos, burners: StoveBurners::off(), cursor: 0 }
    }

    pub fn any_burner_on(&self) -> bool { self.burners.any_on() }

    /// Press: cycle the burner at the cursor, then advance.
    pub fn press(&mut self) {
        let idx = (self.cursor as usize) & 0b11;
        let b = &mut self.burners.burners[idx];
        b.level = b.level.cycle();
        self.cursor = (self.cursor + 1) & 0b11;
    }

    /// Aggregate light contribution from all burners.
    pub fn light_contribution_from_burners(&self) -> Option<LightContribution> {
        // Sum of per-burner intensities, warm orange.
        let mut intensity = 0.0f32;
        for b in self.burners.burners.iter() {
            let i = match b.level {
                BurnerLevel::Off => if b.warmth > 0 { 0.02 } else { 0.0 },
                BurnerLevel::Low => 0.08,
                BurnerLevel::High => 0.15,
            };
            intensity += i;
        }
        if intensity <= 0.0 { return None; }
        Some(LightContribution {
            rgb: (255, 150, 50),
            intensity: intensity.min(0.5),
        })
    }
}

impl Powered for Stove {
    fn power_draw_watts(&self) -> u32 {
        let on_count = self.burners.burners.iter()
            .filter(|b| b.level != BurnerLevel::Off)
            .count() as u32;
        on_count * Self::POWER_PER_ACTIVE_BURNER_W
    }
    fn is_on(&self) -> bool { self.any_burner_on() }
}

impl TimeDecayMinutes for Stove {
    fn tick_minutes(&mut self, dm: u32) {
        for b in self.burners.burners.iter_mut() {
            b.tick_minutes(dm);
        }
    }
}

pub fn burner_bank_watts(n_burners_on: u32) -> u32 {     n_burners_on.saturating_mul(1500)
 }

pub fn joules_delta(watts: u32, seconds: u32) -> u32 {     watts.saturating_mul(seconds)
 }

pub fn heat_stage(warmth: u32) -> u32 { if warmth < 30 { 0 } else if warmth < 60 { 1 } else if warmth < 100 { 2 } else { 3 } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_off() {
        let s = Stove::new(Vec2::new(0.0, 0.0));
        assert!(!s.any_burner_on());
    }
    #[test]
    fn press_cycles_burner_0() {
        let mut s = Stove::new(Vec2::new(0.0, 0.0));
        s.press();
        assert_eq!(s.burners.burners[0].level, BurnerLevel::Low);
        assert_eq!(s.cursor, 1);
    }
    #[test]
    fn four_presses_advance_cursor_around() {
        let mut s = Stove::new(Vec2::new(0.0, 0.0));
        s.press(); s.press(); s.press(); s.press();
        assert_eq!(s.cursor, 0);
    }
    #[test]
    fn burner_warms_to_cooked() {
        let mut b = Burner::off();
        b.level = BurnerLevel::Low;
        b.tick_minutes(40);
        assert!(b.is_cooked());
        assert!(!b.burned);
    }
    #[test]
    fn burner_burns_on_high() {
        let mut b = Burner::off();
        b.level = BurnerLevel::High;
        b.tick_minutes(30);
        assert!(b.burned);
    }
    #[test]
    fn powered_scales_with_burners_on() {
        let mut s = Stove::new(Vec2::new(0.0, 0.0));
        assert_eq!(s.power_draw_watts(), 0);
        s.press();
        assert_eq!(s.power_draw_watts(), Stove::POWER_PER_ACTIVE_BURNER_W);
    }
    #[test]
    fn light_contribution_empty_when_all_cold() {
        let s = Stove::new(Vec2::new(0.0, 0.0));
        assert!(s.light_contribution_from_burners().is_none());
    }
    #[test]
    fn light_contribution_present_when_any_on() {
        let mut s = Stove::new(Vec2::new(0.0, 0.0));
        s.press();
        assert!(s.light_contribution_from_burners().is_some());
    }

    #[test]
    fn warmth_at_or_above_cooked_threshold_boundary() {
        assert!(Burner::warmth_at_or_above_cooked(u32::MAX));
        assert!(Burner::warmth_at_or_above_cooked(Burner::WARMTH_COOKED));
        assert!(!Burner::warmth_at_or_above_cooked(Burner::WARMTH_COOKED - 1));
        assert!(!Burner::warmth_at_or_above_cooked(0));
    }

    #[test]
    fn test_burner_bank_watts() {
                assert_eq!(burner_bank_watts(0), 0);
        assert_eq!(burner_bank_watts(4), 6000);

    }

    #[test]
    fn test_joules_delta() {
                assert_eq!(joules_delta(1500, 60), 90000);
        assert_eq!(joules_delta(0, 100), 0);

    }

    #[test]
    fn test_heat_stage() {
        assert_eq!(heat_stage(0), 0); assert_eq!(heat_stage(29), 0); assert_eq!(heat_stage(30), 1); assert_eq!(heat_stage(75), 2); assert_eq!(heat_stage(150), 3);
    }
}
