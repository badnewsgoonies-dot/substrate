//! Player needs: hunger, tiredness, hygiene, mood.
//!
//! Each need is a u32 score in [0, NEED_MAX]. NEED_MAX means fully
//! satisfied; 0 means maximally deficient. Needs decay over in-game
//! minutes and are restored by specific appliance/object interactions.
//!
//! Decay is tuned so that a need starting at NEED_MAX drops to 0 in
//! roughly one in-game day of inattention. Deliberate abstention from
//! the apartment's rhythm (sleep, eat, wash) will make the player feel it.
//!
//! Design note: kept as a flat struct of primitive counters (no enums,
//! no allocations) so the whole state is trivially Copy + Clone and fits
//! into a single cache line. Mind/routine layers can snapshot it freely.

pub const NEED_MAX: u32 = 10_000;

/// Per-in-game-minute decay rates. A day is 1440 min; at decay=10/min
/// the full 10000 drains in 1000 min (~16.7 h).
pub const HUNGER_DECAY_PER_MIN: u32 = 10;
pub const TIREDNESS_DECAY_PER_MIN: u32 = 7;
pub const HYGIENE_DECAY_PER_MIN: u32 = 5;
pub const MOOD_DECAY_PER_MIN: u32 = 3;

/// Low-threshold boundaries. Below these, the player is 'feeling it'.
pub const HUNGRY_THRESHOLD: u32 = 3_000;
pub const EXHAUSTED_THRESHOLD: u32 = 2_500;
pub const DIRTY_THRESHOLD: u32 = 4_000;
pub const SAD_THRESHOLD: u32 = 3_500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Needs {
    pub hunger: u32,    // 0 = starving, NEED_MAX = sated
    pub tiredness: u32, // 0 = exhausted, NEED_MAX = fully rested
    pub hygiene: u32,   // 0 = filthy, NEED_MAX = clean
    pub mood: u32,      // 0 = miserable, NEED_MAX = content
}

impl Needs {
    /// Start fully satisfied.
    pub const fn new() -> Self {
        Self {
            hunger: NEED_MAX,
            tiredness: NEED_MAX,
            hygiene: NEED_MAX,
            mood: NEED_MAX,
        }
    }

    /// Saturating add, clamped to [0, NEED_MAX]. Used by all restore methods.
    fn saturating_restore(current: u32, amount: u32) -> u32 {
        current.saturating_add(amount).min(NEED_MAX)
    }

    /// Advance all needs by `minutes` of in-game time. Saturates at 0.
    pub fn apply_minute_decay(&mut self, minutes: u32) {
        self.hunger = self.hunger.saturating_sub(HUNGER_DECAY_PER_MIN.saturating_mul(minutes));
        self.tiredness = self.tiredness.saturating_sub(TIREDNESS_DECAY_PER_MIN.saturating_mul(minutes));
        self.hygiene = self.hygiene.saturating_sub(HYGIENE_DECAY_PER_MIN.saturating_mul(minutes));
        self.mood = self.mood.saturating_sub(MOOD_DECAY_PER_MIN.saturating_mul(minutes));
    }

    /// Eating food: restores hunger by `amount` and a touch of mood.
    /// A typical small meal is ~3000; a large dinner ~5000.
    pub fn eat(&mut self, amount: u32) {
        self.hunger = Self::saturating_restore(self.hunger, amount);
        // Eating makes you a bit happier.
        self.mood = Self::saturating_restore(self.mood, amount / 5);
    }

    /// Sleeping: `minutes` slept restores tiredness. Quality of sleep
    /// (e.g. rest_score) should be multiplied into amount by the caller.
    /// A full 480-min night restores the full NEED_MAX.
    pub fn sleep(&mut self, minutes: u32) {
        // Restore ~21 per minute: 480 min * 21 = 10080, just over NEED_MAX.
        let amount = minutes.saturating_mul(21);
        self.tiredness = Self::saturating_restore(self.tiredness, amount);
        self.mood = Self::saturating_restore(self.mood, minutes * 2);
    }

    /// Showering: `seconds` of shower restores hygiene.
    /// ~60 s gives you back a decent share.
    pub fn shower(&mut self, seconds: u32) {
        // 100 per second: 60 s -> 6000 hygiene restored.
        let amount = seconds.saturating_mul(100);
        self.hygiene = Self::saturating_restore(self.hygiene, amount);
    }

    /// Drinking coffee: small mood + tiredness bump (caffeine kick).
    pub fn drink_coffee(&mut self) {
        self.tiredness = Self::saturating_restore(self.tiredness, 1_500);
        self.mood = Self::saturating_restore(self.mood, 500);
    }

    /// Reading a book: mood only.
    pub fn read(&mut self, minutes: u32) {
        self.mood = Self::saturating_restore(self.mood, minutes.saturating_mul(30));
    }

    /// Using the toilet: small hygiene hit then restoration via sink.
    pub fn use_toilet(&mut self) {
        self.hygiene = self.hygiene.saturating_sub(200);
    }

    /// Washing hands at a sink.
    pub fn wash_hands(&mut self) {
        self.hygiene = Self::saturating_restore(self.hygiene, 500);
    }

    pub fn is_hungry(&self) -> bool { self.hunger < HUNGRY_THRESHOLD }
    pub fn is_exhausted(&self) -> bool { self.tiredness < EXHAUSTED_THRESHOLD }
    pub fn needs_shower(&self) -> bool { self.hygiene < DIRTY_THRESHOLD }
    pub fn is_sad(&self) -> bool { self.mood < SAD_THRESHOLD }

    /// Overall wellbeing: mean of the four needs. Useful for AI decisions.
    pub fn wellbeing(&self) -> u32 {
        (self.hunger + self.tiredness + self.hygiene + self.mood) / 4
    }

    /// The most urgent need, as a tag. Ties broken in declared order.
    pub fn most_urgent(&self) -> NeedTag {
        let mut worst = NeedTag::Hunger;
        let mut worst_val = self.hunger;
        if self.tiredness < worst_val { worst = NeedTag::Tiredness; worst_val = self.tiredness; }
        if self.hygiene < worst_val { worst = NeedTag::Hygiene; worst_val = self.hygiene; }
        if self.mood < worst_val { worst = NeedTag::Mood; }
        worst
    }
}

impl Default for Needs {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeedTag { Hunger, Tiredness, Hygiene, Mood }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_fully_satisfied() {
        let n = Needs::new();
        assert_eq!(n.hunger, NEED_MAX);
        assert_eq!(n.tiredness, NEED_MAX);
        assert_eq!(n.hygiene, NEED_MAX);
        assert_eq!(n.mood, NEED_MAX);
        assert!(!n.is_hungry());
        assert!(!n.is_exhausted());
        assert!(!n.needs_shower());
        assert!(!n.is_sad());
    }

    #[test]
    fn decay_drains_hunger_fastest() {
        let mut n = Needs::new();
        n.apply_minute_decay(100); // 100 in-game minutes
        assert_eq!(n.hunger, NEED_MAX - 100 * HUNGER_DECAY_PER_MIN);
        assert_eq!(n.tiredness, NEED_MAX - 100 * TIREDNESS_DECAY_PER_MIN);
        assert!(n.hunger < n.tiredness);
        assert!(n.tiredness < n.hygiene);
        assert!(n.hygiene < n.mood);
    }

    #[test]
    fn decay_saturates_at_zero() {
        let mut n = Needs::new();
        n.apply_minute_decay(100_000);
        assert_eq!(n.hunger, 0);
        assert_eq!(n.tiredness, 0);
        assert_eq!(n.hygiene, 0);
        assert_eq!(n.mood, 0);
    }

    #[test]
    fn eating_restores_hunger_and_a_bit_of_mood() {
        let mut n = Needs { hunger: 2000, tiredness: NEED_MAX, hygiene: NEED_MAX, mood: 5000 };
        n.eat(3000);
        assert_eq!(n.hunger, 5000);
        assert_eq!(n.mood, 5000 + 600); // amount/5 bonus
    }

    #[test]
    fn eating_clamps_at_max() {
        let mut n = Needs::new();
        n.eat(50_000);
        assert_eq!(n.hunger, NEED_MAX);
        assert_eq!(n.mood, NEED_MAX);
    }

    #[test]
    fn full_night_of_sleep_restores_max_tiredness() {
        let mut n = Needs { hunger: NEED_MAX, tiredness: 0, hygiene: NEED_MAX, mood: 5000 };
        n.sleep(480);
        assert_eq!(n.tiredness, NEED_MAX);
        assert!(n.mood > 5000);
    }

    #[test]
    fn shower_restores_hygiene() {
        let mut n = Needs { hunger: NEED_MAX, tiredness: NEED_MAX, hygiene: 0, mood: NEED_MAX };
        n.shower(60);
        assert_eq!(n.hygiene, 6000);
    }

    #[test]
    fn hungry_threshold_predicates() {
        let mut n = Needs::new();
        n.apply_minute_decay(800); // 800 * 10 = 8000 hunger drain -> at 2000
        assert!(n.is_hungry());
        n.eat(3000);
        assert!(!n.is_hungry());
    }

    #[test]
    fn coffee_bumps_tiredness_and_mood() {
        let mut n = Needs { hunger: NEED_MAX, tiredness: 3000, hygiene: NEED_MAX, mood: 4000 };
        n.drink_coffee();
        assert_eq!(n.tiredness, 4500);
        assert_eq!(n.mood, 4500);
    }

    #[test]
    fn wellbeing_is_mean_of_four() {
        let n = Needs { hunger: 10_000, tiredness: 6_000, hygiene: 4_000, mood: 0 };
        assert_eq!(n.wellbeing(), 5_000);
    }

    #[test]
    fn most_urgent_finds_worst_need() {
        let n = Needs { hunger: 8_000, tiredness: 2_000, hygiene: 5_000, mood: 9_000 };
        assert_eq!(n.most_urgent(), NeedTag::Tiredness);

        let n = Needs { hunger: 1_000, tiredness: 2_000, hygiene: 5_000, mood: 9_000 };
        assert_eq!(n.most_urgent(), NeedTag::Hunger);

        let n = Needs { hunger: 8_000, tiredness: 8_000, hygiene: 8_000, mood: 500 };
        assert_eq!(n.most_urgent(), NeedTag::Mood);
    }

    #[test]
    fn toilet_then_wash_is_net_positive_hygiene() {
        let mut n = Needs { hunger: NEED_MAX, tiredness: NEED_MAX, hygiene: 5000, mood: NEED_MAX };
        n.use_toilet();
        assert_eq!(n.hygiene, 4800);
        n.wash_hands();
        assert_eq!(n.hygiene, 5300);
    }

    #[test]
    fn reading_brightens_mood_proportional_to_minutes() {
        let mut n = Needs { hunger: NEED_MAX, tiredness: NEED_MAX, hygiene: NEED_MAX, mood: 1000 };
        n.read(30);
        assert_eq!(n.mood, 1000 + 900); // 30 * 30
    }

    #[test]
    fn a_day_without_care_makes_everything_urgent() {
        let mut n = Needs::new();
        n.apply_minute_decay(1440);
        // After a full day, hunger (10/min) is well past 0.
        assert!(n.is_hungry());
        assert!(n.is_exhausted());
        // Hygiene (5/min) drains 7200 -> 2800, which is < 4000 dirty threshold.
        assert!(n.needs_shower());
        // Mood (3/min) drains 4320 -> 5680, above 3500 sad threshold.
        assert!(!n.is_sad());
    }
}
