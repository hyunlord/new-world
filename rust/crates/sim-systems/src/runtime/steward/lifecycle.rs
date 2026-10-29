//! Steward command provenance and player cancellation protection; no planning or RNG.
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::{
    steward::{rules, OwnedOrder, Target},
    Command,
};
use sim_engine::SimResources;

/// Targets of ordinary commands, captured before mutation.
pub fn targets(res: &SimResources, cmd: &Command) -> Vec<Target> {
    match cmd {
        Command::DeployFoodPot { tile: (x, y) } => vec![Target::Tile(*x, *y)],
        Command::DesignateChopTiles { tiles } => {
            tiles.iter().map(|&(x, y)| Target::Tile(x, y)).collect()
        }
        Command::PlaceBlueprint { kind, x, y } => res.rules.colony.map_or_else(Vec::new, |rule| {
            let (size, _, _) = kind.plan(&rule);
            if x.checked_add(size.0)
                .is_none_or(|right| right > res.tile_grid.width)
                || y.checked_add(size.1)
                    .is_none_or(|bottom| bottom > res.tile_grid.height)
            {
                return Vec::new();
            }
            sim_core::colony::ColonySite {
                kind: *kind,
                top_left: (*x, *y),
                size,
            }
            .tiles()
            .into_iter()
            .map(|(x, y)| Target::Tile(x, y))
            .collect()
        }),
        Command::AddStockpile { x0, y0, x1, y1 }
        | Command::DesignateHarvest { x0, y0, x1, y1 }
        | Command::AddHuntZone { x0, y0, x1, y1 }
        | Command::DesignateQuarry { x0, y0, x1, y1 } => {
            crate::commands::rect_tiles(res, *x0, *y0, *x1, *y1)
                .unwrap_or_default()
                .into_iter()
                .map(|(x, y)| Target::Tile(x, y))
                .collect()
        }
        Command::AddBill {
            workshop, recipe, ..
        } => vec![Target::Bill(*workshop, recipe.code())],
        Command::RemoveBill { workshop, bill }
        | Command::SetBillMode { workshop, bill, .. }
        | Command::SetBillSuspended { workshop, bill, .. }
        | Command::MoveBill { workshop, bill, .. } => res
            .colony
            .workshops
            .get(workshop)
            .and_then(|s| s.bill(*bill))
            .map(|b| vec![Target::Bill(*workshop, b.recipe.code())])
            .unwrap_or_default(),
        Command::OrderTeach {
            learner, knowledge, ..
        } => vec![Target::Teach(*learner, *knowledge)],
        Command::CancelTeach { order } => res
            .colony
            .teach_orders
            .iter()
            .find(|o| o.id == *order)
            .map(|o| vec![Target::Teach(o.learner, o.knowledge)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}
/// Read-only protection proposal captured before an accepted removal erases its targets.
pub struct PlayerProtection {
    targets: Vec<Target>,
    rules: Vec<u16>,
    until: u64,
    target_until: u64,
}
/// Capture intent without mutating state; rejected commands discard the proposal.
pub fn prepare_player(res: &SimResources, cmd: &Command, tick: u64) -> Option<PlayerProtection> {
    // An individual's food harvest remains cancellable even with the steward off.
    let harvest_targets: Vec<_> = match cmd {
        Command::CancelArea { x0, y0, x1, y1 } if res.rules.colony.is_some() => res
            .colony
            .designations
            .iter()
            .filter(|(tile, kind)| {
                **kind == sim_core::colony::Designation::Harvest
                    && tile.0 >= (*x0).min(*x1)
                    && tile.0 <= (*x0).max(*x1)
                    && tile.1 >= (*y0).min(*y1)
                    && tile.1 <= (*y0).max(*y1)
            })
            .map(|(&(x, y), _)| Target::Tile(x, y))
            .collect(),
        _ => Vec::new(),
    };
    if !res.steward.enabled && res.steward.owned.is_empty() && harvest_targets.is_empty() {
        return None;
    }
    let mut affected = targets(res, cmd);
    affected.extend(harvest_targets);
    match cmd {
        Command::CancelArea { x0, y0, x1, y1 }
        | Command::RemoveZoneArea { x0, y0, x1, y1 }
        | Command::CancelHunt { x0, y0, x1, y1 }
        | Command::DesignateDeconstruct { x0, y0, x1, y1 } => {
            for owned in &res.steward.owned {
                // Older saves recorded only the origin; derive the same ordinary footprint before removal.
                let footprint = targets(res, &owned.command);
                affected.extend(owned.targets.iter().chain(footprint.iter()).filter(|target| matches!(target, Target::Tile(x,y) if x >= x0.min(x1) && x <= x0.max(x1) && y >= y0.min(y1) && y <= y0.max(y1))).cloned());
            }
        }
        _ => {}
    }
    let until = tick.saturating_add(2 * res.ticks_per_day);
    let mut cancelled_rules = Vec::new();
    if matches!(
        cmd,
        Command::CancelArea { .. }
            | Command::RemoveZoneArea { .. }
            | Command::CancelHunt { .. }
            | Command::CancelTeach { .. }
            | Command::RemoveBill { .. }
            | Command::DesignateDeconstruct { .. }
    ) {
        for owned in &res.steward.owned {
            let footprint = targets(res, &owned.command);
            if owned
                .targets
                .iter()
                .chain(footprint.iter())
                .any(|target| affected.contains(target))
            {
                cancelled_rules.push(owned.rule);
            }
        }
    }
    let protected_until = if matches!(
        cmd,
        Command::AddBill { .. }
            | Command::SetBillMode { .. }
            | Command::SetBillSuspended { .. }
            | Command::MoveBill { .. }
    ) {
        u64::MAX
    } else {
        until
    };
    Some(PlayerProtection {
        targets: affected,
        rules: cancelled_rules,
        until,
        target_until: protected_until,
    })
}
/// Commit only a successful ordinary player command's captured protection.
pub fn protect_player(res: &mut SimResources, protection: PlayerProtection) {
    for rule in protection.rules {
        let cooldown = res.steward.cooldowns.entry(rule).or_default();
        *cooldown = (*cooldown).max(protection.until);
    }
    for target in protection.targets {
        res.steward.vetoes.insert(target, protection.target_until);
    }
}
/// Apply the same successful source metadata for live and replay commands.
pub fn accepted(
    world: &mut hecs::World,
    res: &mut SimResources,
    rule: u16,
    command: Command,
    targets: Vec<Target>,
) {
    if super::defence::is_rule(rule) {
        if let Command::SetDrafted { agent, drafted } = command {
            if drafted {
                super::defence::assign_arming(world, res, agent);
                res.steward.defence.owned_ids.insert(agent);
            } else {
                res.steward.defence.owned_ids.remove(&agent);
            }
        }
        return;
    }
    let cooldown = rules()
        .ok()
        .and_then(|d| d.rules.iter().find(|r| r.id == rule))
        .map_or(360, |r| r.cooldown);
    res.steward
        .cooldowns
        .insert(rule, res.current_tick.saturating_add(cooldown));
    res.steward.sequence += 1;
    res.steward.last_rule = Some(rule);
    res.steward.last_tick = res.current_tick;
    res.steward.owned.push(OwnedOrder {
        rule,
        command,
        targets,
    });
    let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::StewardAction);
    entry.cause = u32::from(rule);
    entry.value = res.steward.sequence.min(i64::MAX as u64) as i64;
    res.chronicle.push(entry);
}
