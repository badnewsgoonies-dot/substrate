//! Window: Switchable (curtain open/closed) + LightEmitter (daylight).
//!
//! The window is the source of the morning wash. Its light_contribution
//! consults the in-game clock: warm orange 6-9am, cool white 9am-5pm,
//! golden 5-7pm, deep blue or zero at night. When the curtain is closed
//! (state drawn), intensity drops to ~10%.
//!
//! Named `WindowObj` because `Window` would collide with `minifb::Window`.

use crate::kinds::LightContribution;
use crate::space::Vec2;
use crate::time::InGameClock;
use crate::traits::Switchable;

#[derive(Debug, Clone, Copy)]
pub struct WindowObj {
    pub pos: Vec2,
    /// True = curtain is OPEN (light flows in). Matches Switchable "on".
    curtain_open: bool,
}

impl WindowObj {
    pub fn new(pos: Vec2) -> Self { Self { pos, curtain_open: true } }

    /// Light coming through the window, given time of day and curtain.
    pub fn light_contribution(&self, clock: &InGameClock) -> LightContribution {
        let hour = clock.hour_fractional();
        let base_mul = if self.curtain_open { 1.0 } else { 0.10 };

        // Piecewise-linear sun model:
        //   dawn   5-7   : low warm orange rising
        //   morning 7-10 : warm orange peak ~7.5, cooling
        //   midday  10-16: neutral white high intensity
        //   dusk    16-19: golden warm peak ~18
        //   night   19-5 : very faint cool blue moonlight
        let (rgb, intensity) = if (5.0..7.0).contains(&hour) {
            let t = (hour - 5.0) / 2.0;
            ((255, (120.0 + t * 40.0) as u8, (40.0 + t * 40.0) as u8), 0.25 * t)
        } else if (7.0..10.0).contains(&hour) {
            let t = (hour - 7.0) / 3.0;
            let warm = 1.0 - (hour - 7.5).abs() / 1.5;
            let warmth = warm.max(0.0);
            (
                (255, (160.0 + warmth * 40.0) as u8, (80.0 + warmth * 40.0) as u8),
                0.5 * (0.5 + 0.5 * (1.0 - t) + warmth * 0.3),
            )
        } else if (10.0..16.0).contains(&hour) {
            ((245, 245, 235), 0.85)
        } else if (16.0..19.0).contains(&hour) {
            let t = (hour - 16.0) / 3.0;
            let golden = 1.0 - (hour - 18.0).abs() / 2.0;
            let g = golden.max(0.0);
            (
                (255, (180.0 - t * 40.0) as u8, (100.0 - t * 40.0) as u8),
                0.7 * (1.0 - t) + g * 0.2,
            )
        } else {
            ((40, 60, 110), 0.05)
        };

        LightContribution { rgb, intensity: intensity * base_mul }
    }
}

impl Switchable for WindowObj {
    fn is_on(&self) -> bool { self.curtain_open }
    fn toggle(&mut self) { self.curtain_open = !self.curtain_open; }
}

pub fn is_daylight_hour(hour_of_day: u32) -> bool {     hour_of_day >= 6 && hour_of_day < 20
 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_open() {
        let w = WindowObj::new(Vec2::new(0.0, 0.0));
        assert!(w.is_on());
    }
    #[test]
    fn closing_cuts_intensity() {
        let mut w = WindowObj::new(Vec2::new(0.0, 0.0));
        let clock = InGameClock::at_hour(12);
        let open_intensity = w.light_contribution(&clock).intensity;
        w.toggle();
        let closed_intensity = w.light_contribution(&clock).intensity;
        assert!(closed_intensity < open_intensity);
        assert!(closed_intensity > 0.0); // Not totally black.
    }
    #[test]
    fn morning_is_warm_orange() {
        let w = WindowObj::new(Vec2::new(0.0, 0.0));
        let mut clock = InGameClock::at_hour(7);
        clock.advance_in_game_seconds(30 * 60); // 7:30
        let c = w.light_contribution(&clock);
        // r > g > b in a warm orange.
        assert!(c.rgb.0 >= c.rgb.1);
        assert!(c.rgb.1 >= c.rgb.2);
        assert!(c.intensity > 0.3);
    }
    #[test]
    fn midnight_is_dim_cool() {
        let w = WindowObj::new(Vec2::new(0.0, 0.0));
        let clock = InGameClock::at_hour(0);
        let c = w.light_contribution(&clock);
        assert!(c.intensity < 0.15);
        // Blue-ish: b > r.
        assert!(c.rgb.2 > c.rgb.0);
    }
    #[test]
    fn midday_is_bright_neutral() {
        let w = WindowObj::new(Vec2::new(0.0, 0.0));
        let clock = InGameClock::at_hour(12);
        let c = w.light_contribution(&clock);
        assert!(c.intensity > 0.7);
    }

    #[test]
    fn test_is_daylight_hour() {
                assert!(!is_daylight_hour(3));
        assert!(is_daylight_hour(12));
        assert!(is_daylight_hour(6));
        assert!(!is_daylight_hour(20));
        assert!(!is_daylight_hour(23));

    }
}
