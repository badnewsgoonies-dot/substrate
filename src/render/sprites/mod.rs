//! Per-kind pixel-level drawing. One module per kind with a
//! non-trivial sprite. Kinds that just need a filled rectangle
//! (Bed, Book, Window, Mug, Diary, Nightstand) are handled by
//! `render::draw_flat` directly from the `draw_object` dispatcher —
//! no file in this directory for them.
//!
//! Adding a new sprite: create `foo.rs` exposing
//! `pub(super) fn draw_foo(...)`, add `mod foo;` + `pub(super) use
//! foo::draw_foo;` below, and wire up two arms in
//! `render::draw_object` (one for dims in the `(base_h, aspect)`
//! match, one calling `sprites::draw_foo` in the dispatch match).

mod bath_sink;
mod bathtub;
mod coffee_maker;
mod dresser;
mod fridge;
mod lamp;
mod painting;
mod plant;
mod shower;
mod sink;
mod stove;
mod toilet;

pub(super) use bath_sink::draw_bath_sink;
pub(super) use bathtub::draw_bathtub;
pub(super) use coffee_maker::draw_coffee_maker;
pub(super) use dresser::draw_dresser;
pub(super) use fridge::draw_fridge;
pub(super) use lamp::draw_lamp;
pub(super) use painting::draw_painting;
pub(super) use plant::draw_plant;
pub(super) use shower::draw_shower;
pub(super) use sink::draw_sink;
pub(super) use stove::draw_stove;
pub(super) use toilet::draw_toilet;
