//! Read-only activity snapshot, separate from the existing agent schemas.

use godot::prelude::*;
use sim_core::components::{Activity, Agent, AgentId};

/// One active person's activity, ordered by simulation identity by the collector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivitySnapshotRow {
    /// Entity bits, matching `get_agent_snapshot().ids` rather than AgentId.
    pub entity_bits: u64,
    /// Stable simulation identity used only to order rows.
    pub agent_id: AgentId,
    /// TendFire, Craft, Repair, HearthCircle, Explore, Play, SitByFire = 0..6.
    pub kind: u8,
    /// The activity's chosen place, not the person's current walking position.
    pub place: (u32, u32),
    /// Completion tick after arrival; None while walking to the place.
    pub until_tick: Option<u64>,
}

/// Collect only agents with an Activity, without changing the world or its order.
pub fn collect_activity_snapshot(world: &hecs::World) -> Vec<ActivitySnapshotRow> {
    let mut rows: Vec<_> = world
        .query::<(&Agent, &Activity)>()
        .iter()
        .map(|(entity, (agent, activity))| ActivitySnapshotRow {
            entity_bits: entity.to_bits().get(),
            agent_id: agent.id,
            kind: activity.kind.code(),
            place: activity.place,
            until_tick: activity.until_tick,
        })
        .collect();
    rows.sort_by_key(|row| (row.agent_id, row.entity_bits));
    rows
}

/// Marshal five parallel arrays; a walking activity has `until_tick == -1`.
pub(super) fn activity_rows_to_dict(rows: &[ActivitySnapshotRow]) -> VarDictionary {
    let mut ids = PackedInt64Array::new();
    let mut kinds = PackedByteArray::new();
    let mut place_x = PackedInt32Array::new();
    let mut place_y = PackedInt32Array::new();
    let mut until_tick = PackedInt64Array::new();
    for row in rows {
        ids.push(i64::from_ne_bytes(row.entity_bits.to_ne_bytes()));
        kinds.push(row.kind);
        place_x.push(i32::try_from(row.place.0).unwrap_or(i32::MAX));
        place_y.push(i32::try_from(row.place.1).unwrap_or(i32::MAX));
        until_tick.push(
            row.until_tick
                .map_or(-1, |tick| i64::try_from(tick).unwrap_or(i64::MAX)),
        );
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("kinds", kinds);
    dict.set("place_x", place_x);
    dict.set("place_y", place_y);
    dict.set("until_tick", until_tick);
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{ActivityKind, AgentState, Child, TargetKind};

    fn activity(kind: ActivityKind, until_tick: Option<u64>) -> Activity {
        Activity {
            kind,
            settlement: 7,
            place: (12, 34),
            chosen_tick: 10,
            arrived_tick: until_tick.map(|_| 20),
            until_tick,
        }
    }

    #[test]
    fn snapshot_is_empty_when_agents_have_no_activity() {
        // Given agents without activities, and a non-agent holding an activity.
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 },));
        world.spawn((activity(ActivityKind::Play, None),));
        // When collecting the activity surface.
        let rows = collect_activity_snapshot(&world);
        // Then neither is exposed as a person doing an activity.
        assert!(rows.is_empty());
    }

    #[test]
    fn snapshot_orders_identity_across_archetypes_and_preserves_entity_bits() {
        // Given reversed identities in distinct archetypes and walk/arrival phases.
        let mut world = hecs::World::new();
        let later = world.spawn((
            Agent { id: 20 },
            activity(ActivityKind::Craft, Some(110)),
            AgentState::Consuming {
                target: TargetKind::Activity,
            },
        ));
        let earlier = world.spawn((
            Agent { id: 10 },
            activity(ActivityKind::Play, None),
            AgentState::Seeking {
                target: TargetKind::Activity,
            },
            Child { born_tick: 0 },
        ));
        // When collecting twice without advancing the world.
        let rows = collect_activity_snapshot(&world);
        // Then ordering is stable and ids join the existing entity-bit snapshot.
        assert_eq!(
            rows,
            vec![
                ActivitySnapshotRow {
                    entity_bits: earlier.to_bits().get(),
                    agent_id: 10,
                    kind: 5,
                    place: (12, 34),
                    until_tick: None,
                },
                ActivitySnapshotRow {
                    entity_bits: later.to_bits().get(),
                    agent_id: 20,
                    kind: 1,
                    place: (12, 34),
                    until_tick: Some(110),
                },
            ]
        );
        assert_eq!(collect_activity_snapshot(&world), rows);
    }

    #[test]
    fn snapshot_exposes_each_activity_code_and_omits_removed_components() {
        // Given all seven activity kinds, plus a removed activity.
        let mut world = hecs::World::new();
        let kinds = [
            ActivityKind::TendFire,
            ActivityKind::Craft,
            ActivityKind::Repair,
            ActivityKind::HearthCircle,
            ActivityKind::Explore,
            ActivityKind::Play,
            ActivityKind::SitByFire,
        ];
        for (index, kind) in kinds.into_iter().enumerate() {
            world.spawn((
                Agent {
                    id: u64::try_from(index).expect("fixture index"),
                },
                activity(kind, Some(100)),
            ));
        }
        let removed = world.spawn((Agent { id: 99 }, activity(ActivityKind::Repair, None)));
        world
            .remove_one::<Activity>(removed)
            .expect("remove fixture activity");
        // When collecting the current components.
        let rows = collect_activity_snapshot(&world);
        // Then every documented code appears once, without the removed component.
        assert_eq!(
            rows.iter().map(|row| row.kind).collect::<Vec<_>>(),
            (0..=6).collect::<Vec<_>>()
        );
    }
}
