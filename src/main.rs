//! substrate — v0.8.4 (World coordinator wiring).
//!
//! The main loop is thin: handle input into `world.body`, call
//! `world.tick(dt)` to advance the whole simulation (clock, object
//! decay, player needs decay, NPC needs-driven AI), then resolve the
//! crosshair target, react to E-press, and render.

use substrate::interact::{Target, Verb};
use substrate::mind::{Fact, FactKey};
use substrate::render::{self, Frame, HEIGHT, WIDTH};
use substrate::world::World;

use minifb::{Key, KeyRepeat, Window, WindowOptions};

const FOV_DEG: f32 = 60.0;

fn main() {
    let mut world = World::new_apartment();

    let mut window = Window::new(
        "substrate \u{b7} v0.8.4 \u{b7} needs + AI + world",
        WIDTH, HEIGHT,
        WindowOptions { resize: false, ..WindowOptions::default() },
    ).expect("could not open window");
    window.set_target_fps(60);

    let fov_tan = (FOV_DEG.to_radians() * 0.5).tan();
    let mut frame = Frame::new();
    let mut last_mouse_x: Option<f32> = None;
    let mut last_time = std::time::Instant::now();
    let mut e_latched = false;
    let mut interact_pulse: f32 = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let dt = (now - last_time).as_secs_f32().min(0.05);
        last_time = now;

        // --- Player input ---
        if let Some((mx, _)) = window.get_mouse_pos(minifb::MouseMode::Pass) {
            if let Some(last) = last_mouse_x {
                let delta = (mx - last).clamp(-80.0, 80.0);
                world.body.turn(delta * 0.004);
            }
            last_mouse_x = Some(mx);
        }
        if window.is_key_down(Key::Left) { world.body.turn(-1.8 * dt); }
        if window.is_key_down(Key::Right) { world.body.turn(1.8 * dt); }

        let running = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
        let speed = if running { 3.2 } else { 1.8 };
        let mut fwd = 0.0;
        let mut strafe = 0.0;
        if window.is_key_down(Key::W) { fwd += speed * dt; }
        if window.is_key_down(Key::S) { fwd -= speed * dt; }
        if window.is_key_down(Key::A) { strafe -= speed * dt; }
        if window.is_key_down(Key::D) { strafe += speed * dt; }
        world.body.walk(&world.grid, fwd, strafe, 0.22, dt);

        // --- Advance simulation ---
        // Ticks the clock, objects (time + real), player needs decay,
        // and each NPC's needs-driven AI. No other sim tick is needed.
        world.tick(dt);

        // --- Target resolution (passive: update mind when looking) ---
        let target = world.resolve_target(fov_tan);
        if let Some(t) = target {
            match t {
                Target::Object(idx) => {
                    let obj = world.objects.get(idx);
                    world.mind.know(
                        FactKey::SawObject(
                            substrate::mind::MindObjectKind::from_tag(obj.kind_tag()),
                            obj.pos().into(),
                        ),
                        Fact::observed_at(world.clock),
                    );
                }
                Target::Npc(idx) => {
                    let n = &world.npcs[idx];
                    world.mind.know(
                        FactKey::SawPerson(n.name),
                        Fact::observed_at(world.clock),
                    );
                }
            }
        }

        // --- E press = interact ---
        let e_down = window.is_key_down(Key::E);
        if e_down && !e_latched {
            if let Some(t) = target {
                let verb = match t {
                    Target::Object(_) => Verb::UseObject,
                    Target::Npc(_) => Verb::ApproachNpc,
                };
                world.apply_interact(verb, t);
                interact_pulse = 1.0;
            }
        }
        e_latched = e_down;
        interact_pulse = (interact_pulse - dt * 1.5).max(0.0);

        let _ = window.get_keys_pressed(KeyRepeat::No);

        // --- Render ---
        render::render_frame(
            &mut frame,
            &world.grid,
            &world.body,
            &world.npcs,
            &world.objects,
            &world.clock,
            fov_tan,
            target,
            interact_pulse,
        );
        window.update_with_buffer(&frame.pixels, WIDTH, HEIGHT)
            .expect("could not update framebuffer");
    }
}
