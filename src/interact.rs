//! Interaction primitive.

use crate::body::PlayerBody;
use crate::kinds::{Object, ObjectKindTag};
use crate::mind::{Fact, FactKey, Mind, MindObjectKind};
use crate::objects::Objects;
use crate::people::Npc;
use crate::space::TileGrid;
use crate::time::InGameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Object(usize),
    Npc(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    LookAt,
    UseObject,
    ApproachNpc,
}

pub const INTERACT_RADIUS: f32 = 1.8;
pub const CROSSHAIR_HALF_ANGLE: f32 = 0.35;

pub fn resolve_target(
    body: &PlayerBody, _grid: &TileGrid,
    objects: &Objects, npcs: &[Npc], _fov_tan: f32,
) -> Option<Target> {
    let fwd = body.forward();
    let mut best: Option<(Target, f32)> = None;
    for (i, obj) in objects.iter().enumerate() {
        let v = obj.pos() - body.pos;
        let dist = v.length();
        if dist < 0.05 || dist > INTERACT_RADIUS { continue; }
        let cos_angle = (v.x * fwd.x + v.y * fwd.y) / dist;
        if cos_angle < CROSSHAIR_HALF_ANGLE.cos() { continue; }
        if best.map_or(true, |(_, d)| dist < d) {
            best = Some((Target::Object(i), dist));
        }
    }
    for (i, npc) in npcs.iter().enumerate() {
        let v = npc.pos - body.pos;
        let dist = v.length();
        if dist < 0.05 || dist > INTERACT_RADIUS { continue; }
        let cos_angle = (v.x * fwd.x + v.y * fwd.y) / dist;
        if cos_angle < CROSSHAIR_HALF_ANGLE.cos() { continue; }
        if best.map_or(true, |(_, d)| dist < d) {
            best = Some((Target::Npc(i), dist));
        }
    }
    best.map(|(t, _)| t)
}

pub fn apply(
    verb: Verb, target: Target,
    objects: &mut Objects, npcs: &mut [Npc],
    body: &mut PlayerBody, mind: &mut Mind, clock: &InGameClock,
) {
    match (verb, target) {
        (Verb::UseObject, Target::Object(idx)) => {
            let obj = objects.get_mut(idx);
            obj.use_action();
            let kind_tag = obj.kind_tag();
            let pos = obj.pos();
            let bed_is_unmade = matches!(obj, Object::Bed(b) if b.is_unmade());
            if matches!(kind_tag, ObjectKindTag::Bed) {
                body.posture = if bed_is_unmade {
                    crate::body::Posture::Lying
                } else {
                    crate::body::Posture::Standing
                };
            }
            match kind_tag {
                ObjectKindTag::Bed => body.needs.sleep(60),
                ObjectKindTag::Fridge => body.needs.eat(3_000),
                ObjectKindTag::CoffeeMaker => body.needs.drink_coffee(),
                ObjectKindTag::Shower => body.needs.shower(60),
                ObjectKindTag::Bathtub => {
                    body.needs.shower(120);
                    body.needs.read(1);
                }
                ObjectKindTag::Sink | ObjectKindTag::BathSink => body.needs.wash_hands(),
                ObjectKindTag::Toilet => {
                    body.needs.use_toilet();
                    body.needs.wash_hands();
                }
                ObjectKindTag::Book => body.needs.read(30),
                ObjectKindTag::Stove
                | ObjectKindTag::Window
                | ObjectKindTag::Lamp
                | ObjectKindTag::Nightstand
                | ObjectKindTag::Dresser
                | ObjectKindTag::Mug
                | ObjectKindTag::Plant
                | ObjectKindTag::Painting
                | ObjectKindTag::Chair
                | ObjectKindTag::Diary => {}
            }
            mind.know(
                FactKey::SawObject(MindObjectKind::from_tag(kind_tag), pos.into()),
                Fact::observed_at(*clock),
            );
        }
        (Verb::ApproachNpc, Target::Npc(idx)) => {
            let npc = &npcs[idx];
            mind.know(FactKey::SawPerson(npc.name), Fact::observed_at(*clock));
            body.needs.mood = body.needs.mood.saturating_add(500).min(crate::needs::NEED_MAX);
        }
        (Verb::LookAt, _) => {}
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::{
        BathSink, Bathtub, Bed, Book, CoffeeMaker, Diary, Fridge, Mug, Painting, Plant, Shower, Sink, Stove, Toilet,
    };
    use crate::needs::{Needs, NEED_MAX};
    use crate::objects::Objects;
    use crate::routine::Schedule;
    use crate::space::{TileGrid, Vec2};
    use crate::traits::{Openable, Switchable};

    fn grid() -> TileGrid {
        TileGrid::from_ascii(&["11111", "1...1", "1...1", "1...1", "11111"])
    }

    fn ctx() -> (Objects, Vec<Npc>, Mind, InGameClock, PlayerBody) {
        (Objects::new(), vec![], Mind::new(),
         InGameClock::at_hour(8), PlayerBody::new(Vec2::new(2.0, 2.0), 0.0))
    }

    #[test]
    fn resolves_object_in_front_of_body() {
        let body = PlayerBody::new(Vec2::new(2.0, 2.0), 0.0);
        let mut objs = Objects::new();
        objs.add(Object::Book(Book::new(Vec2::new(3.0, 2.0))));
        let npcs: Vec<Npc> = vec![];
        let tgt = resolve_target(&body, &grid(), &objs, &npcs, 1.0).unwrap();
        assert_eq!(tgt, Target::Object(0));
    }

    #[test]
    fn ignores_object_behind_body() {
        let body = PlayerBody::new(Vec2::new(2.0, 2.0), 0.0);
        let mut objs = Objects::new();
        objs.add(Object::Book(Book::new(Vec2::new(1.0, 2.0))));
        let npcs: Vec<Npc> = vec![];
        assert!(resolve_target(&body, &grid(), &objs, &npcs, 1.0).is_none());
    }

    #[test]
    fn use_fridge_toggles_open_closed() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        if let Object::Fridge(f) = objs.get(0) { assert!(f.is_open()); } else { panic!(); }
    }

    #[test]
    fn use_sink_toggles_on_off() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Sink(Sink::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        if let Object::Sink(s) = objs.get(0) { assert!(s.is_on()); } else { panic!(); }
    }

    #[test]
    fn use_stove_turns_first_burner_on() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Stove(Stove::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        if let Object::Stove(s) = objs.get(0) { assert!(s.any_burner_on()); } else { panic!(); }
    }

    #[test]
    fn use_bed_changes_posture_to_lying() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Bed(Bed::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.posture, crate::body::Posture::Standing);
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.posture, crate::body::Posture::Lying);
    }

    #[test]
    fn use_diary_toggles_open_closed() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Diary(Diary::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        if let Object::Diary(d) = objs.get(0) { assert!(d.is_open()); } else { panic!(); }
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        if let Object::Diary(d) = objs.get(0) { assert!(!d.is_open()); } else { panic!(); }
    }

    #[test]
    fn approach_npc_records_memory() {
        let (mut objs, _, mut mind, clock, mut body) = ctx();
        let sch = Schedule::new(vec![]);
        let mut npcs = vec![Npc::with_schedule("friend", Vec2::new(2.9, 2.0), 0.0, sch)];
        apply(Verb::ApproachNpc, Target::Npc(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert!(mind.recall(&FactKey::SawPerson("friend")).is_some());
    }

    fn pre_needs(mood: u32) -> Needs {
        Needs { hunger: 5_000, tiredness: 5_000, hygiene: 5_000, mood }
    }

    #[test]
    fn using_fridge_restores_hunger() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(5_000);
        objs.add(Object::Fridge(Fridge::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.hunger, 8_000);
    }

    #[test]
    fn using_bed_restores_tiredness() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(5_000);
        body.needs.tiredness = 2_000;
        objs.add(Object::Bed(Bed::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.tiredness, 2_000 + 1_260);
    }

    #[test]
    fn using_shower_restores_hygiene() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(5_000);
        body.needs.hygiene = 1_000;
        objs.add(Object::Shower(Shower::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.hygiene, 7_000);
    }

    #[test]
    fn using_bath_sink_washes_hands() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(5_000);
        body.needs.hygiene = 3_000;
        objs.add(Object::BathSink(BathSink::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.hygiene, 3_500);
    }

    #[test]
    fn using_toilet_is_net_positive_hygiene() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(5_000);
        body.needs.hygiene = 5_000;
        objs.add(Object::Toilet(Toilet::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.hygiene, 5_300);
    }

    #[test]
    fn using_book_brightens_mood() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(2_000);
        objs.add(Object::Book(Book::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.mood, 2_000 + 900);
    }

    #[test]
    fn using_coffee_maker_bumps_alertness() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(4_000);
        body.needs.tiredness = 3_000;
        objs.add(Object::CoffeeMaker(CoffeeMaker::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.tiredness, 3_000 + 1_500);
        assert_eq!(body.needs.mood, 4_000 + 500);
    }

    #[test]
    fn using_bathtub_restores_hygiene_bigger_than_shower() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(5_000);
        body.needs.hygiene = 0;
        objs.add(Object::Bathtub(Bathtub::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.hygiene, NEED_MAX);
    }

    #[test]
    fn approaching_an_npc_brightens_mood() {
        let (mut objs, _, mut mind, clock, mut body) = ctx();
        body.needs = pre_needs(4_000);
        let sch = Schedule::new(vec![]);
        let mut npcs = vec![Npc::with_schedule("friend", Vec2::new(2.9, 2.0), 0.0, sch)];
        apply(Verb::ApproachNpc, Target::Npc(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs.mood, 4_500);
    }

    #[test]
    fn decorative_objects_do_not_change_needs() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        let before = body.needs;
        objs.add(Object::Stove(Stove::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs, before);
    }

    #[test]
    fn using_a_mug_does_not_change_needs() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        let before = body.needs;
        objs.add(Object::Mug(Mug::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs, before);
    }

    #[test]
    fn using_a_mug_records_a_memory() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Mug(Mug::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert!(!mind.is_empty());
    }

    #[test]
    fn using_a_plant_does_not_change_needs() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        let before = body.needs;
        objs.add(Object::Plant(Plant::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs, before);
    }

    #[test]
    fn using_a_plant_records_a_memory() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Plant(Plant::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert!(!mind.is_empty());
    }

    #[test]
    fn using_a_painting_does_not_change_needs() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        let before = body.needs;
        objs.add(Object::Painting(Painting::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs, before);
    }

    #[test]
    fn using_a_painting_records_a_memory() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Painting(Painting::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert!(!mind.is_empty());
    }

    #[test]
    fn using_a_diary_does_not_change_needs() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        let before = body.needs;
        objs.add(Object::Diary(Diary::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert_eq!(body.needs, before);
    }

    #[test]
    fn using_a_diary_records_a_memory() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        objs.add(Object::Diary(Diary::new(Vec2::new(2.9, 2.0))));
        apply(Verb::UseObject, Target::Object(0), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        assert!(!mind.is_empty());
    }

    #[test]
    fn a_full_morning_loop_is_game_shaped() {
        let (mut objs, mut npcs, mut mind, clock, mut body) = ctx();
        body.needs = Needs { hunger: 3_000, tiredness: 2_500, hygiene: 4_000, mood: 5_000 };
        let starting = body.needs.wellbeing();
        objs.add(Object::Bed(Bed::new(Vec2::new(2.9, 2.0))));
        objs.add(Object::Toilet(Toilet::new(Vec2::new(2.9, 2.0))));
        objs.add(Object::Shower(Shower::new(Vec2::new(2.9, 2.0))));
        objs.add(Object::Fridge(Fridge::new(Vec2::new(2.9, 2.0))));
        objs.add(Object::CoffeeMaker(CoffeeMaker::new(Vec2::new(2.9, 2.0))));
        objs.add(Object::Book(Book::new(Vec2::new(2.9, 2.0))));
        for i in 0..6 {
            apply(Verb::UseObject, Target::Object(i), &mut objs, &mut npcs, &mut body, &mut mind, &clock);
        }
        body.tick_needs(120);
        assert!(body.needs.wellbeing() > starting);
        assert!(!body.needs.is_hungry());
        assert!(!body.needs.needs_shower());
        assert!(!body.needs.is_exhausted());
    }

    #[test]
    fn a_full_day_of_neglect_is_worse_than_nothing() {
        let mut body = PlayerBody::new(Vec2::new(0.0, 0.0), 0.0);
        body.tick_needs(1_440);
        assert!(body.needs.is_hungry());
        assert!(body.needs.is_exhausted());
        assert!(body.needs.needs_shower());
    }
}
