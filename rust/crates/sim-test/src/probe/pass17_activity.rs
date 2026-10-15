//! Exact, read-only routine accounting and compressed consuming intervals.
use serde_json::{json, Value};
use sim_core::components::{
    Activity, ActivityKind, AgentState, RoutineObservation, RoutineOutcome, TargetKind,
};

pub(super) const KINDS: [&str; 7] = [
    "TendFire",
    "Craft",
    "Repair",
    "HearthCircle",
    "Explore",
    "Play",
    "SitByFire",
];

#[derive(Debug, Default)]
pub(super) struct Counts {
    started: [u64; 7],
    minutes: [u64; 7],
    completed: u64,
    completed_minutes: u64,
    redraws: u64,
    failures: u64,
}

impl Counts {
    pub(super) fn json(&self) -> Value {
        json!({"activities_started":self.started,"activity_minutes":self.minutes,
            "completed_activities":self.completed,"completed_minutes":self.completed_minutes,
            "no_place_redraws":self.redraws,"no_place_failures":self.failures})
    }
}

#[derive(Clone, Copy, Debug)]
struct Episode {
    activity: Option<Activity>,
    kind: ActivityKind,
    chosen_tick: u64,
    minutes: u64,
    adult_minutes: u64,
}

#[derive(Clone, Copy, Debug)]
struct Interval {
    start: u64,
    end: u64,
    chosen: u64,
    kind: ActivityKind,
    stage: &'static str,
}

/// Snapshot after one system tick; no references survive the observation.
pub(super) struct Observation {
    pub(super) tick: u64,
    pub(super) stage: &'static str,
    pub(super) state: AgentState,
    pub(super) activity: Option<Activity>,
    pub(super) events: Vec<RoutineObservation>,
}

#[derive(Debug, Default)]
pub(super) struct Person {
    pub(super) all: Counts,
    pub(super) adult: Counts,
    pub(super) misses: u64,
    episode: Option<Episode>,
    interval: Option<Interval>,
}

impl Person {
    pub(super) fn observe(&mut self, o: Observation) -> Vec<Value> {
        let mut rows = Vec::new();
        let consuming = matches!(
            o.state,
            AgentState::Consuming {
                target: TargetKind::Activity
            }
        );
        let same = self.episode.is_some_and(|old| {
            o.activity
                .is_some_and(|a| a.chosen_tick == old.chosen_tick && a.kind == old.kind)
        });
        if self
            .interval
            .is_some_and(|i| !consuming || !same || i.stage != o.stage || i.end != o.tick)
        {
            self.flush_interval(&mut rows);
        }
        for event in o.events.iter().filter(|event| event.tick == o.tick) {
            let (name, redrawn, failed) = match event.outcome {
                RoutineOutcome::Started { redrawn } => {
                    if self.episode.is_some() {
                        self.misses += 1;
                        self.flush_interval(&mut rows);
                        self.end_episode(o.tick, false, "replaced_without_end", &mut rows);
                    }
                    if let Some(kind) = event.kind {
                        self.all.started[kind.index()] += 1;
                        if o.stage == "adult" {
                            self.adult.started[kind.index()] += 1;
                        }
                        self.episode = Some(Episode {
                            activity: None,
                            kind,
                            chosen_tick: event.tick,
                            minutes: 0,
                            adult_minutes: 0,
                        });
                    } else {
                        self.misses += 1;
                    }
                    ("started", redrawn, false)
                }
                RoutineOutcome::NoPlace { redrawn } => ("no_place", redrawn, true),
                RoutineOutcome::Ended { completed } => {
                    self.flush_interval(&mut rows);
                    if self.episode.is_none()
                        || self.episode.is_some_and(|e| Some(e.kind) != event.kind)
                    {
                        self.misses += 1;
                    }
                    self.end_episode(o.tick, completed, "observed", &mut rows);
                    ("end_observation", false, false)
                }
            };
            self.all.redraws += u64::from(redrawn);
            self.all.failures += u64::from(failed);
            if o.stage == "adult" {
                self.adult.redraws += u64::from(redrawn);
                self.adult.failures += u64::from(failed);
            }
            rows.push(json!({"event":name,"tick":o.tick,"kind":event.kind.map(|k| KINDS[k.index()]),
                "redrawn":redrawn,"life_stage":o.stage,
                "activity":o.activity.filter(|a| a.chosen_tick == event.tick && Some(a.kind) == event.kind),
                "completed":match event.outcome { RoutineOutcome::Ended { completed } => Some(completed), RoutineOutcome::Started { .. } | RoutineOutcome::NoPlace { .. } => None }}));
        }
        let same = self.episode.is_some_and(|old| {
            o.activity
                .is_some_and(|a| a.chosen_tick == old.chosen_tick && a.kind == old.kind)
        });
        if self.episode.is_some() && !same {
            self.flush_interval(&mut rows);
            self.end_episode(o.tick, false, "state_or_component_changed", &mut rows);
        }
        if let Some(activity) = o.activity {
            if self.episode.is_none() {
                self.misses += 1;
                self.episode = Some(Episode {
                    activity: None,
                    kind: activity.kind,
                    chosen_tick: activity.chosen_tick,
                    minutes: 0,
                    adult_minutes: 0,
                });
            }
            if let Some(episode) = &mut self.episode {
                if activity.arrived_tick.is_some()
                    && episode.activity.and_then(|a| a.arrived_tick).is_none()
                {
                    rows.push(json!({"event":"arrived","tick":o.tick,"kind":KINDS[activity.kind.index()],
                        "chosen_tick":activity.chosen_tick,"arrived_tick":activity.arrived_tick,"until_tick":activity.until_tick,
                        "place":activity.place,"settlement":activity.settlement}));
                }
                episode.activity = Some(activity);
            }
        }
        if consuming {
            if let Some(episode) = &mut self.episode {
                let kind = episode.kind;
                self.all.minutes[kind.index()] += 1;
                episode.minutes += 1;
                if o.stage == "adult" {
                    self.adult.minutes[kind.index()] += 1;
                    episode.adult_minutes += 1;
                }
                if let Some(interval) = &mut self.interval {
                    interval.end = o.tick + 1;
                } else {
                    self.interval = Some(Interval {
                        start: o.tick,
                        end: o.tick + 1,
                        chosen: episode.chosen_tick,
                        kind,
                        stage: o.stage,
                    });
                }
            } else {
                self.misses += 1;
            }
        }
        let tag = super::fingerprint::state_tag(Some(&o.state));
        for row in &mut rows {
            row["current_state_tag"] = json!(tag);
            row["current_target_kind"] = json!(if matches!(
                o.state,
                AgentState::Seeking {
                    target: TargetKind::Activity
                } | AgentState::Consuming {
                    target: TargetKind::Activity
                }
            ) {
                Some(8u8)
            } else {
                None
            });
            row["current_state"] = json!(o.state);
        }
        rows
    }

    fn end_episode(&mut self, tick: u64, completed: bool, source: &str, rows: &mut Vec<Value>) {
        let Some(episode) = self.episode.take() else {
            return;
        };
        let arrived_tick = episode.activity.and_then(|a| a.arrived_tick);
        let until_tick = episode.activity.and_then(|a| a.until_tick);
        let arrived = arrived_tick.is_some();
        if completed && !arrived {
            self.misses += 1;
        }
        if completed && arrived {
            self.all.completed += 1;
            self.all.completed_minutes += episode.minutes;
            if episode.adult_minutes > 0 {
                self.adult.completed += 1;
                self.adult.completed_minutes += episode.adult_minutes;
            }
        }
        rows.push(
            json!({"event":"ended","tick":tick,"kind":KINDS[episode.kind.index()],
            "chosen_tick":episode.chosen_tick,"arrived_tick":arrived_tick,
            "until_tick":until_tick,"completed":completed && arrived,"source":source,
            "consuming_minutes":episode.minutes,"adult_consuming_minutes":episode.adult_minutes}),
        );
    }

    fn flush_interval(&mut self, rows: &mut Vec<Value>) {
        if let Some(i) = self.interval.take() {
            rows.push(
                json!({"event":"consuming_interval","start_tick":i.start,"end_tick":i.end,
                "chosen_tick":i.chosen,"kind":KINDS[i.kind.index()],"life_stage":i.stage,
                "minutes":i.end-i.start}),
            );
        }
    }

    pub(super) fn disappeared(&mut self, tick: u64) -> Vec<Value> {
        let mut rows = Vec::new();
        self.flush_interval(&mut rows);
        self.end_episode(tick, false, "agent_disappeared", &mut rows);
        rows
    }

    pub(super) fn final_interval(&self) -> Option<Value> {
        self.interval.map(|i| {
            json!({"event":"consuming_interval","start_tick":i.start,"end_tick":i.end,
            "chosen_tick":i.chosen,"kind":KINDS[i.kind.index()],"life_stage":i.stage,
            "minutes":i.end-i.start,"open_at_end":true})
        })
    }
}
