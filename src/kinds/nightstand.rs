//! Nightstand: small bedside furniture with one storage slot.
// Number of slots available on a nightstand
pub const NIGHTSTAND_SLOT_COUNT: u32 = 1;
// Maximum weight in kilograms that a nightstand can support
pub const NIGHTSTAND_MAX_WEIGHT_KG: u8 = 15;
// Standard width of the nightstand in centimetres
pub const NIGHTSTAND_WIDTH_CM: u32 = 45;




use crate::space::Vec2;
use crate::traits::Container;

#[derive(Debug, Clone, Copy)]
pub struct Nightstand {
    pub pos: Vec2,
}

impl Nightstand {
    pub fn new(pos: Vec2) -> Self { Self { pos } }
}

impl Container for Nightstand {
    fn slot_count(&self) -> usize { 1 }
    fn slot_is_empty(&self, _idx: usize) -> bool { true }
    fn slot_label(&self, _idx: usize) -> Option<&'static str> { None }
}

pub fn nightstand_can_hold(idx: u32) -> bool {     idx < 1
 }

pub fn slot_offset(idx: usize) -> usize {     idx * 64
 }

pub fn nightstand_volume_cm3(depth: u32, width: u32, height: u32) -> u32 {     depth * width * height
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_one_empty_slot() {
        let n = Nightstand::new(Vec2::new(0.0, 0.0));
        assert_eq!(n.slot_count(), 1);
        assert!(n.slot_is_empty(0));
        assert!(n.slot_label(0).is_none());
    }

    #[test]
    fn test_nightstand_can_hold() {
                assert!(nightstand_can_hold(0));
        assert!(!nightstand_can_hold(1));

    }

    #[test]
    fn test_slot_offset() {
            assert_eq!(slot_offset(0), 0);
    assert_eq!(slot_offset(3), 192);

    }

    #[test]
    fn test_nightstand_volume_cm3() {
            assert_eq!(nightstand_volume_cm3(30, 45, 60), 81000);

    }
}
