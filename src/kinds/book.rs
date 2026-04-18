//! Book: Openable only. Closed <-> Open.

use crate::space::Vec2;
use crate::traits::Openable;

#[derive(Debug, Clone, Copy)]
pub struct Book {
    pub pos: Vec2,
    open: bool,
}

impl Book {
    pub fn new(pos: Vec2) -> Self { Self { pos, open: false } }
}

impl Openable for Book {
    fn is_open(&self) -> bool { self.open }
    fn toggle_open(&mut self) { self.open = !self.open; }
}

pub fn pages_read_in_minutes(minutes: u32, pages_per_minute: u32) -> u32 {     minutes.saturating_mul(pages_per_minute)
 }

pub fn chapters_covered(total_pages: u32, pages_per_chapter: u32) -> u32 {     if pages_per_chapter == 0 { return 0; }
    total_pages / pages_per_chapter
 }

pub fn estimate_reading_time_minutes(word_count: u32, words_per_minute: u32) -> u32 { if words_per_minute == 0 { return 0; } word_count / words_per_minute }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_closed() {
        let b = Book::new(Vec2::new(0.0, 0.0));
        assert!(!b.is_open());
    }
    #[test]
    fn toggle_opens_and_closes() {
        let mut b = Book::new(Vec2::new(0.0, 0.0));
        b.toggle_open(); assert!(b.is_open());
        b.toggle_open(); assert!(!b.is_open());
    }

    #[test]
    fn test_pages_read_in_minutes() {
                assert_eq!(pages_read_in_minutes(0, 5), 0);
        assert_eq!(pages_read_in_minutes(30, 2), 60);

    }

    #[test]
    fn test_chapters_covered() {
                assert_eq!(chapters_covered(100, 25), 4);
        assert_eq!(chapters_covered(20, 0), 0);
        assert_eq!(chapters_covered(0, 25), 0);

    }

    #[test]
    fn test_estimate_reading_time_minutes() {
        assert_eq!(estimate_reading_time_minutes(3000, 200), 15); assert_eq!(estimate_reading_time_minutes(1000, 0), 0);
    }
}
