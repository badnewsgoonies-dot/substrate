//! Composable traits for object behaviors.
//!
//! The ten traits in this module are how every object kind declares what
//! it is and what it does. The discipline is strict:
//!
//!   * A trait captures ONE axis of behavior. If two axes share a method
//!     signature but mean different things in two kinds, they're two traits.
//!   * Interface before implementer: a trait should make sense even if no
//!     kind implements it yet.
//!   * Everything else stays kind-specific. These traits are a shallow
//!     cross-section, not a hierarchy.

// -- Openable ---------------------------------------------------------

/// Closed/open where "open" means the space beyond is accessible: see
/// inside, take things out, walk through. Fridge doors, cabinets, books,
/// apartment doors. NOT: light switches (Switchable), beds.
pub trait Openable {
    fn is_open(&self) -> bool;
    fn toggle_open(&mut self);
}

// -- Switchable --------------------------------------------------------

/// Binary state where "on" means the world reacts: water flows, power
/// is drawn, light is emitted. Sink taps, lamps, TVs, curtains.
pub trait Switchable {
    fn is_on(&self) -> bool;
    fn toggle(&mut self);
}

// -- Cyclable ----------------------------------------------------------

/// N ordered states, Use advances through them. Coffee maker brew state,
/// stove burner level, TV channel.
pub trait Cyclable {
    fn state_index(&self) -> usize;
    fn state_count(&self) -> usize;
    fn cycle(&mut self);
}

// -- Container ---------------------------------------------------------

/// Holds up to N items. Item type is kind-specific; this trait is the
/// cross-kind read surface only. Modifiers stay kind-specific because
/// valid-item rules differ.
pub trait Container {
    fn slot_count(&self) -> usize;
    fn slot_is_empty(&self, idx: usize) -> bool;
    fn slot_label(&self, idx: usize) -> Option<&'static str>;
}

// -- Liquid + LiquidContainer + LiquidSource --------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liquid {
    Water,
    Coffee,
    Tea,
    Juice,
    Milk,
    Beer,
}

/// Holds a liquid. Kettles, bottles, reservoirs, bathtubs.
pub trait LiquidContainer {
    fn liquid(&self) -> Option<(Liquid, u32)>;
    fn capacity_ml(&self) -> u32;
    /// Returns overflow_ml that didn't fit.
    fn add_liquid(&mut self, kind: Liquid, ml: u32) -> u32;
    /// Returns (liquid_kind, actually_removed_ml).
    fn drain_liquid(&mut self, ml: u32) -> (Liquid, u32);
}

/// Produces liquid when activated. Sink taps, kettle pours, fridge
/// water dispensers.
pub trait LiquidSource {
    fn produced_liquid(&self) -> Liquid;
    fn is_producing(&self) -> bool;
    fn produce(&mut self, dt_real_seconds: f32) -> (Liquid, u32);
}

// -- Time decay --------------------------------------------------------

/// State advances per in-game minute. Food spoilage, stove warmth,
/// plant hydration.
pub trait TimeDecayMinutes {
    fn tick_minutes(&mut self, dm: u32);
}

/// State advances per real second. Sink flow counter, coffee brew
/// progress, stopwatch.
pub trait TimeDecaySeconds {
    fn tick_real(&mut self, dt: f32);
}

// -- Powered -----------------------------------------------------------

/// Consumes electricity. Feeds the future circuit-breaker system.
/// The contract is soft: is_on() true means "actively drawing now";
/// is_on() false with a nonzero power_draw_watts() would be a bug.
pub trait Powered {
    fn power_draw_watts(&self) -> u32;
    fn is_on(&self) -> bool;
}

// LightEmitter is expressed via the inherent `as_light_emitter(&clock)`
// method on Object (returns Option<LightContribution>) because some
// emitters need the clock to compute their contribution. Putting the
// clock inside a trait method got ugly with borrows; an inherent method
// on the sum type is cleaner.

// -- Contract tests ---------------------------------------------------

#[cfg(test)]
mod contract_tests {
    use super::*;
    use crate::kinds::{Book, Fridge, Sink, WindowObj};
    use crate::space::Vec2;

    #[test]
    fn openable_book_toggle_twice_is_identity() {
        let mut b = Book::new(Vec2::new(0.0, 0.0));
        let start = b.is_open();
        b.toggle_open();
        b.toggle_open();
        assert_eq!(b.is_open(), start);
    }

    #[test]
    fn openable_fridge_toggle_twice_is_identity() {
        let mut f = Fridge::new(Vec2::new(0.0, 0.0));
        let start = f.is_open();
        f.toggle_open();
        f.toggle_open();
        assert_eq!(f.is_open(), start);
    }

    #[test]
    fn switchable_sink_toggle_twice_is_identity() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        let start = s.is_on();
        s.toggle();
        s.toggle();
        assert_eq!(s.is_on(), start);
    }

    #[test]
    fn switchable_window_toggle_twice_is_identity() {
        let mut w = WindowObj::new(Vec2::new(0.0, 0.0));
        let start = w.is_on();
        w.toggle();
        w.toggle();
        assert_eq!(w.is_on(), start);
    }

    #[test]
    fn liquid_container_zero_add_is_zero_overflow() {
        use crate::kinds::CoffeeMaker;
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        assert_eq!(cm.add_liquid(Liquid::Water, 0), 0);
    }

    #[test]
    fn liquid_container_over_drain_is_bounded() {
        use crate::kinds::CoffeeMaker;
        let mut cm = CoffeeMaker::new(Vec2::new(0.0, 0.0));
        // Output bowl starts empty; draining 1000 ml gets 0 ml.
        let (_, taken) = cm.drain_liquid(1000);
        assert_eq!(taken, 0);
    }

    #[test]
    fn liquid_source_off_produces_zero() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        assert!(!s.is_producing());
        let (_, ml) = s.produce(1.0);
        assert_eq!(ml, 0);
    }

    #[test]
    fn liquid_source_on_produces_positive() {
        let mut s = Sink::new(Vec2::new(0.0, 0.0));
        s.toggle();
        assert!(s.is_producing());
        let (kind, ml) = s.produce(1.0);
        assert_eq!(kind, Liquid::Water);
        assert!(ml > 0);
    }
}
