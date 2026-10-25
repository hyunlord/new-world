//! Priority 125, interval 360: after assignment, plan ordinary orders in data order.
//! Reads: colony, people, registered rules. Writes: ordinary command effects and steward provenance.
//! Events: StewardAction for accepted ordinary orders; no RNG draws.
mod arming;
mod completion;
pub mod defence;
pub mod lifecycle;
mod planning;
mod production;
use hecs::World;
use sim_core::colony::{steward::rules, Command, CommandRecord};
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
            if !res.steward.scope.permits(rule.scope)
                || res
                    .steward
                    .cooldowns
                    .get(&rule.id)
                    .is_some_and(|t| res.current_tick < *t)
            {
                continue;
            }
            for command in planning::plan(world, res, rule) {
                let targets = lifecycle::targets(res, &command);
                if targets
                    .iter()
                    .any(|t| res.steward.vetoed(t, res.current_tick))
                {
                    continue;
                }
                issue(
                    world,
                    res,
                    Command::Steward {
                        rule: rule.id,
                        command: Box::new(command),
                    },
                );
            }
        }
    }
}
fn issue(world: &mut World, res: &mut SimResources, command: Command) {
    let result = crate::commands::apply_command(world, res, command.clone());
    res.colony.command_log.push(CommandRecord {
        tick: res.current_tick,
        command,
        result,
    });
}
