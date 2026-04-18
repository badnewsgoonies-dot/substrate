//! Bed: two-state (Made/Unmade). Stays its own kind for now — not joining
//! Openable because the Made/Unmade semantics are distinct from accessibility.

use crate::space::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedState { Made, Unmade }

#[derive(Debug, Clone, Copy)]
pub struct Bed {
    pub pos: Vec2,
    pub state: BedState,
}

impl Bed {
    pub fn new(pos: Vec2) -> Self { Self { pos, state: BedState::Unmade } }
    pub fn is_unmade(&self) -> bool { self.state == BedState::Unmade }
    pub fn use_action(&mut self) {
        self.state = match self.state {
            BedState::Made => BedState::Unmade,
            BedState::Unmade => BedState::Made,
        };
    }
}

pub fn rest_score(minutes_slept: u32) -> u8 {     if minutes_slept >= 480 { return 100; }
    (minutes_slept / 5).min(100) as u8
 }

pub fn sleep_deficit_minutes(target: u32, actual: u32) -> u32 {     target.saturating_sub(actual)
 }

pub fn minutes_until_alarm(now_hour: u32, now_minute: u32, alarm_hour: u32, alarm_minute: u32) -> u32 { let now = now_hour.saturating_mul(60).saturating_add(now_minute); let alarm = alarm_hour.saturating_mul(60).saturating_add(alarm_minute); if alarm > now { alarm - now } else { 1440u32.saturating_sub(now).saturating_add(alarm) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bed_starts_unmade() {
        let b = Bed::new(Vec2::new(0.0, 0.0));
        assert!(b.is_unmade());
    }
    #[test]
    fn use_toggles_state() {
        let mut b = Bed::new(Vec2::new(0.0, 0.0));
        b.use_action();
        assert!(!b.is_unmade());
        b.use_action();
        assert!(b.is_unmade());
    }

    #[test]
    fn test_rest_score() {
                assert_eq!(rest_score(0), 0);
        assert_eq!(rest_score(480), 100);
        assert_eq!(rest_score(1000), 100);
        assert_eq!(rest_score(60), 12);

    }

    #[test]
    fn test_sleep_deficit_minutes() {
                assert_eq!(sleep_deficit_minutes(480, 400), 80);
        assert_eq!(sleep_deficit_minutes(480, 500), 0);

    }

    #[test]
    fn test_minutes_until_alarm() {
        assert_eq!(minutes_until_alarm(8, 0, 9, 30), 90); assert_eq!(minutes_until_alarm(23, 0, 1, 0), 120); assert_eq!(minutes_until_alarm(9, 30, 9, 30), 1440);
    }
}
