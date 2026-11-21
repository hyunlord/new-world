//! Project a completed tick from existing owners. No systems, effects, draws or cache writes.

mod job;

use std::collections::{BTreeMap, BTreeSet};

use hecs::{Entity, World};
use sim_core::colony::food::FoodOwner;
use sim_core::colony::people::{DraftOrder, Drafted, Fleeing};
use sim_core::colony::visible_action::{
    CarriedItemView, Phase, VisibleActionKind as Action, VisibleActionRow, VisibleTarget as Target,
    VisibleTargetKind as Kind,
};
use sim_core::colony::wildlife::Fighter;
use sim_core::colony::{Carry, Job, Tool};
use sim_core::components::{
    Activity, ActivityKind, Agent, AgentId, AgentState, BuildingKind, ConstructionSite, Founding,
    Inventory, PorterStage, PorterTrip, Position, SeekTarget, SettlementMigrant, TargetKind,
};
use sim_engine::SimResources;

use super::{items::chebyshev, pot_supply};
use crate::runtime::mind::{self, context::TargetCandidate, ActionTarget};

type Tile = (u32, u32);

#[derive(Clone, Copy)]
struct Person {
    id: AgentId,
    entity: Entity,
    tile: Tile,
    state: AgentState,
    seek: Option<Tile>,
}

struct Projection<'a> {
    world: &'a World,
    res: &'a SimResources,
    people: BTreeMap<AgentId, Person>,
    pot_sources: pot_supply::SourceIndex,
    open_sites: BTreeSet<Tile>,
}

/// Collect current visible actions in AgentId order. Every field comes from
/// committed movement and lifecycle state at the read boundary.
pub fn visible_action_rows(world: &World, res: &SimResources) -> Vec<VisibleActionRow> {
    let people = world
        .query::<(&Agent, &Position, Option<&AgentState>, Option<&SeekTarget>)>()
        .iter()
        .map(|(entity, (agent, pos, state, seek))| {
            (
                agent.id,
                Person {
                    id: agent.id,
                    entity,
                    tile: (pos.x, pos.y),
                    state: state.copied().unwrap_or_default(),
                    seek: seek.map(|target| target.tile),
                },
            )
        })
        .collect();
    let projection = Projection {
        world,
        res,
        people,
        pot_sources: pot_supply::SourceIndex::default(),
        open_sites: world
            .query::<&ConstructionSite>()
            .iter()
            .filter(|(_, site)| !site.is_complete())
            .map(|(_, site)| (site.position.x, site.position.y))
            .collect(),
    };
    // Bedrolls are exclusive. Legacy huts/shelters can sleep multiple people at
    // one tile; only one pose can use the fixture's anchor.
    let mut displayed_beds = BTreeSet::new();
    projection
        .people
        .values()
        .map(|&person| {
            let mut row = projection.row(person);
            if let Some(target) = &mut row.target {
                if target.kind == Kind::Bed && !displayed_beds.insert(person.tile) {
                    *target = Target::at(Kind::Tile, person.tile);
                }
            }
            row
        })
        .collect()
}

impl Projection<'_> {
    fn get<T: Copy + hecs::Component>(&self, person: Person) -> Option<T> {
        self.world.get::<&T>(person.entity).ok().map(|value| *value)
    }

    fn row(&self, person: Person) -> VisibleActionRow {
        let held = self
            .get::<Carry>(person)
            .filter(|load| load.count > 0)
            .map(|load| CarriedItemView::of(load.spec(), load.count));
        let mut row = VisibleActionRow {
            agent_id: person.id,
            action: Action::Idle,
            phase: Phase::Idle,
            goal: None,
            target: None,
            partner: None,
            held,
            tool: self.get::<Tool>(person),
            recipe: None,
            started_at_tick: None,
            until_tick: None,
        };
        if let Some(flight) = self.get::<Fleeing>(person) {
            row.action = Action::Flee;
            row.goal = person.seek.filter(|&tile| tile == flight.to);
            if row.goal.is_some() {
                row.phase = Phase::Approach;
            }
            row.target = Some(Target::at(Kind::Tile, flight.to));
            row.started_at_tick = Some(flight.since);
            row.until_tick = Some(flight.until);
        } else if let Some(draft) = self.get::<Drafted>(person) {
            self.draft(person, draft, &mut row);
        } else if let Some(trip) = self.get::<PorterTrip>(person) {
            let to = match trip.stage {
                PorterStage::Outbound => trip.to,
                PorterStage::Return => trip.from,
            };
            row.goal = person.seek;
            row.target = self.res.settlements.get(&to).map(|settlement| {
                Target::identified(
                    Kind::SettlementStore,
                    settlement.formation_tile,
                    u64::from(to),
                )
            });
            row.held = held.or_else(|| {
                let resource = trip.kind.resource();
                let inventory = self.world.get::<&Inventory>(person.entity).ok()?;
                CarriedItemView::from_legacy_resource(resource, inventory.get(resource))
            });
            row.phase = if row.held.is_some() {
                Phase::Transport
            } else {
                Phase::Approach
            };
            row.action = Action::Carry;
            row.started_at_tick = Some(trip.depart_tick);
        } else if let Some(job) = self
            .get::<Job>(person)
            .filter(|job| self.owns_job(person, job))
        {
            self.job(person, job, &mut row);
        } else {
            self.fsm(person, &mut row);
        }
        row
    }

    fn draft(&self, person: Person, draft: Drafted, row: &mut VisibleActionRow) {
        if let Some(tile) = draft
            .arming
            .filter(|tile| self.res.colony.ground_items.contains_key(tile))
        {
            row.action = Action::Equip;
            row.phase = Phase::Approach;
            row.goal = Some(tile);
            row.target = Some(Target::at(Kind::Stack, tile));
            return;
        }
        match draft.order {
            DraftOrder::Hold => {}
            DraftOrder::MoveTo(tile) if !self.res.nav.arrived(person.tile, tile) => {
                row.action = Action::DraftMove;
                row.phase = Phase::Approach;
                row.goal = Some(tile);
                row.target = Some(Target::at(Kind::Tile, tile));
            }
            DraftOrder::MoveTo(_) => {}
            DraftOrder::Attack(fighter) => {
                row.target = match fighter {
                    Fighter::Person(id) if id != person.id => {
                        self.people.get(&id).map(agent_target)
                    }
                    Fighter::Person(_) => None,
                    Fighter::Animal(id) => self.animal(id),
                };
                if let Some(target) = row.target {
                    let me = Fighter::Person(person.id);
                    row.action = Action::Strike;
                    if self.res.colony.fights.contains_key(&(me, fighter))
                        && target
                            .tile
                            .is_some_and(|tile| chebyshev(tile, person.tile) <= 1)
                    {
                        row.phase = Phase::Perform;
                    } else {
                        row.phase = Phase::Approach;
                        row.goal = target.tile;
                    }
                }
            }
        }
    }

    fn fsm(&self, person: Person, row: &mut VisibleActionRow) {
        let (kind, phase, tile) = match person.state {
            AgentState::Idle => {
                self.inventory_haul(person, row);
                return;
            }
            AgentState::Seeking { target } => (target, Phase::Approach, person.seek),
            AgentState::Consuming { target } => (target, Phase::Perform, Some(person.tile)),
        };
        let Some(tile) = tile else { return };
        let res = self.res;
        let (action, target) = match kind {
            TargetKind::Food => (Action::Eat, self.food_target(person, tile)),
            TargetKind::Water => (
                Action::Drink,
                res.water_tiles
                    .get(&tile)
                    .filter(|&&amount| amount > 0)
                    .map(|_| Target::at(Kind::Water, tile)),
            ),
            TargetKind::Sleep => {
                let bed = phase == Phase::Perform
                    && res.bed_of.get(&person.id) == Some(&tile)
                    && (res.colony.beds.contains_key(&tile)
                        || matches!(
                            res.building_kinds.get(&tile),
                            Some(BuildingKind::Shelter | BuildingKind::Hut)
                        ));
                (
                    Action::Sleep,
                    Some(Target::at(if bed { Kind::Bed } else { Kind::Tile }, tile)),
                )
            }
            TargetKind::ConstructionSite
                if res.rules.colony.is_none() && self.open_sites.contains(&tile) =>
            {
                (Action::Build, Some(self.site(tile)))
            }
            TargetKind::Agent(id) if id != person.id => {
                if phase == Phase::Approach
                    && (self.get::<Founding>(person).is_some()
                        || self.get::<SettlementMigrant>(person).is_some())
                {
                    row.goal = Some(tile);
                    row.phase = Phase::Approach;
                    return;
                }
                let Some(peer) = self.people.get(&id) else {
                    return;
                };
                let reciprocal = peer.state
                    == (AgentState::Consuming {
                        target: TargetKind::Agent(person.id),
                    })
                    && peer.tile == person.tile;
                if self.get::<PorterTrip>(*peer).is_some()
                    || self.get::<Drafted>(*peer).is_some()
                    || self.get::<Fleeing>(*peer).is_some()
                    || (phase == Phase::Perform && !reciprocal)
                {
                    return;
                }
                let committed = mind::visible_action_target(
                    res,
                    person.id,
                    person.state,
                    None,
                    TargetCandidate {
                        tile: peer.tile,
                        agent: Some(id),
                        food: None,
                    },
                );
                let active_chase = if res.mind.policy_source.is_some() {
                    matches!(committed, Some(ActionTarget::Agent { agent, .. }) if agent == id)
                } else {
                    res.social_chase_since.contains_key(&person.id)
                };
                if !reciprocal && !active_chase {
                    return;
                }
                row.partner = Some(id);
                (Action::Talk, Some(agent_target(peer)))
            }
            TargetKind::GatherFood => (
                Action::Harvest,
                res.food_tiles
                    .get(&tile)
                    .filter(|&&amount| amount > 0)
                    .map(|_| Target::at(Kind::Plant, tile)),
            ),
            TargetKind::GatherWood => (
                Action::Carry,
                res.wood_tiles
                    .get(&tile)
                    .filter(|&&amount| amount > 0)
                    .map(|_| Target::at(Kind::Stack, tile)),
            ),
            TargetKind::Activity => {
                let Some(activity) = self.get::<Activity>(person) else {
                    return;
                };
                // Children are allowed to play within one tile of the reserved place.
                let at_place = activity.place == tile
                    || (phase == Phase::Perform
                        && activity.kind == ActivityKind::Play
                        && chebyshev(activity.place, tile) <= 1);
                if !at_place
                    || (phase == Phase::Perform
                        && (activity.arrived_tick.is_none()
                            || activity
                                .until_tick
                                // SIMFIX1 (F8): reads come after a tick ran (`current_tick` is the
                                // next tick); the mind ends the Activity only when that tick runs.
                                .is_some_and(|end| end < res.current_tick)))
                {
                    return;
                }
                let action = match activity.kind {
                    ActivityKind::TendFire => Action::TendFire,
                    ActivityKind::Craft | ActivityKind::Repair => Action::RoutineHandwork,
                    ActivityKind::HearthCircle | ActivityKind::SitByFire => Action::Sit,
                    ActivityKind::Explore => Action::Explore,
                    ActivityKind::Play => Action::Play,
                };
                let target = self.at_tile(activity.place);
                if phase == Phase::Perform {
                    row.started_at_tick = activity.arrived_tick;
                    row.until_tick = activity.until_tick;
                }
                (action, Some(target))
            }
            TargetKind::Agent(id) if id == person.id && phase == Phase::Approach => {
                row.goal = Some(tile);
                row.phase = Phase::Approach;
                return;
            }
            TargetKind::ConstructionSite | TargetKind::Agent(_) | TargetKind::Job => return,
        };
        if target.is_some() {
            row.action = action;
            row.phase = phase;
            row.target = target;
            row.goal = (phase == Phase::Approach).then_some(tile);
        }
    }

    fn inventory_haul(&self, person: Person, row: &mut VisibleActionRow) {
        if !self.res.haulers.contains(&person.id) {
            return;
        }
        let Ok(inv) = self.world.get::<&Inventory>(person.entity) else {
            return;
        };
        let Some(load) = inv
            .items
            .iter()
            .find_map(|(&kind, &count)| CarriedItemView::from_legacy_resource(kind, count))
        else {
            return;
        };
        let Some((&sid, home)) = self
            .res
            .settlements
            .iter()
            .filter(|(_, settlement)| settlement.member_agents.contains(&person.id))
            .min_by_key(|(&id, _)| id)
        else {
            return;
        };
        row.action = Action::Carry;
        row.phase = Phase::Transport;
        row.goal = Some(home.formation_tile);
        row.target = Some(Target::identified(
            Kind::SettlementStore,
            home.formation_tile,
            u64::from(sid),
        ));
        row.held = row.held.or(Some(load));
    }

    fn food_target(&self, person: Person, tile: Tile) -> Option<Target> {
        let res = self.res;
        if res.rules.colony.is_none() {
            return res
                .food_tiles
                .get(&tile)
                .filter(|&&amount| amount > 0)
                .map(|_| Target::at(Kind::FoodStore, tile));
        }
        let owner = res.food.active_targets.get(&person.id).copied()?;
        let tile = crate::runtime::food::items::edible_tile(
            self.world,
            res,
            person.entity,
            person.id,
            person.tile,
            owner,
        )?;
        if res
            .nav
            .preview_reachable_tiles(person.tile, [tile])
            .is_empty()
        {
            return None;
        }
        if matches!(person.state, AgentState::Consuming { .. }) && tile != person.tile {
            return None;
        }
        Some(self.food_owner(owner, tile))
    }

    fn food_owner(&self, owner: FoodOwner, tile: Tile) -> Target {
        match owner {
            FoodOwner::Ground(_) => Target::at(Kind::Stack, tile),
            FoodOwner::Pot { id, .. } => Target::identified(Kind::Pot, tile, u64::from(id)),
            FoodOwner::Carried(id) => Target::identified(Kind::PersonalFood, tile, id),
            FoodOwner::Shelf { workshop, .. } => {
                Target::identified(Kind::Workshop, tile, u64::from(workshop))
            }
        }
    }

    fn at_tile(&self, tile: Tile) -> Target {
        if self.res.building_kinds.get(&tile) == Some(&BuildingKind::Hearth) {
            Target::at(Kind::Hearth, tile)
        } else {
            Target::at(Kind::Tile, tile)
        }
    }

    fn animal(&self, id: u32) -> Option<Target> {
        let animal = self
            .res
            .colony
            .wildlife
            .animals
            .get(&id)
            .filter(|animal| animal.hp > 0)?;
        Some(Target::identified(
            Kind::WildAnimal,
            animal.pos,
            u64::from(id),
        ))
    }

    fn site(&self, tile: Tile) -> Target {
        site_target(self.res, tile)
    }
}

pub(crate) fn site_target(res: &SimResources, tile: Tile) -> Target {
    use sim_core::colony::building_site::PiecePhase;
    let id = res.colony.buildings.sites.values().find_map(|site| {
        site.pieces
            .iter()
            .any(|piece| {
                piece.centre() == tile
                    && matches!(piece.phase, PiecePhase::Active | PiecePhase::Replacing)
            })
            .then_some(site.id)
    });
    id.map_or_else(
        || Target::at(Kind::Site, tile),
        |id| Target::identified(Kind::Site, tile, u64::from(id)),
    )
}

fn agent_target(person: &Person) -> Target {
    Target::identified(Kind::Agent, person.tile, person.id)
}
