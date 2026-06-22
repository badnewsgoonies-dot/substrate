//! World: single coordinator owning every simulation system.

use crate::ai::ai_tick_npc;
use crate::body::PlayerBody;
use crate::interact::{apply, resolve_target, Target, Verb};
use crate::kinds::{
    BathSink, Bathtub, Bed, Book, Chair, CoffeeMaker, Diary, Dresser, Fridge, Lamp, Mug, Nightstand,
    Object, Painting, Plant, Shower, Sink, Stove, Toilet, WindowObj,
};
use crate::mind::Mind;
use crate::objects::Objects;
use crate::people::Npc;
use crate::routine::Schedule;
use crate::space::{TileGrid, Vec2};
use crate::time::InGameClock;
use crate::traits::Openable;

pub struct World {
    pub grid: TileGrid,
    pub body: PlayerBody,
    pub objects: Objects,
    pub npcs: Vec<Npc>,
    pub mind: Mind,
    pub clock: InGameClock,
    last_total_minutes: u64,
}

impl World {
    /// Canonical v0.8 apartment with 19 interactable objects + one
    /// roommate NPC. All five generated kinds are live in the world:
    /// Mug on the nightstand, Plant in the kitchen nook by the window,
    /// Painting on the bedroom wall by the dresser, Chair on the kitchen
    /// floor (drawn by the spritegen-authored sprite), Diary on the
    /// nightstand beside the Mug.
    pub fn new_apartment() -> Self {
        let map: Vec<&str> = vec![
            "11111111111111111111",
            "1..............21111",
            "1..............21111",
            "1......3.......21111",
            "1......3.......21111",
            "1......3.......21111",
            "1......3.......21111",
            "1113331333.333324444",
            "2..............2...4",
            "2..............2...4",
            "2.......R..........4",
            "2..............2...4",
            "2.....3333.....24444",
            "2..............22222",
            "2..............22222",
            "22222222222222222222",
        ];
        let grid = TileGrid::from_ascii(&map);
        let body = PlayerBody::new(Vec2::new(3.5, 2.5), std::f32::consts::FRAC_PI_2);

        let mut objects = Objects::new();
        objects.add(Object::Bed(Bed::new(Vec2::new(12.5, 3.5))));
        objects.add(Object::Nightstand(Nightstand::new(Vec2::new(11.3, 3.5))));
        objects.add(Object::Lamp(Lamp::new(Vec2::new(11.3, 2.8))));
        objects.add(Object::Dresser(Dresser::new(Vec2::new(2.5, 2.5))));
        objects.add(Object::CoffeeMaker(CoffeeMaker::new(Vec2::new(7.0, 12.5))));
        objects.add(Object::Book(Book::new(Vec2::new(8.0, 12.5))));
        objects.add(Object::Window(WindowObj::new(Vec2::new(8.0, 14.5))));
        objects.add(Object::Fridge(Fridge::new(Vec2::new(2.5, 12.5))));
        objects.add(Object::Stove(Stove::new(Vec2::new(13.0, 12.5))));
        objects.add(Object::Sink(Sink::new(Vec2::new(6.0, 12.5))));
        objects.add(Object::Toilet(Toilet::new(Vec2::new(17.5, 8.4))));
        objects.add(Object::BathSink(BathSink::new(Vec2::new(16.4, 8.5))));
        objects.add(Object::Shower(Shower::new(Vec2::new(18.5, 8.5))));
        let mut bathtub = Bathtub::new(Vec2::new(17.5, 11.3));
        bathtub.toggle_open();
        objects.add(Object::Bathtub(bathtub));
        objects.add(Object::Mug(Mug::new(Vec2::new(11.3, 3.3))));
        objects.add(Object::Plant(Plant::new(Vec2::new(9.0, 14.0))));
        objects.add(Object::Painting(Painting::new(Vec2::new(4.5, 3.5))));
        objects.add(Object::Chair(Chair::new(Vec2::new(5.0, 9.5))));
        objects.add(Object::Diary(Diary::new(Vec2::new(11.3, 3.7))));

        let roommate = Npc::with_schedule(
            "roommate",
            Vec2::new(8.5, 10.5),
            std::f32::consts::PI,
            Schedule::roommate_default(),
        );
        let npcs = vec![roommate];
        let mind = Mind::new();

        let mut clock = InGameClock::at_hour(7);
        clock.advance_in_game_seconds(14 * 60);
        objects.tick_time(&clock);
        let last_total_minutes = clock_total_minutes(&clock);

        Self { grid, body, objects, npcs, mind, clock, last_total_minutes }
    }

    pub fn tick(&mut self, dt: f32) {
        self.clock.advance_real_seconds_f32(dt);
        self.objects.tick_time(&self.clock);
        self.objects.tick_real_seconds(dt);

        let now_total = clock_total_minutes(&self.clock);
        let delta_minutes_u64 = now_total.saturating_sub(self.last_total_minutes);
        let delta_minutes = delta_minutes_u64.min(u32::MAX as u64) as u32;
        if delta_minutes > 0 {
            self.body.tick_needs(delta_minutes);
            self.last_total_minutes = now_total;
        }

        for npc in self.npcs.iter_mut() {
            ai_tick_npc(npc, &mut self.objects, &self.grid, &self.clock, dt, delta_minutes);
        }
    }

    pub fn resolve_target(&self, fov_tan: f32) -> Option<Target> {
        resolve_target(&self.body, &self.grid, &self.objects, &self.npcs, fov_tan)
    }

    pub fn apply_interact(&mut self, verb: Verb, target: Target) {
        apply(
            verb, target,
            &mut self.objects, &mut self.npcs, &mut self.body, &mut self.mind, &self.clock,
        );
    }
}

fn clock_total_minutes(clock: &InGameClock) -> u64 {
    (clock.day as u64) * 24 * 60 + (clock.second_of_day as u64) / 60
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::ObjectKindTag;
    use crate::needs::NEED_MAX;

    #[test]
    fn new_apartment_has_19_objects_and_1_npc() {
        let w = World::new_apartment();
        assert_eq!(w.objects.len(), 19);
        assert_eq!(w.npcs.len(), 1);
        assert_eq!(w.npcs[0].name, "roommate");
    }

    #[test]
    fn new_apartment_has_a_mug_on_the_nightstand() {
        let w = World::new_apartment();
        let c = (0..w.objects.len())
            .filter(|&i| w.objects.get(i).kind_tag() == ObjectKindTag::Mug).count();
        assert_eq!(c, 1);
    }

    #[test]
    fn new_apartment_has_a_plant_in_the_kitchen() {
        let w = World::new_apartment();
        let c = (0..w.objects.len())
            .filter(|&i| w.objects.get(i).kind_tag() == ObjectKindTag::Plant).count();
        assert_eq!(c, 1);
    }

    #[test]
    fn new_apartment_has_a_painting_in_the_bedroom() {
        let w = World::new_apartment();
        let c = (0..w.objects.len())
            .filter(|&i| w.objects.get(i).kind_tag() == ObjectKindTag::Painting).count();
        assert_eq!(c, 1);
    }

    #[test]
    fn new_apartment_has_a_diary_on_the_nightstand() {
        let w = World::new_apartment();
        let diary_positions: Vec<Vec2> = (0..w.objects.len())
            .filter(|&i| w.objects.get(i).kind_tag() == ObjectKindTag::Diary)
            .map(|i| w.objects.get(i).pos())
            .collect();
        assert_eq!(diary_positions.len(), 1);
        // Should be on the nightstand (x ~= 11.3), bedroom side (y < 8)
        let p = diary_positions[0];
        assert!((p.x - 11.3).abs() < 0.5, "diary x expected near 11.3, got {}", p.x);
        assert!(p.y < 8.0, "diary should be in bedroom (y < 8), got {}", p.y);
    }

    #[test]
    fn new_apartment_starts_with_full_player_needs() {
        let w = World::new_apartment();
        assert_eq!(w.body.needs.hunger, NEED_MAX);
        assert_eq!(w.body.needs.tiredness, NEED_MAX);
        assert_eq!(w.body.needs.hygiene, NEED_MAX);
        assert_eq!(w.body.needs.mood, NEED_MAX);
    }

    #[test]
    fn new_apartment_starts_at_seven_fourteen_am() {
        let w = World::new_apartment();
        assert_eq!(w.clock.hour(), 7);
    }

    #[test]
    fn zero_dt_tick_is_a_no_op() {
        let mut w = World::new_apartment();
        let before_hunger = w.body.needs.hunger;
        let before_pos = w.npcs[0].pos;
        w.tick(0.0);
        assert_eq!(w.body.needs.hunger, before_hunger);
        assert_eq!(w.npcs[0].pos, before_pos);
    }

    #[test]
    fn one_real_second_advances_clock() {
        let mut w = World::new_apartment();
        let before_hour = w.clock.hour();
        let before_sec = w.clock.second_of_day;
        w.tick(1.0);
        assert!(w.clock.second_of_day > before_sec || w.clock.hour() != before_hour);
    }

    #[test]
    fn ten_real_seconds_decays_player_needs() {
        let mut w = World::new_apartment();
        let before = w.body.needs;
        w.tick(10.0);
        assert!(w.body.needs.hunger < before.hunger);
        assert!(w.body.needs.tiredness < before.tiredness);
    }

    #[test]
    fn long_simulation_eventually_makes_player_hungry() {
        let mut w = World::new_apartment();
        for _ in 0..500 { w.tick(1.0); }
        for _ in 0..100 { w.tick(1.0); }
        assert!(w.body.needs.is_hungry());
    }

    #[test]
    fn npc_ai_is_live_after_world_tick() {
        let mut w = World::new_apartment();
        w.npcs[0].needs.hunger = 1_000;
        w.npcs[0].pos = Vec2::new(2.4, 12.5);
        let before = w.npcs[0].needs.hunger;
        w.tick(0.1);
        assert!(w.npcs[0].needs.hunger > before);
    }

    #[test]
    fn player_interact_fridge_restores_hunger_through_world() {
        let mut w = World::new_apartment();
        w.body.needs.hunger = 2_000;
        w.body.pos = Vec2::new(2.4, 12.5);
        w.apply_interact(Verb::UseObject, Target::Object(7));
        assert_eq!(w.body.needs.hunger, 5_000);
    }

    #[test]
    fn long_run_of_idle_world_does_not_crash() {
        let mut w = World::new_apartment();
        for _ in 0..1_000 { w.tick(0.1); }
        let _ = w.body.needs.wellbeing();
        let _ = w.npcs[0].needs.wellbeing();
    }
}
