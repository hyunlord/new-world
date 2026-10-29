//! Historical demand is projected only from the actual saved winter-plan evidence.
use crate::ffi::{chronicle::ChronicleRows, story::WhyRows};
use godot::prelude::*;
use sim_core::colony::{
    food::FoodOwner,
    food_evidence::{AcquisitionReason, OperationEvidence},
    CancelReason,
};
use sim_engine::SimResources;

fn demand(res: &SimResources, index: i64) -> i64 {
    u32::try_from(index)
        .ok()
        .and_then(|index| res.food.winter_plans.get(&index))
        .map_or(-1, |row| {
            i64::try_from(row.daily_demand_milli).unwrap_or(i64::MAX)
        })
}

fn evidence_indexes(rows: &WhyRows) -> Vec<Option<u32>> {
    rows.link
        .iter()
        .enumerate()
        .map(|(i, link)| {
            if *link == 0 {
                rows.code.get(i).and_then(|code| u32::try_from(*code).ok())
            } else {
                None
            }
        })
        .collect()
}

fn number(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn owner_dict(owner: FoodOwner) -> VarDictionary {
    let mut out = VarDictionary::new();
    match owner {
        FoodOwner::Ground((x, y)) => {
            out.set("kind", "ground");
            out.set("x", i64::from(x));
            out.set("y", i64::from(y));
        }
        FoodOwner::Carried(agent) => {
            out.set("kind", "carried");
            out.set("agent", number(agent));
        }
        FoodOwner::Shelf { workshop, key } => {
            out.set("kind", "shelf");
            out.set("workshop", i64::from(workshop));
            out.set("item", i64::from(key.kind.code()));
            out.set("quality_bucket", i64::from(key.quality_bucket));
            out.set(
                "material",
                key.material
                    .map_or(-1, |material| i64::from(material.raw())),
            );
        }
        FoodOwner::Pot { id, key } => {
            out.set("kind", "pot");
            out.set("id", i64::from(id));
            out.set("item", i64::from(key.kind.code()));
            out.set("quality_bucket", i64::from(key.quality_bucket));
            out.set(
                "material",
                key.material
                    .map_or(-1, |material| i64::from(material.raw())),
            );
        }
    }
    out
}

fn evidence(res: &SimResources, index: Option<u32>) -> VarDictionary {
    let mut out = VarDictionary::new();
    let Some(index) = index else {
        return out;
    };
    let Some(record) = res.food.evidence.operations.get(&index) else {
        return out;
    };
    let Some(entry) = res.chronicle.entry(index) else {
        return out;
    };
    out.set("schema_version", 1_i64);
    out.set("entry_index", i64::from(index));
    out.set("recorded_tick", number(entry.tick));
    out.set("subject", entry.subject.map_or(-1, number));
    match *record {
        OperationEvidence::Acquisition { receipt, flow } => {
            out.set("type", "acquisition");
            let mut detail = VarDictionary::new();
            detail.set("tick", number(receipt.tick));
            let reason = match receipt.reason {
                AcquisitionReason::TargetUnavailable => "target_unavailable",
                AcquisitionReason::OutputPlacementFull => "output_placement_full",
                AcquisitionReason::OutputRejected => "output_rejected",
                AcquisitionReason::Unfinished => "unfinished",
                AcquisitionReason::WorkDisabled => "work_disabled",
                AcquisitionReason::NoRipeInRegion => "no_ripe_in_region",
                AcquisitionReason::Cancelled(reason) => {
                    detail.set(
                        "cancel_reason",
                        match reason {
                            CancelReason::Needs => "needs",
                            CancelReason::TargetGone => "target_gone",
                            CancelReason::Command => "command",
                            CancelReason::Stuck => "stuck",
                            CancelReason::Interrupted => "interrupted",
                            CancelReason::Died => "died",
                            CancelReason::Unreachable => "unreachable",
                        },
                    );
                    "cancelled"
                }
            };
            detail.set("reason", reason);
            if let Some((x, y)) = receipt.target {
                detail.set(
                    "target",
                    PackedInt64Array::from([i64::from(x), i64::from(y)].as_slice()),
                );
            }
            if let Some(tick) = receipt.job_started {
                detail.set("job_started", number(tick));
            }
            let mut interval = VarDictionary::new();
            interval.set("scope", "colony");
            interval.set("from_tick", number(flow.from_tick));
            interval.set("to_tick", number(flow.to_tick));
            for (key, value) in [
                "harvested_units",
                "craft_input_units",
                "craft_output_units",
                "eaten_units",
                "spoiled_units",
                "lost_units",
                "imported_units",
                "exported_units",
            ]
            .into_iter()
            .zip(flow.units.0)
            {
                interval.set(key, number(value));
            }
            out.set("receipt", detail);
            out.set("flow", interval);
        }
        OperationEvidence::Intake {
            owner,
            item,
            born_day,
            lifetime_days,
        } => {
            out.set("type", "intake");
            out.set("owner", owner_dict(owner));
            out.set("item", i64::from(item.code()));
            out.set("born_day", i64::from(born_day));
            out.set("lifetime_days", i64::from(lifetime_days));
        }
        OperationEvidence::Stored {
            owner,
            item,
            pieces,
            born_day,
        } => {
            out.set("type", "stored");
            out.set("owner", owner_dict(owner));
            out.set("item", i64::from(item.code()));
            out.set("born_day", i64::from(born_day));
            out.set("pieces", i64::from(pieces));
        }
    }
    out
}

pub(in crate::ffi) fn food_chronicle_rows_to_dict(
    rows: ChronicleRows,
    page: bool,
    res: &SimResources,
) -> VarDictionary {
    let food_evidence: Array<VarDictionary> = rows
        .index
        .iter()
        .map(|index| evidence(res, u32::try_from(*index).ok()))
        .collect();
    let values: Vec<i64> = rows.index.iter().map(|index| demand(res, *index)).collect();
    let mut result = if page {
        crate::ffi::chronicle::rows_to_dict(rows, true)
    } else {
        crate::ffi::chronicle::rows_to_dict(rows, false)
    };
    result.set(
        "food_daily_milli",
        PackedInt64Array::from(values.as_slice()),
    );
    result.set("food_evidence", food_evidence);
    result
}

pub(in crate::ffi) fn food_why_to_dict(rows: &WhyRows, res: &SimResources) -> VarDictionary {
    let food_evidence: Array<VarDictionary> = evidence_indexes(rows)
        .into_iter()
        .map(|index| evidence(res, index))
        .collect();
    let values: Vec<i64> = rows
        .link
        .iter()
        .enumerate()
        .map(|(i, link)| {
            if *link == 0 {
                rows.code.get(i).map_or(-1, |index| demand(res, *index))
            } else {
                -1
            }
        })
        .collect();
    let mut result = crate::ffi::story::why_to_dict(rows);
    result.set(
        "food_daily_milli",
        PackedInt64Array::from(values.as_slice()),
    );
    result.set("food_evidence", food_evidence);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{colony::food::WinterPlanEvidence, material::MaterialRegistry};

    #[test]
    fn historical_demand_uses_entry_identity_and_preserves_missing_and_zero() {
        let mut res = SimResources::new(8, 8, MaterialRegistry::new());
        res.food.winter_plans.insert(
            12,
            WinterPlanEvidence {
                tick: 77,
                daily_demand_milli: 2400,
                stored_units: 12,
                required_units: 15,
            },
        );
        res.food.winter_plans.insert(
            19,
            WinterPlanEvidence {
                tick: 90,
                daily_demand_milli: 0,
                stored_units: 0,
                required_units: 0,
            },
        );
        let indexes = [19, -1, 12, 1, 12];
        let observed: Vec<_> = indexes.iter().map(|index| demand(&res, *index)).collect();
        assert_eq!(observed, [0, -1, 2400, -1, 2400]);
        assert_eq!(observed.len(), indexes.len());
        res.current_tick = 100_000;
        assert_eq!(demand(&res, 12), 2400);
    }
}

#[cfg(test)]
mod evidence_tests {
    use super::*;
    #[test]
    fn food_evidence_identity_never_treats_modifier_codes_as_entries() {
        let rows = WhyRows {
            link: vec![0, 1, 2, 0],
            code: vec![19, 19, 19, -1],
            ..WhyRows::default()
        };
        assert_eq!(evidence_indexes(&rows), vec![Some(19), None, None, None]);
    }
}
