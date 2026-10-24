//! Packed-column Godot representation of the MUX bulk observer.

use super::{PersonFact, UiFacts};
use godot::prelude::*;

fn ints(values: impl Iterator<Item = i64>) -> PackedInt64Array {
    PackedInt64Array::from(values.collect::<Vec<_>>().as_slice())
}
fn floats(values: impl Iterator<Item = f64>) -> PackedFloat64Array {
    PackedFloat64Array::from(values.collect::<Vec<_>>().as_slice())
}
fn bytes(values: impl Iterator<Item = bool>) -> PackedByteArray {
    PackedByteArray::from(values.map(u8::from).collect::<Vec<_>>().as_slice())
}
fn signed(value: u64) -> i64 {
    i64::from_ne_bytes(value.to_ne_bytes())
}
fn tick(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
fn optional(value: Option<u64>) -> i64 {
    value.map_or(-1, signed)
}

fn people_to_dict(rows: &[PersonFact]) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("ids", ints(rows.iter().map(|r| signed(r.id))));
    d.set(
        "household_ids",
        ints(rows.iter().map(|r| signed(r.household_id))),
    );
    d.set(
        "entity_ids",
        ints(rows.iter().map(|r| signed(r.entity_bits))),
    );
    d.set("xs", ints(rows.iter().map(|r| i64::from(r.tile.0))));
    d.set("ys", ints(rows.iter().map(|r| i64::from(r.tile.1))));
    d.set("hps", floats(rows.iter().map(|r| r.hp.unwrap_or(-1.0))));
    d.set(
        "max_hps",
        floats(rows.iter().map(|r| r.max_hp.unwrap_or(-1.0))),
    );
    d.set("children", bytes(rows.iter().map(|r| r.child)));
    d.set("infants", bytes(rows.iter().map(|r| r.infant)));
    d.set(
        "born_ticks",
        ints(rows.iter().map(|r| r.born_tick.map_or(-1, tick))),
    );
    d.set("idle", bytes(rows.iter().map(|r| r.idle)));
    d.set("sleeping", bytes(rows.iter().map(|r| r.sleeping)));
    d.set("roofed", bytes(rows.iter().map(|r| r.roofed)));
    d.set(
        "bed_xs",
        ints(rows.iter().map(|r| r.bed.map_or(-1, |t| i64::from(t.0)))),
    );
    d.set(
        "bed_ys",
        ints(rows.iter().map(|r| r.bed.map_or(-1, |t| i64::from(t.1)))),
    );
    d.set("has_job", bytes(rows.iter().map(|r| r.has_job)));
    d.set("drafted", bytes(rows.iter().map(|r| r.drafted)));
    d.set("sick", bytes(rows.iter().map(|r| r.sick)));
    d.set("breakdown", bytes(rows.iter().map(|r| r.breakdown)));
    d.set("nights", ints(rows.iter().map(|r| i64::from(r.nights))));
    d.set("mothers", ints(rows.iter().map(|r| optional(r.mother))));
    d.set("fathers", ints(rows.iter().map(|r| optional(r.father))));
    d.set("partners", ints(rows.iter().map(|r| optional(r.partner))));
    d
}

pub(in crate::ffi) fn ui_facts_to_dict(v: &UiFacts) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("schema_version", 1);
    d.set("enabled", v.enabled);
    d.set("tick", tick(v.tick));
    d.set("day", tick(v.day));
    d.set("ticks_per_day", tick(v.ticks_per_day));
    d.set("clock_offset_ticks", tick(v.clock_offset_ticks));
    d.set(
        "season",
        v.season.map_or(-1, |s| i64::try_from(s.0).unwrap_or(-1)),
    );
    d.set("season_day", v.season.map_or(0, |s| i64::from(s.1)));
    d.set("days_per_season", i64::from(v.days_per_season));
    d.set("meat_spoil_days", v.meat_spoil_days.map_or(-1, i64::from));
    d.set("taught_count", tick(v.taught_count));
    d.set("wolves_spawned", tick(v.wolves_spawned));
    d.set("migrant_waves_arrived", tick(v.migrant_waves_arrived));
    d.set("first_winter_all_alive", v.first_winter_all_alive);
    d.set("people", people_to_dict(&v.people));
    let mut settlements = VarDictionary::new();
    settlements.set("ids", ints(v.settlements.iter().map(|r| i64::from(r.id))));
    settlements.set(
        "xs",
        ints(v.settlements.iter().map(|r| i64::from(r.tile.0))),
    );
    settlements.set(
        "ys",
        ints(v.settlements.iter().map(|r| i64::from(r.tile.1))),
    );
    settlements.set("food", ints(v.settlements.iter().map(|r| tick(r.food))));
    settlements.set(
        "food_need",
        floats(v.settlements.iter().map(|r| r.food_need)),
    );
    settlements.set("wood", ints(v.settlements.iter().map(|r| tick(r.wood))));
    settlements.set(
        "wood_target",
        floats(v.settlements.iter().map(|r| r.wood_target)),
    );
    settlements.set(
        "winter_wood_need",
        floats(v.settlements.iter().map(|r| r.winter_wood_need)),
    );
    d.set("settlements", settlements);
    let mut storage = VarDictionary::new();
    storage.set(
        "zones",
        ints(v.stockpile_tiles.iter().map(|r| i64::from(r.zone))),
    );
    storage.set(
        "xs",
        ints(v.stockpile_tiles.iter().map(|r| i64::from(r.tile.0))),
    );
    storage.set(
        "ys",
        ints(v.stockpile_tiles.iter().map(|r| i64::from(r.tile.1))),
    );
    storage.set(
        "can_hold",
        bytes(v.stockpile_tiles.iter().map(|r| r.can_hold)),
    );
    storage.set(
        "rooms",
        ints(
            v.stockpile_tiles
                .iter()
                .map(|r| r.room.map_or(-1, i64::from)),
        ),
    );
    storage.set(
        "room_roles",
        ints(
            v.stockpile_tiles
                .iter()
                .map(|r| r.room_role.map_or(-1, i64::from)),
        ),
    );
    storage.set("roofed", bytes(v.stockpile_tiles.iter().map(|r| r.roofed)));
    d.set("stockpile_tiles", storage);
    let mut stacks = VarDictionary::new();
    stacks.set("xs", ints(v.stacks.iter().map(|r| i64::from(r.tile.0))));
    stacks.set("ys", ints(v.stacks.iter().map(|r| i64::from(r.tile.1))));
    stacks.set("kinds", ints(v.stacks.iter().map(|r| i64::from(r.kind))));
    stacks.set("counts", ints(v.stacks.iter().map(|r| i64::from(r.count))));
    stacks.set(
        "capacities",
        ints(v.stacks.iter().map(|r| i64::from(r.capacity))),
    );
    stacks.set("reserved", bytes(v.stacks.iter().map(|r| r.reserved)));
    stacks.set(
        "zones",
        ints(v.stacks.iter().map(|r| r.zone.map_or(-1, i64::from))),
    );
    d.set("stacks", stacks);
    let mut unreachable = VarDictionary::new();
    unreachable.set(
        "agent_ids",
        ints(v.unreachable.iter().map(|r| signed(r.agent))),
    );
    unreachable.set(
        "xs",
        ints(v.unreachable.iter().map(|r| i64::from(r.tile.0))),
    );
    unreachable.set(
        "ys",
        ints(v.unreachable.iter().map(|r| i64::from(r.tile.1))),
    );
    unreachable.set(
        "until_ticks",
        ints(v.unreachable.iter().map(|r| tick(r.until))),
    );
    d.set("unreachable", unreachable);
    let mut lots = VarDictionary::new();
    lots.set("days", ints(v.meat_lots.iter().map(|r| i64::from(r.0))));
    lots.set("counts", ints(v.meat_lots.iter().map(|r| i64::from(r.1))));
    d.set("meat_lots", lots);
    d.set("tech", super::super::knowledge::tech_to_dict(&v.tech));
    d.set("waits", super::super::knowledge::waits_to_dict(&v.waits));
    d.set(
        "neighbours",
        super::super::neighbours::neighbours_to_dict(&v.neighbours),
    );
    d.set("animals", super::super::story::animals_to_dict(&v.animals));
    d.set("rooms", super::super::space::rooms_to_dict(&v.rooms));
    d.set("story", super::super::story::story_to_dict(&v.story));
    d.set(
        "workshops",
        super::super::economy::workshops_to_dict(&v.workshops),
    );
    d
}
