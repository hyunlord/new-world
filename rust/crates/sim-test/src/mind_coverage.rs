//! Independent tick-boundary audit of runtime choices against retained reasons.
//! Counts first choices, behavior changes, node-only changes and observable same-behavior
//! restarts separately. A restart is finished -> active; a same-node active -> finished
//! transition is completion, not a new choice. Intra-tick changes returning to the same
//! boundary state are outside this observer's denominator (never inferred from records).
use serde::Serialize;
use sim_core::colony::mind::RunningChoice;
use sim_core::components::AgentId;
use sim_engine::SimResources;
use std::collections::BTreeMap;

/// Tick-boundary coverage, with bounded diagnostic examples rather than bounded counters.
#[derive(Default)]
pub struct MindCoverage {
    before: BTreeMap<AgentId, RunningChoice>,
    report: CoverageReport,
}
/// Persisted probe evidence. Absence of mind is N/A, not a successful coverage result.
#[derive(Default, Serialize)]
pub struct CoverageReport {
    pub observed_mind_ticks: u64,
    pub first_choices: u64,
    pub behaviour_switches: u64,
    pub node_only_choices: u64,
    pub same_behaviour_restarts: u64,
    pub expected_records: u64,
    pub covered_records: u64,
    pub behaviour_expected: u64,
    pub behaviour_covered: u64,
    pub node_expected: u64,
    pub node_covered: u64,
    pub critical_choice_expected: u64,
    pub critical_choice_covered: u64,
    pub critical_expected: u64,
    pub critical_covered: u64,
    pub failures: u64,
    pub examples: Vec<CoverageFailure>,
}
/// First sixteen failures in deterministic agent/tick order.
#[derive(Serialize)]
pub struct CoverageFailure {
    tick: u64,
    agent: AgentId,
    node: usize,
    problem: &'static str,
}
impl MindCoverage {
    /// Snapshot runtime state before systems run; never derive expectations from the log.
    pub fn before_tick(&mut self, resources: &SimResources) {
        if resources.mind.policy_source.is_none() && resources.mind.running.is_empty() {
            return;
        }
        self.before.clone_from(&resources.mind.running);
    }
    /// Validate the final runtime choice against this tick's independent retained record.
    pub fn after_tick(&mut self, resources: &SimResources) {
        let state = &resources.mind;
        if state.history.is_none() && state.running.is_empty() && state.policy_source.is_none() {
            return;
        }
        self.report.observed_mind_ticks += 1;
        for (&agent, running) in &state.running {
            let previous = self.before.get(&agent);
            let changed = previous
                .is_none_or(|old| old.node != running.node || old.revision != running.revision);
            let restarted = previous.is_some_and(|old| old.finished && !running.finished);
            if !changed && !restarted {
                continue;
            }
            self.report.expected_records += 1;
            let dictionary = state.history.as_ref().map(|history| history.dictionary());
            let selected = dictionary.and_then(|d| d.node_label(running.node.index()));
            let prior = previous.and_then(|old| {
                dictionary
                    .filter(|d| d.revision() == old.revision.code())
                    .and_then(|d| d.node_label(old.node.index()))
            });
            let behaviour_change = previous.is_none()
                || selected
                    .zip(prior)
                    .is_some_and(|(new, old)| new.behaviour != old.behaviour);
            self.report.behaviour_expected += u64::from(behaviour_change);
            self.report.node_expected += u64::from(!behaviour_change);
            if previous.is_none() {
                self.report.first_choices += 1;
            } else if selected
                .zip(prior)
                .is_some_and(|(new, old)| new.behaviour != old.behaviour)
            {
                self.report.behaviour_switches += 1;
            } else if restarted {
                self.report.same_behaviour_restarts += 1;
            } else {
                self.report.node_only_choices += 1;
            }
            let critical = selected.is_some_and(|node| node.band == 0);
            self.report.critical_expected += u64::from(critical && behaviour_change);
            self.report.critical_choice_expected += u64::from(critical);
            let problem = match (
                dictionary,
                state.history.as_ref().and_then(|h| h.latest(agent)),
            ) {
                (None, _) => Some("missing_dictionary"),
                (_, None) => Some("missing_record"),
                (Some(dictionary), Some(packed)) => match packed.decode_display(dictionary) {
                    Err(_) => Some("invalid_record"),
                    Ok(record)
                        if record.agent != agent
                            || record.selected.node != running.node.index()
                            || record.revision != running.revision.code() =>
                    {
                        Some("wrong_choice")
                    }
                    Ok(record) if record.tick != resources.current_tick => Some("wrong_tick"),
                    Ok(record) if record.facts.is_empty() => Some("empty_facts"),
                    Ok(_) => None,
                },
            };
            if let Some(problem) = problem {
                self.report.failures += 1;
                if self.report.examples.len() < 16 {
                    self.report.examples.push(CoverageFailure {
                        tick: resources.current_tick,
                        agent,
                        node: running.node.index(),
                        problem,
                    });
                }
            } else {
                self.report.covered_records += 1;
                self.report.critical_covered += u64::from(critical && behaviour_change);
                self.report.critical_choice_covered += u64::from(critical);
                self.report.behaviour_covered += u64::from(behaviour_change);
                self.report.node_covered += u64::from(!behaviour_change);
            }
        }
    }
    /// Whether any mind state was observed; legacy probes omit the entire report.
    pub fn active(&self) -> bool {
        self.report.observed_mind_ticks != 0
    }
    /// Ratios use independent observed transitions; zero denominators are explicitly N/A.
    pub fn report(&self) -> serde_json::Value {
        let mut value = serde_json::to_value(&self.report).expect("coverage primitives serialize");
        value["status"] = serde_json::json!(if self.report.observed_mind_ticks == 0 {
            "N/A"
        } else if self.report.failures != 0 {
            "FAIL"
        } else if self.report.expected_records == 0 {
            "N/A"
        } else {
            "PASS"
        });
        value["coverage"] = ratio(
            self.report.behaviour_covered,
            self.report.behaviour_expected,
        );
        value["node_coverage"] = ratio(self.report.node_covered, self.report.node_expected);
        value["critical_choice_coverage"] = ratio(
            self.report.critical_choice_covered,
            self.report.critical_choice_expected,
        );
        value["critical_coverage"] =
            ratio(self.report.critical_covered, self.report.critical_expected);
        value["definition"] = serde_json::json!("Coverage denominator: first choices plus registered behavior changes only. Node coverage separately counts node-only changes and observable finished-to-active same-behavior restarts. Same-node completion and invisible intra-tick roundtrips excluded. Critical denominator: first choices or behavior changes ending in band 0. Stricter critical_choice denominator: every first/node/behavior/restart transition ending in band 0; use this for the critical gate. Zero denominator is N/A.");
        value
    }
}
fn ratio(covered: u64, expected: u64) -> serde_json::Value {
    if expected == 0 {
        serde_json::Value::Null
    } else {
        serde_json::json!(covered as f64 / expected as f64)
    }
}
#[cfg(test)]
#[path = "mind_coverage_tests.rs"]
mod tests;
