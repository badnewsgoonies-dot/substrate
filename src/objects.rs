//! Objects collection: the `Vec<Object>` world state.
//!
//! All cross-kind tick dispatch lives here, driven by trait accessors
//! on `Object`. Adding a new kind needs exactly zero changes to this
//! file — the accessors in `kinds::Object` do the routing.

use crate::kinds::{LightContribution, Object};
use crate::space::Vec2;
use crate::time::InGameClock;

pub struct Objects {
    items: Vec<Object>,
    /// Last total in-game minutes observed. `None` means never ticked;
    /// the first tick establishes the baseline without advancing decay.
    /// `Some(0)` is a valid observed state (clock exactly at day 0 / 0s).
    last_total_minutes: Option<u64>,
}

impl Objects {
    pub fn new() -> Self {
        Self { items: Vec::new(), last_total_minutes: None }
    }
    pub fn add(&mut self, obj: Object) { self.items.push(obj); }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
    pub fn get(&self, idx: usize) -> &Object { &self.items[idx] }
    pub fn get_mut(&mut self, idx: usize) -> &mut Object { &mut self.items[idx] }
    pub fn iter(&self) -> impl Iterator<Item = &Object> { self.items.iter() }

    pub fn nearest_within(&self, pos: Vec2, radius: f32) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, o) in self.items.iter().enumerate() {
            let d = o.pos() - pos;
            let d2 = d.x * d.x + d.y * d.y;
            if d2 > radius * radius { continue; }
            if best.map_or(true, |(_, b)| d2 < b) { best = Some((i, d2)); }
        }
        best.map(|(i, _)| i)
    }

    /// Advance all TimeDecayMinutes implementers against the clock.
    /// First call only establishes the baseline — decay starts from
    /// the second call onward. The baseline is an `Option<u64>` so
    /// that a clock reading of 0 minutes is distinguished from the
    /// never-ticked case.
    pub fn tick_time(&mut self, clock: &InGameClock) {
        let total_minutes = (clock.day as u64) * 24 * 60
            + (clock.second_of_day as u64) / 60;
        let prev = match self.last_total_minutes {
            None => {
                self.last_total_minutes = Some(total_minutes);
                return;
            }
            Some(p) => p,
        };
        let delta = total_minutes.saturating_sub(prev);
        if delta == 0 { return; }
        self.last_total_minutes = Some(total_minutes);
        let dm = delta as u32;
        for obj in self.items.iter_mut() {
            if let Some(decay) = obj.as_time_decay_minutes_mut() {
                decay.tick_minutes(dm);
            }
        }
    }

    /// Advance all TimeDecaySeconds implementers each frame.
    pub fn tick_real_seconds(&mut self, dt: f32) {
        for obj in self.items.iter_mut() {
            if let Some(decay) = obj.as_time_decay_seconds_mut() {
                decay.tick_real(dt);
            }
        }
    }

    /// Sum all LightEmitter contributions into a single ambient value.
    /// The window is the dominant contributor at most hours of the day;
    /// lamps add on top when switched on at night.
    pub fn aggregate_ambient(&self, clock: &InGameClock) -> LightContribution {
        let mut r = 0.0f32;
        let mut g = 0.0f32;
        let mut b = 0.0f32;
        let mut total_intensity = 0.0f32;
        for obj in self.items.iter() {
            if let Some(lc) = obj.as_light_emitter(clock) {
                let i = lc.intensity;
                if i <= 0.0 { continue; }
                r += lc.rgb.0 as f32 * i;
                g += lc.rgb.1 as f32 * i;
                b += lc.rgb.2 as f32 * i;
                total_intensity += i;
            }
        }
        if total_intensity <= 0.0 {
            return LightContribution::ZERO;
        }
        LightContribution {
            rgb: (
                (r / total_intensity).min(255.0) as u8,
                (g / total_intensity).min(255.0) as u8,
                (b / total_intensity).min(255.0) as u8,
            ),
            intensity: total_intensity.min(1.0),
        }
    }

    /// Total power draw from all Powered implementers.
    pub fn total_power_watts(&self) -> u32 {
        let mut sum = 0u32;
        for obj in self.items.iter() {
            if let Some(p) = obj.as_powered() {
                sum = sum.saturating_add(p.power_draw_watts());
            }
        }
        sum
    }
}

impl Default for Objects {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::{Bed, Fridge, Sink, Stove, WindowObj};

    #[test]
    fn empty_returns_nothing() {
        let os = Objects::new();
        assert!(os.is_empty());
        assert!(os.nearest_within(Vec2::new(0.0, 0.0), 100.0).is_none());
    }

    #[test]
    fn tick_time_decays_fridge() {
        let mut os = Objects::new();
        os.add(Object::Fridge(Fridge::new(Vec2::new(0.0, 0.0))));
        let mut c = InGameClock::at_hour(0);
        os.tick_time(&c);
        let start_min = match os.get(0) {
            Object::Fridge(f) => f.slots[0].minutes_remaining,
            _ => unreachable!(),
        };
        c.advance_in_game_seconds(60 * 60);
        os.tick_time(&c);
        let end_min = match os.get(0) {
            Object::Fridge(f) => f.slots[0].minutes_remaining,
            _ => unreachable!(),
        };
        assert_eq!(start_min - end_min, 60);
    }

    #[test]
    fn tick_real_accumulates_sink() {
        let mut os = Objects::new();
        os.add(Object::Sink(Sink::new(Vec2::new(0.0, 0.0))));
        os.get_mut(0).use_action(); // turn on
        os.tick_real_seconds(1.5);
        match os.get(0) {
            Object::Sink(s) => assert!((s.seconds_running - 1.5).abs() < 1e-4),
            _ => unreachable!(),
        }
    }

    #[test]
    fn aggregate_ambient_reflects_time() {
        let mut os = Objects::new();
        os.add(Object::Window(WindowObj::new(Vec2::new(0.0, 0.0))));
        let clock = InGameClock::at_hour(12);
        let amb = os.aggregate_ambient(&clock);
        assert!(amb.intensity > 0.5);
    }

    #[test]
    fn total_power_sums_powered_objects() {
        let mut os = Objects::new();
        os.add(Object::Fridge(Fridge::new(Vec2::new(0.0, 0.0))));
        os.add(Object::Bed(Bed::new(Vec2::new(0.0, 0.0)))); // unpowered
        let p = os.total_power_watts();
        assert_eq!(p, Fridge::POWER_W);
    }

    #[test]
    fn stove_power_scales_with_burners() {
        let mut os = Objects::new();
        os.add(Object::Stove(Stove::new(Vec2::new(0.0, 0.0))));
        assert_eq!(os.total_power_watts(), 0);
        os.get_mut(0).use_action(); // burner 0 low
        assert_eq!(os.total_power_watts(), Stove::POWER_PER_ACTIVE_BURNER_W);
    }
}
