//! Routine primitive — unchanged from v0.5.

use crate::space::Vec2;
use crate::time::InGameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity { Sleeping, InKitchen, Out }

#[derive(Debug, Clone, Copy)]
pub struct RoutineSlot {
    pub start_hour: u32,
    pub end_hour: u32,
    pub activity: Activity,
}

impl RoutineSlot {
    pub fn contains_hour(&self, hour: u32) -> bool {
        let h = hour % 24;
        if self.start_hour <= self.end_hour {
            h >= self.start_hour && h < self.end_hour
        } else {
            h >= self.start_hour || h < self.end_hour
        }
    }
}

#[derive(Debug, Clone)]
pub struct Schedule { pub slots: Vec<RoutineSlot> }

impl Schedule {
    pub fn new(slots: Vec<RoutineSlot>) -> Self { Self { slots } }
    pub fn activity_at(&self, clock: &InGameClock) -> Activity {
        let h = clock.hour();
        for slot in &self.slots {
            if slot.contains_hour(h) { return slot.activity; }
        }
        Activity::Out
    }
    pub fn roommate_default() -> Self {
        Self::new(vec![
            RoutineSlot { start_hour: 22, end_hour: 7, activity: Activity::Sleeping },
            RoutineSlot { start_hour: 7, end_hour: 9, activity: Activity::InKitchen },
            RoutineSlot { start_hour: 9, end_hour: 17, activity: Activity::Out },
            RoutineSlot { start_hour: 17, end_hour: 20, activity: Activity::InKitchen },
            RoutineSlot { start_hour: 20, end_hour: 22, activity: Activity::Out },
        ])
    }
}

pub fn activity_target(a: Activity) -> Option<Vec2> {
    match a {
        Activity::Sleeping => Some(Vec2::new(12.5, 3.5)),
        Activity::InKitchen => Some(Vec2::new(8.5, 10.5)),
        Activity::Out => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrap_midnight_slot_includes_0() {
        let s = RoutineSlot { start_hour: 22, end_hour: 7, activity: Activity::Sleeping };
        assert!(s.contains_hour(0));
        assert!(s.contains_hour(23));
        assert!(!s.contains_hour(7));
    }
    #[test]
    fn roommate_default_covers_typical_hours() {
        let sch = Schedule::roommate_default();
        assert_eq!(sch.activity_at(&InGameClock::at_hour(7)), Activity::InKitchen);
        assert_eq!(sch.activity_at(&InGameClock::at_hour(12)), Activity::Out);
        assert_eq!(sch.activity_at(&InGameClock::at_hour(23)), Activity::Sleeping);
    }
    #[test]
    fn activity_target_out_is_none() {
        assert!(activity_target(Activity::Out).is_none());
    }
}
