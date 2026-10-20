# ADR 0012 — Player orders are recorded commands

## Context
M1 gives the player verbs: mark trees for felling, add and remove stockpile zones, place a hut blueprint, cancel. The simulation must stay deterministic and replayable while the player gives orders at any moment, paused or not, between ticks the live game feeds at a varying rate.

## Decision
Every order is a `Command` value (`rust/crates/sim-core/src/colony/command.rs`). `sim_systems::commands::apply_command(world, resources, command) -> CommandResult` is the only path that changes designations, zones and blueprints. It validates the order (out of map, water, blocked, nothing to do; drafting is rejected until M5) and returns `{accepted, count, reason}`. The bridge (`cmd_*` funcs) applies a command at once through `commands::submit`. That call also appends `(tick, command, result)` to `colony.command_log`, where `tick` is the engine's next tick: a command applied between ticks t−1 and t is replayed right before tick t's systems. `worldsim_probe --scene colony --commands FILE` replays a script or a recorded log (`--commands-out`) and produces a byte-identical summary. Without `rules.colony` every command is rejected (`NoColony`): the baseline world takes no orders.

## Alternatives considered
- Orders as bridge calls that mutate state directly (no log): not replayable, and validation would sit in the bridge.
- Orders queued and applied by a system at a fixed priority: one extra tick of latency, and a paused game could not show the result of an order at once.
- Logging only the effects (the changed tiles): larger logs, and a replay would not exercise validation.

## Consequences
A run is its seed plus its command log; bugs reproduce from both. New verbs (M2+) are new `Command` variants with a validation table and a test per reject reason (`worldsim-architecture`, "add a command"). The UI shows the result count or the reason as a toast.

## Date
2026-10-20

## Source
docs/v8/m1/M1_GOAL.md 6.3, 6.4; docs/v8/m1/M1_DECISIONS.md D14, D16; rust/crates/sim-test/tests/colony_commands.rs
