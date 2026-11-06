//! Physical bed-anchored dwelling predicates and continuous deficiency hysteresis.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::people::Apparel;
use sim_core::colony::thermal::DwellingStageDef;
use sim_core::components::{Agent, Child, Position};
use sim_core::rooms::Tile;
use sim_engine::SimResources;

/// Native dwelling facts shared by gameplay, mood and presentation.
#[derive(Debug, Clone)]
pub struct DwellingView {
    /// Earned authored identity.
    pub stage: Option<u16>,
    /// Earned localized label.
    pub label_key: Option<String>,
    /// Highest currently satisfied identity.
    pub candidate: Option<u16>,
    /// First unmet physical condition key.
    pub reason: &'static str,
    /// Continuous deficiency deadline.
    pub deficiency_deadline: Option<u64>,
    /// Current physical roof.
    pub roofed: bool,
    /// Actual packed floor coverage.
    pub floor_coverage_permille: u16,
    /// Current fully paid positive primary hearth.
    pub paid_heat: bool,
    /// At least one actual occupant, all currently warm.
    pub warm_occupants: bool,
    /// Stored reachable food satisfies the food-requiring stage.
    pub nearby_food: bool,
    /// One authored mood contribution for this bed's resident.
    pub mood_permille: i32,
}

/// Evaluate actual geometry/people/storage once in native code; never mutates state.
pub fn dwelling_view(world: &World, res: &SimResources, bed: Tile) -> Option<DwellingView> {
    let thermal = res.thermal.as_ref()?;
    if !res.colony.beds.contains_key(&bed) {
        return None;
    }
    let room = crate::runtime::rooms::room_at(res, bed);
    let roofed = room.is_some_and(|r| r.roofed);
    // Permanent footprints cannot receive floors through ordinary commands, even if
    // a floor predates their construction. Exclude them in either construction order.
    let eligible = |tile: &Tile| {
        !res.building_kinds.contains_key(tile)
            && !matches!(
                res.colony.occupied.get(tile),
                Some(sim_core::colony::Occupant::Hut(_) | sim_core::colony::Occupant::Workshop(_))
            )
            && !res
                .colony
                .workshops
                .values()
                .any(|shop| shop.work_tile() == *tile)
    };
    let area = room.map_or(0, |r| r.tiles.iter().filter(|tile| eligible(tile)).count());
    let floors = room.map_or(0, |r| {
        r.tiles
            .iter()
            .filter(|tile| eligible(tile))
            .filter(|&&(x, y)| res.tile_grid.floor_material[res.tile_grid.idx(x, y)].is_some())
            .count()
    });
    let coverage = room.map_or(0, |_| {
        u16::try_from(floors * 1000 / area.max(1)).unwrap_or(0)
    });
    let beds = room.map_or(0, |r| crate::runtime::rooms::beds_in(res, r));
    let paid = room.is_some_and(|r| super::room_paid(res, r.id));
    let mut occupants = 0_u64;
    let mut adults = 0_u64;
    let mut warm = true;
    for (_, (agent, pos, child, apparel)) in world
        .query::<(&Agent, &Position, Option<&Child>, Option<&Apparel>)>()
        .iter()
    {
        if child.is_none()
            && res
                .bed_of
                .get(&agent.id)
                .is_some_and(|bed| room.is_some_and(|r| r.tiles.contains(bed)))
        {
            adults += 1;
        }
        if room.is_some_and(|r| r.tiles.contains(&(pos.x, pos.y))) {
            occupants += 1;
            warm &= super::felt_temperature(res, apparel, (pos.x, pos.y))
                .is_some_and(|t| t >= thermal.rules.cold_threshold_d10);
        }
    }
    warm &= occupants > 0
        && room
            .and_then(|r| thermal.rooms.get(&r.id))
            .and_then(|r| r.last_integrated_tick)
            .is_some_and(|tick| tick / 60 == res.current_tick / 60);
    let mut candidate: Option<&DwellingStageDef> = None;
    let mut reason = if roofed { "beds" } else { "roof" };
    let mut nearby_food = false;
    for stage in &thermal.rules.stages {
        let food = super::food::food_near(world, res, bed, stage, adults);
        if stage.nearby_food_adult_days > 0 {
            nearby_food |= food;
        }
        let failed = if !roofed {
            Some("roof")
        } else if beds < u32::from(stage.min_beds) {
            Some("beds")
        } else if coverage < stage.floor_coverage_permille
            || (stage.floor_coverage_permille > 0 && area == 0)
        {
            Some("floor")
        } else if stage.requires_paid_heat && !paid {
            Some("fuel")
        } else if stage.requires_warm_occupants && !warm {
            Some("warmth")
        } else if !food {
            Some("food")
        } else {
            None
        };
        if failed.is_none() && candidate.is_none_or(|old| stage.rank > old.rank) {
            candidate = Some(stage);
            reason = "ready";
        } else if candidate.is_some_and(|old| stage.rank > old.rank) {
            reason = failed.unwrap_or("ready");
        }
    }
    let saved = thermal.dwellings.get(&bed);
    let earned = saved
        .and_then(|s| s.stage)
        .and_then(|id| thermal.rules.stages.iter().find(|s| s.id == id));
    Some(DwellingView {
        stage: earned.map(|s| s.id),
        label_key: earned.map(|s| s.label_key.clone()),
        candidate: candidate.map(|s| s.id),
        reason,
        deficiency_deadline: saved
            .and_then(|s| s.deficient_since_tick)
            .zip(earned)
            .map(|(tick, stage)| tick.saturating_add(stage.downgrade_delay_ticks)),
        roofed,
        floor_coverage_permille: coverage,
        paid_heat: paid,
        warm_occupants: warm,
        nearby_food,
        mood_permille: if roofed {
            earned.map_or(0, |s| s.mood_permille)
        } else {
            0
        },
    })
}

pub(super) fn update(world: &World, res: &mut SimResources) {
    let beds: Vec<_> = res.colony.beds.keys().copied().collect();
    let views: Vec<_> = beds
        .iter()
        .filter_map(|&bed| dwelling_view(world, res, bed).map(|v| (bed, v)))
        .collect();
    let Some(thermal) = res.thermal.as_mut() else {
        return;
    };
    thermal
        .dwellings
        .retain(|tile, _| res.colony.beds.contains_key(tile));
    let mut improved = Vec::new();
    for (bed, view) in views {
        let rank = |id: Option<u16>| {
            id.and_then(|id| thermal.rules.stages.iter().find(|s| s.id == id))
                .map_or(0, |s| u32::from(s.rank) + 1)
        };
        let history = thermal.dwellings.entry(bed).or_default();
        if rank(view.candidate) >= rank(history.stage) {
            if rank(view.candidate) > rank(history.stage) {
                improved.push((bed, view.candidate));
            }
            history.stage = view.candidate;
            history.deficient_since_tick = None;
        } else {
            let since = *history.deficient_since_tick.get_or_insert(res.current_tick);
            let delay = thermal
                .rules
                .stages
                .iter()
                .find(|s| Some(s.id) == history.stage)
                .map_or(0, |s| s.downgrade_delay_ticks);
            if res.current_tick.saturating_sub(since) >= delay {
                history.stage = view.candidate;
                history.deficient_since_tick = None;
            }
        }
    }
    for (bed, stage) in improved {
        let Some(stage) = stage else {
            continue;
        };
        let room = res.colony.room_of_tile.get(&bed).copied().unwrap_or(0);
        let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::DwellingImproved);
        entry.cause = u32::from(stage);
        entry.value = i64::from(room);
        crate::runtime::chronicle_log::record(world, res, entry);
    }
}
