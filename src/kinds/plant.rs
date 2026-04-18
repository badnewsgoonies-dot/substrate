//! Plant: a decorative potted houseplant.

use crate::space::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Plant {
    pub pos: Vec2,
}

impl Plant {
    pub fn new(pos: Vec2) -> Self {
        Self { pos }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_pos() {
        let x = Plant::new(Vec2::new(3.0, 4.0));
        assert_eq!(x.pos, Vec2::new(3.0, 4.0));
    }
}
