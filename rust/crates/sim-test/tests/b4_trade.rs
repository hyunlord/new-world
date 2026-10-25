//! T phase-one behavior regressions using only APIs present in the unchanged P build.
#[path = "b4_trade/support.rs"]
mod support;
use support::*;
#[path = "b4_trade/boundaries.rs"]
mod boundaries;
#[path = "b4_trade/policy.rs"]
mod policy;
#[path = "b4_trade/validation.rs"]
mod validation;

#[test]
fn t_public_data_parser_accepts_the_authored_scarcity_policy() {
    let rule =
        ron::to_string(&neighbour_data().expect("data").rule).expect("serialize existing rule");
    let text=format!("(schema_version:1,rule:{rule},names:[],goods:[],goods_policy:Some(Scarce(maximum_practitioners:2,scarce_weight:30,staple_weight:10,rare_weight:1,next_era_cutoff:750,rare_era:Neolithic)))");
    assert!(
        ron::from_str::<NeighbourData>(&text).is_ok(),
        "valid registered top-level policy must deserialize without changing saved rule shape"
    );
}

#[test]
fn t_one_and_two_current_adult_practitioners_keep_scarce_goods() {
    let node = node(Era::Paleolithic);
    for count in [1, 2] {
        let mut observable = false;
        for seed in 0..8 {
            let mut e = fixture(seed, node, 750);
            for _ in 0..count {
                holder(&mut e, node, 1000, TODAY, false, true);
            }
            let expected = expected(&e, node, count, 750);
            observable |= has_artifact(&expected, node);
            assert_eq!(generate(&mut e), expected, "seed={seed} count={count}");
        }
        assert!(
            observable,
            "fixed calibration set actually samples the affected artifact"
        );
    }
}

#[test]
fn t_children_and_non_agents_do_not_supply_practitioner_counts() {
    let node = node(Era::Paleolithic);
    for (child, agent) in [(true, true), (false, false)] {
        for seed in 0..8 {
            let mut e = fixture(seed, node, 750);
            for _ in 0..3 {
                holder(&mut e, node, 1000, TODAY, child, agent);
            }
            assert_ne!(
                trade::colony_practice(&e.world) & knowledge::bit(node),
                0,
                "legacy mask remains independently populated"
            );
            let expected = expected(&e, node, 0, 750);
            assert_eq!(
                generate(&mut e),
                expected,
                "seed={seed} child={child} agent={agent}"
            );
        }
    }
}

#[test]
fn t_stale_holders_do_not_supply_current_practitioner_counts() {
    let node = node(Era::Paleolithic);
    for seed in 0..8 {
        let mut e = fixture(seed, node, 750);
        let window = sim_systems::runtime::knowledge::rule(&e.resources)
            .expect("knowledge rule")
            .practice_days;
        let stale = TODAY - u16::try_from(window).expect("small practice window") - 1;
        for _ in 0..3 {
            holder(&mut e, node, 1000, stale, false, true);
        }
        assert_ne!(trade::colony_practice(&e.world) & knowledge::bit(node), 0);
        let expected = expected(&e, node, 0, 750);
        assert_eq!(generate(&mut e), expected, "seed={seed} stale holders");
    }
}

#[test]
fn t_threshold_and_inclusive_recency_boundary_control_three_practitioners() {
    let node = node(Era::Paleolithic);
    let threshold = knowledge::defs().threshold(node);
    for recent in [true, false] {
        for proficient in [true, false] {
            for seed in 0..8 {
                let mut e = fixture(seed, node, 750);
                let window = sim_systems::runtime::knowledge::rule(&e.resources)
                    .expect("knowledge rule")
                    .practice_days;
                let last = TODAY - u16::try_from(window).expect("window") - u16::from(!recent);
                for _ in 0..3 {
                    holder(
                        &mut e,
                        node,
                        threshold - u16::from(!proficient),
                        last,
                        false,
                        true,
                    );
                }
                let count = if recent && proficient { 3 } else { 0 };
                let expected = expected(&e, node, count, 750);
                assert_eq!(
                    generate(&mut e),
                    expected,
                    "seed={seed} recent={recent} threshold_met={proficient}"
                );
            }
        }
    }
}

#[test]
fn t_neolithic_unknown_rare_priority_ends_at_progress_750() {
    let node = node(Era::Neolithic);
    for progress in [249, 250, 749, 750] {
        for seed in 0..8 {
            let mut e = fixture(seed, node, progress);
            let expected = expected(&e, node, 0, progress);
            assert_eq!(
                generate(&mut e),
                expected,
                "seed={seed} progress={progress}"
            );
        }
    }
}

#[test]
fn t_three_practitioners_remain_omitted_and_legacy_mask_is_unchanged() {
    let node = node(Era::Paleolithic);
    for seed in 0..8 {
        let mut e = fixture(seed, node, 750);
        for _ in 0..3 {
            holder(&mut e, node, 1000, TODAY, false, true);
        }
        let mask = trade::colony_practice(&e.world);
        let expected = expected(&e, node, 3, 750);
        assert!(!has_artifact(&expected, node));
        assert_eq!(generate(&mut e), expected);
        assert_eq!(
            trade::colony_practice(&e.world),
            mask,
            "goods generation does not rewrite historical practice semantics"
        );
    }
}
