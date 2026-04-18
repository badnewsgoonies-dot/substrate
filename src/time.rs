//! Time primitive — unchanged from v0.5.

pub const REAL_SECONDS_PER_IN_GAME_DAY: u32 = 1200;
pub const IN_GAME_SECONDS_PER_IN_GAME_DAY: u32 = 86_400;
pub const COMPRESSION_RATIO: u32 =
    IN_GAME_SECONDS_PER_IN_GAME_DAY / REAL_SECONDS_PER_IN_GAME_DAY;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InGameClock {
    pub day: u32,
    pub second_of_day: u32,
    pub sub_seconds: f32,
}

impl InGameClock {
    pub fn new() -> Self { Self::default() }
    pub fn at_hour(hour: u32) -> Self {
        Self { day: 0, second_of_day: (hour % 24) * 3600, sub_seconds: 0.0 }
    }
    pub fn advance_real_seconds(&mut self, real_seconds: u32) {
        let in_game_delta = real_seconds.saturating_mul(COMPRESSION_RATIO);
        self.advance_in_game_seconds(in_game_delta);
    }
    pub fn advance_real_seconds_f32(&mut self, real_seconds: f32) {
        let total_in_game = real_seconds * (COMPRESSION_RATIO as f32) + self.sub_seconds;
        let whole = total_in_game.floor();
        self.sub_seconds = total_in_game - whole;
        if whole > 0.0 { self.advance_in_game_seconds(whole as u32); }
    }
    pub fn advance_in_game_seconds(&mut self, delta: u32) {
        let total = self.second_of_day as u64 + delta as u64;
        let extra_days = (total / IN_GAME_SECONDS_PER_IN_GAME_DAY as u64) as u32;
        self.day = self.day.saturating_add(extra_days);
        self.second_of_day = (total % IN_GAME_SECONDS_PER_IN_GAME_DAY as u64) as u32;
    }
    pub fn hour_fractional(&self) -> f32 { self.second_of_day as f32 / 3600.0 }
    pub fn hour(&self) -> u32 { self.second_of_day / 3600 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ratio_is_72() { assert_eq!(COMPRESSION_RATIO, 72); }
    #[test]
    fn twenty_real_minutes_rolls_one_day() {
        let mut c = InGameClock::new();
        c.advance_real_seconds(REAL_SECONDS_PER_IN_GAME_DAY);
        assert_eq!(c.day, 1);
        assert_eq!(c.second_of_day, 0);
    }
    #[test]
    fn float_advance_accumulates_remainder() {
        let mut c = InGameClock::new();
        for _ in 0..5 { c.advance_real_seconds_f32(1.0 / 60.0); }
        assert_eq!(c.second_of_day, 6);
    }
}
