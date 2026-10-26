//! Shadow accounting retains raw old evidence, errors and uncovered denominators.
use crate::mind_observer::{Band, PersonTick, Verdict};
use serde::Serialize;
use sim_core::components::ActivityKind;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
/// One pure selector decision and its exact factual witnesses.
pub struct Prediction {
    pub(super) node: String,
    pub(super) behaviour: String,
    pub(super) band: String,
    pub(super) score: i64,
    pub(super) facts: Vec<(String, i64)>,
    pub(super) routine_entry_draw: bool,
}
#[derive(Default, Debug, Serialize)]
/// Unique person-tick and evaluation counts; timings are deliberately absent.
pub struct Counts {
    pub(super) person_ticks: u64,
    pub(super) evaluations: u64,
    pub(super) port_person_ticks: u64,
    pub(super) port_behaviour_matches: u64,
    pub(super) port_band_matches: u64,
    pub(super) port_both_matches: u64,
    pub(super) uncovered: u64,
    pub(super) missing_prediction: u64,
    pub(super) pending_settlement: u64,
    pub(super) routine_entry_draw_replacements: u64,
}
#[derive(Debug, Serialize)]
/// Bounded raw old and predicted evidence for a disagreement category.
pub struct Example {
    pub(super) tick: u64,
    pub(super) agent: u64,
    pub(super) old_arm: String,
    pub(super) old_exit: Option<String>,
    pub(super) old_behaviour: String,
    pub(super) old_band: Option<String>,
    pub(super) old_verdict: String,
    pub(super) predicted: Option<Prediction>,
    pub(super) category: String,
}
#[derive(Default, Debug, Serialize)]
/// Coverage and agreement remain separate; errors cannot silently leave the denominator.
pub struct Summary {
    pub(super) counts: Counts,
    pub(super) by_arm: BTreeMap<String, Counts>,
    pub(super) by_exit: BTreeMap<String, u64>,
    pub(super) disagreement_categories: BTreeMap<String, u64>,
    pub(super) examples: BTreeMap<String, Vec<Example>>,
    pub(super) errors: Vec<String>,
    pub(super) error_count: u64,
    pub(super) unmatched_predictions: u64,
    pub(super) prediction_evaluations: u64,
    pub(super) replay_evaluations: u64,
}
fn band_name(band: Band) -> &'static str {
    match band {
        Band::Critical => "critical",
        Band::Duty => "duty",
        Band::Need => "need",
        Band::Work => "work",
        Band::Leisure => "leisure",
        Band::Idle => "idle",
    }
}
fn routine_name(kind: ActivityKind) -> &'static str {
    match kind {
        ActivityKind::TendFire => "routine_tend_fire",
        ActivityKind::Craft => "routine_craft",
        ActivityKind::Repair => "routine_repair",
        ActivityKind::HearthCircle => "routine_hearth_circle",
        ActivityKind::Explore => "routine_explore",
        ActivityKind::Play => "routine_play",
        ActivityKind::SitByFire => "routine_sit_by_fire",
    }
}
pub(super) fn observed_behaviour(choice: crate::mind_observer::Choice) -> &'static str {
    if choice.behaviour == "routine" {
        choice
            .activity
            .map(routine_name)
            .unwrap_or("unresolved_routine")
    } else {
        choice.behaviour
    }
}
impl Summary {
    pub(super) fn error(&mut self, error: String) {
        self.error_count += 1;
        if self.errors.len() < 16 {
            self.errors.push(error);
        }
    }
    /// True only when all observed person-ticks have known old evidence and predictions.
    pub fn coverage_complete(&self) -> bool {
        self.error_count == 0
            && self.counts.person_ticks > 0
            && self.counts.uncovered == 0
            && self.counts.missing_prediction == 0
            && self.counts.pending_settlement == 0
            && self.unmatched_predictions == 0
    }
    pub(super) fn compare(&mut self, old: PersonTick, predicted: Option<Prediction>) {
        let choice = old.final_choice;
        if let Some(exit) = choice.exit {
            *self.by_exit.entry(exit.into()).or_default() += 1;
        }
        let behaviour = observed_behaviour(choice);
        let band = choice.mapped_band.map(band_name);
        let behaviour_matches = predicted.as_ref().is_some_and(|p| p.behaviour == behaviour);
        let band_matches = predicted
            .as_ref()
            .is_some_and(|p| Some(p.band.as_str()) == band);
        // Only an admitted routine-to-routine activity draw is the declared replacement.
        // Failed eligibility/admission and every routine continuation stay in the port set.
        let routine_draw = choice.arm == "routine_started"
            && choice.activity.is_some()
            && predicted.as_ref().is_some_and(|p| {
                p.routine_entry_draw
                    && matches!(
                        p.behaviour.as_str(),
                        "routine_tend_fire"
                            | "routine_craft"
                            | "routine_repair"
                            | "routine_hearth_circle"
                            | "routine_explore"
                            | "routine_play"
                            | "routine_sit_by_fire"
                    )
            });
        let port = choice.verdict == Verdict::Port && !routine_draw;
        let uncovered = choice.verdict == Verdict::Uncovered;
        let missing = predicted.is_none();
        let replay = old.evaluations.iter().filter(|e| e.replay).count() as u64;
        self.replay_evaluations += replay;
        for counts in [
            &mut self.counts,
            self.by_arm.entry(choice.arm.into()).or_default(),
        ] {
            counts.person_ticks += 1;
            counts.evaluations += old.evaluations.len() as u64;
            counts.port_person_ticks += u64::from(port);
            counts.port_behaviour_matches += u64::from(port && behaviour_matches);
            counts.port_band_matches += u64::from(port && band_matches);
            counts.port_both_matches += u64::from(port && behaviour_matches && band_matches);
            counts.uncovered += u64::from(uncovered);
            counts.missing_prediction += u64::from(missing);
            counts.pending_settlement += u64::from(old.pending_settlement);
            counts.routine_entry_draw_replacements += u64::from(routine_draw);
        }
        if !behaviour_matches || !band_matches || uncovered || old.pending_settlement {
            let category = format!(
                "{}:{}:{}->{}:{}{}",
                choice.arm,
                behaviour,
                band.unwrap_or("unmapped"),
                predicted
                    .as_ref()
                    .map_or("missing", |p| p.behaviour.as_str()),
                predicted.as_ref().map_or("missing", |p| p.band.as_str()),
                if routine_draw {
                    ":declared_routine_draw"
                } else {
                    ":unreviewed"
                }
            );
            *self
                .disagreement_categories
                .entry(category.clone())
                .or_default() += 1;
            let examples = self.examples.entry(category.clone()).or_default();
            if examples.len() < 3 {
                examples.push(Example {
                    tick: old.tick,
                    agent: old.agent,
                    old_arm: choice.arm.into(),
                    old_exit: choice.exit.map(str::to_owned),
                    old_behaviour: choice.behaviour.into(),
                    old_band: band.map(str::to_owned),
                    old_verdict: format!("{:?}", choice.verdict),
                    predicted,
                    category,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind_observer::{Choice, Evaluation};
    fn row(choice: Choice) -> PersonTick {
        PersonTick {
            tick: 1,
            agent: 7,
            evaluations: vec![Evaluation {
                replay: false,
                choice,
            }],
            final_choice: choice,
            settlement: None,
            pending_settlement: false,
        }
    }
    fn prediction(behaviour: &str, band: &str) -> Prediction {
        Prediction {
            node: "test".into(),
            behaviour: behaviour.into(),
            band: band.into(),
            score: 0,
            facts: vec![],
            routine_entry_draw: behaviour == "routine_play",
        }
    }
    #[test]
    fn replay_counts_once_and_band_agreement_is_independent() {
        let choice = Choice::new("natural", "eat", Some(Band::Need), Verdict::Port);
        let mut old = row(choice);
        old.evaluations.push(Evaluation {
            replay: true,
            choice,
        });
        let mut summary = Summary::default();
        summary.compare(old, Some(prediction("eat", "critical")));
        assert_eq!(summary.counts.person_ticks, 1);
        assert_eq!(summary.counts.evaluations, 2);
        assert_eq!(summary.replay_evaluations, 1);
        assert_eq!(summary.counts.port_behaviour_matches, 1);
        assert_eq!(summary.counts.port_band_matches, 0);
        assert!(summary.coverage_complete());
    }
    #[test]
    fn missing_and_uncovered_do_not_disappear_or_pass_coverage() {
        let mut summary = Summary::default();
        summary.compare(
            row(Choice::new(
                "natural",
                "eat",
                Some(Band::Need),
                Verdict::Port,
            )),
            None,
        );
        summary.compare(
            row(Choice::new("unknown", "unknown", None, Verdict::Uncovered)),
            Some(prediction("wander", "idle")),
        );
        assert_eq!(summary.counts.person_ticks, 2);
        assert_eq!(summary.counts.port_person_ticks, 1);
        assert_eq!(summary.counts.port_behaviour_matches, 0);
        assert_eq!(summary.counts.missing_prediction, 1);
        assert_eq!(summary.counts.uncovered, 1);
        assert!(!summary.coverage_complete());
    }
    #[test]
    fn only_actual_routine_entry_draw_is_replacement() {
        let mut choice = Choice::new(
            "routine_started",
            "routine",
            Some(Band::Leisure),
            Verdict::Port,
        );
        choice.activity = Some(ActivityKind::Craft);
        let mut summary = Summary::default();
        summary.compare(row(choice), Some(prediction("routine_play", "leisure")));
        choice.arm = "routine_continuation";
        summary.compare(row(choice), Some(prediction("routine_play", "leisure")));
        choice.arm = "routine_started";
        summary.compare(row(choice), Some(prediction("wander", "idle")));
        let mut continuation = prediction("routine_play", "leisure");
        continuation.routine_entry_draw = false;
        summary.compare(row(choice), Some(continuation));
        assert_eq!(summary.counts.routine_entry_draw_replacements, 1);
        assert_eq!(summary.counts.port_person_ticks, 3);
        assert_eq!(summary.counts.port_behaviour_matches, 0);
    }
    #[test]
    fn completion_keeps_dispatched_behaviour_and_records_exit() {
        let mut choice = Choice::new("consume", "eat", Some(Band::Need), Verdict::Port);
        choice.exit = Some("sated");
        let mut summary = Summary::default();
        summary.compare(row(choice), Some(prediction("eat", "need")));
        assert_eq!(summary.counts.port_both_matches, 1);
        summary.error("missing facts".into());
        assert!(!summary.coverage_complete());
    }
}
