//! Dresser: larger bedroom furniture with four drawer slots for clothes.
// Maximum number of drawers a dresser unit can have
pub const MAX_DRESSER_DRAWERS: u32 = 4;
// Standard dresser drawer interior height in centimetres
pub const DRESSER_DRAWER_HEIGHT_CM: u32 = 25;
// Total storage capacity of the dresser in cubic centimetres
pub const DRESSER_TOTAL_CAPACITY_CM3: u32 = 180000;




use crate::space::Vec2;
use crate::traits::Container;

#[derive(Debug, Clone, Copy)]
pub struct Dresser {
    pub pos: Vec2,
}

impl Dresser {
    pub fn new(pos: Vec2) -> Self { Self { pos } }
}

impl Container for Dresser {
    fn slot_count(&self) -> usize { 4 }
    fn slot_is_empty(&self, _idx: usize) -> bool { true }
    fn slot_label(&self, _idx: usize) -> Option<&'static str> { None }
}

pub fn total_bedroom_slots(dressers: u32, nightstands: u32) -> u32 {     dressers * 4 + nightstands
 }

pub fn is_valid_drawer(idx: u32) -> bool {     idx < 4
 }

pub fn next_drawer(current: u32) -> u32 {     current.saturating_add(1).min(4)
 }

pub fn drawer_delta(a: u32, b: u32) -> i32 {     a as i32 - b as i32
 }

pub fn clamp_drawer_idx(idx: u32) -> u32 {     idx.min(3)
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_four_drawers() {
        let d = Dresser::new(Vec2::new(0.0, 0.0));
        assert_eq!(d.slot_count(), 4);
        for i in 0..4 {
            assert!(d.slot_is_empty(i));
            assert!(d.slot_label(i).is_none());
        }
    }

    #[test]
    fn test_total_bedroom_slots() {
            assert_eq!(total_bedroom_slots(1, 1), 5);
    assert_eq!(total_bedroom_slots(0, 3), 3);

    }

    #[test]
    fn test_is_valid_drawer() {
                assert!(is_valid_drawer(0));
        assert!(is_valid_drawer(3));
        assert!(!is_valid_drawer(4));

    }

    #[test]
    fn test_next_drawer() {
            assert_eq!(next_drawer(0), 1);
    assert_eq!(next_drawer(3), 4);
    assert_eq!(next_drawer(4), 4);

    }

    #[test]
    fn test_drawer_delta() {
            assert_eq!(drawer_delta(4, 1), 3);
    assert_eq!(drawer_delta(1, 4), -3);

    }

    #[test]
    fn test_clamp_drawer_idx() {
            assert_eq!(clamp_drawer_idx(2), 2);
    assert_eq!(clamp_drawer_idx(9), 3);

    }
}
