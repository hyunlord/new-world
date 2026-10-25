//! Defence state must participate in gameplay identity, including when policy is OFF.
use sim_core::colony::steward::DefenceState;
use sim_test::probe::colony::colony_fingerprint;

#[test]
fn planted_defence_state_differences_are_detected_while_overall_steward_is_off() {
    let mut engine = sim_test::colony_engine(0);
    assert!(!engine.resources.steward.enabled);
    assert!(engine.resources.steward.owned.is_empty());
    let baseline = colony_fingerprint(&engine);
    let mut variants = Vec::new();
    let mut state = DefenceState::default();
    state.enabled = false;
    variants.push(("policy", state));
    let mut state = DefenceState::default();
    state.owned_ids.insert(17);
    variants.push(("ownership", state));
    let mut state = DefenceState::default();
    state.manual_until.insert(17, 1440);
    variants.push(("manual deadline", state));
    let mut state = DefenceState::default();
    state.quiet_since = Some(60);
    variants.push(("quiet deadline", state));
    let mut state = DefenceState::default();
    state.retired_pack = Some(42);
    variants.push(("retired pack", state));
    let mut state = DefenceState::default();
    state.responded_pack = Some(42);
    variants.push(("response ownership", state));
    let mut state = DefenceState::default();
    state.last_count = 4;
    variants.push(("batch count", state));
    for (name, state) in variants {
        engine.resources.steward.defence = state;
        assert_ne!(
            colony_fingerprint(&engine),
            baseline,
            "planted {name} must change identity"
        );
    }
    engine.resources.steward.defence = DefenceState::default();
    assert_eq!(
        colony_fingerprint(&engine),
        baseline,
        "default OFF fixture identity remains unchanged"
    );
}
