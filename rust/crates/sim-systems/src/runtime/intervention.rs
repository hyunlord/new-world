//! Revitalize Pass 7 — applying the god's hand between ticks.
//!
//! [`apply_intervention`] changes the world at once, before the next
//! `SimEngine::tick`, appends an [`InterventionRecord`] to
//! `SimResources::intervention_log` (every call, `applied` 0 included) and
//! counts `Tallies::interventions_applied` when something changed. No random
//! number is drawn. Rain and cold snaps leave state that `FireSystem` reads;
//! every other kind changes stockpiles or deadwood directly. The log and the
//! tally are display-only.
//!
//! Pass 11: under `FirewoodRule::rain_soak_permille` (`game_v17`) a rain also
//! removes `floor(wood × permille / 1000)` from the settlement's stored wood at
//! application (`InterventionRecord::soaked_wood`).

use sim_core::components::{ResourceKind, SettlementId};
use sim_engine::{Intervention, InterventionRecord, SimEngine, SimResources};

use crate::runtime::resource_regen::season_at_tick;

/// Winter index of `SeasonRule` seasons.
const WINTER: usize = 3;

/// Kind names shared by schedules and the FFI command.
pub const KINDS: [&str; 7] = ["gift_food", "spoil_food", "gift_wood", "rot_wood", "rain", "cold_snap", "windfall"];

/// Build an intervention from its kind name (see [`KINDS`]). `amount` is
/// hours for `rain`, nights for `cold_snap` and units otherwise; `settlement`
/// is ignored by `cold_snap` and `windfall`. `None` for an unknown kind.
pub fn intervention_from_kind(kind: &str, settlement: SettlementId, amount: u32) -> Option<Intervention> {
    Some(match kind {
        "gift_food" => Intervention::GiftFood { settlement, amount },
        "spoil_food" => Intervention::SpoilFood { settlement, amount },
        "gift_wood" => Intervention::GiftWood { settlement, amount },
        "rot_wood" => Intervention::RotWood { settlement, amount },
        "rain" => Intervention::Rain { settlement, hours: amount },
        "cold_snap" => Intervention::ColdSnap { nights: amount },
        "windfall" => Intervention::Windfall { units: amount },
        _ => return None,
    })
}

/// Whether `kind` names a settlement intervention (it needs a settlement).
pub fn kind_needs_settlement(kind: &str) -> bool {
    !matches!(kind, "cold_snap" | "windfall")
}

/// The lowest settlement id now ("home" in schedules and the FFI), if any.
pub fn home_settlement(resources: &SimResources) -> Option<SettlementId> {
    resources.settlements.keys().copied().min()
}

/// Apply `intervention` now (between ticks) and record it.
pub fn apply_intervention(engine: &mut SimEngine, intervention: Intervention) -> InterventionRecord {
    let now = engine.current_tick();
    let resources = &mut engine.resources;
    let mut soaked_wood = 0u64;
    let applied = match intervention {
        Intervention::GiftFood { settlement, amount } => add(resources, settlement, ResourceKind::Food, amount),
        Intervention::SpoilFood { settlement, amount } => remove(resources, settlement, ResourceKind::Food, amount),
        Intervention::GiftWood { settlement, amount } => add(resources, settlement, ResourceKind::Wood, amount),
        Intervention::RotWood { settlement, amount } => remove(resources, settlement, ResourceKind::Wood, amount),
        Intervention::Rain { settlement, hours } => {
            if hours == 0 || !resources.settlements.contains_key(&settlement) {
                0
            } else {
                let until = now.saturating_add(u64::from(hours) * resources.ticks_per_day / 24);
                let entry = resources.fire_blocked_until.entry(settlement).or_insert(0);
                *entry = (*entry).max(until);
                // Pass 11: the rain soaks a share of the stored wood.
                let soak = resources.rules.society.firewood.map_or(0, |rule| rule.rain_soak_permille);
                if soak > 0 {
                    if let Some(store) = resources.settlements.get_mut(&settlement) {
                        let wood = store.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0);
                        let lost = (u64::from(wood) * u64::from(soak) / 1000) as u32;
                        soaked_wood = u64::from(store.withdraw(ResourceKind::Wood, lost));
                    }
                }
                u64::from(hours)
            }
        }
        Intervention::ColdSnap { nights } => {
            resources.cold_snap_nights = resources.cold_snap_nights.saturating_add(nights);
            u64::from(nights)
        }
        Intervention::Windfall { units } => windfall(resources, now, units),
    };
    let record = InterventionRecord { tick: now, intervention, applied, soaked_wood };
    resources.intervention_log.push(record);
    resources.tallies.interventions_applied += u64::from(applied > 0);
    record
}

fn add(resources: &mut SimResources, sid: SettlementId, kind: ResourceKind, amount: u32) -> u64 {
    resources.settlements.get_mut(&sid).map_or(0, |settlement| {
        let before = settlement.stockpile.get(&kind).copied().unwrap_or(0);
        settlement.store(kind, amount);
        u64::from(settlement.stockpile.get(&kind).copied().unwrap_or(0) - before)
    })
}

fn remove(resources: &mut SimResources, sid: SettlementId, kind: ResourceKind, amount: u32) -> u64 {
    resources
        .settlements
        .get_mut(&sid)
        .map_or(0, |settlement| u64::from(settlement.withdraw(kind, amount)))
}

/// Passes over the registered deadwood spots in key order from the first key
/// (Pass 11 `regional_wood`: nearest to the home settlement first),
/// one unit a spot while it is below its ceiling (its max; in winter
/// `min(max, winter_deadwood_cap)`, unless the windfall lands on the snow,
/// Pass 8 `windfall_over_snow`), until `units` are placed or a pass places
/// none. The winter cursor is left alone. Returns the units placed.
fn windfall(resources: &mut SimResources, now: u64, units: u32) -> u64 {
    let winter = season_at_tick(resources, now).is_some_and(|(season, _)| season == WINTER);
    let cap = resources
        .rules
        .society
        .firewood
        .filter(|rule| !rule.windfall_over_snow)
        .and_then(|rule| rule.winter_deadwood_cap)
        .filter(|_| winter);
    let mut spots: Vec<((u32, u32), u8)> = resources.wood_source_max.iter().map(|(k, v)| (*k, *v)).collect();
    // Pass 11 (`regional_wood`): the home settlement's nearest spots first
    // (Chebyshev distance to its formation tile, then key order).
    if resources.rules.society.firewood.is_some_and(|rule| rule.regional_wood) {
        if let Some(home) = home_settlement(resources).and_then(|sid| resources.settlements.get(&sid)).map(|s| s.formation_tile) {
            spots.sort_by_key(|((x, y), _)| (x.abs_diff(home.0).max(y.abs_diff(home.1)), *x, *y));
        }
    }
    let mut placed = 0u32;
    while placed < units {
        let mut this_pass = 0u32;
        for (key, max) in &spots {
            if placed == units {
                break;
            }
            let ceiling = cap.map_or(*max, |cap| cap.min(*max));
            let cur = resources.wood_tiles.get(key).copied().unwrap_or(0);
            if cur < ceiling {
                resources.wood_tiles.insert(*key, cur + 1);
                placed += 1;
                this_pass += 1;
            }
        }
        if this_pass == 0 {
            break;
        }
    }
    u64::from(placed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::Settlement;
    use sim_core::material::MaterialRegistry;

    fn engine() -> SimEngine {
        let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
        let mut settlement = Settlement::new_with_id(3, 0);
        settlement.store(ResourceKind::Food, 5);
        engine.resources.settlements.insert(3, settlement);
        engine
    }

    #[test]
    fn kinds_round_trip_and_unknown_kinds_are_none() {
        for kind in KINDS {
            assert!(intervention_from_kind(kind, 1, 2).is_some(), "{kind}");
        }
        assert_eq!(intervention_from_kind("flood", 1, 2), None);
        assert!(!kind_needs_settlement("windfall") && kind_needs_settlement("rain"));
    }

    #[test]
    fn spoil_never_goes_below_zero_and_every_call_is_logged() {
        let mut engine = engine();
        assert_eq!(home_settlement(&engine.resources), Some(3));
        let spoiled = apply_intervention(&mut engine, Intervention::SpoilFood { settlement: 3, amount: 9 });
        assert_eq!(spoiled.applied, 5);
        let unknown = apply_intervention(&mut engine, Intervention::GiftFood { settlement: 4, amount: 9 });
        assert_eq!(unknown.applied, 0);
        assert_eq!(engine.resources.intervention_log, vec![spoiled, unknown]);
        assert_eq!(engine.resources.tallies.interventions_applied, 1);
    }

    #[test]
    fn rain_keeps_the_later_end() {
        let mut engine = engine();
        apply_intervention(&mut engine, Intervention::Rain { settlement: 3, hours: 2 });
        apply_intervention(&mut engine, Intervention::Rain { settlement: 3, hours: 1 });
        assert_eq!(engine.resources.fire_blocked_until.get(&3), Some(&120));
        assert_eq!(apply_intervention(&mut engine, Intervention::Rain { settlement: 3, hours: 0 }).applied, 0);
    }
}
