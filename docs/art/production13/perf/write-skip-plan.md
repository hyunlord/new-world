# Candidate (b): conservative property-write skip preparation

Status: implementation preparation only; NOT_IMPLEMENTED, NOT_MEASURED. Inspected HEAD `c1b1358e5d443f2eff11733b0ca8136df32a7d00` during Part 0. No product code or runtime session changed for this document. Re-read current source before Part 1 implementation. The decision remains contingent on GOAL §5.1, not on this plan.

## Boundary and invariants

Current exact write span is `scripts/ui/agent_renderer.gd:263–303` (line numbers drift): from `var sprite_id` and conditional `SpriteLibrary.apply`, through `apply_variant`, body metadata, `CarryLoad.apply`, `CarryLoad.apply_founder`, `_update_activity_icon`. Decisions before that span stay live, including interpolation, movement, stationary and idle timers, `_pose_moments.update_into`, daily/porter resolution, facing, cold pose, frame choice and daily icon decoration.

There are two statements interleaved inside that span that MUST move outside the conditional write guard without changing their per-frame ordering relative to consumers:

- `_selection_index.observe(entity_bits, agent_id)` remains live, because `begin_frame()` empties its lookup every frame.
- `activity_oracle_inputs` metadata remains freshly written whenever `_activity_oracle_enabled`; its animation time, previous facing, movement, pickup/meal, chill alpha and parent visibility must never become a cached diagnostic.

`_porter_lines.routes.append` is after the span and stays live: routes are cleared at the start of each process. `draw_agents.append`, `_apply_drawn_positions`, Cold, MorningChill, drawn-position metadata, settlement/hover outline, shadows, cues, retire, selection ring, link hover and causal ingest all stay live. Never early-continue the agent loop on a key hit.

## Owned exact key

Store one owned value per agent ID, with sprite instance identity and reset generation. Use explicit primitive fields in an Array or Dictionary and exact Variant equality, never approximate floats, a rounded string, or a lossy hash. Do not store the borrowed `moments`, `look`, `choice`, `daily`, `porter`, `tile` or full snapshot objects as the key. Building a fresh primitive-only value avoids deep copying the entire world and preserves the intended savings.

| Key group | Values to include | Why |
|---|---|---|
| Identity | agent ID, entity bits, sprite instance ID | Agent IDs may recur across reset; a pooled sprite is new ownership. |
| Frame | chosen `sprite_id`; chosen `view`, `activity`, `fallback_kind`, `qa_status` with the exact existing default values | Frame application and explicit choice metadata. Atlas geometry/page/anchor is immutable within one candidate session and determined by ID; invalidate cache when frame catalog is rebuilt. |
| Variant | exact `look.tone`, `look.hair`; computed `body` | `apply_variant` consumes tone/hair and the chosen frame's page. `look.phase` is already resolved into the chosen frame and must not substitute for it. |
| Body write metadata | `sheltered_sleep`, `daily_kind`, `arrived`, owned ordered `overrides`, `entity_bits`, body, tone/hair, tile x/y, activity, pose, `str(choice.get('activity', pose))` for cold_pose, base_pose, moving, facing, fallback_kind, qa_status; schema constant 1 | Every explicit metadata value in the guarded span, including duplicate derived fields, must match. Preserve exact types and default expressions. |
| Visibility | `_capture_visible`, computed `sprite.visible = capture && !sheltered_sleep && !sprite_id.is_empty()`, parent `is_visible_in_tree()` | Required capture and parent visibility sensitivity; do not confuse local visible with inherited visible. |
| Carry | effective carry argument (`pose` for a nonempty porter row, else activity), chosen frame ID, final flip, final local visibility, sprite z_index | `CarryLoad.apply` derives load metadata, swaps body texture, hides/reuses its child, and records `load_z_index = sprite.z_index + layer.z_index`. |
| Founder | `founders.has(agent_id)`, activity, exact carried-food float, exact carried-wood int, base_pose, final visibility/flip, chosen frame geometry identity | Drives suppression (`not_founder`, `hidden_agent`, eat/asleep, real_load) and anchor position. Do not reduce actual food/wood inputs to booleans for this conservative trial. |
| Icon | `choice.activity_icon`, `choice.fallback_kind`, camera zoom.x (default 1), porter presence, porter kind and stage if present, final local and inherited visibility, chosen frame offset identity | Daily icon and porter paths differ. Porter stage changes opacity; zoom changes porter scale and daily visibility. Body offset determines icon anchor. |
| Generations | reset/frame-catalog, language, capture visibility and candidate enable-generation | Force first write after lifecycle changes, even if semantic fields happen to compare equal. |

If retaining composite values instead of primitive fields: deep-duplicate `overrides`, `variant`, nested choice `rect`/`anchor`, and any dictionary containing arrays before storing. `_pose_output` is deliberately reused and overwritten for the next agent (`agent_renderer.gd:217`); retaining it would corrupt every cache entry. `_base_looks` is cached; `FrameChoices.daily_pose` explicitly returns borrowed data (`agent_frame_choices_p10.gd:88`), while `choose` only shallow-duplicates an atlas dictionary. `with_daily_icon` mutates the chosen dictionary. Copy after all decoration, not before. Resources such as textures/materials are shared, not made immutable by `Dictionary.duplicate(true)`; use catalog generation plus frame IDs rather than treating a duplicate resource reference as an owned render snapshot.

## Later mutations and why naive equality is insufficient

1. `CarryLoad.apply` changes `sprite.texture` to the carry atlas without changing `sprite_id` (`carry_load_layer.gd:17`). A key must cover effective activity and carry state, not sprite_id alone. On transition out it restores the normal page. Frame apply retains its existing conditional semantics; do not unconditionally reapply every sprite on a miss.
2. `SpriteLibrary.apply_variant` sets material/self_modulate; `SpriteLibrary.apply` can clear material for suppressed frames. Both must execute in original order on misses (`art_sprite_library.gd:43,106`). Cold/chill/settlement outline use child sprites and do not replace the main sprite's material in current source.
3. `_raise_selected_draw_order_p12` runs later and changes/restores body and icon z_index (`agent_renderer.gd:802`). Selected agents always write. Include current body z_index so deselection causes a miss when `load_z_index` metadata needs refreshing. The selection switch can restore the old body's z after that frame's property phase, so preserving original next-frame update behavior is essential.
4. Cold changes main position.x and metadata each frame; MorningChill changes child textures/regions/offsets and metadata; outline/shadow/cue/ring stay live. Do not compare all final sprite metadata wholesale: these legitimate later changes would turn every frame into a miss, and storing references to metadata arrays would risk aliasing.
5. `_ensure_agent` reuses a pooled sprite, whose prior texture, children, metadata and material can survive. Erase cache on retirement and force the first frame after creation/reuse. Do not infer newness solely from sprite instance ID.
6. `_update_activity_icon` lazily creates a child and its two branches reset different properties. Keep the original call ordering and exact branch selection. If an owned icon/CarryLoad/FounderBundle child is recreated or removed independently, invalidate that agent entry rather than assuming equal input means the output still exists. This is a named child-lifecycle case, not an invitation to poll all properties each frame.
7. Parent visibility can change without `_capture_visible` changing. Include inherited visibility; retain `_capture_visible` epoch invalidation as separately required by GOAL. The activity oracle must remain live through hidden/show cycles.

## Integration and forced misses

- Introduce `P13_WRITE_SKIP` default off until acceptance. Keep a renderer-owned key map plus diagnostic hit/miss/forced counters. Counters cannot change rendering or acceptance thresholds.
- `_ready` connects `/root/Locale.language_changed` (actual signal is in `scripts/core/locale.gd:3`) to cache invalidation. A generation or full cache clear happens on every delivered language change; one listener per renderer. No simulation polling or new getters.
- `_on_world_reset` clears keys BEFORE its synchronous `_process(0.0)` at line 458. `refresh_snapshot` is also a conservative invalidation point before its synchronous process; tests should not rely on a frame callback arriving later.
- `set_capture_visible` invalidates before `_process(0.0)` whenever shown changes. The first invoked zero-delta frame is the required forced-write frame. Calling it repeatedly with the same value need not invalidate, but must preserve the existing process call.
- `_ensure_agent`/`_retire_missing` clear the agent key. Detect `new_sprite` before ensuring, so pooled reuse cannot hit.
- Read `AgentInspectorPanel.get_selected_entity_bits()` once before the agent loop and compare it with snapshot entity bits, not art ID. Selection index/ring remain live. Selected agent bypasses equality every frame, including selected-but-hidden agents.
- Candidate switching clears keys. Disabling write skip runs the unchanged original write path; switching it back on cannot hit stale data. Setters for the existing allocation candidate should invalidate if invoked because frame/material caches are reset there.
- Preserve original block order on misses. Save the owned key only after writes succeed. Hits execute oracle/selection observation and every downstream route/draw operation. Avoid adding animation_seconds/tick to the write key when only oracle/post-loop uses them, or there will be no useful hit; decision outputs already encode their relevant animation effects.

## Candidate setter and identity evidence

Existing `set_perf_candidate_p12/get_perf_candidate_p12` only support `declump-kernel` at `agent_renderer.gd:90`. Do not overload the getter's current kernel identity silently: p13 capture harnesses call it and expect `change_id=declump-kernel` to prove `P12_DECLUMP_KERNEL` state.

Preferred bounded adaptation: add P13 candidate setter/getter handling `write-skip`, and teach only the p13 alternating runner/harness to dispatch to the P13 pair for that ID; leave P12 kernel pair unchanged. The write-skip getter must return its active ID, boolean readback and unchanged kernel state so A/B compares only one variable against the last accepted build. P13 identity manifests need additional write-skip readback and request/default provenance; keep the existing kernel fields. A narrower unified setter is possible only if the existing kernel getter retains its old contract and p13 readback is explicit. Do not turn off an accepted kernel to test write skip.

## Required verification before the speed decision

Unit scenarios exercise actual renderer writes/readback, not just equality of a helper:

1. Equal stable inputs: first miss, second hit, identical body/child property metadata; timers still advance and selection index still contains the agent.
2. New sprite and pooled sprite reuse (including same art ID after reset): forced full write even if semantic key values match.
3. Selected agent: every frame full write; selection switch and deselection refresh carry z metadata while ring and raised order remain correct.
4. World reset: clear before synchronous process, no cross-world entity/variant/icon leakage. `refresh_snapshot` also invalidates.
5. Locale change: one forced full write for each live agent even with identical displayed inputs; oracle metadata remains current.
6. Capture false then true and parent visibility false then true: each transition misses, including synchronous zero-delta capture update; shadows/icons/routes follow current visibility.
7. Switch off/on plus allocation-candidate reset: no stale keys; readback names exactly the toggled candidate and preserves kernel state.
8. Frame/facing/variant/life-stage, tile, overrides and cold/base-pose transitions: alter one resolved write input at a time; each misses and matches uncached output.
9. Carry food→no carry→wood, founder real-load suppression and eat/asleep transitions: texture, load metadata, anchors and founder visibility match the original path.
10. Daily↔porter icon, porter kind/stage, zoom across 2, icon/frame offset change and child recreation: miss with correct scale/opacity/placement.
11. Alias regression: after storing agent A key, mutate reused `_pose_output`, nested choice arrays and overrides for agent B; A's saved value must not change.
12. Oracle-enabled equal-key hit: oracle animation/movement inputs update while the guarded property writes stay skipped; post-loop cold/chill/outline/shadow and geometry also continue.

Then execute GOAL §5.1 on the same candidate: all six exact framings, W0/W200/W-multi 600-frame dumps, both W500 240-frame pans; three valid alternating 150-frame pairs on W500 frozen and W200 running. Both pooled ratios <=0.97 and >=2/3 pair improvements required. A build/unit pass proves neither identity nor speed. Retain rejected results; default remains off unless accepted. Consider key construction/deep-copy cost in actual measured results; do not reduce key correctness to obtain speed.
