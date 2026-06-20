//! substrate — first-person life simulation on a raycaster.
//!
//! Modules are flat per-primitive. Per-kind sprites and behaviors live
//! under kinds::. v0.8 apartment: bedroom (bed/nightstand/lamp/dresser)
//! + kitchen (coffee maker/book/window/fridge/stove/sink) + bathroom
//! (toilet/shower/bathtub/bath sink).
//!
//! v0.8.2 adds the Needs layer; v0.8.3 adds the AI layer; v0.8.4 adds
//! the World coordinator; v0.8.5 adds kindgen — declarative-spec-to-
//! Rust-source generator. Mug is the first golden-tested generated kind.

pub mod addressing;
pub mod ai;
pub mod body;
pub mod interact;
pub mod kindgen;
pub mod kinds;
pub mod mind;
pub mod needs;
pub mod objects;
pub mod people;
pub mod render;
pub mod texture;
pub mod routine;
pub mod scene;
pub mod spritegen;
pub mod space;
pub mod time;
pub mod traits;
pub mod world;
