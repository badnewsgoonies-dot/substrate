//! Needs-driven AI for NPCs.

use crate::kinds::ObjectKindTag;
use crate::needs::{NeedTag, Needs};
use crate::objects::Objects;
use crate::people::{Npc, NPC_ARRIVE_RADIUS};
use crate::routine::activity_target;
use crate::space::{TileGrid, Vec2};
use crate::time::InGameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Satisfy { need: NeedTag, obj_idx: usize },
    FollowSchedule,
    Idle,
}

pub fn kinds_for_need(need: NeedTag) -> &'static [ObjectKindTag] {
    match need {
        NeedTag::Hunger => &[ObjectKindTag::Fridge],
        NeedTag::Tiredness => &[ObjectKindTag::Bed],
        NeedTag::Hygiene => &[
            ObjectKindTag::Shower,
            ObjectKindTag::Bathtub,
            ObjectKindTag::BathSink,
            ObjectKindTag::Sink,
        ],
        NeedTag::Mood => &[ObjectKindTag::Book],
    }
}

pub fn restore_for_kind(needs: &mut Needs, tag: ObjectKindTag) {
    match tag {
        ObjectKindTag::Fridge => needs.eat(3_000),
        ObjectKindTag::Bed => needs.sleep(60),
        ObjectKindTag::Shower => needs.shower(60),
        ObjectKindTag::Bathtub => {
            needs.shower(120);
            needs.read(1);
        }
        ObjectKindTag::BathSink | ObjectKindTag::Sink => needs.wash_hands(),
        ObjectKindTag::Toilet => {
            needs.use_toilet();
            needs.wash_hands();
        }
        ObjectKindTag::Book => needs.read(30),
        ObjectKindTag::CoffeeMaker => needs.drink_coffee(),
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
}

pub fn find_nearest_kind(
    pos: Vec2,
    objects: &Objects,
    tags: &[ObjectKindTag],
) -> Option<(usize, Vec2)> {
    let mut best: Option<(usize, f32, Vec2)> = None;
    for (i, obj) in objects.iter().enumerate() {
        if !tags.contains(&obj.kind_tag()) { continue; }
        let d = (obj.pos() - pos).length();
        if best.map_or(true, |(_, bd, _)| d < bd) {
            best = Some((i, d, obj.pos()));
        }
    }
    best.map(|(i, _, p)| (i, p))
}

pub fn pick_urgent_target(
    needs: &Needs,
    pos: Vec2,
    objects: &Objects,
) -> Option<(NeedTag, usize, Vec2)> {
    let checks: [(bool, NeedTag); 4] = [
        (needs.is_hungry(), NeedTag::Hunger),
        (needs.is_exhausted(), NeedTag::Tiredness),
        (needs.needs_shower(), NeedTag::Hygiene),
        (needs.is_sad(), NeedTag::Mood),
    ];
    for (urgent, tag) in checks {
        if !urgent { continue; }
        if let Some((idx, tpos)) = find_nearest_kind(pos, objects, kinds_for_need(tag)) {
            return Some((tag, idx, tpos));
        }
    }
    None
}

pub fn decide_intent(
    npc: &Npc,
    objects: &Objects,
    clock: &InGameClock,
) -> Intent {
    if let Some((need, idx, _)) = pick_urgent_target(&npc.needs, npc.pos, objects) {
        return Intent::Satisfy { need, obj_idx: idx };
    }
    match activity_target(npc.schedule.activity_at(clock)) {
        Some(_) => Intent::FollowSchedule,
        None => Intent::Idle,
    }
}

pub fn ai_tick_npc(
    npc: &mut Npc,
    objects: &mut Objects,
    grid: &TileGrid,
    clock: &InGameClock,
    dt: f32,
    minutes_elapsed: u32,
) -> Intent {
    npc.needs.apply_minute_decay(minutes_elapsed);
    npc.current_activity = npc.schedule.activity_at(clock);

    if let Some((need, idx, tpos)) = pick_urgent_target(&npc.needs, npc.pos, objects) {
        let remaining = npc.walk_toward(grid, tpos, dt);
        if remaining < NPC_ARRIVE_RADIUS {
            let obj = objects.get_mut(idx);
            obj.use_action();
            let tag = obj.kind_tag();
            restore_for_kind(&mut npc.needs, tag);
        }
        return Intent::Satisfy { need, obj_idx: idx };
    }

    match activity_target(npc.current_activity) {
        Some(t) => {
            npc.walk_toward(grid, t, dt);
            Intent::FollowSchedule
        }
        None => Intent::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::{Bed, Book, Fridge, Object, Shower};
    use crate::routine::{RoutineSlot, Activity, Schedule};

    fn open_grid() -> TileGrid {
        TileGrid::from_ascii(&[
            "1111111111111111",
            "1..............1", "1..............1", "1..............1",
            "1..............1", "1..............1", "1..............1",
            "1..............1", "1..............1", "1..............1",
            "1..............1", "1..............1", "1..............1",
            "1..............1", "1..............1",
            "1111111111111111",
        ])
    }

    #[test]
    fn kinds_for_hunger_is_fridge() {
        assert_eq!(kinds_for_need(NeedTag::Hunger), &[ObjectKindTag::Fridge]);
    }

    #[test]
    fn hygiene_kinds_are_ordered_preference() {
        let ks = kinds_for_need(NeedTag::Hygiene);
        assert_eq!(ks[0], ObjectKindTag::Shower);
        assert_eq!(ks[1], ObjectKindTag::Bathtub);
    }

    #[test]
    fn restore_for_kind_fridge_restores_hunger() {
        let mut n = Needs { hunger: 1000, tiredness: 5000, hygiene: 5000, mood: 5000 };
        restore_for_kind(&mut n, ObjectKindTag::Fridge);
        assert_eq!(n.hunger, 4000);
    }

    #[test]
    fn restore_for_kind_mug_is_decorative() {
        let before = Needs::new();
        let mut after = before;
        restore_for_kind(&mut after, ObjectKindTag::Mug);
        assert_eq!(before, after);
    }

    #[test]
    fn restore_for_kind_plant_is_decorative() {
        let before = Needs::new();
        let mut after = before;
        restore_for_kind(&mut after, ObjectKindTag::Plant);
        assert_eq!(before, after);
    }

    #[test]
    fn restore_for_kind_painting_is_decorative() {
        let before = Needs::new();
        let mut after = before;
        restore_for_kind(&mut after, ObjectKindTag::Painting);
        assert_eq!(before, after);
    }

    #[test]
    fn restore_for_kind_diary_is_decorative() {
        let before = Needs::new();
        let mut after = before;
        restore_for_kind(&mut after, ObjectKindTag::Diary);
        assert_eq!(before, after);
    }

    #[test]
    fn find_nearest_picks_closest_matching() {
        let mut objs = Objects::new();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(10.0, 10.0))));
        objs.add(Object::Fridge(Fridge::new(Vec2::new(3.0, 3.0))));
        let (idx, pos) = find_nearest_kind(Vec2::new(2.0, 2.0), &objs, &[ObjectKindTag::Fridge]).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(pos, Vec2::new(3.0, 3.0));
    }

    #[test]
    fn find_nearest_returns_none_when_no_match() {
        let objs = Objects::new();
        assert!(find_nearest_kind(Vec2::new(0.0, 0.0), &objs, &[ObjectKindTag::Fridge]).is_none());
    }

    #[test]
    fn pick_urgent_target_prefers_hunger_first() {
        let needs = Needs { hunger: 1000, tiredness: 1000, hygiene: 1000, mood: 1000 };
        let mut objs = Objects::new();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(3.0, 3.0))));
        objs.add(Object::Bed(Bed::new(Vec2::new(5.0, 5.0))));
        let (tag, _, _) = pick_urgent_target(&needs, Vec2::new(2.0, 2.0), &objs).unwrap();
        assert_eq!(tag, NeedTag::Hunger);
    }

    #[test]
    fn pick_urgent_target_returns_none_when_no_need_urgent() {
        let needs = Needs::new();
        let mut objs = Objects::new();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(3.0, 3.0))));
        assert!(pick_urgent_target(&needs, Vec2::new(2.0, 2.0), &objs).is_none());
    }

    #[test]
    fn decide_intent_idle_when_schedule_out() {
        let npc = Npc::with_schedule("alice", Vec2::new(2.0, 2.0), 0.0, Schedule::new(vec![]));
        let objs = Objects::new();
        let clock = InGameClock::at_hour(12);
        assert_eq!(decide_intent(&npc, &objs, &clock), Intent::Idle);
    }

    #[test]
    fn decide_intent_satisfy_hunger_with_fridge_nearby() {
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        npc.needs.hunger = 1000;
        let mut objs = Objects::new();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(3.0, 3.0))));
        let clock = InGameClock::at_hour(12);
        let intent = decide_intent(&npc, &objs, &clock);
        match intent {
            Intent::Satisfy { need, obj_idx } => {
                assert_eq!(need, NeedTag::Hunger);
                assert_eq!(obj_idx, 0);
            }
            _ => panic!("expected Satisfy"),
        }
    }

    #[test]
    fn hungry_npc_walks_to_fridge_and_eats() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        npc.needs = Needs { hunger: 1000, tiredness: 5000, hygiene: 5000, mood: 5000 };
        let mut objs = Objects::new();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(2.1, 2.1))));
        let clock = InGameClock::at_hour(12);
        let before = npc.needs.hunger;
        ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 1.0, 0);
        assert!(npc.needs.hunger > before);
    }

    #[test]
    fn tired_npc_walks_to_bed() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        npc.needs = Needs { hunger: 8000, tiredness: 500, hygiene: 8000, mood: 8000 };
        let mut objs = Objects::new();
        objs.add(Object::Bed(Bed::new(Vec2::new(2.1, 2.1))));
        let clock = InGameClock::at_hour(23);
        let before = npc.needs.tiredness;
        ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 1.0, 0);
        assert!(npc.needs.tiredness > before);
    }

    #[test]
    fn dirty_npc_walks_to_shower() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        npc.needs = Needs { hunger: 8000, tiredness: 8000, hygiene: 1000, mood: 8000 };
        let mut objs = Objects::new();
        objs.add(Object::Shower(Shower::new(Vec2::new(2.1, 2.1))));
        let clock = InGameClock::at_hour(8);
        let before = npc.needs.hygiene;
        ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 1.0, 0);
        assert!(npc.needs.hygiene > before);
    }

    #[test]
    fn sad_npc_reads_a_book() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        npc.needs = Needs { hunger: 8000, tiredness: 8000, hygiene: 8000, mood: 1000 };
        let mut objs = Objects::new();
        objs.add(Object::Book(Book::new(Vec2::new(2.1, 2.1))));
        let clock = InGameClock::at_hour(12);
        let before = npc.needs.mood;
        ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 1.0, 0);
        assert!(npc.needs.mood > before);
    }

    #[test]
    fn ai_tick_decays_needs_over_minutes() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        let mut objs = Objects::new();
        let clock = InGameClock::at_hour(12);
        let before = npc.needs.hunger;
        ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 0.0, 100);
        assert!(npc.needs.hunger < before);
    }

    #[test]
    fn a_day_of_lone_npc_ai_keeps_needs_bounded() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        let mut objs = Objects::new();
        let clock = InGameClock::at_hour(12);
        for _ in 0..24 { ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 0.1, 60); }
        assert_eq!(npc.needs.hunger, 0);
        assert_eq!(npc.needs.tiredness, 0);
    }

    #[test]
    fn a_stocked_apartment_keeps_npc_bounded_away_from_zero() {
        let grid = open_grid();
        let mut npc = Npc::new("alice", Vec2::new(2.0, 2.0), 0.0);
        let mut objs = Objects::new();
        objs.add(Object::Fridge(Fridge::new(Vec2::new(2.5, 2.0))));
        objs.add(Object::Bed(Bed::new(Vec2::new(3.0, 2.0))));
        objs.add(Object::Shower(Shower::new(Vec2::new(3.5, 2.0))));
        objs.add(Object::Book(Book::new(Vec2::new(4.0, 2.0))));
        let clock = InGameClock::at_hour(8);
        for _ in 0..36 { ai_tick_npc(&mut npc, &mut objs, &grid, &clock, 2.0, 10); }
        assert!(npc.needs.hunger > 0);
        assert!(npc.needs.tiredness > 0);
        assert!(npc.needs.hygiene > 0);
        assert!(npc.needs.mood > 0);
    }

    #[allow(dead_code)]
    fn _slots_reference_guard() -> Vec<RoutineSlot> {
        vec![RoutineSlot { start_hour: 0, end_hour: 24, activity: Activity::Out }]
    }
}
