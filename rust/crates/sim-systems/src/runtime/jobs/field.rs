//! Field job selection and physical prerequisites shared by assignment/execution.
use sim_core::colony::knowledge::KnowledgeMask;
pub(super) mod execute;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::cultivation::{FieldPhase, FieldWork};
use sim_core::colony::{ItemKind, Job, JobKind, JobStage, ReserveKey};
use sim_core::components::AgentId;
use sim_engine::SimResources;

pub(crate) fn tile(kind: JobKind) -> Option<(u32, u32)> {
    match kind {
        JobKind::TillField { tile }
        | JobKind::SowField { tile, .. }
        | JobKind::TendField { tile }
        | JobKind::HarvestField { tile } => Some(tile),
        _ => None,
    }
}
pub(super) fn target_exists(res: &SimResources, id: AgentId, job: &Job) -> bool {
    let Some(tile) = tile(job.kind) else {
        return false;
    };
    let Some(f) = res.cultivation.fields.get(&tile) else {
        return false;
    };
    if !crate::runtime::cultivation::uncovered_soil(res, tile)
        || !f.active
        || f.work == FieldWork::Paused
        || !res
            .settlements
            .get(&f.owner)
            .is_some_and(|s| s.member_agents.contains(&id))
    {
        return false;
    }
    match job.kind {
        JobKind::TillField { .. } => matches!(f.phase, FieldPhase::Untilled | FieldPhase::Fallow),
        JobKind::SowField { from, .. } => {
            let Some(def) = installed_or_bundled().crop(f.crop) else {
                return false;
            };
            matches!(f.phase, FieldPhase::Tilled)
                && crate::runtime::cultivation::sowing_window(res, f.crop)
                && res
                    .cultivation
                    .fertility
                    .get(&tile)
                    .copied()
                    .unwrap_or(def.initial_fertility)
                    >= def.min_fertility
                && (job.stage != JobStage::ToSource
                    || res
                        .colony
                        .ground_items
                        .get(&from)
                        .is_some_and(|s| s.kind == def.seed_item && s.count >= def.seeds_per_tile))
        }
        JobKind::TendField { .. } => {
            !f.tended
                && matches!(
                    f.phase,
                    FieldPhase::Sown { .. } | FieldPhase::Growing { .. }
                )
        }
        JobKind::HarvestField { .. } => matches!(f.phase, FieldPhase::Ripe { .. }),
        _ => false,
    }
}
/// All candidates respect the worker's home, practice, scan radius and nav region.
pub(super) fn pick(
    res: &SimResources,
    id: AgentId,
    practice: KnowledgeMask,
    region: u32,
    pos: (u32, u32),
    radius: u32,
    now: bool,
) -> Option<(JobKind, u16, ItemKind)> {
    let mut fields: Vec<_> = res
        .cultivation
        .fields
        .iter()
        .filter(|(t, f)| {
            f.active
                && crate::runtime::cultivation::uncovered_soil(res, **t)
                && f.work != FieldWork::Paused
                && (f.work == FieldWork::Now) == now
                && super::items::chebyshev(pos, **t) <= radius
                && res.nav.reaches(region, **t)
                && !res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Field(**t))
                && res
                    .colony
                    .unreachable
                    .get(&(id, **t))
                    .is_none_or(|until| *until <= res.current_tick)
                && res
                    .settlements
                    .get(&f.owner)
                    .is_some_and(|s| s.member_agents.contains(&id))
        })
        .collect();
    fields.sort_by_key(|(t, _)| (super::items::chebyshev(pos, **t), t.1, t.0));
    for (&tile, f) in fields {
        let Some(def) = installed_or_bundled().crop(f.crop) else {
            continue;
        };
        let mask = crate::runtime::cultivation::knowledge_mask(f.crop);
        if mask != 0 && practice & mask == 0 {
            continue;
        }
        let placeholder = ItemKind::role("default_placeholder");
        let kind = match f.phase {
            FieldPhase::Untilled | FieldPhase::Fallow => {
                if !now && !crate::runtime::cultivation::sowing_window(res, f.crop) {
                    continue;
                }
                if res
                    .cultivation
                    .fertility
                    .get(&tile)
                    .copied()
                    .unwrap_or(def.initial_fertility)
                    < def.min_fertility
                {
                    continue;
                }
                JobKind::TillField { tile }
            }
            FieldPhase::Tilled => {
                if !crate::runtime::cultivation::sowing_window(res, f.crop)
                    || res
                        .cultivation
                        .fertility
                        .get(&tile)
                        .copied()
                        .unwrap_or(def.initial_fertility)
                        < def.min_fertility
                {
                    continue;
                }
                if crate::runtime::cultivation::ground_seed_count(res, f.owner, f.crop)
                    < u64::from(def.seeds_per_tile)
                {
                    continue;
                }
                let source = res
                    .colony
                    .ground_items
                    .iter()
                    .filter(|(t, s)| {
                        s.kind == def.seed_item
                            && crate::runtime::cultivation::seed_belongs_to(res, f.owner, **t)
                            && s.count >= def.seeds_per_tile
                            && def.seeds_per_tile
                                <= res.rules.colony.map_or(0, |r| r.carry(def.seed_item))
                            && res.nav.reaches(region, **t)
                            && super::items::chebyshev(pos, **t) <= radius
                            && !res
                                .colony
                                .reservations
                                .contains_key(&ReserveKey::Stack(**t))
                    })
                    .min_by_key(|(t, _)| (super::items::chebyshev(pos, **t), t.1, t.0));
                if let Some((&from, _)) = source {
                    return Some((
                        JobKind::SowField { tile, from },
                        def.seeds_per_tile,
                        def.seed_item,
                    ));
                }
                continue;
            }
            FieldPhase::Sown { .. } | FieldPhase::Growing { .. } if !f.tended => {
                JobKind::TendField { tile }
            }
            FieldPhase::Ripe { .. } => JobKind::HarvestField { tile },
            _ => continue,
        };
        return Some((kind, 0, placeholder));
    }
    None
}
