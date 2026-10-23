# ADR 0016 — The simulation contract: reproducible, view-independent, measured

## Context
On 2026-10-21 the owner decided that the simulation is the product. Visuals will change again (perhaps to 3D);
the logic must stay reproducible under any renderer and must stay fast and small as it grows, using
mathematically effective methods rather than brute force. Before M1.5 the contract was only half-true: the sim
was Godot-free, deterministic and replayable from (seed, commands), but there was no save/load, no second
renderer, no standing stress benchmark at colony scale, no memory measurement and no spatial index (job
assignment scanned every candidate; 26 of 28 systems ran every tick). This ADR writes the contract down; every
later pass is held to it, and each clause names the gate step that checks it.

## Decision

**C1 Reproducibility.** The world state at any tick is a pure function of `(rules, seed, command log)`. Any build of the same commit replays a recorded `(seed, commands)` to identical fingerprints (probe summary, raw rows, chronicle). A later commit that changes the colony's behaviour says so in its CHANGELOG entry and re-captures the colony fixtures; the `baseline` world never changes (identity gate). Saved games are a serialization of that state with a format version; `save → load → tick` equals `tick` without the save (7).

**C2 View independence.** `sim-core`, `sim-systems`, `sim-engine` know nothing of pixels, tiles sizes, sprites, cameras, frames or Godot. `sim-bridge` is the only crate that knows Godot, and it only marshals. Renderers consume the documented snapshot schema (`docs/ARCHITECTURE.md` §4 + the schema file of 8.1) and the chronicle/causal log; nothing a renderer needs is computed in GDScript from undocumented state. Proof: a second renderer exists (the text renderer, 8.1) and the GUI test and the text renderer agree on positions.

**C3 Performance budget.** Measured every pass by the standing benchmark (6.1) on the colony world, 192×192, at N = 200 / 1,000 / 2,000 agents: tick mean and p99, per-system share, bytes per agent and per tile. Hard ceilings: at N = 1,000 tick mean ≤ 5 ms; at N = 2,000 ≤ 10 ms; memory ≤ 4 KiB per agent and ≤ 96 B per tile beyond the fixed cost. Regression ceiling: any benchmark statistic worse than the ledger's previous pass by more than 10 % fails the gate unless the pass's CHANGELOG justifies it and the ledger line says so. No system may do O(agents × candidates) work per tick without a spatial index; no per-tick scan of all tiles.

**C4 Mathematical method.** Prefer closed-form state over per-tick accumulation (store `since_tick` + rate; evaluate on read, as `Life::age_ticks` does). Prefer event-driven work (dirty sets, timers, intervals) over polling. Evaluate curves (decay, probabilities, thresholds) from precomputed integer tables, not transcendental calls. New state uses integers or fixed-point (permille / per-64k); existing `f32` needs are grandfathered until a dedicated pass converts them, and no new `f32` state is added to the sim crates. Randomness only through keyed draws. Maps iterate in key order. Tie-breaks are `(y, x)` then id.

**C5 Structure.** Logic lives in the system that owns the component; a system's header lists reads/writes/events and the table in ARCHITECTURE is generated from it. Files over 1,200 lines are reported every pass; the decision tick function is not extended (M2 dismantles it as the job layer takes over).

Each of C1–C5 names the gate step that checks it (after this pass: identity, replay, colony determinism, save/load round trip, benchmark + ledger, text-renderer agreement, docs check for the schema file).

### Clause → gate step (`tools/v8/gate.sh`)

| clause | gate steps |
|---|---|
| C1 | `identity` (baseline never changes), `determinism`, `colony_determinism`, `replay`, `save_roundtrip` |
| C2 | `renderer_agreement` (text renderer = GUI positions), `schema_docs` (`docs/v8/snapshot_schema.md` against the bridge), `bridge_api` |
| C3 | `bench` (standing benchmark + `docs/perf/PERF_LEDGER.md`; > +10 % against the previous ledger line fails; ceilings reported PASS/MISS; since M5.5 the memory ceilings on the allocator counter and the RSS outer bound, median of 5 build-only repeats) |
| C4 | `forbidden_math`, `identity`/`determinism` (keyed draws, key order) |
| C5 | `docs` (system table generated from headers), the pass report's size table |

The final mapping, including any step a pass had to drop, is recorded in the pass report (M1.5:
`docs/v8/m1_5/M1_5_REPORT.md`).

## Alternatives considered
- Leave the contract implicit in the skills: the skills already hold most rules, but nothing tied them to gate
  steps, so a pass could quietly break one (no memory figure existed, no second renderer proved C2).
- Hard performance ceilings from this pass: there was no colony-scale measurement yet; M1.5 reports the ceilings
  as PASS/MISS and makes them hard in M2 only if they are met (otherwise M-PERF goes first).
- Convert every `f32` need now: a large behaviour change across the baseline, which must stay bit-identical;
  deferred to M-PERF and grandfathered here.

## Consequences
- Every pass appends a line to `docs/perf/PERF_LEDGER.md` and is compared with the previous line.
- New state is integer or fixed-point; new randomness is keyed; new candidate searches use the spatial index
  (`sim-core/src/spatial.rs`).
- A renderer (Godot today, maybe 3D later) reads only documented snapshot rows; the text renderer
  (`worldsim_probe --ascii`) is the standing second consumer.
- A saved game is part of the contract once it exists; a format change bumps its version.


## Amendment M5.5 — the memory instrument (2026-10-24; M5.5 goal §3.4, DECISIONS D3)
- **The C3 per-agent and per-tile memory ceilings (4 KiB, 96 B) are judged on the allocator counter**
  (`sim_test::alloc::Counting`, live heap bytes after the build; deterministic) when the run has it, and on RSS
  otherwise. Reason (M5 D19): RSS on macOS is bimodal — one M5 binary read 83.0–117.5 B per tile across sessions and
  the M4 base missed its own 96 B ceiling once — while the allocator read 63.1 B per tile and 1,871 B per agent for
  both binaries in every session.
- **RSS keeps an outer bound**, because fragmentation and allocator overhead are real memory: every `bench` run also
  builds N = 200, N = 2,000 and the 64² scene in `RSS_REPEATS` = 5 separate processes, pairs them by repeat, and the
  median of the 5 RSS slopes must be ≤ 5,120 B per agent and ≤ 128 B per tile (`rss_outer` in `bench.json`, ceilings
  `rss_per_agent_le_5120` and `rss_per_tile_le_128`). It is reported every pass (`bench_ledger.py check` prints it,
  the gate summary carries it) and a MISS fails the gate like any C3 ceiling.
- **Gates never change their instruments mid-run.** M5 moved the ceilings to the allocator by stopping a running gate
  (D19); the change was defensible but the contract did not say it. A gate that finds an instrument wrong is stopped,
  the finding recorded in the pass's DECISIONS, the instrument changed in its own commit, and the gate started again
  from step 1.

## Date
2026-10-21

## Source
docs/v8/m1_5/M1_5_GOAL.md §3 (the contract text, verbatim above), §6.1, §7, §8.1; docs/v8/m5_5/M55_GOAL.md §3.4 and docs/v8/m5_5/M55_DECISIONS.md D3 (the M5.5 amendment); docs/decisions/0012-orders-as-recorded-commands.md; docs/decisions/0014-two-worlds-baseline-and-colony.md
