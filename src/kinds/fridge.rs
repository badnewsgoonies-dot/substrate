//! Fridge: Openable + Container<FoodItem> + Powered + TimeDecayMinutes.
//!
//! Interior freshness ticks in in-game minutes. Three fixed slots for
//! now; when we grow Container to handle variable-size inventories the
//! fridge can expand.

use crate::space::Vec2;
use crate::traits::{Container, Openable, Powered, TimeDecayMinutes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodItem {
    Milk,
    Eggs,
    Leftovers,
}

impl FoodItem {
    pub const fn starting_minutes(self) -> u32 {
        match self {
            FoodItem::Milk => 2 * 24 * 60,
            FoodItem::Eggs => 3 * 24 * 60,
            FoodItem::Leftovers => 1 * 24 * 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FridgeSlot {
    pub item: Option<FoodItem>,
    pub minutes_remaining: u32,
}

impl FridgeSlot {
    pub const fn empty() -> Self { Self { item: None, minutes_remaining: 0 } }
    pub fn stock(item: FoodItem) -> Self {
        Self { item: Some(item), minutes_remaining: item.starting_minutes() }
    }
    pub fn is_fresh(&self) -> bool { self.item.is_some() && self.minutes_remaining > 0 }
    pub fn tick_minutes(&mut self, dm: u32) -> bool {
        if self.item.is_none() { return false; }
        let was_fresh = self.minutes_remaining > 0;
        self.minutes_remaining = self.minutes_remaining.saturating_sub(dm);
        was_fresh && self.minutes_remaining == 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fridge {
    pub pos: Vec2,
    open: bool,
    pub slots: [FridgeSlot; 3],
}

impl Fridge {
    pub const POWER_W: u32 = 100;

    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            open: false,
            slots: [
                FridgeSlot::stock(FoodItem::Milk),
                FridgeSlot::stock(FoodItem::Eggs),
                FridgeSlot::stock(FoodItem::Leftovers),
            ],
        }
    }

    pub fn fresh_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_fresh()).count()
    }
}

impl Openable for Fridge {
    fn is_open(&self) -> bool { self.open }
    fn toggle_open(&mut self) { self.open = !self.open; }
}

impl Powered for Fridge {
    fn power_draw_watts(&self) -> u32 { Self::POWER_W }
    fn is_on(&self) -> bool { true }
}

impl TimeDecayMinutes for Fridge {
    fn tick_minutes(&mut self, dm: u32) {
        for s in self.slots.iter_mut() { s.tick_minutes(dm); }
    }
}

// Container<FoodItem>-style slot accessors. Stays kind-specific for now
// because the slot shape carries freshness, not just the item. When a
// full Container trait emerges (with typed Item enum), fridge will be
// a Container<FoodItem> implementer.
impl Container for Fridge {
    fn slot_count(&self) -> usize { self.slots.len() }
    fn slot_is_empty(&self, idx: usize) -> bool {
        self.slots.get(idx).map_or(true, |s| s.item.is_none())
    }
    fn slot_label(&self, idx: usize) -> Option<&'static str> {
        self.slots.get(idx).and_then(|s| s.item.map(|it| match it {
            FoodItem::Milk => "milk",
            FoodItem::Eggs => "eggs",
            FoodItem::Leftovers => "leftovers",
        }))
    }
}

pub fn minutes_until_spoiled(initial_minutes: u32, elapsed: u32) -> u32 {     initial_minutes.saturating_sub(elapsed)
 }

pub fn fraction_fresh_percent(remaining_minutes: u32, initial_minutes: u32) -> u32 {     if initial_minutes == 0 { return 0; }
    (remaining_minutes.saturating_mul(100)) / initial_minutes
 }

pub fn spoilage_rate_per_hour(is_door_open: bool) -> u32 { if is_door_open { 5 } else { 1 } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_closed_and_stocked() {
        let f = Fridge::new(Vec2::new(0.0, 0.0));
        assert!(!f.is_open());
        assert_eq!(f.fresh_count(), 3);
    }
    #[test]
    fn toggle_open_works() {
        let mut f = Fridge::new(Vec2::new(0.0, 0.0));
        f.toggle_open(); assert!(f.is_open());
        f.toggle_open(); assert!(!f.is_open());
    }
    #[test]
    fn tick_minutes_decays_contents() {
        let mut f = Fridge::new(Vec2::new(0.0, 0.0));
        let before = f.slots[0].minutes_remaining;
        f.tick_minutes(60);
        assert_eq!(f.slots[0].minutes_remaining, before - 60);
    }
    #[test]
    fn spoilage_does_not_underflow() {
        let mut f = Fridge::new(Vec2::new(0.0, 0.0));
        f.tick_minutes(u32::MAX);
        assert_eq!(f.slots[0].minutes_remaining, 0);
        f.tick_minutes(10);
        assert_eq!(f.slots[0].minutes_remaining, 0);
    }
    #[test]
    fn draws_power_always() {
        let f = Fridge::new(Vec2::new(0.0, 0.0));
        assert!(f.is_on());
        assert!(f.power_draw_watts() > 0);
    }
    #[test]
    fn container_labels() {
        let f = Fridge::new(Vec2::new(0.0, 0.0));
        assert_eq!(f.slot_count(), 3);
        assert_eq!(f.slot_label(0), Some("milk"));
        assert_eq!(f.slot_label(1), Some("eggs"));
        assert_eq!(f.slot_label(2), Some("leftovers"));
    }

    #[test]
    fn test_minutes_until_spoiled() {
                assert_eq!(minutes_until_spoiled(1000, 300), 700);
        assert_eq!(minutes_until_spoiled(50, 999), 0);

    }

    #[test]
    fn test_fraction_fresh_percent() {
                assert_eq!(fraction_fresh_percent(0, 100), 0);
        assert_eq!(fraction_fresh_percent(50, 100), 50);
        assert_eq!(fraction_fresh_percent(100, 0), 0);

    }

    #[test]
    fn test_spoilage_rate_per_hour() {
        assert_eq!(spoilage_rate_per_hour(true), 5); assert_eq!(spoilage_rate_per_hour(false), 1);
    }
}
