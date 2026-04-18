//! Lamp: Switchable + Powered + LightEmitter.
//!
//! The first three-trait composition. When on: draws 40W, emits warm
//! yellow light that adds to the room's ambient. First real indoor light
//! source besides the stove's burner glow and the window's daylight.
// Red channel value for a warm incandescent light color
pub const WARM_INCANDESCENT_R: u8 = 255;
// Green channel value for a warm incandescent light color
pub const WARM_INCANDESCENT_G: u8 = 220;
// Blue channel value for a warm incandescent light color
pub const WARM_INCANDESCENT_B: u8 = 150;
// Rated lifetime in hours for a standard lamp bulb
pub const LAMP_BULB_LIFETIME_HOURS: u64 = 10000;
// Colour temperature in Kelvin for soft-white lamp bulbs
pub const SOFT_WHITE_TEMP_KELVIN: u16 = 2700;
// Idle power draw of a lamp in milliwatts
pub const LAMP_IDLE_DRAW_MW: u16 = 500;







use crate::kinds::LightContribution;
use crate::space::Vec2;
use crate::traits::{Powered, Switchable};

#[derive(Debug, Clone, Copy)]
pub struct Lamp {
    pub pos: Vec2,
    on: bool,
}

impl Lamp {
    pub const POWER_W: u32 = 40;

    pub fn new(pos: Vec2) -> Self { Self { pos, on: false } }

    /// Light contribution when on. None when off (caller can skip it
    /// entirely and save the aggregation work).
    pub fn light_contribution(&self) -> Option<LightContribution> {
        if !self.on { return None; }
        Some(LightContribution {
            rgb: (255, 220, 150),  // warm incandescent yellow
            intensity: 0.22,
        })
    }
}

impl Switchable for Lamp {
    fn is_on(&self) -> bool { self.on }
    fn toggle(&mut self) { self.on = !self.on; }
}

impl Powered for Lamp {
    fn power_draw_watts(&self) -> u32 {
        if self.on { Self::POWER_W } else { 0 }
    }
    fn is_on(&self) -> bool { self.on }
}

pub fn watts_consumed_when(on: bool) -> u32 {     if on { 40 } else { 0 }
 }

pub fn is_warm_tone(r: u8, g: u8, b: u8) -> bool {     r >= g && g >= b
 }

pub fn lumens_from_watts(watts: u32) -> u32 {     watts * 80
 }

pub fn safe_lumens(watts: u32) -> Result<u32, String> {     if watts > 1000 { return Err("too many watts".to_string()); }
    Ok(watts * 80)
 }

pub fn log_noop() -> () {  }

pub fn describe_lamp_state(on: bool) -> &'static str {     if on { "on" } else { "off" }
 }

pub fn lamp_bank_watts(lamps_on: u32) -> u32 {     lamps_on * 40
 }

pub fn all_lamps_off(on: u32) -> bool {     on == 0
 }

pub fn bank_lumens(n_on: u32, lumens_per_lamp: u32) -> u32 {     n_on.saturating_mul(lumens_per_lamp)
 }

pub fn watts_for_lumens(lumens: u32, lumens_per_watt: u32) -> u32 {     if lumens_per_watt == 0 { return 0; }
    lumens / lumens_per_watt
 }

pub fn is_dim(lumens: u32) -> bool { lumens < 200 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_off() {
        let l = Lamp::new(Vec2::new(0.0, 0.0));
        assert!(!Switchable::is_on(&l));
        assert_eq!(l.power_draw_watts(), 0);
        assert!(l.light_contribution().is_none());
    }

    #[test]
    fn toggle_turns_on_and_emits_light() {
        let mut l = Lamp::new(Vec2::new(0.0, 0.0));
        l.toggle();
        assert!(Switchable::is_on(&l));
        assert_eq!(l.power_draw_watts(), Lamp::POWER_W);
        let lc = l.light_contribution().unwrap();
        assert!(lc.intensity > 0.0);
        // Warm: r >= g >= b.
        assert!(lc.rgb.0 >= lc.rgb.1);
        assert!(lc.rgb.1 >= lc.rgb.2);
    }

    #[test]
    fn switchable_contract_holds() {
        let mut l = Lamp::new(Vec2::new(0.0, 0.0));
        let start = Switchable::is_on(&l);
        l.toggle();
        l.toggle();
        assert_eq!(Switchable::is_on(&l), start);
    }

    #[test]
    fn power_tracks_state() {
        let mut l = Lamp::new(Vec2::new(0.0, 0.0));
        assert_eq!(l.power_draw_watts(), 0);
        l.toggle();
        assert_eq!(l.power_draw_watts(), Lamp::POWER_W);
        l.toggle();
        assert_eq!(l.power_draw_watts(), 0);
    }

    #[test]
    fn test_watts_consumed_when() {
                assert_eq!(watts_consumed_when(true), 40);
        assert_eq!(watts_consumed_when(false), 0);

    }

    #[test]
    fn test_is_warm_tone() {
            assert_eq!(is_warm_tone(255, 220, 150), true);
    assert_eq!(is_warm_tone(100, 150, 200), false);

    }

    #[test]
    fn test_lumens_from_watts() {
                assert_eq!(lumens_from_watts(40), 3200);
        assert_eq!(lumens_from_watts(0), 0);

    }

    #[test]
    fn test_safe_lumens() {
            assert_eq!(safe_lumens(40), Ok(3200));
    assert!(safe_lumens(1500).is_err());

    }

    #[test]
    fn test_log_noop() {
            log_noop();

    }

    #[test]
    fn test_describe_lamp_state() {
            assert_eq!(describe_lamp_state(true), "on");
    assert_eq!(describe_lamp_state(false), "off");

    }

    #[test]
    fn test_lamp_bank_watts() {
            assert_eq!(lamp_bank_watts(0), 0);
    assert_eq!(lamp_bank_watts(3), 120);

    }

    #[test]
    fn test_all_lamps_off() {
            assert!(all_lamps_off(0));
    assert!(!all_lamps_off(1));

    }

    #[test]
    fn test_bank_lumens() {
                assert_eq!(bank_lumens(0, 800), 0);
        assert_eq!(bank_lumens(3, 500), 1500);

    }

    #[test]
    fn test_watts_for_lumens() {
                assert_eq!(watts_for_lumens(1600, 80), 20);
        assert_eq!(watts_for_lumens(100, 0), 0);

    }

    #[test]
    fn test_is_dim() {
        assert!(is_dim(100)); assert!(is_dim(199)); assert!(!is_dim(200)); assert!(!is_dim(1000));
    }
}
