//! Intervention schedules (Pass 7 section 8.3).
//!
//! A schedule is a JSON list of `{"tick", "kind", "settlement", "amount"}`
//! rows in non-decreasing tick order. `kind` is one of
//! `sim_systems::runtime::intervention::KINDS`; `settlement` is `"home"` (the
//! lowest settlement id at that tick) or a settlement id; `amount` is hours for
//! `rain`, nights for `cold_snap` and units otherwise. `settlement` is ignored
//! by `cold_snap` and `windfall`. Rows apply when `engine.current_tick()`
//! equals their tick, in file order, before `engine.tick()`.
use serde_json::Value;
use sim_core::components::SettlementId;
use sim_engine::{InterventionRecord, SimEngine};
use sim_systems::runtime::intervention::{apply_intervention, home_settlement, intervention_from_kind, KINDS};

/// Settlement of a scheduled intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The lowest settlement id at the row's tick.
    Home,
    /// A settlement id.
    Id(SettlementId),
}

/// One scheduled intervention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleEntry {
    /// Completed ticks at which it applies.
    pub tick: u64,
    /// Kind name (one of `KINDS`).
    pub kind: String,
    /// Settlement it acts on.
    pub settlement: Target,
    /// Hours, nights or units.
    pub amount: u32,
}

/// Settlement id used when "home" names no settlement: 0 is never issued
/// (`SimResources::next_settlement_id` starts at 1), so the row applies 0.
const NO_SETTLEMENT: SettlementId = 0;

/// Parse a schedule; unknown kinds, malformed rows and ticks out of order are errors.
pub fn parse_schedule(text: &str) -> Result<Vec<ScheduleEntry>, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("schedule is not JSON: {e}"))?;
    let rows = value.as_array().ok_or("schedule must be a JSON list")?;
    let mut entries = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let object = row.as_object().ok_or_else(|| format!("row {index} is not an object"))?;
        if object.len() != 4 || !["tick", "kind", "settlement", "amount"].iter().all(|k| object.contains_key(*k)) {
            return Err(format!("row {index} must have exactly tick, kind, settlement and amount"));
        }
        let tick = object["tick"].as_u64().ok_or_else(|| format!("row {index}: tick must be a non-negative integer"))?;
        let kind = object["kind"].as_str().ok_or_else(|| format!("row {index}: kind must be a string"))?;
        if !KINDS.contains(&kind) {
            return Err(format!("row {index}: unknown kind {kind}"));
        }
        let settlement = match &object["settlement"] {
            Value::String(s) if s == "home" => Target::Home,
            other => Target::Id(
                other
                    .as_u64()
                    .and_then(|id| SettlementId::try_from(id).ok())
                    .ok_or_else(|| format!("row {index}: settlement must be \"home\" or a settlement id"))?,
            ),
        };
        let amount = object["amount"]
            .as_u64()
            .and_then(|a| u32::try_from(a).ok())
            .ok_or_else(|| format!("row {index}: amount must be a u32"))?;
        if entries.last().is_some_and(|last: &ScheduleEntry| last.tick > tick) {
            return Err(format!("row {index}: ticks out of order"));
        }
        entries.push(ScheduleEntry { tick, kind: kind.to_string(), settlement, amount });
    }
    Ok(entries)
}

/// Apply every entry whose tick equals `engine.current_tick()`, starting at
/// `*next` (entries before it were applied already), in file order; advance
/// `*next` past them. Returns the records.
pub fn apply_due(engine: &mut SimEngine, entries: &[ScheduleEntry], next: &mut usize) -> Vec<InterventionRecord> {
    let now = engine.current_tick();
    let mut records = Vec::new();
    while let Some(entry) = entries.get(*next).filter(|entry| entry.tick <= now) {
        *next += 1;
        if entry.tick < now {
            continue;
        }
        let sid = match entry.settlement {
            Target::Home => home_settlement(&engine.resources).unwrap_or(NO_SETTLEMENT),
            Target::Id(id) => id,
        };
        if let Some(intervention) = intervention_from_kind(&entry.kind, sid, entry.amount) {
            records.push(apply_intervention(engine, intervention));
        }
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rows_and_rejects_bad_schedules() {
        let entries = parse_schedule(
            r#"[{"tick": 5, "kind": "rain", "settlement": "home", "amount": 13},
                {"tick": 5, "kind": "windfall", "settlement": 2, "amount": 10}]"#,
        )
        .expect("valid");
        assert_eq!(entries[0], ScheduleEntry { tick: 5, kind: "rain".into(), settlement: Target::Home, amount: 13 });
        assert_eq!(entries[1].settlement, Target::Id(2));
        for bad in [
            r#"{"tick": 1}"#,
            r#"[{"tick": 1, "kind": "flood", "settlement": "home", "amount": 1}]"#,
            r#"[{"tick": 1, "kind": "rain", "settlement": "home"}]"#,
            r#"[{"tick": 1, "kind": "rain", "settlement": "away", "amount": 1}]"#,
            r#"[{"tick": -1, "kind": "rain", "settlement": "home", "amount": 1}]"#,
            r#"[{"tick": 1, "kind": "rain", "settlement": "home", "amount": 1, "x": 0}]"#,
            r#"[{"tick": 2, "kind": "rain", "settlement": "home", "amount": 1},
                {"tick": 1, "kind": "rain", "settlement": "home", "amount": 1}]"#,
            "not json",
        ] {
            assert!(parse_schedule(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn due_rows_apply_in_file_order_at_their_tick() {
        let mut engine = crate::scenes::game_v12_scene(2);
        let entries = parse_schedule(
            r#"[{"tick": 0, "kind": "gift_wood", "settlement": "home", "amount": 4},
                {"tick": 1, "kind": "gift_wood", "settlement": "home", "amount": 4},
                {"tick": 1, "kind": "rot_wood", "settlement": "home", "amount": 1},
                {"tick": 3, "kind": "cold_snap", "settlement": "home", "amount": 1}]"#,
        )
        .expect("valid");
        let mut next = 0;
        // Tick 0: no settlement has formed yet, so "home" applies nothing.
        assert!(engine.resources.settlements.is_empty());
        let records = apply_due(&mut engine, &entries, &mut next);
        assert_eq!((records.len(), records[0].applied, next), (1, 0, 1));
        engine.tick();
        assert!(!engine.resources.settlements.is_empty());
        let records = apply_due(&mut engine, &entries, &mut next);
        assert_eq!(records.iter().map(|r| r.applied).collect::<Vec<_>>(), vec![4, 1]);
        engine.tick();
        assert!(apply_due(&mut engine, &entries, &mut next).is_empty());
        engine.tick();
        assert_eq!(apply_due(&mut engine, &entries, &mut next).len(), 1);
        assert_eq!((next, engine.resources.cold_snap_nights), (4, 1));
        assert_eq!(engine.resources.intervention_log.len(), 4);
    }
}
