//! Differential checks for compact stamps under the tick-local monotonic invariant.
use super::*;

const DOMAINS: [Domain; DOMAIN_COUNT] = [
    Domain::Person,
    Domain::Claims,
    Domain::FoodSources,
    Domain::WaterSources,
    Domain::SleepSources,
    Domain::RoutineOccupancy,
    Domain::Invitations,
    Domain::Jobs,
    Domain::Threats,
];

fn context() -> TickContext {
    let mut world = World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let resources = SimResources::new(4, 4, sim_core::MaterialRegistry::new());
    TickContext::capture(&world, &resources).expect("context")
}

fn original_fresh(
    mask: u16,
    captured: &[u64; DOMAIN_COUNT],
    current: &[u64; DOMAIN_COUNT],
) -> bool {
    captured
        .iter()
        .enumerate()
        .all(|(index, version)| mask & (1 << index) == 0 || current[index] == *version)
}

#[test]
fn every_subset_matches_original_across_monotonic_sequences_and_duplicate_domains() {
    let mut ctx = context();
    for initial in [
        [0; DOMAIN_COUNT],
        [u64::MAX - 1; DOMAIN_COUNT],
        std::array::from_fn(|index| (index as u64) * 17),
    ] {
        for mask in 0u16..(1 << DOMAIN_COUNT) {
            // Initialize the simulated tick before constructing any stamp.
            ctx.versions = initial;
            let domains: Vec<_> = DOMAINS
                .iter()
                .copied()
                .filter(|domain| mask & (1 << *domain as usize) != 0)
                .collect();
            let duplicates: Vec<_> = domains
                .iter()
                .copied()
                .chain(domains.iter().rev().copied())
                .collect();
            let stamp = ctx.stamp(123, &domains);
            let duplicate_stamp = ctx.stamp(123, &duplicates);
            assert_eq!(stamp.dependencies, mask);
            assert_eq!(stamp.version_sum, duplicate_stamp.version_sum);
            assert!(stamp.fresh(&ctx.versions));
            for domain in DOMAINS {
                ctx.invalidate(domain)
                    .expect("first increment cannot overflow");
                let expected = original_fresh(mask, &initial, &ctx.versions);
                assert_eq!(
                    stamp.fresh(&ctx.versions),
                    expected,
                    "mask={mask}, domain={domain:?}"
                );
                assert_eq!(duplicate_stamp.fresh(&ctx.versions), expected);
                let restamped = ctx.stamp(456, &duplicates);
                assert!(restamped.fresh(&ctx.versions));
                if initial[domain as usize] == u64::MAX - 1 {
                    let before = ctx.versions;
                    assert_eq!(
                        ctx.invalidate(domain),
                        Err(ContextError::VersionOverflow(domain))
                    );
                    assert_eq!(ctx.versions, before);
                    assert_eq!(stamp.fresh(&ctx.versions), expected);
                    assert!(restamped.fresh(&ctx.versions));
                }
            }
        }
    }
}

#[test]
fn sums_above_u64_max_are_exact_and_empty_dependencies_remain_fresh() {
    let mut ctx = context();
    ctx.versions = [u64::MAX - 1; DOMAIN_COUNT];
    let all = ctx.stamp(1, &DOMAINS);
    let empty = ctx.stamp(2, &[]);
    assert_eq!(
        all.version_sum,
        u128::from(u64::MAX - 1) * DOMAIN_COUNT as u128
    );
    assert!(all.version_sum > u128::from(u64::MAX));
    assert_eq!(empty.version_sum, 0);
    for domain in DOMAINS {
        ctx.invalidate(domain).expect("advance to maximum");
        assert!(!all.fresh(&ctx.versions));
        assert!(empty.fresh(&ctx.versions));
    }
    let maximum = ctx.stamp(3, &DOMAINS);
    assert_eq!(
        maximum.version_sum,
        u128::from(u64::MAX) * DOMAIN_COUNT as u128
    );
    for domain in DOMAINS {
        assert_eq!(
            ctx.invalidate(domain),
            Err(ContextError::VersionOverflow(domain))
        );
        assert!(maximum.fresh(&ctx.versions));
        assert!(empty.fresh(&ctx.versions));
    }
}

#[test]
fn overwrites_restamp_slots_and_dynamic_names_with_new_dependency_sets() {
    let mut ctx = context();
    for name in ["social_candidate", "custom-derived"] {
        ctx.set_derived(7, name, 1, &[Domain::Person, Domain::Person])
            .expect("write");
        ctx.invalidate(Domain::Person).expect("invalidate");
        assert_eq!(
            ctx.derived(7, name),
            Err(ContextError::StaleDerived {
                agent: 7,
                name: name.into()
            })
        );
        ctx.set_derived(7, name, 2, &[Domain::Claims])
            .expect("restamp");
        ctx.invalidate(Domain::Person).expect("unrelated change");
        assert_eq!(ctx.derived(7, name), Ok(2));
        ctx.invalidate(Domain::Claims).expect("related change");
        assert!(matches!(
            ctx.derived(7, name),
            Err(ContextError::StaleDerived { .. })
        ));
        ctx.set_derived(7, name, 3, &[]).expect("freeze");
        ctx.invalidate(Domain::Claims).expect("change after freeze");
        assert_eq!(ctx.derived(7, name), Ok(3));
    }
    for name in ["food", "custom-target"] {
        let target = TargetCandidate {
            food: None,
            tile: (2, 3),
            agent: Some(7),
        };
        ctx.set_target(7, name, Some(target), &[Domain::FoodSources])
            .expect("write");
        ctx.invalidate(Domain::FoodSources).expect("invalidate");
        assert_eq!(
            ctx.target(7, name),
            Err(ContextError::StaleDerived {
                agent: 7,
                name: name.into()
            })
        );
        ctx.set_target(7, name, None, &[]).expect("restamp absence");
        ctx.invalidate(Domain::FoodSources)
            .expect("unrelated change");
        assert_eq!(ctx.target(7, name), Ok(None));
        ctx.set_target(7, name, Some(target), &[Domain::Person])
            .expect("restamp presence");
        assert_eq!(ctx.target(7, name), Ok(Some(target)));
        ctx.invalidate(Domain::Person).expect("related change");
        assert!(matches!(
            ctx.target(7, name),
            Err(ContextError::StaleDerived { .. })
        ));
    }
}
