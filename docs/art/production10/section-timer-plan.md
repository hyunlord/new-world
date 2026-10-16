# Part 1 section-timer implementation plan

Planning only, before Part 0 completes. Inspected `scripts/ui/agent_renderer.gd`, `scripts/test/art/frame_cpu_probe.gd`, `frame_measure_session.gd`, `perf_frames_p10.gd`, and source-static test readers. No production edits or Godot/cargo execution in this task.

## Debug activation and disabled cost

Use an explicit diagnostic mode, default disabled, selected once before the renderer starts processing. Proposed capture-facing API: `configure_section_profile(enabled: bool)` and `section_profile_readout() -> Dictionary`. Reject enabling outside `OS.is_debug_build()`. Do not check environment variables, allocate timer arrays, duplicate readouts, call the clock, emit signals, format labels or invoke a no-op timer helper inside an unprofiled agent loop.

**Literal zero disabled overhead needs a structural choice.** A runtime flag checked at every checkpoint still adds branches, even when its clock calls are skipped. Do not describe that as zero cost. A static-false constant is only acceptable if the installed Godot compiler demonstrably removes its branches; that has not been verified here. Preferred strict solution: keep production `_process` and `_apply_drawn_positions` untouched by timing checkpoints; generate an opt-in diagnostic script from the current source with inserted checkpoints and load that script before instantiating the diagnostic scene. The ordinary renderer never loads the diagnostic script. The generator should fail closed on absent/ambiguous markers and record both source and generated hashes. This avoids maintaining a hand-copied renderer whose behavior can diverge. It satisfies off-by-default/no off-path runtime cost, but requires documenting that the debug flag selects an instrumented source variant rather than a per-frame branch.

A leader choosing simpler inline guarded instrumentation must explicitly record that residual boolean checks remain and prove their impact; it cannot claim the brief's literal zero-cost condition without further evidence. This is an implementation choice to resolve autonomously in DECISIONS before edits, not an owner question.

## Checkpoint map and attribution

Record integer microseconds in preallocated indexed counters; map indexes to names only in the out-of-bracket readout. Clock each actual boundary once. For nested spans, record inclusive parents separately; never sum parent and children as exclusive totals. Keep execution order, method names and the literal `SpriteLibrary.apply(sprite, sprite_id)` call.

| Output key | Exact boundary / purpose |
|---|---|
| `reads_usec` | `SimReads.read` for world info, agents, society, time; add separately if calls are disjoint. Activity context getter reads belong to its inclusive span; do not count twice in an exclusive total. |
| `checksum_talk_usec` | `_prepare_talk_cache`, `_snapshot_checksum_from`, and the checksum-changed interpolation refresh through `_lerp_accumulator = 0.0`. |
| `chill_ingest_usec` | `ingest_chill_snapshot(snap, tick)`, including MorningChill ingest. |
| `activity_context_usec` | `_activity_context()` inclusive. |
| `tile_contexts_usec` | Per-agent tile-context cache lookup and optional `_tile_context` call; also count misses. |
| `agent_setup_usec` | Agent ID/live map, interpolated position, `_ensure_agent`, movement bookkeeping, stationary seconds and render-position update, excluding look span. |
| `look_usec` | `_base_look`, life-stage lookup, `Activity.elder_look`, body derivation; body derivation can be a second accumulated interval. |
| `pose_usec` | Target/carried inputs, `_pose_moments.update`, resolve, idle elapsed, talk delta, facing, camp pose, meal and cold overrides. These retain delta and per-frame ordering. |
| `frame_choice_usec` | `frame_choice` plus sprite ID extraction and conditional `SpriteLibrary.apply`/empty-texture branch. |
| `variant_usec` | `SpriteLibrary.apply_variant` only. |
| `meta_usec` | Flip, sheltered visibility and per-agent metadata writes; retain ordering around children. |
| `children_usec` | `CarryLoad.apply`, `apply_founder`, `_update_activity_icon`. |
| `draw_rows_usec` | `draw_agents.append` allocation, outside children span. |
| `seats_usec` | `_camp_seats.update`; blocked tile construction reported separately as `blocked_usec`. |
| `footprints_usec` | Spread offsets, targets and merged frame/CarryLoad rectangles before declump. |
| `declump_usec` | `CampSeatsScript.declump` only, inclusive of its internal loops. |
| `easing_usec` | Apply declump result, exponential easing, rounding, seat/position metadata, outline call and construction of rounded positions. Never omit this stage because spatial work cached. |
| `walker_offsets_usec` | `CampSeatsScript.walker_offsets` only. |
| `walker_apply_usec` | Apply lateral offsets and associated metadata after helper returns. |
| `post_loop_usec` | Cold/shiver/breath, morning chill, outline, shadow and cue updates. |
| `retire_usec` | `_retire_missing(live)`. |
| `selection_usec` | `_update_selection_ring()`. |
| `causal_usec` | `_ingest_causal_events` final call, not silently folded into selection. |
| `total_usec` | Whole AgentRenderer callback, including clock/counter overhead in diagnostic mode. |

The checksum section calls `_base_look` only on changed snapshots: keep that work attributed to checksum refresh rather than counting it again as per-agent look. Do not move reads or split behavioral helpers merely to get a cleaner timer table. `FoundingGroups` is a child with its own process callback; parent's `_process` timing does **not** include it. Measure it separately with a diagnostic whole-callback bracket, and label it separately from the parent total. Node ablation of AgentRenderer plus children is a different metric.

## Frame-aligned output

`FrameCpuProbe` stamps priorities -1000/+1000 and publishes deferred. `Session.Runtime.next_frame()` resumes from that deferred completion, invokes `force_draw(false)` outside the process bracket, then reads frame metrics. Attach the section readout in `perf_frames_p10.gd::_sample` alongside FFI counters; do not serialize from the renderer's `_process`.

Proposed additive sample field:

```json
{"agent_sections":{"schema_version":1,"process_frame":123,"valid":true,"complete":true,"agent_count":500,"tile_context_misses":23,"instrumented":true,"unit":"usec","total_usec":46000,"exclusive_usec":{"reads_usec":100,"checksum_talk_usec":200,"chill_ingest_usec":100,"activity_context_usec":300,"tile_contexts_usec":600,"look_usec":1000},"founding_groups_usec":400,"clock_reads":12000}}
```

The abbreviated example is illustrative only; actual output must contain every named exclusive key, initialized to zero each profiled callback. Include remainder/unattributed time as a diagnostic, never clamp negative remainder to zero: negative values indicate nested/double accounting or invalid timestamps. An early return sets `complete=false` with an error reason; disabled or ablated renderers produce unavailable data, never a fabricated all-zero completed sample. Require `agent_sections.process_frame == sample.process_frame` just as the current FFI-counter check does. Preserve existing primary CPU definitions and metric keys; don't add section time to primary time, because it is already included in process time.

A diagnostic-only W500 frozen Z4 session: same seed 15100, tick1440, camera/viewport, window lock and quiet hold as the matrix; 30 warmup then 300 measured frames. No fixed FPS in frame-time mode. Record source/build hashes, flag, process-frame alignment, population/tick, clock resolution and timer-call count. Report median/p90/p99/max per section in `perf/agent_sections.md`. Use timers to rank work, not as uninstrumented H1–H4 evidence. Accepted-change alternating pairs and final matrix run without instrumentation; repeat diagnostic session after Part 1 for before/after section attribution.

## Source-static constraints

There are 22 test files mentioning `agent_renderer.gd`; count includes broad file-existence/source readers, not 22 guaranteed failures. Additive instrumentation that leaves production statements in place should need no semantic repin.

- `harness_p11_alpha_agent_renderer.rs:393`: `SpriteLibrary.apply(sprite, sprite_id)` literal and absence of state tints/scale boosts. `:410` requires `states: PackedByteArray = snap.get(...)`. `:438` onwards requires previous/current positions, `lerp_t`, `.lerp(..., lerp_t)`, `_move_duration` and world-info reads. Keep these statements, names and math.
- `harness_p14_alpha_agent_sprite_overhaul.rs:433`: exact agent-ID extraction, `draw_agents.append({"id": agent_id,`, post-loop ID extraction, cue call and cue geometry/color strings. Do not rewrite dictionary construction to time it.
- `harness_p14_epsilon_activity_trails.rs:194` and interaction cue tests extract named function bodies through the next `\nfunc ` boundary. Extracting `_process` work into renamed helpers may invalidate body-level checks; additive markers avoid that.
- Other source readers to inspect before moving helpers: `harness_p13_gamma_interaction_cues`, `harness_p14_gamma_click_inspector`, `harness_p14_zeta_zoom_adaptive`, `harness_d_phase_a_runtime_warnings`, `harness_p8_delta_memory_ui`, `harness_p9_delta_combat_ui`.
- `harness_art_p9_sim_reads.rs` scans protected raw getters/mutators across `.gd` files. An instrumented/generated script must retain `SimReads.read` and approved mutation routing; naming a diagnostic file does not exempt direct world getter calls. Do not relax scanner rules.

After implementation, inspect actual failing assertions and only re-pin moved equivalent semantics with one Art Pass 10 ledger row each. No source-static assertion should be changed merely because profiling changes a file hash. Validate generated diagnostic behavior against the original using running dumps and frozen captures, then validate ordinary off-path identity after any production edits.

## Prepared diagnostic implementation (offline checks only)

The strict off-cost choice is now implemented in new diagnostic files:

- `tools/art_pipeline/generate_agent_sections_p10.py`: indentation-sensitive unique markers in `_process` and `_apply_drawn_positions`; rejects missing/duplicate/reordered markers, unexpected return counts, diagnostic-name collisions and reinstrumentation. It verifies that removing only its tagged insertion lines restores every nonblank ordinary statement in original order. Emits SHA-256 for source, generated script, helper and generator in `OUTPUT.manifest.json`. Source files are never rewritten.
- `scripts/test/art/agent_section_clock_p10.gd`: 25 exclusive counters (all planned spans plus `other_usec`), whole callback, frame number, call count, agent/tile counts, early-return/incomplete status. Adjacent intervals telescope to total; timer/mark overhead is included in measured intervals and is not corrected away. `FoundingGroups` is explicitly unavailable, never included in parent total.
- `scripts/test/art/section_profile_loader_p10.gd`: debug-only opt-in installation on detached Main, validates source/output/helper hashes before replacing only the detached AgentRenderer's script; readout rejects stale probe frames.
- `tools/art_pipeline/test_generate_agent_sections_p10.py`: current-source statement preservation, all section indexes, both early returns and normal completion, removed markers, duplicate markers, new return and reinstrumentation rejection.

Generation command (generated output is a diagnostic artifact, not a production source replacement):

```sh
python3 tools/art_pipeline/generate_agent_sections_p10.py \
  scripts/ui/agent_renderer.gd \
  scripts/test/art/generated_p10/agent_renderer_profile_p10.gd
python3 tools/art_pipeline/test_generate_agent_sections_p10.py
```

The generator was exercised against the current source into `/tmp/agent_renderer_profile_p10.gd`, not installed in the project. Offline tests passed. Runtime parsing, loader replacement, visual identity and time sampling remain NOT VERIFIED until Part 0 completes. An existing future optimized renderer may legitimately fail the current marker map; inspect changed flow and revise markers explicitly before regenerating, never silently skip sections.

Harness integration sequence:

1. Explicit `--agent-sections` diagnostic flag must be opt-in and rejected outside debug builds. Instantiate main; keep it detached. Ordinary runs omit loader calls entirely.
2. Preload `section_profile_loader_p10.gd` in the diagnostic harness and call `install(main, "res://scripts/test/art/generated_p10/agent_renderer_profile_p10.gd", "res://scripts/test/art/generated_p10/agent_renderer_profile_p10.manifest.json")`. Abort and preserve the error if `valid` is false; do not fall back to uninstrumented timings. Copy the successful installation provenance into session metadata.
3. Only after successful install, perform the existing world setup, add main to the tree and follow the normal W500 frozen Z4 session setup. Do not change renderer priority or give it a second manual `_process` call.
4. After `await runtime.next_frame()`, set `sample["agent_sections"] = Loader.collect(main.get_node("AgentRenderer"), int(sample["process_frame"]))`. Abort classification on incomplete/invalid/misaligned samples. This readout allocation occurs after the CPU bracket.
5. Save raw samples and summaries under production10 evidence. Call the generated renderer only through the normal scene process surface. Restore/quit through existing runtime cleanup. Treat instrumented section results as diagnostic attribution, not H1–H4 timing evidence.
6. Before accepting the generator in live measurements, run the required pixel/running identity between generated and ordinary renderers and check every protected getter scan. No baseline or final benchmark should accidentally load the generated script.

No full launch harness was added: wiring it into existing measurement control flow before the baseline finishes risks changing active tooling. The two-method loader is the bounded integration surface; its Godot behavior is still unverified. Production scripts and scenes are unchanged by this subtask.
