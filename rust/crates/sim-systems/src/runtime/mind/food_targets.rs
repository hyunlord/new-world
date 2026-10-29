//! Factual food capability filters; authored nodes decide preference between them.
use super::context::{PersonSnapshot, TargetCandidate};
use crate::runtime::food::{self, choice::FoodChoiceIndex};
use sim_core::colony::content::FoodDef;
use sim_core::colony::food::FoodOwner;
use sim_engine::SimResources;
use std::cmp::Reverse;

/// Data-independent capability classes exposed to authored target bindings.
#[derive(Clone, Copy)]
pub(super) enum FoodFilter {
    Any,
    Cooked,
    Novel,
    CookedNovel,
}
impl FoodFilter {
    pub(super) fn binding(name: &str) -> Option<Self> {
        match name {
            "food" => Some(Self::Any),
            "food_cooked" => Some(Self::Cooked),
            "food_novel" => Some(Self::Novel),
            "food_cooked_novel" => Some(Self::CookedNovel),
            _ => None,
        }
    }
    fn accepts(self, food: &FoodDef, person: &PersonSnapshot, res: &SimResources) -> bool {
        let novel = || {
            food.groups.iter().any(|group| {
                res.food
                    .meals
                    .get(&person.agent)
                    .and_then(|history| history.last_group_tick.get(group))
                    .is_none_or(|tick| {
                        res.current_tick.saturating_sub(*tick)
                            >= res.rules.calendar.days_to_ticks(3)
                    })
            })
        };
        food.raw_edible
            && match self {
                Self::Any => true,
                Self::Cooked => food.cooked,
                Self::Novel => novel(),
                Self::CookedNovel => food.cooked && novel(),
            }
    }
}

pub(super) fn candidate(
    index: &FoodChoiceIndex,
    person: &PersonSnapshot,
    res: &SimResources,
    filter: FoodFilter,
) -> Option<TargetCandidate> {
    // Capture fixes public owners for this decision phase; live eligibility below
    // rejects depletion. Job-release drops occur after all decisions and replays.
    let origin = (person.position.x, person.position.y);
    let mut best = None;
    let mut consider = |kind: sim_core::colony::ItemKind, tile: (u32, u32), owner: FoodOwner| {
        let Some(food) = &kind.definition().food else {
            return;
        };
        if !filter.accepts(food, person, res) {
            return;
        }
        let distance = origin.0.abs_diff(tile.0).max(origin.1.abs_diff(tile.1));
        // Nearest wins within the authored capability; item preference is a
        // data-defined tie breaker, followed by stable tile and owner identity.
        let key = (distance, Reverse(food.preference), tile.1, tile.0, owner);
        if best.as_ref().is_none_or(|(previous, _)| key < *previous) {
            best = Some((
                key,
                TargetCandidate {
                    tile,
                    agent: None,
                    food: Some(owner),
                },
            ));
        }
    };
    for kind in index.kinds() {
        let Some(def) = &kind.definition().food else {
            continue;
        };
        if !filter.accepts(def, person, res) {
            continue;
        }
        if let Some((tile, owner)) = index.nearest(res, kind, origin, u32::MAX, |_, tile| {
            res.nav.reachable(origin, tile)
        }) {
            consider(kind, tile, owner);
        }
    }
    if let Some(carry) = person.carry {
        let owner = FoodOwner::Carried(person.agent);
        let day = food::day(res);
        if carry.count > 0
            && carry
                .kind
                .definition()
                .food
                .as_ref()
                .is_some_and(|definition| {
                    res.food.ages.get(&owner).is_some_and(|age| {
                        age.born_day <= day
                            && age.remaining(
                                day,
                                definition.lifetime_days(food::storage::class(res, owner)),
                            ) > 0
                    })
                })
        {
            consider(carry.kind, origin, owner);
        }
    }
    best.map(|(_, target)| target)
}

#[cfg(test)]
mod tests {
    use super::super::context::TickContext;
    use super::*;
    use crate::runtime::food::items::{self, FoodSource};
    use hecs::World;
    use sim_core::colony::{ItemKind, ItemSpec};
    use sim_core::components::{Agent, AgentState, Position};
    use sim_core::rules::WorldRules;

    fn scene() -> (World, SimResources) {
        let mut world = World::new();
        world.spawn((Agent { id: 1 }, Position::new(2, 2), AgentState::Idle));
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = WorldRules::colony(0);
        res.nav.install(sim_core::nav::CostMap::new(8, 8));
        (world, res)
    }

    fn create(res: &mut SimResources, name: &str) {
        let kind = ItemKind::from_name(name).unwrap();
        items::create_ground(
            res,
            (2, 2),
            ItemSpec::raw(kind, None),
            1,
            0,
            FoodSource::Harvest,
        )
        .unwrap();
    }

    fn target(context: &TickContext, res: &SimResources) -> Option<TargetCandidate> {
        candidate(
            &context.physical_food.borrow(),
            context.row(1).unwrap(),
            res,
            FoodFilter::Any,
        )
    }

    #[test]
    fn exhausted_owner_is_skipped_inside_the_same_decision_phase() {
        let (mut world, mut res) = scene();
        create(&mut res, "roots");
        let context = TickContext::capture(&world, &res).unwrap();
        assert!(target(&context, &res).is_some());
        items::take(&mut world, &mut res, FoodOwner::Ground((2, 2)), 1, 0).unwrap();
        assert!(target(&context, &res).is_none());
    }

    #[test]
    fn deferred_food_is_discovered_by_the_next_phase_capture() {
        let (world, mut res) = scene();
        let previous = TickContext::capture(&world, &res).unwrap();
        assert!(target(&previous, &res).is_none());
        // Job release creates public food only after all decisions and replays.
        create(&mut res, "berries");
        assert!(target(&previous, &res).is_none());
        res.current_tick += 1;
        let next = TickContext::capture(&world, &res).unwrap();
        assert_eq!(
            target(&next, &res).unwrap().food,
            Some(FoodOwner::Ground((2, 2)))
        );
    }
}
