//! Additive per-seed block (Pass 20, `game_v25` and later relations scenes;
//! absent in every other scene, so no earlier summary gains a key). Pass 21
//! adds a `pass21` key only under a dispute cap (`super::relation_caps`).
//!
//! `tallies`: the Pass 20 counters. `pairs_end`: every living pair at the end
//! as `[a, b, value, estranged 0/1, base]`. `trips`: every porter trip, ended
//! (in end order) then in flight (by porter), as `[porter, from, to, kind 0
//! food / 1 wood, cause, withdrawn, delivered, returned, lost, failed 0/1,
//! depart_tick, end_tick or -1]`. `in_flight_units`: units still carried at
//! the end. `days`: every ordered pair at every decision minute as `[tick,
//! from, to, value, estranged, in_reach, from_food_days_permille, from_food,
//! from_wood_nights, from_wood, to_food_days_permille, to_food,
//! to_wood_nights, to_wood, aid_food, aid_wood, exchange, sent_kind or -1,
//! sent_amount]` (booleans 0/1).
use serde_json::{json, Value};
use sim_core::components::TripRecord;
use sim_engine::SimEngine;

fn trip_row(record: &TripRecord) -> Value {
    json!([
        record.porter,
        record.trip.from,
        record.trip.to,
        record.trip.kind.as_u8(),
        record.cause,
        record.trip.amount,
        record.delivered,
        record.returned,
        record.lost,
        u8::from(record.failed),
        record.trip.depart_tick,
        record
            .end_tick
            .map_or(-1, |tick| i64::try_from(tick).unwrap_or(i64::MAX)),
    ])
}

/// The per-seed block, `None` without the relations rule.
pub(super) fn seed_block(engine: &SimEngine) -> Option<Value> {
    let r = &engine.resources;
    r.rules.society.relations?;
    let t = &r.tallies;
    let relations = &r.settlement_relations;
    let pairs: Vec<Value> = relations
        .pairs
        .iter()
        .map(|((a, b), s)| json!([a, b, s.value, u8::from(s.estranged), s.base]))
        .collect();
    let trips: Vec<Value> = relations
        .trips
        .iter()
        .chain(relations.active.values())
        .map(trip_row)
        .collect();
    let in_flight: u64 = relations
        .active
        .values()
        .map(|record| {
            u64::from(
                record
                    .trip
                    .amount
                    .saturating_sub(record.delivered + record.returned + record.lost),
            )
        })
        .sum();
    let days: Vec<Value> = relations
        .days
        .iter()
        .map(|d| {
            let (kind, amount) = d
                .sent
                .map_or((-1, 0), |(kind, amount)| (i64::from(kind.as_u8()), amount));
            json!([
                d.tick,
                d.from,
                d.to,
                d.value,
                u8::from(d.estranged),
                u8::from(d.in_reach),
                d.from_measure[0],
                d.from_measure[1],
                d.from_measure[2],
                d.from_measure[3],
                d.to_measure[0],
                d.to_measure[1],
                d.to_measure[2],
                d.to_measure[3],
                u8::from(d.aid_food),
                u8::from(d.aid_wood),
                u8::from(d.exchange),
                kind,
                amount,
            ])
        })
        .collect();
    let mut block = json!({
        "tallies": {
            "porter_trips": t.porter_trips, "porters_departed": t.porters_departed,
            "porter_deliveries": t.porter_deliveries, "porter_turnbacks": t.porter_turnbacks,
            "porter_loads_lost": t.porter_loads_lost, "porter_units_withdrawn": t.porter_units_withdrawn,
            "porter_food_delivered": t.porter_food_delivered, "porter_wood_delivered": t.porter_wood_delivered,
            "porter_units_returned": t.porter_units_returned, "porter_units_lost": t.porter_units_lost,
            "forage_disputes": t.forage_disputes, "quarrels": t.quarrels,
            "villages_estranged": t.villages_estranged, "villages_reconciled": t.villages_reconciled,
        },
        "pairs_end": pairs,
        "trips": trips,
        "in_flight_units": in_flight,
        "days": days,
    });
    // Pass 21: the capped rule's keys (`game_v26`; absent in `game_v25`).
    if let (Some(extra), Some(object)) =
        (super::relation_caps::block(engine), block.as_object_mut())
    {
        object.insert("pass21".into(), extra);
    }
    Some(block)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_block_is_present_with_the_relations_rule_only() {
        let mut engine = crate::scenes::baseline_scene(3);
        let block = super::seed_block(&engine).expect("baseline block");
        // Tick 0: no porter has left, nothing is in flight.
        assert_eq!(block["tallies"]["porters_departed"], 0);
        assert_eq!(block["in_flight_units"], 0);
        assert!(block["trips"].as_array().is_some_and(Vec::is_empty));
        engine.resources.rules.society.relations = None;
        assert!(super::seed_block(&engine).is_none());
    }
}
