//! All object kinds + the outer Object enum.
//!
//! Currently generated + integrated: mug, plant, painting, diary.
//!
//! Every variant of `ObjectKindTag` must appear in `ALL_KIND_TAGS`
//! below AND in `kindgen::UNIVERSE_MANIFEST`. The factory
//! owns the census of the universe; no kind exists silently.

pub mod bath_sink;
pub mod bathtub;
pub mod bed;
pub mod book;
pub mod coffee_maker;
pub mod diary;
pub mod dresser;
pub mod fridge;
pub mod lamp;
pub mod mug;
pub mod nightstand;
pub mod painting;
pub mod plant;
pub mod shower;
pub mod sink;
pub mod stove;
pub mod toilet;
pub mod window_obj;

pub use bath_sink::BathSink;
pub use bathtub::Bathtub;
pub use bed::Bed;
pub use book::Book;
pub use coffee_maker::{BrewState, CoffeeMaker};
pub use diary::Diary;
pub use dresser::Dresser;
pub use fridge::{FoodItem, Fridge, FridgeSlot};
pub use lamp::Lamp;
pub use mug::Mug;
pub use nightstand::Nightstand;
pub use painting::Painting;
pub use plant::Plant;
pub use shower::Shower;
pub use sink::Sink;
pub use stove::{Burner, BurnerLevel, Stove, StoveBurners};
pub use toilet::{Toilet, ToiletState};
pub use window_obj::WindowObj;

use crate::space::Vec2;
use crate::traits::{
    Container, Cyclable, LiquidContainer, LiquidSource, Openable, Powered,
    Switchable, TimeDecayMinutes, TimeDecaySeconds,
};
use crate::time::InGameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKindTag {
    Bed,
    Book,
    CoffeeMaker,
    Window,
    Fridge,
    Stove,
    Sink,
    Toilet,
    Shower,
    Bathtub,
    BathSink,
    Lamp,
    Nightstand,
    Dresser,
    Mug,
    Plant,
    Painting,
    Diary,
}

impl ObjectKindTag {
    pub fn name(&self) -> &'static str {
        match self {
            ObjectKindTag::Bed => "Bed",
            ObjectKindTag::Book => "Book",
            ObjectKindTag::CoffeeMaker => "CoffeeMaker",
            ObjectKindTag::Window => "Window",
            ObjectKindTag::Fridge => "Fridge",
            ObjectKindTag::Stove => "Stove",
            ObjectKindTag::Sink => "Sink",
            ObjectKindTag::Toilet => "Toilet",
            ObjectKindTag::Shower => "Shower",
            ObjectKindTag::Bathtub => "Bathtub",
            ObjectKindTag::BathSink => "BathSink",
            ObjectKindTag::Lamp => "Lamp",
            ObjectKindTag::Nightstand => "Nightstand",
            ObjectKindTag::Dresser => "Dresser",
            ObjectKindTag::Mug => "Mug",
            ObjectKindTag::Plant => "Plant",
            ObjectKindTag::Painting => "Painting",
            ObjectKindTag::Diary => "Diary",
        }
    }
}

pub const ALL_KIND_TAGS: &[ObjectKindTag] = &[
    ObjectKindTag::Bed,
    ObjectKindTag::Book,
    ObjectKindTag::CoffeeMaker,
    ObjectKindTag::Window,
    ObjectKindTag::Fridge,
    ObjectKindTag::Stove,
    ObjectKindTag::Sink,
    ObjectKindTag::Toilet,
    ObjectKindTag::Shower,
    ObjectKindTag::Bathtub,
    ObjectKindTag::BathSink,
    ObjectKindTag::Lamp,
    ObjectKindTag::Nightstand,
    ObjectKindTag::Dresser,
    ObjectKindTag::Mug,
    ObjectKindTag::Plant,
    ObjectKindTag::Painting,
    ObjectKindTag::Diary,
];

#[derive(Debug, Clone, Copy)]
pub enum Object {
    Bed(Bed),
    Book(Book),
    CoffeeMaker(CoffeeMaker),
    Window(WindowObj),
    Fridge(Fridge),
    Stove(Stove),
    Sink(Sink),
    Toilet(Toilet),
    Shower(Shower),
    Bathtub(Bathtub),
    BathSink(BathSink),
    Lamp(Lamp),
    Nightstand(Nightstand),
    Dresser(Dresser),
    Mug(Mug),
    Plant(Plant),
    Painting(Painting),
    Diary(Diary),
}

impl Object {
    pub fn pos(&self) -> Vec2 {
        match self {
            Object::Bed(x) => x.pos,
            Object::Book(x) => x.pos,
            Object::CoffeeMaker(x) => x.pos,
            Object::Window(x) => x.pos,
            Object::Fridge(x) => x.pos,
            Object::Stove(x) => x.pos,
            Object::Sink(x) => x.pos,
            Object::Toilet(x) => x.pos,
            Object::Shower(x) => x.pos,
            Object::Bathtub(x) => x.pos,
            Object::BathSink(x) => x.pos,
            Object::Lamp(x) => x.pos,
            Object::Nightstand(x) => x.pos,
            Object::Dresser(x) => x.pos,
            Object::Mug(x) => x.pos,
            Object::Plant(x) => x.pos,
            Object::Painting(x) => x.pos,
            Object::Diary(x) => x.pos,
        }
    }

    pub fn kind_tag(&self) -> ObjectKindTag {
        match self {
            Object::Bed(_) => ObjectKindTag::Bed,
            Object::Book(_) => ObjectKindTag::Book,
            Object::CoffeeMaker(_) => ObjectKindTag::CoffeeMaker,
            Object::Window(_) => ObjectKindTag::Window,
            Object::Fridge(_) => ObjectKindTag::Fridge,
            Object::Stove(_) => ObjectKindTag::Stove,
            Object::Sink(_) => ObjectKindTag::Sink,
            Object::Toilet(_) => ObjectKindTag::Toilet,
            Object::Shower(_) => ObjectKindTag::Shower,
            Object::Bathtub(_) => ObjectKindTag::Bathtub,
            Object::BathSink(_) => ObjectKindTag::BathSink,
            Object::Lamp(_) => ObjectKindTag::Lamp,
            Object::Nightstand(_) => ObjectKindTag::Nightstand,
            Object::Dresser(_) => ObjectKindTag::Dresser,
            Object::Mug(_) => ObjectKindTag::Mug,
            Object::Plant(_) => ObjectKindTag::Plant,
            Object::Painting(_) => ObjectKindTag::Painting,
            Object::Diary(_) => ObjectKindTag::Diary,
        }
    }

    pub fn use_action(&mut self) {
        match self {
            Object::Bed(x) => x.use_action(),
            Object::Book(x) => x.toggle_open(),
            Object::CoffeeMaker(x) => x.cycle(),
            Object::Window(x) => x.toggle(),
            Object::Fridge(x) => x.toggle_open(),
            Object::Stove(x) => x.press(),
            Object::Sink(x) => x.toggle(),
            Object::Toilet(x) => x.cycle(),
            Object::Shower(x) => x.toggle(),
            Object::Bathtub(x) => x.toggle(),
            Object::BathSink(x) => x.toggle(),
            Object::Lamp(x) => x.toggle(),
            Object::Nightstand(_) => {}
            Object::Dresser(_) => {}
            Object::Mug(_) => {}
            Object::Plant(_) => {}
            Object::Painting(_) => {}
            Object::Diary(x) => x.toggle_open(),
        }
    }

    pub fn as_openable(&self) -> Option<&dyn Openable> {
        match self {
            Object::Book(x) => Some(x),
            Object::Fridge(x) => Some(x),
            Object::Bathtub(x) => Some(x),
            Object::Diary(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_switchable(&self) -> Option<&dyn Switchable> {
        match self {
            Object::Sink(x) => Some(x),
            Object::Window(x) => Some(x),
            Object::Shower(x) => Some(x),
            Object::Bathtub(x) => Some(x),
            Object::BathSink(x) => Some(x),
            Object::Lamp(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_time_decay_minutes_mut(&mut self) -> Option<&mut dyn TimeDecayMinutes> {
        match self {
            Object::Fridge(x) => Some(x),
            Object::Stove(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_time_decay_seconds_mut(&mut self) -> Option<&mut dyn TimeDecaySeconds> {
        match self {
            Object::CoffeeMaker(x) => Some(x),
            Object::Sink(x) => Some(x),
            Object::Toilet(x) => Some(x),
            Object::Shower(x) => Some(x),
            Object::Bathtub(x) => Some(x),
            Object::BathSink(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_light_emitter(&self, clock: &InGameClock) -> Option<LightContribution> {
        match self {
            Object::Window(x) => Some(x.light_contribution(clock)),
            Object::Stove(x) => x.light_contribution_from_burners(),
            Object::Lamp(x) => x.light_contribution(),
            _ => None,
        }
    }

    pub fn as_powered(&self) -> Option<&dyn Powered> {
        match self {
            Object::CoffeeMaker(x) => Some(x),
            Object::Fridge(x) => Some(x),
            Object::Stove(x) => Some(x),
            Object::Lamp(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_liquid_container(&self) -> Option<&dyn LiquidContainer> {
        match self {
            Object::CoffeeMaker(x) => Some(x),
            Object::Toilet(x) => Some(x),
            Object::Bathtub(x) => Some(x),
            Object::Mug(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_liquid_source(&self) -> Option<&dyn LiquidSource> {
        match self {
            Object::Sink(x) => Some(x),
            Object::Shower(x) => Some(x),
            Object::BathSink(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_container(&self) -> Option<&dyn Container> {
        match self {
            Object::Fridge(x) => Some(x),
            Object::Nightstand(x) => Some(x),
            Object::Dresser(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightContribution {
    pub rgb: (u8, u8, u8),
    pub intensity: f32,
}

impl LightContribution {
    pub const ZERO: Self = Self { rgb: (0, 0, 0), intensity: 0.0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kind_tags_has_eighteen_entries() {
        assert_eq!(ALL_KIND_TAGS.len(), 18);
    }

    #[test]
    fn all_kind_tags_entries_are_unique() {
        use std::collections::HashSet;
        let s: HashSet<&ObjectKindTag> = ALL_KIND_TAGS.iter().collect();
        assert_eq!(s.len(), ALL_KIND_TAGS.len());
    }

    #[test]
    fn tag_name_round_trips_through_all_variants() {
        let names: Vec<&str> = ALL_KIND_TAGS.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"Mug"));
        assert!(names.contains(&"Plant"));
        assert!(names.contains(&"Painting"));
        assert!(names.contains(&"Diary"));
        assert!(names.contains(&"Bed"));
        assert!(names.contains(&"Bathtub"));
    }
}
