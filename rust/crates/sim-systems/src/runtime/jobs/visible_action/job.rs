//! Job stages, physical target namespaces and paired apprenticeship.

use super::*;
use crate::runtime::jobs::{execute, livestock};
use crate::runtime::livestock::{has_capture_slot, pen_layout};
use sim_core::colony::{JobKind, JobStage};

impl Projection<'_> {
    pub(super) fn owns_job(&self, person: Person, job: &Job) -> bool {
        let res = self.res;
        let space = matches!(job.kind, JobKind::Build { site } if res.colony.space_sites.contains_key(&site));
        if person.state != execute::expected_state(job.kind, job.stage, space)
            || job
                .keys()
                .iter()
                .any(|key| res.colony.reservations.get(key) != Some(&person.id))
        {
            return false;
        }
        if let JobKind::Capture { animal, pen } = job.kind {
            // Age admission uses keyed demographics in the executor. The read
            // only checks the surviving order, physical target and reservation.
            return res.livestock.capture_orders.contains_key(&animal)
                && self.animal(animal).is_some()
                && pen_layout(res, pen)
                    .is_ok_and(|pen| has_capture_slot(res, &pen, Some(person.id)));
        }
        if let (JobKind::Gather { spot, .. }, JobStage::Working { .. }) = (job.kind, job.stage) {
            return res.wood_tiles.get(&spot).is_some_and(|&count| count > 0);
        }
        if let JobKind::Build { site } = job.kind {
            if !self.open_sites.contains(&site) {
                return false;
            }
        }
        execute::target_exists(
            self.world,
            res,
            person.id,
            person.entity,
            job,
            &self.pot_sources,
            |from, to| !res.nav.preview_reachable_tiles(from, [to]).is_empty(),
        )
    }

    pub(super) fn job(&self, person: Person, job: Job, row: &mut VisibleActionRow) {
        if matches!(job.kind, JobKind::Teach { .. } | JobKind::Learn { .. }) {
            self.lesson(person, job, row);
            return;
        }
        let Some(target) = self.job_target(job) else {
            return;
        };
        let phase = match job.stage {
            JobStage::ToSource => Phase::Approach,
            JobStage::Working { .. } => {
                if !self.still_at_work(person.tile, job) {
                    Phase::Approach
                } else {
                    Phase::Perform
                }
            }
            JobStage::ToDest if row.held.is_some_and(|held| held.item == job.item) => {
                Phase::Transport
            }
            JobStage::ToDest => return,
        };
        if matches!(phase, Phase::Approach | Phase::Transport) {
            let goal = match job.stage {
                JobStage::ToDest => job.kind.dest(),
                JobStage::ToSource | JobStage::Working { .. } => job
                    .fetch
                    .or_else(|| livestock::target_tile(self.res, &job))
                    .or(target.tile),
            };
            if goal.is_some_and(|goal| self.res.nav.blocked.get(&person.id) == Some(&goal)) {
                return;
            }
            row.goal = goal;
        }
        row.action = job_action(job.kind);
        row.phase = phase;
        row.target = Some(target);
        if let JobKind::Craft { workshop, bill, .. } = job.kind {
            row.recipe = self
                .res
                .colony
                .workshops
                .get(&workshop)
                .and_then(|shop| shop.bill(bill))
                .map(|bill| bill.recipe);
        }
    }

    fn still_at_work(&self, pos: Tile, job: Job) -> bool {
        match job.kind {
            JobKind::Hunt { animal } => self
                .animal(animal)
                .and_then(|target| target.tile)
                .zip(self.res.rules.colony.and_then(|colony| colony.wildlife))
                .is_some_and(|(tile, rule)| chebyshev(pos, tile) <= rule.hunt_reach),
            JobKind::Capture { animal, .. } => self
                .animal(animal)
                .and_then(|target| target.tile)
                .is_some_and(|tile| chebyshev(pos, tile) <= 1),
            JobKind::Chop { .. }
            | JobKind::Quarry { .. }
            | JobKind::Gather { .. }
            | JobKind::Harvest { .. }
            | JobKind::Dig { .. }
            | JobKind::DigClay { .. }
            | JobKind::Mine { .. }
            | JobKind::Deconstruct { .. } => chebyshev(pos, job.kind.source()) <= 1,
            JobKind::SowField { tile, .. } => self.res.nav.arrived(pos, tile),
            _ => self.res.nav.arrived(pos, job.kind.source()),
        }
    }

    fn job_target(&self, job: Job) -> Option<Target> {
        if let Some(fetch) = job.fetch.filter(|_| job.stage == JobStage::ToSource) {
            return self
                .res
                .colony
                .ground_items
                .get(&fetch)
                .filter(|stack| stack.count > 0)
                .map(|_| Target::at(Kind::Stack, fetch));
        }
        if job.stage == JobStage::ToDest {
            let dest = job.kind.dest()?;
            return Some(match job.kind {
                JobKind::Deliver { .. } => self.site(dest),
                JobKind::Supply { workshop, .. } => {
                    Target::identified(Kind::Workshop, dest, u64::from(workshop))
                }
                JobKind::SowField { .. } => Target::at(Kind::Field, dest),
                JobKind::Trade { .. } => {
                    let caravan = self.res.colony.neighbours.trading_caravan()?;
                    Target::identified(Kind::Trade, dest, u64::from(caravan.id))
                }
                _ => self.res.colony.zone_of_tile.get(&dest).map_or_else(
                    || self.at_tile(dest),
                    |&zone| Target::identified(Kind::Stockpile, dest, u64::from(zone)),
                ),
            });
        }
        let source = job.kind.source();
        Some(match job.kind {
            JobKind::Chop { .. } => Target::at(Kind::Tree, source),
            JobKind::Build { .. } | JobKind::Deconstruct { .. } => self.site(source),
            JobKind::Quarry { .. }
            | JobKind::Dig { .. }
            | JobKind::DigClay { .. }
            | JobKind::Mine { .. } => Target::at(Kind::Deposit, source),
            JobKind::Craft { workshop, .. } => {
                Target::identified(Kind::Workshop, source, u64::from(workshop))
            }
            JobKind::Supply { .. } => {
                let (owner, _) = pot_supply::source(self.res, source, job.item)?;
                self.food_owner(owner, source)
            }
            JobKind::Hunt { animal } | JobKind::Capture { animal, .. } => self.animal(animal)?,
            JobKind::Milk { animal, .. } | JobKind::Slaughter { animal, .. } => {
                // Domesticated animals have a pen association, but no per-tick
                // position. The admitted gate is the physical work point.
                Target::identified(Kind::Livestock, source, u64::from(animal))
            }
            JobKind::Harvest { .. } => Target::at(Kind::Plant, source),
            JobKind::TillField { .. }
            | JobKind::TendField { .. }
            | JobKind::HarvestField { .. } => Target::at(Kind::Field, source),
            JobKind::SowField { tile, .. } if matches!(job.stage, JobStage::Working { .. }) => {
                Target::at(Kind::Field, tile)
            }
            JobKind::Haul { .. }
            | JobKind::Deliver { .. }
            | JobKind::Gather { .. }
            | JobKind::Clear { .. }
            | JobKind::Equip { .. }
            | JobKind::Trade { .. }
            | JobKind::SowField { .. } => Target::at(Kind::Stack, source),
            JobKind::Teach { .. } | JobKind::Learn { .. } => return None,
        })
    }

    fn lesson(&self, person: Person, job: Job, row: &mut VisibleActionRow) {
        let (id, at, teaching) = match job.kind {
            JobKind::Teach { order, at } => (order, at, true),
            JobKind::Learn { order, at } => (order, at, false),
            _ => return,
        };
        let Some(order) = self
            .res
            .colony
            .teach_orders
            .iter()
            .find(|order| order.id == id)
        else {
            return;
        };
        let (actor, partner) = if teaching {
            (order.teacher, order.learner)
        } else {
            (order.learner, order.teacher)
        };
        if actor != person.id || actor == partner {
            return;
        }
        let Some(&peer) = self.people.get(&partner) else {
            return;
        };
        let partner_job = self.get::<Job>(peer).filter(|other| {
            let expected = if teaching {
                JobKind::Learn { order: id, at }
            } else {
                JobKind::Teach { order: id, at }
            };
            other.kind == expected
                && self.owns_job(peer, other)
                && self.get::<Drafted>(peer).is_none()
                && self.get::<Fleeing>(peer).is_none()
                && self.get::<PorterTrip>(peer).is_none()
        });
        row.action = job_action(job.kind);
        row.target = Some(self.at_tile(at));
        row.partner = partner_job.map(|_| partner);
        if matches!(job.stage, JobStage::Working { .. })
            && chebyshev(person.tile, at) <= 1
            && chebyshev(peer.tile, at) <= 1
            && partner_job.is_some_and(|other| matches!(other.stage, JobStage::Working { .. }))
        {
            row.phase = Phase::Perform;
        } else if chebyshev(person.tile, at) > 1 {
            row.phase = Phase::Approach;
            row.goal = Some(at);
        }
    }
}

fn job_action(kind: JobKind) -> Action {
    match kind {
        JobKind::Chop { .. } => Action::Chop,
        JobKind::Haul { .. }
        | JobKind::Deliver { .. }
        | JobKind::Supply { .. }
        | JobKind::Gather { .. }
        | JobKind::Clear { .. }
        | JobKind::Trade { .. } => Action::Carry,
        JobKind::Build { .. } => Action::Build,
        JobKind::Quarry { .. } | JobKind::Dig { .. } | JobKind::DigClay { .. } => Action::Dig,
        JobKind::Deconstruct { .. } => Action::Dismantle,
        JobKind::Mine { .. } => Action::Mine,
        JobKind::Craft { .. } => Action::Craft,
        JobKind::Teach { .. } => Action::Teach,
        JobKind::Learn { .. } => Action::Learn,
        JobKind::Hunt { .. } => Action::Hunt,
        JobKind::Equip { .. } => Action::Equip,
        JobKind::Harvest { .. } | JobKind::HarvestField { .. } => Action::Harvest,
        JobKind::TillField { .. } => Action::Till,
        JobKind::SowField { .. } => Action::Sow,
        JobKind::TendField { .. } => Action::TendCrop,
        JobKind::Capture { .. } => Action::Capture,
        JobKind::Milk { .. } => Action::Milk,
        JobKind::Slaughter { .. } => Action::Slaughter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_activity_projects_every_job_kind() {
        let at = (2, 2);
        let from = (1, 1);
        let cases = [
            (JobKind::Chop { tree: at }, Action::Chop),
            (JobKind::Haul { from, to: at }, Action::Carry),
            (JobKind::Deliver { from, site: at }, Action::Carry),
            (JobKind::Build { site: at }, Action::Build),
            (JobKind::Quarry { node: at }, Action::Dig),
            (
                JobKind::Supply {
                    from,
                    workshop: 1,
                    to: at,
                },
                Action::Carry,
            ),
            (
                JobKind::Craft {
                    workshop: 1,
                    bill: 2,
                    at,
                },
                Action::Craft,
            ),
            (JobKind::Gather { spot: from, to: at }, Action::Carry),
            (JobKind::Dig { tile: at }, Action::Dig),
            (JobKind::Deconstruct { tile: at }, Action::Dismantle),
            (JobKind::Clear { from, to: at }, Action::Carry),
            (JobKind::DigClay { node: at }, Action::Dig),
            (JobKind::Teach { order: 1, at }, Action::Teach),
            (JobKind::Learn { order: 1, at }, Action::Learn),
            (JobKind::Hunt { animal: 1 }, Action::Hunt),
            (JobKind::Equip { from }, Action::Equip),
            (JobKind::Trade { from, to: at }, Action::Carry),
            (JobKind::Harvest { plant: at }, Action::Harvest),
            (JobKind::TillField { tile: at }, Action::Till),
            (JobKind::SowField { tile: at, from }, Action::Sow),
            (JobKind::TendField { tile: at }, Action::TendCrop),
            (JobKind::HarvestField { tile: at }, Action::Harvest),
            (JobKind::Capture { animal: 1, pen: 2 }, Action::Capture),
            (JobKind::Milk { animal: 1, at }, Action::Milk),
            (JobKind::Slaughter { animal: 1, at }, Action::Slaughter),
            (JobKind::Mine { node: at }, Action::Mine),
        ];
        assert_eq!(cases.len() + 1, JobKind::NAMES.len());
        for (i, (job, action)) in cases.into_iter().enumerate() {
            assert_eq!(usize::from(job.code()), i + 1, "{job:?}");
            assert_eq!(job_action(job), action, "{job:?}");
        }
        println!("job_kinds=26/26 projection_errors=0");
    }
}
