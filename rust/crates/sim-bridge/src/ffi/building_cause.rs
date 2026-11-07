//! Persisted building provenance. NOT_COMPILED/NOT_RUN.
use super::origin;
use godot::prelude::*;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::building_site::BuildCause;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::{Command, CommandRecord};
use sim_engine::SimResources;

fn inner(record: &CommandRecord) -> &Command {
    match &record.command {
        Command::Steward { command, .. } => command,
        command => command,
    }
}

fn build_pose(command: &Command) -> Option<((u32, u32), u8)> {
    match command {
        Command::BuildTemplate {
            anchor, rotation, ..
        } => Some((*anchor, *rotation)),
        _ => None,
    }
}

/// Historical upgrade poses come from the retained original build command witness.
fn pose(res: &SimResources, site: u32, record: u32, command: &Command) -> Option<((u32, u32), u8)> {
    if let Some(pose) = build_pose(command) {
        return Some(pose);
    }
    res.chronicle.causes().find_map(|(index, cause)| {
        let CauseRef::Building {
            site: old_site,
            record: old_record,
            ..
        } = cause
        else {
            return None;
        };
        if old_site != site || old_record >= record {
            return None;
        }
        res.chronicle
            .entry(index)
            .filter(|e| e.kind == ChronicleKind::BuildingComplete)?;
        let source = res
            .colony
            .command_log
            .get(usize::try_from(old_record).ok()?)?;
        source.result.accepted.then_some(())?;
        build_pose(inner(source))
    })
}

fn source_in_time(
    source: &CommandRecord,
    executed_tick: u64,
    command_boundary: u64,
    completed_tick: Option<u64>,
) -> bool {
    source.result.accepted
        && match completed_tick {
            Some(tick) => source.tick <= tick && tick <= executed_tick,
            None => source.tick <= command_boundary,
        }
}

/// Resolve only an exact saved site/source pair; return the UI's fail-closed receipt.
pub fn collect(res: &SimResources, command_boundary: u64, site: i64, record: i64) -> VarDictionary {
    let mut row = VarDictionary::new();
    row.set("valid", false);
    let (Ok(site), Ok(record)) = (u32::try_from(site), u32::try_from(record)) else {
        return row;
    };
    let catalog = installed_or_bundled();
    let historical = res.chronicle.causes().find_map(|(index, cause)| {
        let CauseRef::Building {
            site: old_site,
            record: old_record,
            origin,
        } = cause
        else {
            return None;
        };
        if old_site != site || old_record != record {
            return None;
        }
        let entry = res
            .chronicle
            .entry(index)
            .filter(|e| e.kind == ChronicleKind::BuildingComplete)?;
        // The save loader pins the catalogue revision; ordinal is immutable within that revision.
        let template = catalog
            .building_templates()
            .buildings
            .get(usize::try_from(entry.value).ok()?)?;
        Some((template, origin, Some(index)))
    });
    let witness = historical.or_else(|| {
        let current = res
            .colony
            .buildings
            .sites
            .get(&site)
            .filter(|s| s.source_record == record)?;
        Some((
            catalog.building_templates().get(&current.template)?,
            current.cause,
            None,
        ))
    });
    let Some((template, cause, completion)) = witness else {
        return row;
    };
    let Some(source) = usize::try_from(record)
        .ok()
        .and_then(|index| res.colony.command_log.get(index))
    else {
        return row;
    };
    let completed_tick = completion.and_then(|index| res.chronicle.entry(index).map(|e| e.tick));
    if !source_in_time(source, res.current_tick, command_boundary, completed_tick) {
        return row;
    }
    let command = inner(source);
    let command_key = match command {
        Command::BuildTemplate { id, .. } if id == &template.id => "BUILD_CAUSE_ORDER_BUILD",
        Command::UpgradeBuilding { site: target } if *target == site => "BUILD_CAUSE_ORDER_UPGRADE",
        _ => return row,
    };
    let Some((anchor, rotation)) = pose(res, site, record, command) else {
        return row;
    };
    row.set("valid", true);
    row.set("site", i64::from(site));
    row.set("record", i64::from(record));
    row.set("template_id", template.id.as_str());
    row.set("name_key", template.name_key.as_str());
    row.set("tier", i64::from(template.tier));
    row.set("command_key", command_key);
    row.set("tick", i64::try_from(source.tick).unwrap_or(i64::MAX));
    row.set("x", i64::from(anchor.0));
    row.set("y", i64::from(anchor.1));
    row.set("rotation", i64::from(rotation));
    row.set("completion_entry", completion.map(i64::from).unwrap_or(-1));
    origin(&mut row, cause);
    row.set(
        "cause_kind",
        match cause {
            BuildCause::Command => "command",
            BuildCause::Zone(_) => "zone",
            BuildCause::Steward(_) => "steward",
        },
    );
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::CommandResult;

    fn source(tick: u64) -> CommandRecord {
        CommandRecord {
            tick,
            command: Command::BuildTemplate {
                id: "hut".into(),
                anchor: (1, 1),
                rotation: 0,
                station: None,
            },
            result: CommandResult::ok(1),
        }
    }

    #[test]
    fn paused_building_cause_accepts_only_the_actual_command_boundary() {
        let mut record = source(8);
        assert!(source_in_time(&record, 7, 8, None));
        assert!(!source_in_time(&record, 7, 7, None));
        record.tick = 9;
        assert!(!source_in_time(&record, 7, 8, None));
        record.tick = 8;
        record.result.accepted = false;
        assert!(!source_in_time(&record, 7, 8, None));
    }

    #[test]
    fn historical_building_cause_requires_an_executed_completion_after_its_source() {
        assert!(source_in_time(&source(6), 7, 8, Some(7)));
        assert!(!source_in_time(&source(8), 7, 8, Some(8)));
        assert!(!source_in_time(&source(8), 7, 8, Some(7)));
    }
}
