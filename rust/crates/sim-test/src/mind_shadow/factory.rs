//! Explicit old brain with production generation, ordinary colony setup and all systems.
use crate::probe::colony::ColonyArgs;
use sim_core::colony::{steward::StewardScope, Command};
use sim_engine::SimEngine;
use sim_systems::commands::submit;

pub(super) fn world(args: &ColonyArgs, seed: u64) -> Result<SimEngine, String> {
    let mut engine = sim_systems::worldgen::build_shadow_world_with(
        &sim_systems::worldgen::WorldGenConfig::colony(seed),
        sim_core::rules::WorldRules::colony(seed),
    );
    let names = engine.system_names();
    if names
        .iter()
        .filter(|name| **name == "AgentDecisionSystem")
        .count()
        != 1
        || names.contains(&"ColonyMindSystem")
    {
        return Err(
            "mind-shadow factory must register exactly one legacy brain and no colony mind".into(),
        );
    }
    if let Some(rule) = engine.resources.rules.colony.as_mut() {
        if args.neighbours == "off" {
            rule.neighbours = None;
        }
        match args.story.as_str() {
            "off" => rule.story = None,
            "no-beasts" => {
                if let Some(story) = rule.story.as_mut() {
                    story.beasts = false;
                }
            }
            "on" => {}
            other => return Err(format!("invalid story mode {other}")),
        }
    }
    if args.steward
        && !submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All,
            },
        )
        .accepted
    {
        return Err("mind-shadow steward setup rejected".into());
    }
    if let Some(on) = args.defence {
        if !submit(&mut engine, Command::SetStewardDefence { on }).accepted {
            return Err("mind-shadow defence setup rejected".into());
        }
    }
    Ok(engine)
}
