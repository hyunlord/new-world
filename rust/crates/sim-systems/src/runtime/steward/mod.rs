//! Priority 125, interval 360: after assignment, plan ordinary orders in data order.
//! Reads: colony, people, registered rules.
//! Writes: ordinary command effects and steward provenance.
//! Events: StewardAction for accepted ordinary orders; no RNG draws.
mod arming;
mod completion;
pub mod defence;
pub mod food;
mod house_profile;
mod housing;
pub mod lifecycle;
mod planning;
mod production;
use hecs::World;
use sim_core::colony::{steward::rules, Command};
use sim_engine::{RuntimeSystem, SimResources};

/// Deterministic optional colony assistant; default state is disabled.
#[derive(Default)]
pub struct StewardSystem;
impl RuntimeSystem for StewardSystem {
    fn name(&self) -> &str {
        "StewardSystem"
    }
    fn priority(&self) -> u32 {
        125
    }
    fn tick_interval(&self) -> u64 {
        360
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        if res.rules.colony.is_none() {
            return;
        }
        if res.steward.replay {
            let pending = std::mem::take(&mut res.steward.pending);
            for (tick, command) in pending {
                if tick <= res.current_tick && !defence::source(&command) {
                    issue(world, res, command);
                } else {
                    res.steward.pending.push((tick, command));
                }
            }
            return;
        }
        if !res.steward.enabled {
            return;
        }
        let Ok(data) = rules() else {
            return;
        };
        for rule in &data.rules {
            let mut profile = RuleProfile::new(res.current_tick, rule.id);
            if !res.steward.scope.permits(rule.scope) {
                profile.skip(true);
                continue;
            }
            if res
                .steward
                .cooldowns
                .get(&rule.id)
                .is_some_and(|t| res.current_tick < *t)
            {
                profile.skip(false);
                continue;
            }
            profile.begin_plan();
            let commands = planning::plan(world, res, rule);
            profile.planned(commands.len());
            for command in commands {
                let targets = lifecycle::targets(res, &command);
                if targets
                    .iter()
                    .any(|t| res.steward.vetoed(t, res.current_tick))
                {
                    profile.vetoed();
                    continue;
                }
                let accepted = issue(
                    world,
                    res,
                    Command::Steward {
                        rule: rule.id,
                        command: Box::new(command),
                    },
                );
                profile.issued(accepted);
            }
        }
    }
}
fn issue(world: &mut World, res: &mut SimResources, command: Command) -> bool {
    crate::commands::apply_and_log(world, res, command, res.current_tick).accepted
}

// Optional call-local telemetry: elapsed time and counters never enter simulation decisions.
struct RuleSample {
    tick: u64,
    rule: u16,
    start: std::time::Instant,
    plan_start: Option<std::time::Instant>,
    dispatch_start: Option<std::time::Instant>,
    plan_ns: u64,
    planned: usize,
    vetoed: usize,
    issued: usize,
    accepted: usize,
    skipped_scope: bool,
    skipped_cooldown: bool,
}
struct RuleProfile(Option<RuleSample>);
impl RuleProfile {
    fn new(tick: u64, rule: u16) -> Self {
        if std::env::var("WORLDSIM_MUX2_PROFILE").as_deref() != Ok("1") {
            return Self(None);
        }
        Self(Some(RuleSample {
            tick,
            rule,
            start: std::time::Instant::now(),
            plan_start: None,
            dispatch_start: None,
            plan_ns: 0,
            planned: 0,
            vetoed: 0,
            issued: 0,
            accepted: 0,
            skipped_scope: false,
            skipped_cooldown: false,
        }))
    }
    fn skip(&mut self, scope: bool) {
        if let Some(sample) = self.0.as_mut() {
            sample.skipped_scope = scope;
            sample.skipped_cooldown = !scope;
        }
    }
    fn begin_plan(&mut self) {
        if let Some(sample) = self.0.as_mut() {
            sample.plan_start = Some(std::time::Instant::now());
        }
    }
    fn planned(&mut self, count: usize) {
        if let Some(sample) = self.0.as_mut() {
            let now = std::time::Instant::now();
            sample.plan_ns = sample
                .plan_start
                .map_or(0, |start| nanos(now.duration_since(start)));
            sample.dispatch_start = Some(now);
            sample.planned = count;
        }
    }
    fn vetoed(&mut self) {
        if let Some(sample) = self.0.as_mut() {
            sample.vetoed += 1;
        }
    }
    fn issued(&mut self, accepted: bool) {
        if let Some(sample) = self.0.as_mut() {
            sample.issued += 1;
            sample.accepted += usize::from(accepted);
        }
    }
}
fn nanos(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
impl Drop for RuleProfile {
    fn drop(&mut self) {
        let Some(sample) = self.0.as_ref() else {
            return;
        };
        let now = std::time::Instant::now();
        let dispatch = sample
            .dispatch_start
            .map_or(0, |start| nanos(now.duration_since(start)));
        eprintln!(
            "MUX2_STEWARD_RULE_PROFILE {{\"tick\":{},\"rule\":{},\"total_ns\":{},\"plan_ns\":{},\"dispatch_ns\":{},\"planned\":{},\"vetoed\":{},\"issued\":{},\"accepted\":{},\"rejected\":{},\"skipped_scope\":{},\"skipped_cooldown\":{}}}",
            sample.tick, sample.rule, nanos(now.duration_since(sample.start)), sample.plan_ns, dispatch,
            sample.planned, sample.vetoed, sample.issued, sample.accepted, sample.issued - sample.accepted,
            sample.skipped_scope, sample.skipped_cooldown,
        );
    }
}
