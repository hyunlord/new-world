# Part 3 measured optimization plan

Initial post-Part1 W500 ablation: `perf/p3-initial.json`, source session retained externally. The 150-frame off / neighbouring on blocks rank AgentRenderer (46.6435 ms), BuildingRenderer (1.3675 ms), NatureRenderer (1.1000 ms). This session was contended; keep its raw evidence and negative rows. Selection follows these measured rows, not baseline rankings.

## Row 1: AgentRenderer

1. Preserve regression behavior before editing: add a focused coordinate-neighbour cache test against the original ordered scan, covering ties, coincidence, self exclusion, no neighbour, ordered-array and length changes, in-place same-frame mutations, lazy new indices and reset. Existing animation, facing, camp, elder, selection and causal tests remain unchanged.
2. Cache only the pure nearest talking partner vector by row index while exact copied coordinate-array contents remain unchanged. Compare inputs once per process invocation after existing parallel-array validation, never once per agent. Keep direct `_talk_delta` behavior available for reference/test callers.
3. Reuse immutable base appearance by agent ID and derive life-stage appearance without mutating it; retire/reset caches. Reuse tile-context calculations only within one process invocation. Do not persist activity context because seat-blocked data is mutated by drawn-position work.
4. Keep all frame delta, interpolation, idle timers, pose moments, chill/cold, cue expiry, selection and facing evaluation running as before. No tick-only gating of animation.
5. Run focused regression tests when no timing hold is active, then all six fixed-frame B captures and exact RGBA comparison against retained A/A-prime. Record a fresh W500 ablation cost using the unchanged driver before committing/pushing this row.

## Rows 2 and 3

Read BuildingRenderer and NatureRenderer dependencies independently, but implement each only after the previous row's identity verdict. Write the concrete cache boundary and regression evidence here before editing. Repeat identity and 150-frame cost evidence after each row. Preserve the brief's time-budget drop order if required measurements are threatened.

No shader, asset, localization, simulation parameter, process priority or rendered output changes are authorized.

## Row 1 result

The pre-edit semantic test, post-edit cache test, ten existing focused tests and strict parser/FFI check passed. All six fresh B captures have exactly zero RGBA differences; independent visual review opened P1/P5 and recomputed all six. AgentRenderer's measured ablation cost changed from46.6435ms to45.6475ms (150-frame off blocks with300 neighbouring reference samples). The initial session was contended and the new session was not, so this modest difference is an observation rather than an isolated causal speedup claim. The retained cache removes proven repeated scans without changing animation; AgentRenderer remains the dominant cost. Evidence: `gates/p3-agent/`, `identity/p3-agent-attempt1/`, `perf/p3-agent*`.

## Row 2 concrete boundary: BuildingRenderer

Cache only SHELTER_COLD.sleeper_counts. Compare exact duplicated agent columns xs/ys/states/target_kinds/colds and building columns xs/ys/kinds, including presence and packed type, plus world identity. Reset in _reset_cold. Keep cold apply, frost fade, breath, tint, elapsed flame animation, resource updates and label processing live. Same-tick/in-place edits must invalidate through content comparison, not tick or hash. First add regression coverage against the uncached function for repeated inputs with advancing delta, each relevant column mutation, missing/wrong-type columns, shelter insertion/removal, and reset. Run existing shelter_cold,structure_states,every_hearth,ysort tests. Then six exact identity captures and a fresh alternating cost session, comparing to row1's unchanged BuildingRenderer cost under the same protocol.

## Row 2 result

The56-reference behavior test passed before and after; cache-reuse assertion, four existing focused tests and strict42-file/FFI checks passed. All six B frames remain exactly identical; independent review opened P2/P6 and decoded every framing. BuildingRenderer cost changed from0.2815ms in the preceding row1 session to0.0800ms in the row2 session, both admitted noncontended. Initial selection cost1.3675ms came from the earlier contended session and is retained separately. The modest measured reduction does not remove the dominant AgentRenderer cost.
