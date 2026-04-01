use std::collections::BTreeMap;

use hecs::World;
use sim_core::components::{Age, Behavior, Identity, Position};
use sim_core::config;
use sim_core::ids::{BandId, EntityId};
use sim_engine::{SimResources, SimSystem};

/// Warm-tier runtime system that refreshes promoted-band steering state.
#[derive(Debug, Clone)]
pub struct BandBehaviorSystem {
    priority: u32,
    tick_interval: u64,
}

impl BandBehaviorSystem {
    /// Creates a new band-behavior runtime system.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for BandBehaviorSystem {
    fn name(&self) -> &'static str {
        "band_behavior_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        refresh_band_behavior_state(world, resources);
    }
}

/// Recomputes promoted-band centers and applies band steering hints to members.
pub(crate) fn refresh_band_behavior_state(world: &mut World, resources: &SimResources) {
    let position_by_id: BTreeMap<EntityId, (f64, f64)> = world
        .query::<(&Position, Option<&Age>)>()
        .iter()
        .filter_map(|(entity, (position, age_opt))| {
            if matches!(age_opt, Some(age) if !age.alive) {
                None
            } else {
                Some((EntityId(entity.id() as u64), (position.x, position.y)))
            }
        })
        .collect();

    let mut band_centers: BTreeMap<BandId, (f64, f64)> = BTreeMap::new();
    for band in resources.band_store.all().filter(|band| band.is_promoted) {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut count = 0_u32;
        for member_id in &band.members {
            if let Some((x, y)) = position_by_id.get(member_id) {
                sum_x += *x;
                sum_y += *y;
                count = count.saturating_add(1);
            }
        }
        if count > 0 {
            band_centers.insert(band.id, (sum_x / count as f64, sum_y / count as f64));
        }
    }

    let mut query = world.query::<(&Identity, &mut Behavior, Option<&Age>)>();
    for (_, (identity, behavior, age_opt)) in &mut query {
        if matches!(age_opt, Some(age) if !age.alive) {
            behavior.band_center_x = None;
            behavior.band_center_y = None;
            behavior.outsider_separation_mult = 1.0;
            continue;
        }

        if let Some(band_id) = identity.band_id {
            if let Some((center_x, center_y)) = band_centers.get(&band_id).copied() {
                behavior.band_center_x = Some(center_x);
                behavior.band_center_y = Some(center_y);
                behavior.outsider_separation_mult = config::BAND_OUTSIDER_SEPARATION_MULT;
                continue;
            }
        }

        behavior.band_center_x = None;
        behavior.band_center_y = None;
        behavior.outsider_separation_mult = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::band::Band;
    use sim_core::components::Identity;
    use sim_core::{config::GameConfig, GameCalendar, WorldMap};

    fn resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 11);
        SimResources::new(calendar, map, 17)
    }

    #[test]
    fn band_behavior_sets_center_for_members() {
        let mut world = World::new();
        let mut resources = resources();
        let band_id = resources.band_store.allocate_id();

        let a = world.spawn((
            Position::from_f64(1.0, 1.0),
            Identity {
                band_id: Some(band_id),
                ..Identity::default()
            },
            Behavior::default(),
        ));
        let b = world.spawn((
            Position::from_f64(5.0, 1.0),
            Identity {
                band_id: Some(band_id),
                ..Identity::default()
            },
            Behavior::default(),
        ));
        let c = world.spawn((
            Position::from_f64(3.0, 4.0),
            Identity {
                band_id: Some(band_id),
                ..Identity::default()
            },
            Behavior::default(),
        ));

        resources.band_store.insert(Band {
            id: band_id,
            name: "band_1".to_string(),
            members: vec![
                EntityId(a.id() as u64),
                EntityId(b.id() as u64),
                EntityId(c.id() as u64),
            ],
            leader: None,
            provisional_since: 10,
            promoted_tick: Some(20),
            is_promoted: true,
            settlement_id: None,
        });

        refresh_band_behavior_state(&mut world, &resources);

        let behavior = world.get::<&Behavior>(a).expect("behavior");
        assert_eq!(behavior.band_center_x, Some(3.0));
        assert_eq!(behavior.band_center_y, Some(2.0));
        assert_eq!(
            behavior.outsider_separation_mult,
            config::BAND_OUTSIDER_SEPARATION_MULT
        );
    }

    #[test]
    fn band_behavior_clears_center_for_loners() {
        let mut world = World::new();
        let resources = resources();
        let entity = world.spawn((
            Position::from_f64(2.0, 2.0),
            Identity::default(),
            Behavior {
                band_center_x: Some(1.0),
                band_center_y: Some(1.0),
                outsider_separation_mult: 4.0,
                ..Behavior::default()
            },
        ));

        refresh_band_behavior_state(&mut world, &resources);

        let behavior = world.get::<&Behavior>(entity).expect("behavior");
        assert_eq!(behavior.band_center_x, None);
        assert_eq!(behavior.band_center_y, None);
        assert_eq!(behavior.outsider_separation_mult, 1.0);
    }

    #[test]
    fn band_behavior_ignores_provisional_bands() {
        let mut world = World::new();
        let mut resources = resources();
        let band_id = resources.band_store.allocate_id();

        let entity = world.spawn((
            Position::from_f64(2.0, 2.0),
            Identity {
                band_id: Some(band_id),
                ..Identity::default()
            },
            Behavior::default(),
        ));

        resources.band_store.insert(Band {
            id: band_id,
            name: "band_1".to_string(),
            members: vec![EntityId(entity.id() as u64)],
            leader: None,
            provisional_since: 10,
            promoted_tick: None,
            is_promoted: false,
            settlement_id: None,
        });

        refresh_band_behavior_state(&mut world, &resources);

        let behavior = world.get::<&Behavior>(entity).expect("behavior");
        assert_eq!(behavior.band_center_x, None);
        assert_eq!(behavior.band_center_y, None);
        assert_eq!(behavior.outsider_separation_mult, 1.0);
    }
}
