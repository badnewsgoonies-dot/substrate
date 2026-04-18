//! Mind primitive.

use std::collections::HashMap;

use crate::kinds::ObjectKindTag;
use crate::space::Vec2;
use crate::time::InGameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MindObjectKind {
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

impl MindObjectKind {
    pub fn from_tag(t: ObjectKindTag) -> Self {
        match t {
            ObjectKindTag::Bed => Self::Bed,
            ObjectKindTag::Book => Self::Book,
            ObjectKindTag::CoffeeMaker => Self::CoffeeMaker,
            ObjectKindTag::Window => Self::Window,
            ObjectKindTag::Fridge => Self::Fridge,
            ObjectKindTag::Stove => Self::Stove,
            ObjectKindTag::Sink => Self::Sink,
            ObjectKindTag::Toilet => Self::Toilet,
            ObjectKindTag::Shower => Self::Shower,
            ObjectKindTag::Bathtub => Self::Bathtub,
            ObjectKindTag::BathSink => Self::BathSink,
            ObjectKindTag::Lamp => Self::Lamp,
            ObjectKindTag::Nightstand => Self::Nightstand,
            ObjectKindTag::Dresser => Self::Dresser,
            ObjectKindTag::Mug => Self::Mug,
            ObjectKindTag::Plant => Self::Plant,
            ObjectKindTag::Painting => Self::Painting,
            ObjectKindTag::Diary => Self::Diary,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactKey {
    SawObject(MindObjectKind, TileCoord),
    SawPerson(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord { pub x: i32, pub y: i32 }

impl TileCoord {
    pub fn from_pos(pos: Vec2) -> Self {
        Self { x: pos.x.floor() as i32, y: pos.y.floor() as i32 }
    }
}

impl From<Vec2> for TileCoord {
    fn from(v: Vec2) -> Self { TileCoord::from_pos(v) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fact {
    pub observed_at: InGameClock,
}

impl Fact {
    pub fn observed_at(clock: InGameClock) -> Self { Self { observed_at: clock } }
}

pub struct Mind {
    facts: HashMap<FactKey, Fact>,
}

impl Mind {
    pub fn new() -> Self { Self { facts: HashMap::new() } }
    pub fn know(&mut self, key: FactKey, fact: Fact) { self.facts.insert(key, fact); }
    pub fn recall(&self, key: &FactKey) -> Option<&Fact> { self.facts.get(key) }
    pub fn len(&self) -> usize { self.facts.len() }
    pub fn is_empty(&self) -> bool { self.facts.is_empty() }
}

impl Default for Mind {
    fn default() -> Self { Self::new() }
}
