//! Diary: a private journal with a pressed-leather cover.

use crate::space::Vec2;
use crate::traits::Openable;

#[derive(Debug, Clone, Copy)]
pub struct Diary {
    pub pos: Vec2,
    open: bool,
}

impl Diary {
    pub fn new(pos: Vec2) -> Self {
        Self { pos, open: false }
    }
}

impl Openable for Diary {
    fn is_open(&self) -> bool { self.open }
    fn toggle_open(&mut self) { self.open = !self.open; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed() {
        let x = Diary::new(Vec2::new(0.0, 0.0));
        assert!(!x.is_open());
    }

    #[test]
    fn toggle_opens_and_closes() {
        let mut x = Diary::new(Vec2::new(0.0, 0.0));
        x.toggle_open();
        assert!(x.is_open());
        x.toggle_open();
        assert!(!x.is_open());
    }
}
