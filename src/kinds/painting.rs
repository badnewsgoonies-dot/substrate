//! Painting: a framed piece of art on the wall.

use crate::space::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Painting {
    pub pos: Vec2,
}

impl Painting {
    pub fn new(pos: Vec2) -> Self {
        Self { pos }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_pos() {
        let x = Painting::new(Vec2::new(3.0, 4.0));
        assert_eq!(x.pos, Vec2::new(3.0, 4.0));
    }
}
