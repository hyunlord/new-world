//! Capture order provenance at mutation boundaries and copy it at job assignment.
use hecs::World;
use sim_core::colony::knowledge::TeachOrder;
use sim_core::colony::mind_job_origins::{MindDraftOrigin, MindJobOrigin, MindOrderKey};
use sim_core::colony::mind_records::MindFactSource;
use sim_core::colony::people::{DraftOrder, Drafted};
use sim_core::colony::{Bill, ColonySite, Job, JobKind};
use sim_core::components::{Agent, AgentId};
use sim_engine::SimResources;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum OrderValue {
    Present,
    Site(ColonySite),
    Bill(Bill, usize),
    Teach(TeachOrder),
    Quantities(Vec<u16>),
    Enabled(bool),
}
#[derive(Default)]
pub(crate) struct OrderSnapshot(
    BTreeMap<MindOrderKey, OrderValue>,
    BTreeMap<AgentId, (JobKind, u64)>,
    BTreeMap<AgentId, (u64, DraftOrder)>,
);

pub(crate) fn snapshot(world: &World, res: &SimResources) -> Option<OrderSnapshot> {
    res.mind.policy_source.as_ref()?;
    let mut orders = BTreeMap::new();
    for (&tile, &kind) in &res.colony.designations {
        orders.insert(
            MindOrderKey::Designation {
                tile,
                kind: kind.code(),
            },
            OrderValue::Present,
        );
    }
    for (entity, site) in world.query::<&ColonySite>().iter() {
        orders.insert(
            MindOrderKey::Site {
                entity: entity.to_bits().get(),
                centre: site.centre(),
            },
            OrderValue::Site(*site),
        );
    }
    for (&zone, value) in &res.colony.zones {
        for &tile in &value.tiles {
            orders.insert(MindOrderKey::Stockpile { zone, tile }, OrderValue::Present);
        }
    }
    for (&workshop, value) in &res.colony.workshops {
        for (position, bill) in value.bills.iter().enumerate() {
            orders.insert(
                MindOrderKey::Bill {
                    workshop,
                    bill: bill.id,
                },
                OrderValue::Bill(*bill, position),
            );
        }
    }
    for &order in &res.colony.teach_orders {
        orders.insert(
            MindOrderKey::Teach { order: order.id },
            OrderValue::Teach(order),
        );
    }
    for &animal in &res.colony.wildlife.marks {
        orders.insert(MindOrderKey::HuntAnimal { animal }, OrderValue::Present);
    }
    for &rect in &res.colony.wildlife.zones {
        orders.insert(MindOrderKey::HuntZone { rect }, OrderValue::Present);
    }
    for caravan in &res.colony.neighbours.caravans {
        for &(from, item, quantity) in &caravan.owed {
            let value = orders
                .entry(MindOrderKey::Trade {
                    caravan: caravan.id,
                    from,
                    item,
                })
                .or_insert_with(|| OrderValue::Quantities(Vec::new()));
            if let OrderValue::Quantities(quantities) = value {
                quantities.push(quantity);
            }
        }
    }
    orders.insert(
        MindOrderKey::Dress,
        OrderValue::Enabled(!res.colony.dress_off),
    );
    let jobs = world
        .query::<(&Agent, &Job)>()
        .iter()
        .map(|(_, (agent, job))| (agent.id, (job.kind, job.taken_at)))
        .collect();
    let drafted = world
        .query::<(&Agent, &Drafted)>()
        .iter()
        .map(|(_, (agent, drafted))| (agent.id, (drafted.since, drafted.order)))
        .collect();
    Some(OrderSnapshot(orders, jobs, drafted))
}

fn reconcile(
    origins: &mut BTreeMap<MindOrderKey, MindFactSource>,
    before: &OrderSnapshot,
    after: &OrderSnapshot,
    source: Option<MindFactSource>,
) {
    origins.retain(|key, _| after.0.contains_key(key));
    for (key, value) in &after.0 {
        if before.0.get(key) != Some(value) {
            origins.remove(key);
            if let Some(source) = source {
                origins.insert(*key, source);
            }
        }
    }
}

pub(crate) fn applied(
    world: &World,
    res: &mut SimResources,
    before: Option<OrderSnapshot>,
    source: Option<MindFactSource>,
) {
    let Some(before) = before else {
        return;
    };
    let Some(after) = snapshot(world, res) else {
        return;
    };
    reconcile(&mut res.mind.order_origins, &before, &after, source);
    res.mind
        .draft_origins
        .retain(|agent, _| after.2.contains_key(agent));
    for (&agent, &(since, order)) in &after.2 {
        if before.2.get(&agent) != Some(&(since, order)) {
            res.mind.draft_origins.remove(&agent);
            if let Some(source) = source {
                res.mind.draft_origins.insert(
                    agent,
                    MindDraftOrigin {
                        since,
                        order,
                        source,
                    },
                );
            }
        }
    }
    let changed: Vec<_> = world
        .query::<(&Agent, &Job)>()
        .iter()
        .filter(|(_, (agent, job))| {
            before
                .1
                .get(&agent.id)
                .is_some_and(|identity| *identity != (job.kind, job.taken_at))
        })
        .map(|(_, (agent, job))| (agent.id, *job))
        .collect();
    for (agent, job) in changed {
        assigned(world, res, agent, job);
    }
}

fn site_key(world: &World, tile: (u32, u32), footprint: bool) -> Option<MindOrderKey> {
    world
        .query::<&ColonySite>()
        .iter()
        .find_map(|(entity, site)| {
            (site.centre() == tile || (footprint && site.tiles().contains(&tile))).then_some(
                MindOrderKey::Site {
                    entity: entity.to_bits().get(),
                    centre: site.centre(),
                },
            )
        })
}

fn origin(world: &World, res: &SimResources, job: Job) -> Option<MindFactSource> {
    let designation = |tile| {
        res.colony
            .designations
            .get(&tile)
            .map(|kind| MindOrderKey::Designation {
                tile,
                kind: kind.code(),
            })
    };
    let key = match job.kind {
        JobKind::Chop { tree } => designation(tree),
        JobKind::Quarry { node } | JobKind::DigClay { node } => designation(node),
        JobKind::Dig { tile } | JobKind::Deconstruct { tile } => designation(tile),
        JobKind::Build { site } | JobKind::Deliver { site, .. } => site_key(world, site, false),
        JobKind::Clear { from, .. } => site_key(world, from, true),
        JobKind::Haul { to, .. } => res
            .colony
            .zone_of_tile
            .get(&to)
            .map(|&zone| MindOrderKey::Stockpile { zone, tile: to }),
        JobKind::Craft { workshop, bill, .. } => Some(MindOrderKey::Bill { workshop, bill }),
        JobKind::Supply { workshop, .. } => res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|shop| shop.active)
            .map(|bill| MindOrderKey::Bill { workshop, bill }),
        JobKind::Teach { order, .. } | JobKind::Learn { order, .. } => {
            Some(MindOrderKey::Teach { order })
        }
        JobKind::Equip { .. } => Some(MindOrderKey::Dress),
        JobKind::Trade { from, to } => res
            .colony
            .neighbours
            .caravans
            .iter()
            .find(|caravan| {
                caravan.spot.unwrap_or(caravan.pos) == to
                    && caravan
                        .owed
                        .iter()
                        .any(|&(tile, item, _)| tile == from && item == job.item)
            })
            .map(|caravan| MindOrderKey::Trade {
                caravan: caravan.id,
                from,
                item: job.item,
            }),
        JobKind::Hunt { animal } => {
            if res.colony.wildlife.marks.contains(&animal) {
                Some(MindOrderKey::HuntAnimal { animal })
            } else {
                let animal = res.colony.wildlife.animals.get(&animal)?;
                let mut sources = res
                    .colony
                    .wildlife
                    .zones
                    .iter()
                    .filter(|&&(x0, y0, x1, y1)| {
                        (x0..=x1).contains(&animal.pos.0) && (y0..=y1).contains(&animal.pos.1)
                    })
                    .filter_map(|&rect| {
                        res.mind
                            .order_origins
                            .get(&MindOrderKey::HuntZone { rect })
                            .copied()
                    });
                let first = sources.next()?;
                return sources.all(|source| source == first).then_some(first);
            }
        }
        JobKind::Gather { .. } => None,
    }?;
    res.mind.order_origins.get(&key).copied()
}

pub(crate) fn assigned(world: &World, res: &mut SimResources, agent: AgentId, job: Job) {
    if res.mind.policy_source.is_none() {
        return;
    }
    let source = origin(world, res, job);
    res.mind.job_origins.remove(&agent);
    if let Some(source) = source {
        res.mind.job_origins.insert(
            agent,
            MindJobOrigin {
                kind: job.kind,
                taken_at: job.taken_at,
                source,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn single(key: MindOrderKey) -> OrderSnapshot {
        OrderSnapshot(
            BTreeMap::from([(key, OrderValue::Present)]),
            BTreeMap::new(),
            BTreeMap::new(),
        )
    }
    #[test]
    fn cancelled_then_recreated_key_gets_new_command() {
        let key = MindOrderKey::Designation {
            tile: (2, 3),
            kind: 0,
        };
        let empty = OrderSnapshot::default();
        let live = single(key);
        let mut origins = BTreeMap::new();
        reconcile(
            &mut origins,
            &empty,
            &live,
            Some(MindFactSource::Command(4)),
        );
        reconcile(
            &mut origins,
            &live,
            &empty,
            Some(MindFactSource::Command(5)),
        );
        assert!(!origins.contains_key(&key));
        reconcile(
            &mut origins,
            &empty,
            &live,
            Some(MindFactSource::Command(6)),
        );
        assert_eq!(origins[&key], MindFactSource::Command(6));
    }
    #[test]
    fn unrelated_command_does_not_relabel_existing_order() {
        let key = MindOrderKey::Teach { order: 7 };
        let live = single(key);
        let mut origins = BTreeMap::from([(key, MindFactSource::Command(1))]);
        reconcile(&mut origins, &live, &live, Some(MindFactSource::Command(2)));
        assert_eq!(origins[&key], MindFactSource::Command(1));
    }
    #[test]
    fn unlogged_replacement_clears_old_source() {
        let key = MindOrderKey::Dress;
        let before = OrderSnapshot(
            BTreeMap::from([(key, OrderValue::Enabled(true))]),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        let after = OrderSnapshot(
            BTreeMap::from([(key, OrderValue::Enabled(false))]),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        let mut origins = BTreeMap::from([(key, MindFactSource::Command(1))]);
        reconcile(&mut origins, &before, &after, None);
        assert!(origins.is_empty());
    }
    #[test]
    fn command_boundary_uses_actual_log_index_with_same_tick_commands() {
        use sim_core::colony::Command;
        let mut world = World::new();
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.mind.policy_source = Some("fixture".into());
        let unrelated = Command::SetStewardDefence { on: true };
        assert!(crate::commands::apply_and_log(&mut world, &mut res, unrelated, 10).accepted);
        let stockpile = Command::AddStockpile {
            x0: 2,
            y0: 2,
            x1: 2,
            y1: 2,
        };
        assert!(crate::commands::apply_and_log(&mut world, &mut res, stockpile, 10).accepted);
        let zone = res.colony.zone_of_tile[&(2, 2)];
        let key = MindOrderKey::Stockpile { zone, tile: (2, 2) };
        assert_eq!(res.mind.order_origins[&key], MindFactSource::Command(1));
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::SetStewardDefence { on: false },
                10
            )
            .accepted
        );
        assert_eq!(res.mind.order_origins[&key], MindFactSource::Command(1));
    }
    #[test]
    fn pre_switch_command_does_not_create_provenance_state() {
        let mut world = World::new();
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        let command = sim_core::colony::Command::AddStockpile {
            x0: 2,
            y0: 2,
            x1: 2,
            y1: 2,
        };
        assert!(crate::commands::apply_and_log(&mut world, &mut res, command, 10).accepted);
        assert!(res.mind.order_origins.is_empty());
        assert!(res.mind.job_origins.is_empty());
    }
    #[test]
    fn direct_draft_mutations_capture_only_the_actual_command() {
        use sim_core::colony::Command;
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, sim_core::components::AgentState::Idle));
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.rules.colony.as_mut().expect("colony").wildlife =
            Some(sim_core::rules::WildlifeRule::M5);
        res.mind.policy_source = Some("fixture".into());
        res.current_tick = 10;
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::SetDrafted {
                    agent: 7,
                    drafted: true
                },
                10
            )
            .accepted
        );
        assert_eq!(
            res.mind.draft_origins[&7].source,
            MindFactSource::Command(0)
        );
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::DraftMove {
                    agent: 7,
                    x: 2,
                    y: 3
                },
                10
            )
            .accepted
        );
        assert_eq!(
            res.mind.draft_origins[&7].source,
            MindFactSource::Command(1)
        );
        assert_eq!(res.mind.draft_origins[&7].order, DraftOrder::MoveTo((2, 3)));
        world.get::<&mut Drafted>(entity).expect("drafted").arming = Some((1, 1));
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::SetStewardDefence { on: true },
                10
            )
            .accepted
        );
        assert_eq!(
            res.mind.draft_origins[&7].source,
            MindFactSource::Command(1)
        );
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::SetDrafted {
                    agent: 7,
                    drafted: false
                },
                10
            )
            .accepted
        );
        assert!(!res.mind.draft_origins.contains_key(&7));
    }
}
