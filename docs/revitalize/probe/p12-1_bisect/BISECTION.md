# Why the afternoon work fell — the J6 bisection (Pass 12 Part 1, PASS12_GOAL.md 6)

## 6.1 Every behaviour difference between `game_v16` and `game_v17_nofission`

Read from `git diff 14372ea2..31559232 -- rust/crates/sim-core rust/crates/sim-engine rust/crates/sim-systems` and a
grep of every new rule field.

| # | Difference | Gate | Acts without interventions in v17_nofission? |
|---|---|---|---|
| 1 | Wild regions in worldgen (`camp::regions`: 1–3 regions, bushes and deadwood 22–28 tiles out, beyond the camp's forage and wood radii) | `WorldGenConfig::wild_regions > 0` (v17 worldgen) | **yes** |
| 2 | Exposure damage [9, 5, 6] instead of [10, 4, 5] | `ExposureRule::damage_per_cold_hour` | yes (winter nights) |
| 3 | Hunger adds to the cold | `ExposureRule::hunger_weight` 0.5 | yes |
| 4 | Hardiness (frailty factor; `Hardiness` on founders and newborns) | `ExposureRule::use_hardiness` | yes |
| 5 | Regional winter fall (per-area accumulators, camp keeps v16's fall) | `FirewoodRule::regional_wood` and a non-empty `wood_spot_area` (needs regions) | yes (winter) |
| 6 | Windfall nearest the home first | `regional_wood` | no (interventions only) |
| 7 | Rain soaks the woodpile | `FirewoodRule::rain_soak_permille` | no (interventions only) |
| 8 | Founding markers, sticky membership, shared-bush supply 1/n, founders excluded from cold/meals/tethers/haul | `SocietyRules::fission.is_some()` or `Founding` present | no (no fission rule in v17_nofission; verified: each site in `movement.rs`, `agent_decision.rs`, `food_economy.rs`, `settlement_system.rs`, `stockpile_consume.rs`, `fire.rs` is gated on `fission.is_some()` or on a `Founding` component, which only `FissionSystem` inserts) |
| 9 | Formation scan ignores anchors of settlements no longer in `settlements`; retired hearths never anchor | ungated | no: before Pass 11 both maps always hold the same ids (SettlementSystem clears both on dissolution) and `retired_hearths` is filled only by `FissionSystem` |
| 10 | `exclusivity_moves`, `dissolutions` tallies | ungated | no (measurement only) |

## 6.2 The ablation

Test-only scenes in `sim-test/src/probe/ablation.rs` (`abl_*` = v17_nofission with one difference reverted, `fwd_*` =
v16 with one difference added; no `sim-core` constructor). Tuning seeds 0–31 × 120 days, Pass 11's flags, every
determinism twin passed. Y9 is computed with `evaluate_pass6.py`'s code (verified: `game_v16` 0.358 / 0.254 / 0.621
and `game_v17_nofission` 0.345 / 0.119 / 0.635 equal Pass 11's J6 output for the same seeds). Activity at 15:00 from
the new schema-12 clock keys `adult_act_*` (probe row keys, `pass12.rs`). Full table: `bisect.md` / `bisect.json`.

| Run | work_1500 | share of the gap (0.254 − 0.119 = **0.135**) | forage share of adults at 15:00 | idle share |
|---|---:|---:|---:|---:|
| game_v16 | 0.254 | — | 0.178 | 0.592 |
| game_v17_nofission | 0.119 | — | 0.128 | 0.656 |
| **abl_no_regions** | 0.254 | **1.000** | 0.182 | 0.590 |
| abl_v16_damage | 0.118 | −0.007 | 0.128 | 0.656 |
| abl_no_hunger_weight | 0.119 | 0.000 | 0.128 | 0.656 |
| abl_no_hardiness | 0.122 | 0.027 | 0.130 | 0.655 |
| abl_no_regional_wood | 0.119 | 0.000 | 0.128 | 0.656 |
| abl_no_rain_soak | 0.119 | 0.000 | 0.128 | 0.656 |
| abl_v16_exposure (2+3+4) | 0.104 | −0.107 | 0.128 | 0.656 |
| **fwd_regions** | 0.104 | **1.107** | 0.128 | 0.656 |
| fwd_regions_regional_wood | 0.104 | 1.107 | 0.128 | 0.656 |
| **fwd_regions_bushes_only** | 0.104 | **1.107** | 0.128 | 0.656 |
| fwd_regions_wood_only | 0.254 | 0.000 | 0.178 | 0.592 |
| fwd_v17_damage | 0.225 | 0.211 | 0.180 | 0.592 |
| fwd_hunger_weight | 0.254 | 0.000 | 0.178 | 0.592 |
| fwd_hardiness | 0.227 | 0.200 | 0.177 | 0.593 |
| fwd_exposure_v2 (2+3+4) | 0.254 | 0.000 | 0.182 | 0.590 |

**The decisive difference is the wild regions — precisely their bushes**: reverting them restores 100 % of the gap,
adding them to v16 causes 111 %, adding only their deadwood causes nothing. The exposure, rain and regional-wood
differences are within noise (the two forward exposure arms at 0.21 / 0.20 move the seed-median statistic but leave the
pooled 15:00 activity shares exactly at v16's; their combination `fwd_exposure_v2` is 0.000). No pair interacts: the
regions' effect is the same with or without regional wood, and with or without exposure v2.

**The activity that took the afternoon: foraging** (adults foraging at 15:00 0.178 → 0.128), replaced by idling
(0.592 → 0.656); wood gathering (0.031), hauling and drinking are unchanged.

**Mechanism.** The forage arm runs only while a settlement's food in hand is below its store target. With region bushes
the store is fuller from the first day (day 1 08:00: 50.5 vs 39.0 median; day 10: 408 vs 339) while forage trips and
haul deposits are equal at first: the food arrives through the incidental **Gather arm** (`CascadeArm::Gather`, scan
radius 8 around the agent, no forage trip counted) — wood gatherers working 16–18 tiles out and foragers at the edge of
the camp's 16-tile radius pick up food at full region bushes a few tiles beyond the camp's reach and carry it home. At
15:00 in spring–autumn the store holds its target in 93 % of rows (v16 82.5 %), so the forage arm is off (forage trips
over 120 days 809 vs 1,882; food harvested is the same, 12,288 vs 12,290). A second, smaller channel: once the camp's
bushes are bare, a hungry adult's meal target (`nearest_resource_tile`, the nearest food tile anywhere) is a region
bush (681 agent-hours seeking food at a region bush in 6 seeds × 30 days, 0 in v16; recorded with `wm_p0 --agents`
into `worldsim-wm-data/p12/j6diag/`), and food is "available" world-wide while any region bush holds food.

**Fix for v18 (Part 2, 7.5)**, checked here as the test scene `abl_local_food` (v17_nofission + the flag):
`AgentRules::local_food_availability` — a settlement member's food world is its catchment (formation tile ±
`catchment_radius` = 16): food availability, the meal target and the Gather arm's tile only count food tiles inside it.
Three versions were tried on the tuning seeds, each a separate run of the same scene name:

| Version | work_1500 | share of the gap restored | forage share at 15:00 |
|---|---:|---:|---:|
| availability only (`abl_local_food`, `probe_p1b`) | 0.123 | 0.034 | 0.134 |
| + meal target in the catchment (`abl_local_food2`, `probe_p1c`) | 0.111 | −0.054 | 0.137 |
| + Gather arm in the catchment (`abl_local_food3`, `probe_p1d`; the v18 flag) | 0.214 | **0.707** | 0.168 |

The Gather-arm channel carries the effect; the flag restores 0.71 of the gap. The remaining 0.29 is not explained
(hypothesis: ground meals of members while away from camp — 265 vs 180 per seed in `abl_local_food2` — and members not
yet in a roster; not pursued further).
