//! CoffeeMaker.
//!
//! Now with real internal structure: a water reservoir (LiquidContainer,
//! accepts only water) and a coffee output (LiquidContainer, holds
//! coffee). A brew-state machine transitions Idle -> Brewing -> Done on
//! Use presses, but only if the preconditions hold: water present, power
//! available (Powered trait).
//!
//! For v0.6 the reservoir starts full — nothing in the world can pour
//! water yet (that's the Kettle in v0.7+). The mental model is correct
//! even though the inputs are temporarily magic.

use crate::space::Vec2;
use crate::traits::{
    Cyclable, Liquid, LiquidContainer, Powered, TimeDecaySeconds,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrewState {
    Idle,
    Brewing, // pressing again is no-op; brew completes on tick
    Done,
}

#[derive(Debug, Clone, Copy)]
pub struct CoffeeMaker {
    pub pos: Vec2,
    reservoir_ml: u32,      // water, up to RESERVOIR_CAPACITY
    output_ml: u32,         // coffee, up to OUTPUT_CAPACITY
    brew: BrewState,
    brew_progress_s: f32,   // seconds into a brew cycle
}

impl CoffeeMaker {
    pub const RESERVOIR_CAPACITY: u32 = 500;
    pub const OUTPUT_CAPACITY: u32 = 500;
    pub const BREW_DURATION_SEC: f32 = 12.0;
    pub const WATER_PER_BREW_ML: u32 = 250;
    pub const COFFEE_PER_BREW_ML: u32 = 250;
    pub const POWER_BREWING_W: u32 = 800;

    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            reservoir_ml: Self::RESERVOIR_CAPACITY, // magic-full for v0.6
            output_ml: 0,
            brew: BrewState::Idle,
            brew_progress_s: 0.0,
        }
    }

    pub fn brew_state(&self) -> BrewState { self.brew }
    pub fn reservoir_ml(&self) -> u32 { self.reservoir_ml }
    pub fn output_ml(&self) -> u32 { self.output_ml }
}

impl Cyclable for CoffeeMaker {
    fn state_index(&self) -> usize {
        match self.brew { BrewState::Idle => 0, BrewState::Brewing => 1, BrewState::Done => 2 }
    }
    fn state_count(&self) -> usize { 3 }
    fn cycle(&mut self) {
        self.brew = match self.brew {
            BrewState::Idle => {
                // Only start brewing if we have water.
                if self.reservoir_ml >= Self::WATER_PER_BREW_ML {
                    self.brew_progress_s = 0.0;
                    BrewState::Brewing
                } else { BrewState::Idle }
            }
            // Pressing while brewing is a no-op — you can't rush it.
            BrewState::Brewing => BrewState::Brewing,
            BrewState::Done => {
                // Pour out (magic-drain for v0.6; future: fills the cup held by player).
                self.output_ml = 0;
                BrewState::Idle
            }
        };
    }
}

impl LiquidContainer for CoffeeMaker {
    fn liquid(&self) -> Option<(Liquid, u32)> {
        // The output bowl is the semantically-visible liquid (the coffee).
        if self.output_ml > 0 { Some((Liquid::Coffee, self.output_ml)) }
        else if self.reservoir_ml > 0 { Some((Liquid::Water, self.reservoir_ml)) }
        else { None }
    }
    fn capacity_ml(&self) -> u32 { Self::OUTPUT_CAPACITY }
    fn add_liquid(&mut self, kind: Liquid, ml: u32) -> u32 {
        // Only accepts water into the reservoir.
        if kind != Liquid::Water { return ml; }
        let room = Self::RESERVOIR_CAPACITY.saturating_sub(self.reservoir_ml);
        let taken = ml.min(room);
        self.reservoir_ml += taken;
        ml - taken
    }
    fn drain_liquid(&mut self, ml: u32) -> (Liquid, u32) {
        // Draining drains coffee (the output), not water.
        let taken = ml.min(self.output_ml);
        self.output_ml -= taken;
        (Liquid::Coffee, taken)
    }
}

impl Powered for CoffeeMaker {
    fn power_draw_watts(&self) -> u32 {
        if self.brew == BrewState::Brewing { Self::POWER_BREWING_W } else { 0 }
    }
    fn is_on(&self) -> bool { self.brew == BrewState::Brewing }
}

impl TimeDecaySeconds for CoffeeMaker {
    fn tick_real(&mut self, dt: f32) {
        if self.brew == BrewState::Brewing {
            self.brew_progress_s += dt;
            if self.brew_progress_s >= Self::BREW_DURATION_SEC {
                // Brew completes: drain water, fill output.
                self.reservoir_ml = self.reservoir_ml.saturating_sub(Self::WATER_PER_BREW_ML);
                self.output_ml = (self.output_ml + Self::COFFEE_PER_BREW_ML).min(Self::OUTPUT_CAPACITY);
                self.brew = BrewState::Done;
                self.brew_progress_s = 0.0;
            }
        }
    }
}

pub fn brew_progress_percent(elapsed_seconds: u32, total_seconds: u32) -> u32 {     if total_seconds == 0 { return 0; }
    (elapsed_seconds.saturating_mul(100)) / total_seconds
 }

pub fn cups_from_ml(total_ml: u32, cup_size_ml: u32) -> u32 {     if cup_size_ml == 0 { return 0; }
    total_ml / cup_size_ml
 }

pub fn water_for_n_cups(n_cups: u32, ml_per_cup: u32) -> u32 { n_cups.saturating_mul(ml_per_cup) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_idle_with_water_no_coffee() {
        let cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        assert_eq!(cm.brew, BrewState::Idle);
        assert!(cm.reservoir_ml > 0);
        assert_eq!(cm.output_ml, 0);
    }
    #[test]
    fn cycle_starts_brew_when_water_present() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        cm.cycle();
        assert_eq!(cm.brew, BrewState::Brewing);
    }
    #[test]
    fn brew_completes_after_duration() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        cm.cycle();
        cm.tick_real(CoffeeMaker::BREW_DURATION_SEC + 0.1);
        assert_eq!(cm.brew, BrewState::Done);
        assert_eq!(cm.output_ml, CoffeeMaker::COFFEE_PER_BREW_ML);
    }
    #[test]
    fn cycle_from_done_empties_output() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        cm.cycle();
        cm.tick_real(CoffeeMaker::BREW_DURATION_SEC + 0.1);
        cm.cycle(); // Done -> Idle, pours out.
        assert_eq!(cm.output_ml, 0);
        assert_eq!(cm.brew, BrewState::Idle);
    }
    #[test]
    fn cycle_without_water_stays_idle() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        // Drain the reservoir.
        for _ in 0..10 {
            cm.cycle();
            cm.tick_real(CoffeeMaker::BREW_DURATION_SEC + 1.0);
            cm.cycle();
        }
        // Reservoir exhausted.
        assert!(cm.reservoir_ml < CoffeeMaker::WATER_PER_BREW_ML);
        let before = cm.brew;
        cm.cycle();
        assert_eq!(cm.brew, before);
    }
    #[test]
    fn powered_only_while_brewing() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        assert_eq!(cm.power_draw_watts(), 0);
        cm.cycle();
        assert_eq!(cm.power_draw_watts(), CoffeeMaker::POWER_BREWING_W);
    }
    #[test]
    fn add_water_up_to_capacity_returns_overflow() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        // Reservoir starts full.
        let overflow = cm.add_liquid(Liquid::Water, 100);
        assert_eq!(overflow, 100);
    }
    #[test]
    fn add_wrong_liquid_returns_all_as_overflow() {
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        let overflow = cm.add_liquid(Liquid::Coffee, 100);
        assert_eq!(overflow, 100);
    }

    #[test]
    fn test_brew_progress_percent() {
                assert_eq!(brew_progress_percent(0, 60), 0);
        assert_eq!(brew_progress_percent(30, 60), 50);
        assert_eq!(brew_progress_percent(60, 60), 100);
        assert_eq!(brew_progress_percent(10, 0), 0);

    }

    #[test]
    fn test_cups_from_ml() {
                assert_eq!(cups_from_ml(750, 250), 3);
        assert_eq!(cups_from_ml(100, 0), 0);
        assert_eq!(cups_from_ml(0, 250), 0);

    }

    #[test]
    fn test_water_for_n_cups() {
        assert_eq!(water_for_n_cups(4, 250), 1000); assert_eq!(water_for_n_cups(0, 250), 0);
    }
}
