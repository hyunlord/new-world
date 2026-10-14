# WorldSim performance budget (Revitalize Pass 15)

**Timed batches and load.** Section 1 asks for a 1-minute load < 4 before every timed batch. This Mac carried 2–13 of foreign load throughout (processes this run did not start). The overhead check (start load 3.33), the natural sweep (2.95) and the supplementary sizes (2.09) started below 4; the fed (13.41), repeatability (9.85), raw (9.85) and map (7.02) sweeps started above it inside one gated batch and are contended in that sense, although their JSON says `contended: false` (the gate was checked once for the batch; review round 1, M1). P2's fed n = 500 runs and the repeatability runs were therefore rerun with one gate each (section 1.1, "recheck").

Measured 2026-10-15 on one Mac — a **MacBook Pro, Apple M4 Max (14 cores), 36 GiB, macOS 27.0, rustc 1.93.1**, release build — at code commit `8f052142` (behaviour-identical to `f5373ab2`: I1–I3 equal, METRICS Pass 15 Part 2). The simulation is single-threaded. Every number below is this machine's; for slower machines use a factor-2 margin (`N_max / 2`). Tables and per-run files: `docs/revitalize/probe/p15-4_scaling/scaling.md` (+ `scaling.json`), runs in `p15-1_natural/`, `p15-2_stress/`, `p15-3_map/`. The profiler's own cost is −0.22 % (natural seed 15000, 30 days, 3 + 3 alternating runs, same fingerprint each time), i.e. within noise.

**Frame budget (4.3).** 60 FPS = 16.67 ms. The simulation may use half a frame at 4× speed (2 ticks per frame on average): **B1** mean tick ≤ 4.0 ms; **B2** p99.9 of tick times ≤ 8.0 ms (also the maximum and the ticks > 16.67 ms); **B3** one call of every per-frame snapshot getter (Rust side), summed, ≤ 2.0 ms. Target population 500 alive.

## 1. Today

### The P table

| # | question | rule | measured | verdict |
|---|---|---|---|---|
| P1 | the game as it is (`game_v19`, natural, 32 worlds, 120 days) | every world meets B1 and B2 | worst world mean 0.129 ms, worst p99.9 0.304 ms, max 1.935 ms; no tick > 8 ms | **PASS** |
| P2 | 500 people (fed stress, n = 500, 8 worlds, 2 days) | median world meets B1; pooled p99.9 meets B2 | median mean **0.303 ms**; pooled p99.9 **0.634 ms** (22,560 ticks); per-world max 0.785–0.827 ms; 0 ticks > 16.67 ms | **PASS** |
| P3 | the UI's reads at 500 people | B3 on single calls | **0.026 ms** on both branches (art figure partial: 2 art-only per-frame getters untimed); ×callers 0.099 ms (this branch), 0.285 ms (art branch) | **PASS** |
| P4 | headroom | N_max reported; comfortable if ≥ 1,000 | every size passes: **N_max ≥ 2,000**; fitted line reaches 4.0 ms at **5,142** alive (extrapolated; the supplementary sizes measure ≈ 4,350 for B1 and ≈ 3,600 for B2, see 1.6) | **comfortable** |
| P5 | growth | clean if no system whose natural slope is superlinear or worse has ≥ 5 % at n = 500 | no system has a superlinear natural slope; only `AgentMovementSystem`'s stress slope (1.21, 1.4 % of tick time) crosses 1.2 | **clean** |

### 1.1 Tick time per sweep

| sweep | worlds | mean alive | median mean tick | pooled p99.9 | worst p99.9 | max | µs per agent-tick |
|---|---:|---:|---:|---:|---:|---:|---:|
| natural `game_v19`, 120 days | 32 | 48.0 | 0.081 ms | 0.233 ms | 0.304 ms | 1.935 ms | 1.72 |
| stress fed n = 50 | 8 | 50 | 0.038 ms | 0.089 ms | 0.105 ms | 0.130 ms | 0.76 |
| stress fed n = 500 | 8 | 500 | 0.303 ms | 0.634 ms | 0.712 ms | 0.827 ms | **0.61** |
| stress fed n = 1,000 | 8 | 1,000 | 0.673 ms | 1.508 ms | 1.804 ms | 1.997 ms | 0.67 |
| stress fed n = 2,000 | 8 | 2,000 | 1.622 ms | 3.624 ms | 4.344 ms | 4.746 ms | 0.81 |
| stress raw n = 2,000 | 8 | 1,688 | 1.404 ms | 2.641 ms | 2.779 ms | 3.681 ms | 0.84 |
| map 64 / 128 / 256 (founders as `game_v19`) | 8 each | 24 | 0.027 / 0.030 / 0.037 ms | — | — | 0.12 / 0.10 / 2.16 ms | — |

No tick of any planned run exceeded 8 ms (so none exceeded 16.67 ms). The natural world's per-agent cost (1.7 µs) is higher than the stress world's (0.6 µs) because fixed per-tick work (influence grid, settlement and causal-log bookkeeping) is spread over ~48 people instead of 500. Repeatability (seed 15100, n = 500, fed, three runs, each batch gated at load < 4): 0.3084 / 0.2987 / 0.2998 ms, coefficient of variation **1.8 %** (the contended first set: 0.3035 / 0.3034 / 0.3062 ms, 0.5 %). **Recheck of P2** (review round 1, M1): the eight fed n = 500 runs rerun behind their own gate (start load 2.94) give median mean 0.2971 ms (−2.1 % against the contended sweep), pooled p99.9 0.608 ms, max 0.823 ms, and the same end fingerprint on every seed — P2 passes either way. Start loads of every batch: `scaling.md`, "Timed batches".

### 1.2 Who costs what

| system | natural (pooled) | n = 50 | n = 500 | n = 2,000 |
|---|---:|---:|---:|---:|
| AgentDecisionSystem | 53.6 % | 54.4 % | 60.0 % | 69.4 % |
| SettlementSystem | 21.0 % | 20.5 % | 21.9 % | 16.6 % |
| MemorySystem | 10.2 % | 6.4 % | 7.0 % | 5.8 % |
| StockpileConsumeSystem | 1.2 % | 2.1 % | 2.4 % | 1.8 % |
| CombatSystem | 1.4 % | 1.9 % | 1.7 % | 1.5 % |
| AgentMovementSystem | 0.8 % | 0.9 % | 1.4 % | 1.3 % |
| StarvationSystem | 2.2 % | 1.7 % | 0.7 % | 0.4 % |
| InfluenceUpdateSystem | 1.8 % | 3.1 % | 0.5 % | 0.1 % |
| StaleSeekTargetSystem | 1.2 % | 1.1 % | 0.6 % | 0.4 % |
| (the other 17 systems) | < 1.2 % each | | | |

Three systems are 81–92 % of every tick at every size (natural 85 %, n = 50 81 %, n = 500 89 %, n = 2,000 92 %); `AgentDecisionSystem` alone is 54–69 % and its share grows with the population.

### 1.3 How cost grows (slopes, 95 % intervals)

Whole tick: natural **0.94** [0.92, 0.96] (per day, over ≈ 15–150 alive), stress fed **1.06** [1.04, 1.09], raw 1.08 [1.05, 1.11]; against map tiles 0.12 [0.09, 0.15] — the cost is people, not area.

| system | natural slope | stress slope (fed) | class |
|---|---|---|---|
| AgentDecisionSystem | 0.98 [0.96, 1.00] | 1.13 [1.10, 1.16] | linear or better |
| SettlementSystem | 1.08 [1.06, 1.10] | 0.98 [0.97, 0.99] | linear or better |
| MemorySystem | 0.94 [0.92, 0.96] | 1.03 [1.01, 1.04] | linear or better |
| AgentMovementSystem | 0.82 [0.80, 0.83] | 1.21 [1.19, 1.23] | natural linear; stress superlinear by 0.01 (not crowding-driven: +0.39 < 0.5) |
| every other system ≥ 1 % somewhere | 0.05–0.95 | 0.10–0.99 | linear or better |

The natural slope pools days, so it also absorbs time trends (more buildings and settlements, winter firewood). The stress slope is a density-confounded upper bound: every founder starts on the camp ring and the first settlement admits everyone in radius. **No system is near-quadratic and none is crowding-driven.** The static audit (`scaling.md`, "Static audit") finds no O(A²) loop on the `game_v19` path: the mild superlinearity is log-factor bookkeeping (BTree maps and sorts of every agent rebuilt each tick in four systems) and same-tile peer scans that grow with crowding.

### 1.4 Slow ticks

- **Stress worlds:** the slowest ticks are the morning wake-up at 06:01 (and 14:01, 21:00 clock edges): every agent leaves night rest and re-decides at once, so the tick takes ≈ 2.7–2.9× the mean, nearly all of it in `AgentDecisionSystem` (n = 500: 0.70 of 0.83 ms; n = 2,000: 4.3 of 4.7 ms). None of these is a daily-interval system — the work happens inside interval-1 systems at clock boundaries.
- **Natural worlds:** isolated 0.5–1.9 ms single ticks against a 0.08 ms mean — one system each time: `AgentDecisionSystem` (474 of 640 kept slow ticks), `SettlementSystem` (74; settlement founding/membership changes), `FissionSystem` (32; a village split), `MemorySystem` (28). The worst, 1.935 ms (seed 15013, day 112 13:36), is 12 % of a frame. Some of these single spikes may include OS preemption (foreign load ~3–13 on this machine during the runs); none comes near 8 ms.
- **Map 256:** one 2.16 ms `ConstructionSystem` tick (seed 15104, day 1 16:32).

### 1.5 The UI's reads (Rust side, n = 500 fed)

| getter (Rust body) | one call | calls per frame, this branch | calls per frame, art branch |
|---|---:|---:|---:|
| `get_settlement_snapshot` (`collect_settlement_snapshot` + wood targets) | 17.3 µs | 3 | 10 + 2H (H = hearths) |
| `get_agent_snapshot` (rows + ages + colds) | 6.0 µs | 6 | 9 |
| `get_resource_snapshot` | 2.7 µs | 1 | 5 |
| `get_tile_causal_history` (one tile / all T agent tiles) | 0.08 µs / 4.1 µs | 2T (T = 69 at n = 500) | 2T |
| `get_building_snapshot`, `get_construction_snapshot`, `get_terrain_snapshot` | ≈ 0.1 µs | 1, 2, 0 | 4, 4, 1 |
| `get_world_info`, `get_time_snapshot`, `get_tallies`, `get_recent_deaths` | < 0.05 µs (clock resolution) | 1, 2, 2, 1 | 2, 13 + 5H, 2, 3 |

Single calls summed: **26 µs** (limit 2,000 µs). Per frame with today's callers: **99 µs** on this branch, **285 µs** on the art branch (H = 1 in the stress world; with several villages H grows) — plus `get_society_snapshot` (5 per frame) and `get_intervention_state` (4 per frame), which exist only on the art branch and could not be timed here. The Godot-side `*_to_dict` conversions are not included (they need a Godot runtime; the art line measures them, 3). Full call-site inventory: `scaling.md`, "Per-frame caller inventory".

### 1.6 How many people fit (N_max)

- Largest measured size whose median fed mean tick ≤ 4.0 ms: **≥ 2,000** (every pre-registered size passes; n = 2,000 uses 1.62 ms, 41 % of the budget).
- The fitted whole-tick line (stress fed, mean alive ≥ 80) reaches 4.0 ms at **≈ 5,100 alive** — an extrapolation beyond the largest size.
- With the factor-2 margin for slower machines: **≥ 1,000** measured, ≈ 2,600 extrapolated.
- **Supplementary sizes (outside the pre-registered P table; seeds 15100–15107, fed, 2 days, gated batch started at load 2.09, not contended):**

  | n | median mean tick | pooled p99.9 | max | ticks > 16.67 ms | µs per agent-tick |
  |---:|---:|---:|---:|---:|---:|
  | 3,000 | 2.54 ms | 5.91 ms | 9.17 ms | 0 | 0.85 |
  | 4,000 | 3.65 ms | **9.52 ms** | 14.47 ms | 0 | 0.91 |
  | 6,000 | **5.72 ms** | 16.79 ms | 30.69 ms | 24 | 0.95 |
  | 8,000 | 8.17 ms | 27.11 ms | 52.32 ms | 35 | 1.02 |

  Measured, the mean tick reaches 4.0 ms at **≈ 4,350 alive** (log-log between 4,000 and 6,000; the fit over all fed sizes, slope 1.12 [1.10, 1.13], says ≈ 4,480) — close to the pre-registered extrapolation's 5,142 but lower, because the cost per person keeps rising (0.61 µs at 500 → 1.02 µs at 8,000). **B2 fails first:** the pooled p99.9 crosses 8 ms between 3,000 (5.9 ms) and 4,000 (9.5 ms), ≈ **3,600** by log-log interpolation, because the 06:01 wake-up spike grows faster than the mean (p99.9 / mean 2.3 at 3,000, 2.6 at 4,000, 2.9 at 6,000, 3.3 at 8,000). From n = 6,000 single ticks exceed 16.67 ms (dropped frames at any speed). **Both budgets together hold up to ≈ 3,000–3,600 people on this Mac (≈ 1,500–1,800 with the factor-2 margin)** — all in the pessimistic one-camp stress world.

So on this Mac the current code simulates the 500-person target in **0.30 ms per tick — 7.6 % of B1** and about 3.6 % of a 60 FPS frame at 4× speed.

## 2. The rule for later game passes

Copy this into every future pass goal that adds or changes a simulation system:

> **Performance rule (from Pass 15).** Before and after the change, on the same Mac, run `rust/target/release/worldsim_perf quick --out <dir>` (release build; seed 15100, n = 500 stress world, 64 × 64, 1 day, fed) through `tools/revitalize/perf_batch_p15.sh` (load < 4, no Godot / cargo running), and report:
> (a) **the change in mean tick at n = 500 fed** (`tick_summary.mean_ms`). The pass **fails** if the mean tick rises by more than **15 %**, or rises above **4.0 ms**. (Pass 15 reference on this Mac: ≈ 0.30 ms — 0.297 ms median over 8 seeds uncontended; run-to-run variation 1.8 %, so the 15 % line is about eight times the noise.)
> (b) **the new system's stress slope** from a three-size stress run, `worldsim_perf stress --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed`, fitted as in `tools/revitalize/perf_p15.py` (log mean µs per call on log mean alive), compared with this pass's stress slopes of comparable existing systems **on the same design** (seed 15100, n = 100 / 500 / 1,000; `scaling.md`, "Reference slopes"): decision 1.06, settlement 0.96, memory 0.98, movement 1.23, combat 0.93, stockpile consumption 0.99; whole tick 1.00. (With three points the 95 % intervals are wide — e.g. decision [0.39, 1.73] — so compare slopes, not intervals; the eight-seed, five-size slopes of section 1.3 are the precise reference.) A near-quadratic slope (≥ 1.7) **fails** unless it is no steeper than the crowding-driven slopes measured here — Pass 15 measured **none**, so in practice any near-quadratic system fails unless the pass's goal accepts it explicitly.
> The per-system table of the quick run's JSON (`systems[].share_of_tick_time`) names the new system's share; a new system above 10 % of tick time at n = 500 should say why.

## 3. For the art line

1. **Measure frame times on a crowded world:** call `WorldSimNode.reset_world_stress(seed, 500)` (debug `#[func]`, benchmark only; agents clamped to 1..5,000) and record frame times at 1×, 2× and 4×. The simulation side is ≈ 0.3 ms per tick at 500 people, so a frame budget miss at 500 would be the renderer's or the GDScript reads', not the tick's.
2. **Read each per-frame getter once per frame and share the result.** Today's duplicate callers (`scaling.md` inventory): on the art branch `get_settlement_snapshot` 10 + 2 per hearth (17 µs each in Rust before the dictionary conversion — the largest single item, ≈ 0.2 ms per frame at one hearth), `get_agent_snapshot` 9 (6 µs each, plus a 500-row dictionary each time), `get_time_snapshot` 13 + 5 per hearth, `get_resource_snapshot` 5, `get_society_snapshot` 5, `get_building_snapshot` 4, `get_construction_snapshot` 4, `get_intervention_state` 4, and `get_tile_causal_history` twice per distinct agent tile (2T = 138 calls per frame at 500 people). On this branch: `get_agent_snapshot` 6, `get_settlement_snapshot` 3. One cached snapshot per frame (or per tick change) removes most of the Rust cost and, more importantly, the repeated Godot dictionary building, which this pass could not time.
3. **Rust-side costs** to expect per call at 500 people: settlement 17 µs, agent 6 µs, resource 2.7 µs, all tiles' causal history 4 µs, everything else < 0.2 µs (table 1.5).

## 4. Next optimisation candidates (share × stress slope at n = 500)

None is needed for the 500-person target on this machine. In order of share × slope:

1. **`AgentDecisionSystem`** (60 % × 1.13 = 0.68) — `sim-systems/src/runtime/decision/agent_decision.rs`: ~7 per-tick BTree maps/sets of every member/child/elder rebuilt from scratch (L612–652, L763–765 and L794–796 build `member_of` twice, L817–829) and 10–15 BTree lookups per agent (L1020–1025, L1076, L1499–1567); three per-settlement full-world scans with sorts (L766, L799–800); two full collect-and-sort passes (L998–1001, L2179–2183); same-tile peer `min()` scans that grow with crowding (L1106–1114, L1135–1143, L1284–1288). Remedy: one shared per-tick agent/member index and a per-tile peer bucket.
2. **`SettlementSystem`** (22 % × 0.98 = 0.21) — `runtime/settlement/settlement_system.rs`: `resolve_exclusivity` sorts members and allocates one `Vec` per member every tick even with one settlement (L341–361); `sync_membership` builds and sorts the in-radius list and then breaks at once because the cap is reached (L288–302); `run_formation_scan` collects and sorts all positions it then ignores under the hearth anchor (L415–417); `ingest_community_history` scans every causal-log tile (L588–617). Root cause of M = A in the stress world: the first settlement admits everyone in radius without the `SETTLEMENT_MAX_POP` cap (L480–485) — a rule question, not a performance fix.
3. **`MemorySystem`** (7 % × 1.03 = 0.07) — `runtime/memory/memory_system.rs`: every agent × memory entry decayed each tick with a linear `recalled.any` scan (L109–120); two full causal-log scans per tick (L88–98, L124–140; `CausalLogStorage::lookup` is linear, `sim-core/src/causal/storage.rs` L99). Remedy: a per-tick event list and an `EventId → tile` index in `CausalLogStorage` (also helps 1 and 2).
4. **`AgentMovementSystem`** (1.4 % × 1.21 = 0.02) — `runtime/agent/movement.rs` L189–205 rebuilds a `BTreeMap<AgentId, home>` of every member each tick (the source of its 1.21 stress slope); the shared index of 1 removes it.
5. **UI reads** (outside the tick) — the duplicate getter calls of section 3.2, cheapest to fix and the only item the player's frame rate is likely to notice at 500 people.

Part 5's optional fix was not triggered (P2 passes; no system is near-quadratic), so no candidate was attempted in this pass.
