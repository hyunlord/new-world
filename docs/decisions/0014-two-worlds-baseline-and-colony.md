# ADR 0014 — Two worlds: `baseline` is frozen for identity, `colony` is the game

## Context
V8-0 proved, with `tools/v8/identity.sh`, that its cleanup changed nothing: the `baseline` world is the V7 game, bit for bit. M1 builds a different game (a 192-tile map, 10 founders, orders, items and jobs), and later milestones keep changing it. Without a fixed reference, every later refactor would lose its proof of "no behaviour change".

## Decision
- **`baseline`** (`WorldGenConfig::baseline` + `WorldRules::baseline`) stays the V7 world. Every new behaviour is gated by a rule that is off there (`rules.colony = None`, `WorldGenConfig::colony = None`). Its rule and worldgen dumps change only by added fields with their off value. `tools/v8/identity.sh` against the base data must pass after every change to `rust/**`.
- **`colony`** (`WorldGenConfig::colony` + `WorldRules::colony` = baseline + the colony rule, no automatic shelter planning, no fission, a step every 3 ticks) is the game. `build_game_engine(seed)` builds it; the probe (`--scene colony`), the perf tool (`natural --scene colony`) and the tests (`sim_test::colony_engine`) use it.

## Alternatives considered
- Retire baseline and re-capture identity on the colony: the colony keeps changing in every milestone, so identity would prove nothing across them.
- Keep the game on baseline and add orders to the V7 world: the 64-tile, 18–30-founder world is not the game VISION describes (ADR 0003).

## Consequences
New code is written behind the colony gate, and identity keeps catching leaks (a new behaviour that runs in baseline). The colony has its own determinism and replay checks (gate steps `colony_determinism`, `replay`). When baseline can no longer host a needed refactor, a later ADR may retire it.

## Date
2026-10-20

## Source
docs/v8/m1/M1_GOAL.md "Two worlds", 4.1, 8.1; docs/decisions/0011-one-baseline-instead-of-version-chains.md
