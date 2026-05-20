# Phase 8-δ — Memory UI Integration

## Section 1: Implementation Intent

V7 Phase 8-δ surfaces the Memory System's `MemoryRecalled` causal events in the
Godot UI layer. This is Stage 2 / 3 of the all-chain δ dispatch
(7-δ Social UI → **8-δ Memory UI** → 9-δ Combat UI).

**Scope lock** (phase8.md §3, Phase 8-δ):
- `CausalPanel` GDScript: render `MemoryRecalled` events and `MemoryReason`
  decision reason with distinct formatting.
- `AgentRenderer` GDScript: brief "recalling" visual indicator when a memory
  cascade-bias recall fires for an agent.
- Locale keys: 7 new `UI_*` keys in `localization/fluent/{en,ko}/messages.ftl`
  **and** `localization/compiled/{en,ko}.json`.
- **No new SimBridge Rust FFI**: `MemoryRecalled` events are already serialized
  by the existing causal chain FFI (`kind: "memory_recalled"` in world_node.rs
  line ~544). No new `collect_memory_snapshot` or `get_memory_snapshot` is
  needed for this sub-stage.

**Phase 7-δ precedent** (813e2d06, 95/100 APPROVE):
- CausalPanel `_format_event()` match arms for social events → mirror for memory.
- 7-key locale pattern with `UI_CAUSAL_*` prefix.
- `localization/compiled/en.json` and `ko.json` updated via Python JSON merge.

**Phase 9-δ scope intact**: `MemoryRecallTrigger::CombatContext` UI is Phase 9-δ
scope — do NOT render it distinctly here. Treat it the same as `CascadeBias` at
most, or leave it as a generic "recall" label.

---

## Section 2: What to Build

### 2-A: CausalPanel GDScript (`scripts/ui/panels/causal_panel.gd`)

Add match arms in the `_format_event()` function (or equivalent event-rendering
method):

1. Event kind `"memory_recalled"`:
   - Display a memory icon label (e.g., `[MEMO]` or locale key `UI_CAUSAL_EVENT_MEMORY_RECALLED`).
   - Include the `triggered_by` field if available in the event dict. Phase 8-β
     only emits `CascadeBias`; format it via `UI_MEMORY_RECALL_TRIGGER_CASCADE`.
   - Show `recalled_event` id if present (as a short hex or numeric id).

2. Decision reason `"memory_reason"` (when `event.kind == "agent_decision"` and
   `event.reason == "memory_reason"`):
   - Display using locale key `UI_CAUSAL_REASON_MEMORY`.
   - Mirror the Phase 7-δ `"social_reason"` pattern exactly.

### 2-B: AgentRenderer GDScript (`scripts/ui/renderers/agent_renderer.gd`)

Add a brief "recalling" visual indicator:
- Subscribe to `SimulationBus.causal_event_fired` (or equivalent signal) for
  events of kind `"memory_recalled"`.
- When such an event fires for a given agent entity, activate a brief
  visual cue (e.g., blue tint modulate or a thought-bubble sprite) for 0.5–1
  second, then revert.
- Do **not** extend the `state_tag` byte in `AgentSnapshotRow` — that would
  break Phase 7-δ harness test A22 which pins `state_tag ∈ {0,1,2,3}`.
- Use a GDScript-side timer dict (`_recalling_agents: Dictionary`) keyed by
  entity id, value = remaining display ticks or time.

### 2-C: Locale keys

Add to `localization/fluent/en/messages.ftl` **and** `localization/fluent/ko/messages.ftl`:

| Key | en value | ko value |
|-----|----------|----------|
| `UI_CAUSAL_REASON_MEMORY` | `Memory recall` | `기억 회상` |
| `UI_CAUSAL_EVENT_MEMORY_RECALLED` | `Memory recalled` | `기억 회상됨` |
| `UI_CAUSAL_EVENT_MEMORY_RECALLED_CASCADE` | `Memory recalled (cascade)` | `기억 회상됨 (연쇄)` |
| `UI_AGENT_STATE_RECALLING` | `Recalling` | `회상 중` |
| `UI_MEMORY_RECALL_TRIGGER_CASCADE` | `Cascade bias` | `연쇄 편향` |
| `UI_MEMORY_RECALL_TRIGGER_SIMILARITY` | `Similarity search` | `유사도 탐색` |
| `UI_MEMORY_RECALL_TRIGGER_PERIODIC` | `Periodic` | `주기적` |

Then update `localization/compiled/en.json` and `localization/compiled/ko.json`
by adding all 7 keys under the `strings` dict (JSON `indent=2`, `ensure_ascii=False`).

---

## Section 3: How to Implement

### 3-A: CausalPanel

Read `scripts/ui/panels/causal_panel.gd`. Find `_format_event()` or the event
rendering switch. Add cases for `"memory_recalled"` and `"memory_reason"`.
Follow the Phase 7-δ `"social_interaction_started"` / `"social_reason"` pattern
exactly.

### 3-B: AgentRenderer

Read `scripts/ui/renderers/agent_renderer.gd`. Find where `states` PackedByteArray
is processed (Phase 7-δ added `state_tag == 2` tint). Add the recalling timer
pattern using a GDScript Dictionary.

Connect to SimulationBus causal event signal if available; if not, read from the
causal events dict returned by existing SimBridge methods per tick.

### 3-C: Locale / Compiled JSON

```python
import json

new_en = {
    "UI_CAUSAL_REASON_MEMORY": "Memory recall",
    "UI_CAUSAL_EVENT_MEMORY_RECALLED": "Memory recalled",
    "UI_CAUSAL_EVENT_MEMORY_RECALLED_CASCADE": "Memory recalled (cascade)",
    "UI_AGENT_STATE_RECALLING": "Recalling",
    "UI_MEMORY_RECALL_TRIGGER_CASCADE": "Cascade bias",
    "UI_MEMORY_RECALL_TRIGGER_SIMILARITY": "Similarity search",
    "UI_MEMORY_RECALL_TRIGGER_PERIODIC": "Periodic",
}
new_ko = {
    "UI_CAUSAL_REASON_MEMORY": "기억 회상",
    "UI_CAUSAL_EVENT_MEMORY_RECALLED": "기억 회상됨",
    "UI_CAUSAL_EVENT_MEMORY_RECALLED_CASCADE": "기억 회상됨 (연쇄)",
    "UI_AGENT_STATE_RECALLING": "회상 중",
    "UI_MEMORY_RECALL_TRIGGER_CASCADE": "연쇄 편향",
    "UI_MEMORY_RECALL_TRIGGER_SIMILARITY": "유사도 탐색",
    "UI_MEMORY_RECALL_TRIGGER_PERIODIC": "주기적",
}
for path, new_keys in [
    ("localization/compiled/en.json", new_en),
    ("localization/compiled/ko.json", new_ko),
]:
    with open(path) as f:
        d = json.load(f)
    d["strings"].update(new_keys)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(d, f, ensure_ascii=False, indent=2)
```

### 3-D: No Rust changes required

`sim-bridge`, `sim-core`, `sim-systems`, `sim-engine` are **not modified**.
The causal chain FFI already serializes `MemoryRecalled` events. All changes are
GDScript and locale files.

---

## Section 4: Locale Contract

Exactly 7 new keys, all with `UI_` prefix. Pattern mirrors Phase 7-δ exactly:

**en thresholds** (from Phase 7-δ A19 pattern):
- All 7 keys present in `localization/compiled/en.json` under `strings`
- Each value: character length ≥ 3, ≥ 2 ASCII alphabetic chars [A-Za-z]
- All 7 values pairwise distinct (no copy-paste collision)

**ko thresholds** (from Phase 7-δ A20 pattern):
- All 7 keys present in `localization/compiled/ko.json` under `strings`
- Each value: character length ≥ 2, ≥ 1 Hangul syllable char (U+AC00–U+D7A3)
- Each value differs (byte-wise) from the corresponding en value

---

## Section 5: Verification

### Rust harness tests (`rust/crates/sim-test/tests/harness_p8_delta_memory_ui.rs`)

Write harness tests that verify:

1. **locale_compiled_contains_all_seven_keys_en** (Type A): parse
   `localization/compiled/en.json` (relative path from workspace root), check all
   7 keys present with value length ≥ 3 and ≥ 2 ASCII alpha chars.

2. **locale_compiled_contains_all_seven_keys_ko** (Type A): same for ko.json,
   checking ≥ 1 Hangul syllable (U+AC00–U+D7A3) per value, each differs from en.

3. **locale_seven_keys_pairwise_distinct_en** (Type A): all 7 en values pairwise
   distinct (0 collisions among C(7,2)=21 pairs).

4. **memory_recalled_causal_event_ffi_kind** (Type A): after a full Phase 8
   scenario run (20-agent stage1 engine, `register_default_runtime_systems`,
   `Social::new(0.0, 1.0)` to force frequent social interactions → memory
   accumulation), run 200 ticks. Then read causal events from
   `engine.resources().causal_ring` (or per-entity ring). Among events of type
   `CausalEvent::MemoryRecalled`, verify that each has `triggered_by ==
   MemoryRecallTrigger::CascadeBias` (Phase 8-β only wires CascadeBias).
   Threshold: if any `MemoryRecalled` events exist, all have `triggered_by ==
   CascadeBias`. If 0 events after 200 ticks, the test PASSes vacuously — log
   a notice but do not fail (MemoryRecalled depends on prior social interactions
   creating memories; 200 ticks may not be enough to trigger the recall cascade).

5. **memory_recalled_ffi_serialization_includes_triggered_by** (Type A): use the
   SimBridge causal event serialization (call `WorldSimNode`'s causal event FFI if
   accessible from sim-test; else test via `sim_bridge::ffi::world_node` helpers
   directly). For a `CausalEvent::MemoryRecalled { triggered_by: CascadeBias, .. }`
   constructed manually, verify the serialized dict includes key `"triggered_by"`
   with a non-empty string value. This exercises the world_node.rs serialization
   branch at line ~544.

6. **memory_recalled_ffi_kind_string** (Type A): verify that the serialized
   `kind` field for `CausalEvent::MemoryRecalled` equals the literal string
   `"memory_recalled"` (not `"MemoryRecalled"` or similar). Direct string
   comparison, no tolerance.

7. **memory_reason_decision_as_str** (Type A): verify
   `DecisionReason::MemoryReason.as_str() == "memory_reason"` (exact equality).
   Tests that the CausalPanel's match on `event.reason == "memory_reason"` will
   hit the correct case.

**Test naming**: `harness_p8_delta_memory_*` prefix for all 7 tests.

### Build verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -5
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: all existing tests pass (phase 3-9γ + phase 7-δ regressions CLEAN),
7 new p8-δ tests pass, clippy clean.

---

## Section 6: Pipeline Lane

**Lane: `--full`**

Rationale: GDScript UI changes (CausalPanel + AgentRenderer) require plan debate
to lock the rendering contract, and VLM visual verification to confirm the
recalling indicator appears correctly.

```bash
bash tools/harness/harness_pipeline.sh \
    p8-delta-memory-ui \
    .harness/prompts/p8-delta-memory-ui.md \
    --full
```

---

## Section 7: VLM Verification Checklist

The pipeline VLM step should verify:

1. **CausalPanel MemoryReason rendering**: When an agent's causal panel is open
   during a memory cascade-bias recall tick, the panel shows a memory recall
   event entry with the `UI_CAUSAL_EVENT_MEMORY_RECALLED` label (or similar
   localized text).

2. **CausalPanel MemoryReason decision**: When an agent makes a decision driven
   by memory recall, the decision reason shows `UI_CAUSAL_REASON_MEMORY` text.

3. **AgentRenderer recalling indicator**: When a memory recall fires for an
   agent, a brief visual cue (blue tint or similar) appears on that agent sprite
   for ~0.5–1 second, then disappears. It should NOT persist permanently and
   should NOT appear on agents without memory recall events.

4. **No regression**: The Phase 7-δ socializing green tint still works correctly
   for agents in `Consuming { target: Agent(_) }` state.

**A broken implementation could look like**: memory recall text never appearing
in CausalPanel, recalling indicator persisting forever, Phase 7-δ green tint
disappearing (regression), or locale keys showing raw key strings.

---

## Out of Scope

- `MemoryRecallTrigger::CombatContext` distinct UI (Phase 9-δ scope).
- Memory entries list panel (requires new `Memory::entries_for_agent` FFI).
- New `AgentState::Recalling` variant (not added in Phase 8-β; extending
  `state_tag` would break Phase 7-δ harness test A22).
- `collect_memory_snapshot` or any new `#[func]` in `WorldSimNode`.
- Any modification to `sim-core`, `sim-systems`, `sim-engine`, or `sim-bridge`
  Rust crates.
