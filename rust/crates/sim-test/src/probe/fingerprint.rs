//! Portable, sorted FNV-1a trajectory fingerprints.
use sim_core::components::{
    Agent, AgentState, BodyHealth, Hunger, Position, ResourceKind, Sleep, Social, TargetKind,
    Thirst,
};
use sim_engine::SimEngine;

const OFFSET: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x100000001b3;

pub(super) fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(PRIME);
    }
}

/// Map states exactly as the agent snapshot collector does.
pub(super) const fn state_tag(state: Option<&AgentState>) -> u8 {
    match state {
        None | Some(AgentState::Idle) => 0,
        Some(
            AgentState::Seeking {
                target: TargetKind::Activity,
            }
            | AgentState::Consuming {
                target: TargetKind::Activity,
            },
        ) => 4,
        Some(AgentState::Seeking { .. }) => 1,
        Some(AgentState::Consuming {
            target: TargetKind::Agent(_),
        }) => 2,
        Some(AgentState::Consuming { .. }) => 3,
    }
}

/// Hash sorted agent native scalar bits, cumulative tallies, then sorted settlements.
/// Hunger retains its legacy f32 bit width; all other need/health fields are f64.
/// Settlement ids use u32, member counts u64, and food quantities u32, all LE.
pub fn fingerprint(engine: &SimEngine) -> String {
    let mut hash = OFFSET;
    let mut query = engine.world.query::<(
        &Agent,
        &Position,
        Option<&AgentState>,
        Option<&Hunger>,
        Option<&Thirst>,
        Option<&Sleep>,
        Option<&Social>,
        Option<&BodyHealth>,
    )>();
    let mut agents: Vec<_> = query.iter().map(|(_, row)| row).collect();
    agents.sort_by_key(|row| row.0.id);
    for (agent, pos, state, hunger, thirst, sleep, social, health) in agents {
        feed(&mut hash, &agent.id.to_le_bytes());
        feed(&mut hash, &pos.x.to_le_bytes());
        feed(&mut hash, &pos.y.to_le_bytes());
        feed(&mut hash, &[state_tag(state)]);
        feed(
            &mut hash,
            &hunger.map_or(0.0, |v| v.value).to_bits().to_le_bytes(),
        );
        for value in [
            thirst.map_or(0.0, |v| v.value),
            sleep.map_or(0.0, |v| v.fatigue),
            social.map_or(0.0, |v| v.loneliness),
            health.map_or(0.0, |v| v.hp),
        ] {
            feed(&mut hash, &value.to_bits().to_le_bytes());
        }
    }
    let t = engine.resources.tallies;
    for value in [
        t.births,
        t.deaths_starvation,
        t.deaths_dehydration,
        t.deaths_combat,
    ] {
        feed(&mut hash, &value.to_le_bytes());
    }
    let mut settlements: Vec<_> = engine.resources.settlements.values().collect();
    settlements.sort_by_key(|s| s.settlement_id);
    for settlement in settlements {
        feed(&mut hash, &settlement.settlement_id.to_le_bytes());
        feed(
            &mut hash,
            &(settlement.member_agents.len() as u64).to_le_bytes(),
        );
        feed(
            &mut hash,
            &settlement
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0)
                .to_le_bytes(),
        );
    }
    super::mind_fingerprint::append(&mut hash, &engine.resources);
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fnv_matches_known_vector() {
        let mut hash = OFFSET;
        feed(&mut hash, b"hello");
        assert_eq!(hash, 0xa430d84680aabd0b);
    }
    #[test]
    fn independent_baseline_engines_match() {
        assert_eq!(
            fingerprint(&crate::scenes::baseline_scene(0)),
            fingerprint(&crate::scenes::baseline_scene(0))
        );
        assert_ne!(
            fingerprint(&crate::scenes::baseline_scene(0)),
            fingerprint(&crate::scenes::baseline_scene(1))
        );
    }
    #[test]
    fn tally_changes_are_observable() {
        let mut engine = crate::scenes::baseline_scene(0);
        let before = fingerprint(&engine);
        engine.resources.tallies.births += 1;
        assert_ne!(before, fingerprint(&engine));
    }
}
