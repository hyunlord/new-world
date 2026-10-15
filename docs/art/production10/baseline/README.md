# Part 0 live-world census

Source merge: `0caabf5fd1c62b9d3c99845daab1805e79799db7`. Debug bridge built locally; no renderer changes. Source `world_node.rs:193` builds live `game_v22`; stress builder remains `game_v19` at line 185. Runtime census: `live-day30-1000.json`.

Seed 20261010, displayed day 30 at 10:00 is tick 41880 (day_index 29, clock offset applied). Population 29; settlements 1; lit fires 0. Seven state-tag-4 agents: TendFire 1, Craft 0, Repair 0, HearthCircle 3, Explore 0, Play 1, SitByFire 2. Unknown activity kinds: 0. Five family/temperament/detail getter samples are preserved in the raw JSON. Probe elapsed 200.007 seconds, zero failures.

This establishes bridge outputs, not visual correctness. Original six framing census and replacement selection are still pending.

Framing census in progress: P1 tick40560 retains one settlement/population29; P2 tick31320 retains one lit settlement/population29. P3 tick46189 retains two settlements/population37. P9's actual P3 sidecar (`soc_d33_0949_FV_all.entities.json`) also had two settlements, so P3 still meets its established multi-village framing purpose without moving the tick. P4 remains pending.

P6 fixture correction: previous P9 final P6 image was identical to P1 and had no inspector, because deferred world_reset closed it. P10 opens the inspector after deferred reset delivery (frame12), retains simulation setup frame10 and capture130, and fails if the panel is not visible. Prior P9 evidence remains unchanged.

## Running harness control
W200 baseline versus unchanged-renderer control: PASS, 600 frames and 300 mutations (1440→1740). Raw evidence: `/Users/rexxa/work/art-production10-evidence/running/{base,control}/W200.jsonl`; comparison: `running/control-W200-comparison.json`. This validates the harness, not the final Part1 optimization.

## Six-framing census completed
All original seeds/ticks remain valid under game_v22 (P5 remains game_v19).

| Framing | Population | Settlements | Lit fires |
|---|---:|---:|---:|
| P1 | 29 | 1 | 0 |
| P2 | 29 | 1 | 1 |
| P3 | 37 | 2 | 0 |
| P4 | 66 | 3 | 0 |
| P5 | 500 | 1 | 0 |
| P6 | 29 | 1 | 0 |

Raw census `framings-census.json`: complete=true, failures=[], elapsed1482.304s. P6 fixture capture exit0 and visually inspected: resident inspector and GodHand panels are visible.

## Independent baseline repeat
A and Aprime completed six independent launches each. `control-comparison.json` verifies equal fixed60FPS/120-frame settle schedules and exact RGBA equality for all six framings. Candidate Part1 captures therefore require exact equality for every framing. This control result is not H5 acceptance.
