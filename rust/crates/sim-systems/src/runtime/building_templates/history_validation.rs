//! Immutable generation witnesses; catalogue revision pins the stored template ordinals.
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::building_site::{BuildCause, BuildingSite};
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::Command;
use sim_engine::SimResources;
use std::collections::BTreeMap;

pub(super) fn source<'a>(
    res: &'a SimResources,
    site: &BuildingSite,
    record: u32,
    origin: BuildCause,
    template_id: &str,
    command_boundary: u64,
) -> Result<&'a Command, String> {
    let record = res
        .colony
        .command_log
        .get(record as usize)
        .ok_or("missing building source command")?;
    if !record.result.accepted || record.tick > command_boundary {
        return Err("unaccepted/future building source".into());
    }
    let (command, steward) = match &record.command {
        Command::Steward { rule, command } => (command.as_ref(), Some(*rule)),
        command => (command, None),
    };
    let origin_valid = match (origin, steward) {
        (BuildCause::Command, None) => true,
        (BuildCause::Steward(request), Some(rule)) => request == u32::from(rule),
        (BuildCause::Zone(zone), Some(_)) => zone < res.colony.next_zone_id,
        _ => false,
    };
    if !origin_valid {
        return Err("building origin disagrees with source envelope".into());
    }
    let catalog = installed_or_bundled();
    let template = catalog
        .building_templates()
        .get(template_id)
        .ok_or("unknown source template")?;
    match command {
        Command::BuildTemplate {
            id,
            anchor,
            rotation,
            station,
        } => {
            let selected = station.as_ref().or_else(|| template.stations.first());
            if id != template_id
                || *anchor != site.anchor
                || *rotation != site.rotation
                || selected != site.station.as_ref()
            {
                return Err("building source plan identity mismatch".into());
            }
        }
        Command::UpgradeBuilding { site: target }
            if *target == site.id && template.upgrade_from.is_some() => {}
        _ => return Err("building source is not the matching build/upgrade".into()),
    }
    Ok(command)
}

pub(super) fn validate(res: &SimResources, command_boundary: u64) -> Result<(), String> {
    let catalog = installed_or_bundled();
    let mut generations = BTreeMap::<(u32, u32), u32>::new();
    let mut previous = BTreeMap::<u32, (u32, usize)>::new();
    for (index, entry) in res.chronicle.entries().iter().enumerate() {
        if entry.kind != ChronicleKind::BuildingComplete {
            continue;
        }
        let code =
            usize::try_from(entry.value).map_err(|_| "negative building template ordinal")?;
        let template = catalog
            .building_templates()
            .buildings
            .get(code)
            .ok_or("building template ordinal out of range")?;
        let Some(CauseRef::Building {
            site,
            record,
            origin,
        }) = res
            .chronicle
            .cause_of(u32::try_from(index).map_err(|_| "chronicle index overflow")?)
        else {
            return Err("building completion has no immutable cause witness".into());
        };
        let saved = res
            .colony
            .buildings
            .sites
            .get(&site)
            .ok_or("completion points at unknown site")?;
        let command = source(res, saved, record, origin, &template.id, command_boundary)?;
        let source_tick = res.colony.command_log[record as usize].tick;
        let expected_cause = match origin {
            BuildCause::Command => 0,
            BuildCause::Zone(_) => 1,
            BuildCause::Steward(_) => 2,
        };
        if entry.cause != expected_cause
            || entry.tick < source_tick
            || entry.tick > res.current_tick
        {
            return Err("building completion time/origin mismatch".into());
        }
        if matches!(command, Command::UpgradeBuilding { .. }) {
            let Some((prior_record, prior_code)) = previous.get(&site) else {
                return Err("upgrade lacks prior completion".into());
            };
            if *prior_record >= record
                || template.upgrade_from.as_deref()
                    != Some(
                        catalog.building_templates().buildings[*prior_code]
                            .id
                            .as_str(),
                    )
            {
                return Err("upgrade predecessor mismatch".into());
            }
        }
        let index = u32::try_from(index).map_err(|_| "chronicle index overflow")?;
        if generations.insert((site, record), index).is_some() {
            return Err("duplicate generation completion".into());
        }
        previous.insert(site, (record, code));
    }
    for site in res.colony.buildings.sites.values() {
        if let Some(index) = site.completion_entry {
            let entry = res
                .chronicle
                .entry(index)
                .ok_or("missing current completion entry")?;
            let code =
                usize::try_from(entry.value).map_err(|_| "negative current template ordinal")?;
            if generations.get(&(site.id, site.source_record)) != Some(&index)
                || catalog
                    .building_templates()
                    .buildings
                    .get(code)
                    .is_none_or(|template| template.id != site.template)
                || res.chronicle.cause_of(index)
                    != Some(CauseRef::Building {
                        site: site.id,
                        record: site.source_record,
                        origin: site.cause,
                    })
            {
                return Err("current building completion witness mismatch".into());
            }
        }
    }
    Ok(())
}
