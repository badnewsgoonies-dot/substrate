# substrate

First-person life-sim in pure Rust. A small raycast-rendered apartment with 18 interactive object kinds, a roommate NPC driven by needs-based AI, and an in-game clock at 72:1 compression.

## Architecture

### Factory-driven kind creation

`src/kindgen/` is a declarative factory: a `KindSpec` plus an `Archetype` generates a complete kind module. Four archetypes exist today:

- `SimpleLiquidContainer` — a vessel with a capacity and `LiquidContainer` impl (Mug)
- `SwitchableFaucet` — an on/off water source with flow rate (would cover Sink/Shower/BathSink; those are hand-crafted for now)
- `DecorativeItem` — position-only world object (Plant, Painting)
- `SimpleOpenable` — open/closed toggle via `Openable` trait (Diary)

Emitted source is byte-exact golden-tested against the on-disk file: `assert_eq!(emit_kind_source(&DIARY_SPEC), include_str!("../kinds/diary.rs"))`. If the factory and the kind file drift, the test fails on both sides.

### Universe manifest

`UNIVERSE_MANIFEST` is a static list of every kind that exists — generated or hand-crafted — with its archetype conformance. Meta-tests enforce:

- `UNIVERSE_MANIFEST` is bijective with `ALL_KIND_TAGS` (every kind tag has a manifest entry and vice versa)
- Every `GENERATED_SPECS` entry has its integration fragments (`pub mod`, `ObjectKindTag` variant, `Object` variant, match arms in `mind.rs`/`ai.rs`/`interact.rs`) literally present on disk
- Hand-crafted kinds declare at least one archetype conformance name

It is impossible to silently add or remove a kind from the universe.

### Render

`src/render.rs` is a raycast renderer with the dispatcher and primitives. Each non-trivial kind has its own sprite file under `src/render/sprites/<kind>.rs`. Simple rectangular kinds (Bed, Book, Window, Mug, Diary, Nightstand) are handled inline by `draw_flat`.

Adding a new sprite is three small edits: one new file, one line in `sprites/mod.rs`, two match arms in `draw_object`.

### Domain layers

- `body.rs` — player position, posture, momentum, needs decay
- `needs.rs` — four-need model: hunger, tiredness, hygiene, mood (u32, decays over time)
- `ai.rs` — needs-driven NPC behavior: if hungry, walk to fridge; if tired, walk to bed; etc.
- `interact.rs` — player use/approach verbs and their effects on needs + memory
- `mind.rs` — observational memory: `Mind` records `Fact(kind, coord, time)` tuples from interactions
- `objects.rs` — vector-like container for `Object` with aggregate ambient-light computation
- `routine.rs` — NPC daily schedules (hour-slot -> activity)
- `space.rs` — tile grid + DDA raycasting
- `time.rs` — in-game clock at 72:1 (20 real minutes = 1 game day)
- `traits.rs` — Openable, Switchable, Powered, LightEmitter, LiquidContainer, LiquidSource, Container, TimeDecayMinutes, TimeDecaySeconds, Cyclable, plus contract tests
- `world.rs` — single coordinator owning every subsystem

## The apartment

18 objects + 1 NPC. Bedroom has bed, nightstand (with mug + diary), lamp, dresser, and a painting by the dresser. Kitchen has fridge, stove, sink, coffee maker, book on the counter, plant by the window. Bathroom has toilet, bath-sink, shower, bathtub. One roommate with a daily schedule.

Game starts at 7:14 am with full needs. Neglect it and you will get hungry, tired, dirty, sad — in that order of urgency.

## Build

    cargo run         # play
    cargo test        # full suite
    cargo test --lib  # lib tests only

## Counts (as of this commit)

- 49 source files
- ~6,500 LOC
- 268 tests (1 test per ~24 LOC)
- 4 archetypes
- 18 kinds (14 hand-crafted + 4 generated)
- 1 NPC

## License

TBD.
