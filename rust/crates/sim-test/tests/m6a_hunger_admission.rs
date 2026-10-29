//! Intentionally induced ordinary-player starvation scenario, not calibration evidence.
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::food_evidence::{AcquisitionReason, OperationEvidence};
use sim_core::colony::steward::StewardScope;
use sim_core::colony::{Command, WorkType};
use sim_core::components::Agent;

#[test]
#[ignore = "root serializes this ordinary-command evidence scenario"]
fn ordinary_disabled_harvest_hunger_has_historical_admission_reason() {
    let mut engine = sim_test::colony_engine(0);
    let result = sim_systems::commands::apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::SetSteward {
            on: false,
            scope: StewardScope::All,
        },
    );
    assert!(result.accepted);
    let mut agents: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    agents.sort_unstable();
    for agent in agents {
        let result = sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::SetWorkPriority {
                agent,
                work: WorkType::Haul,
                level: 0,
            },
        );
        assert!(result.accepted);
    }
    let end = engine.resources.ticks_per_day * 12;
    for _ in 0..end {
        engine.tick();
        let res = &engine.resources;
        for (index, death) in res.chronicle.entries().iter().enumerate() {
            if death.kind != ChronicleKind::Died {
                continue;
            }
            let mut link = index as u32;
            for _ in 0..16 {
                match res.chronicle.cause_of(link) {
                    Some(CauseRef::Entry(next)) => {
                        link = next;
                        if let Some(OperationEvidence::Acquisition { receipt, flow }) =
                            res.food.evidence.operations.get(&next)
                        {
                            if receipt.reason == AcquisitionReason::WorkDisabled {
                                assert_eq!(
                                    res.chronicle.entry(next).unwrap().subject,
                                    death.subject
                                );
                                assert!(receipt.tick <= death.tick);
                                assert!(flow.from_tick <= receipt.tick);
                                assert_eq!(flow.to_tick, death.tick);
                                assert_eq!(receipt.job_started, None);
                                return;
                            }
                        }
                    }
                    Some(CauseRef::Sample { sample, .. }) => {
                        let Some(next) = res.chronicle.sample(sample).and_then(|s| s.entry) else {
                            break;
                        };
                        link = next;
                    }
                    _ => break,
                }
            }
        }
    }
    panic!("12-day ordinary-command scenario produced no actual hunger chain with a recorded disabled admission");
}
