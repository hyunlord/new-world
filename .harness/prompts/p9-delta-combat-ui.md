# Phase 9-δ — Combat UI Integration

## Section 1: Implementation Intent

V7 Phase 9-δ surfaces the Combat System's `CombatStarted` / `CombatCompleted`
causal events in the Godot UI layer. This is Stage 3 / 3 of the all-chain δ
dispatch (7-δ Social UI → 8-δ Memory UI → **9-δ Combat UI**).

**Scope lock** (phase9.md §3, Phase 9-δ):
- `CausalPanel` GDScript: render `CombatStarted`, `CombatCompleted` events and
  `CombatReason` decision reason with distinct formatting.
- `AgentRenderer` GDScript: brief "in combat" visual indicator when a
  `combat_started` causal event fires for an agent.
- `world_node.rs` FFI: extend `CombatStarted` and `CombatCompleted`
  serialization to expose `defender_id` (currently `defender: _`). The
  world_node.rs comment at line ~602 explicitly defers this to "later phases
  that surface combat UI" — Phase 9-δ is that phase.
- Locale keys: 5 new `UI_*` keys in `localization/fluent/{en,ko}/messages.ftl`
  **and** `localization/compiled/{en,ko}.json`.

**Precedent — Phase 8-δ** (commit `7ce33a33`):
- CausalPanel `_format_event()` match arms for memory events → mirror for combat.
- AgentRenderer `_recalling_agents` pattern → mirror as `_combating_agents`.
- `_ingest_memory_recalls()` tile-poll pattern → mirror as `_ingest_combat_events()`.
- `mark_agent_recalling()` + `recall_cue_remaining()` → mirror for combat cue.
- Single-source-of-truth `event_view_to_owned_dict()` → `event_view_to_dict()`
  delegation pattern for the FFI dict (Phase 8-δ Evaluator fix — replicate for
  combat fields).
- `agent_ids: PackedInt64Array` parallel array in snapshot → already present,
  use for `AgentId` ↔ rendered-row mapping.

**Phase 8-δ AgentRenderer pattern (MUST replicate exactly):**
```
_combating_agents: Dictionary = {}        # agent_id → frames remaining
_seen_combat_event_ids: Dictionary = {}   # dedupe set
const _SEEN_COMBAT_MAX := 512
const COMBAT_CUE_FRAMES := 36             # ~0.6s at 60 FPS
const COMBAT_CUE_SCALE_BOOST := 1.3      # 30% scale pulse
const COMBAT_CUE_TINT := Color(1.0, 0.3, 0.3, 1.0)  # red cue (future shader)
```
**Do NOT extend `state_tag`** — Phase 7-δ A22 contract pins `state_tag ∈
{0,1,2,3}`. Combat detection goes through the causal ring poll, not the
snapshot byte.

---

## Section 2: What to Build

### 2-A: world_node.rs FFI extension

In `rust/crates/sim-bridge/src/ffi/world_node.rs`, extend `CausalEventView`
and `event_view_to_owned_dict()` to expose `defender_id`:

1. Add `defender_id: Option<AgentId>` field to `CausalEventView`.
2. In `CausalEvent::CombatStarted` arm of `CausalEventView::from_event()`,
   set `defender_id: Some(*defender)` instead of `defender: _`.
3. In `CausalEvent::CombatCompleted` arm, similarly set
   `defender_id: Some(*defender)`.
4. In `event_view_to_owned_dict()`, insert `"defender_id"` key (as
   `FfiFieldValue::Int` from `agent_id.0.get() as i64`) when
   `self.defender_id.is_some()`.
5. Verify `event_view_to_dict()` delegates to `event_view_to_owned_dict()`
   (Phase 8-δ single-source-of-truth fix is already in place — do NOT
   regress it).

**Note**: `hp_after` is already serialized as `new_value` in
`CombatCompleted`. No change needed for that field.

### 2-B: CausalPanel GDScript (`scripts/ui/panels/causal_panel.gd`)

Add match arms in `_format_event()`:

1. Event kind `"combat_started"`:
   - Display `UI_CAUSAL_EVENT_COMBAT_STARTED` label.
   - Show `hp_after` is N/A here; show position if available.
   - Show `defender_id` if present in event dict.

2. Event kind `"combat_completed"`:
   - Display `UI_CAUSAL_EVENT_COMBAT_COMPLETED` label.
   - Show `hp_after` from `new_value` field if present.
   - If `hp_after == 0.0` (or absent), note the defender may be dead.

3. Decision reason `"combat_reason"` (`event.kind == "agent_decision"` and
   `event.reason == "combat_reason"`):
   - Display using locale key `UI_CAUSAL_REASON_COMBAT`.
   - Mirror the Phase 8-δ `"memory_reason"` pattern exactly.

**Localization rule**: All player-facing text MUST use `_ltr()`. No hardcoded
English strings of length ≥ 4 in the memory/combat rendering region.

### 2-C: AgentRenderer GDScript (`scripts/ui/agent_renderer.gd`)

Add combat cue alongside the existing recall cue:

1. Add constants `COMBAT_CUE_FRAMES`, `COMBAT_CUE_SCALE_BOOST`,
   `COMBAT_CUE_TINT` (values above).
2. Add `_combating_agents: Dictionary = {}` and
   `_seen_combat_event_ids: Dictionary = {}`.
3. Add `_clear_expired_combating()` — mirrors `_clear_expired_recalling()`.
4. In `_process()`, call `_clear_expired_combating()` first, then apply
   `COMBAT_CUE_SCALE_BOOST` for agents in `_combating_agents` (by `agent_ids`
   array lookup, same as Phase 8-δ `_recalling_agents` pattern).
   - If both recall AND combat cue are active, apply the larger boost.
5. Add `_ingest_combat_events(ids, agent_ids, xs, ys, n)` — mirrors
   `_ingest_memory_recalls()`. Polls `get_tile_causal_history()` per unique
   tile, filters `kind == "combat_started"`, dedupes by event id, calls
   `mark_agent_in_combat(agent_id)` for the attacker.
6. Add `mark_agent_in_combat(agent_id: int)` and
   `combat_cue_remaining(agent_id: int) -> int` public API.
7. Call `_ingest_combat_events()` at the end of `_process()`, after the
   existing `_ingest_memory_recalls()` call.
8. **Do NOT remove** any Phase 7-δ or Phase 8-δ code.

### 2-D: Locale keys (exactly 5 new keys)

Add to `localization/fluent/en/messages.ftl` **and**
`localization/fluent/ko/messages.ftl`:

| Key | en value | ko value |
|-----|----------|----------|
| `UI_CAUSAL_REASON_COMBAT` | `Combat decision` | `전투 결정` |
| `UI_CAUSAL_EVENT_COMBAT_STARTED` | `Combat started` | `전투 시작됨` |
| `UI_CAUSAL_EVENT_COMBAT_COMPLETED` | `Combat completed` | `전투 완료됨` |
| `UI_AGENT_STATE_IN_COMBAT` | `In combat` | `전투 중` |
| `UI_COMBAT_HP_AFTER` | `HP after` | `이후 HP` |

Then update `localization/compiled/en.json` and `localization/compiled/ko.json`
via Python JSON merge (same as Phase 8-δ):

```python
import json

new_en = {
    "UI_CAUSAL_REASON_COMBAT": "Combat decision",
    "UI_CAUSAL_EVENT_COMBAT_STARTED": "Combat started",
    "UI_CAUSAL_EVENT_COMBAT_COMPLETED": "Combat completed",
    "UI_AGENT_STATE_IN_COMBAT": "In combat",
    "UI_COMBAT_HP_AFTER": "HP after",
}
new_ko = {
    "UI_CAUSAL_REASON_COMBAT": "전투 결정",
    "UI_CAUSAL_EVENT_COMBAT_STARTED": "전투 시작됨",
    "UI_CAUSAL_EVENT_COMBAT_COMPLETED": "전투 완료됨",
    "UI_AGENT_STATE_IN_COMBAT": "전투 중",
    "UI_COMBAT_HP_AFTER": "이후 HP",
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

---

## Section 3: How to Implement

### 3-A: world_node.rs

Read `rust/crates/sim-bridge/src/ffi/world_node.rs`. Find `CausalEventView`
struct (~line 912). Add `defender_id: Option<AgentId>`. Find the
`CombatStarted` and `CombatCompleted` arms in `CausalEventView::from_event()`
(~line 602). Replace `defender: _` with `defender_id: Some(*defender)`.
Add `"defender_id"` to `event_view_to_owned_dict()`. Confirm
`event_view_to_dict()` still delegates to `event_view_to_owned_dict()`.

### 3-B: CausalPanel

Read `scripts/ui/panels/causal_panel.gd`. Find `_format_event()` match. Add
cases for `"combat_started"`, `"combat_completed"`, and `"combat_reason"`.
Follow the Phase 8-δ `"memory_recalled"` / `"memory_reason"` pattern exactly.
No hardcoded English.

### 3-C: AgentRenderer

Read `scripts/ui/agent_renderer.gd`. Follow the Phase 8-δ `_recalling_agents`
pattern exactly for `_combating_agents`. The `agent_ids: PackedInt64Array`
array is already in the snapshot — use it (Phase 8-δ already wired it).

### 3-D: Locale / Compiled JSON

Apply the Python JSON merge above. Verify 5 keys in both compiled files.

---

## Section 4: Locale Contract

Exactly 5 new keys, all with `UI_` prefix.

**en thresholds:**
- All 5 keys present in `localization/compiled/en.json` under `strings`
- Each value: character length ≥ 2, ≥ 2 ASCII alphabetic chars [A-Za-z]
- All 5 values pairwise distinct (no copy-paste collision)

**ko thresholds:**
- All 5 keys present in `localization/compiled/ko.json` under `strings`
- Each value: character length ≥ 2, ≥ 1 Hangul syllable char (U+AC00–U+D7A3)
- Each value differs (byte-wise) from the corresponding en value

---

## Section 5: Verification

### Rust harness tests (`rust/crates/sim-test/tests/harness_p9_delta_combat_ui.rs`)

Write harness tests:

1. **locale_compiled_en_five_keys_present** (Type A): parse
   `localization/compiled/en.json`, check all 5 keys present with value
   length ≥ 2 and ≥ 2 ASCII alpha chars.

2. **locale_compiled_ko_five_keys_present** (Type A): same for ko.json,
   checking ≥ 1 Hangul syllable per value, each differs from en.

3. **locale_five_keys_pairwise_distinct_en** (Type A): all 5 en values
   pairwise distinct (0 collisions among C(5,2)=10 pairs).

4. **combat_started_ffi_kind_string** (Type A): construct
   `CausalEvent::CombatStarted { .. }`, pass through
   `CausalEventView::from_event()` then `event_view_to_owned_dict()` (the
   single-source-of-truth helper, per Phase 8-δ fix). Assert
   `dict["kind"] == "combat_started"` (exact literal).

5. **combat_completed_ffi_kind_string** (Type A): same for
   `CausalEvent::CombatCompleted`, assert `dict["kind"] == "combat_completed"`.

6. **combat_ffi_defender_id_exposed** (Type A): construct
   `CausalEvent::CombatStarted` with a known `defender` AgentId. Pass through
   `event_view_to_owned_dict()`. Assert `dict.contains_key("defender_id")` and
   the value is an `FfiFieldValue::Int` matching the defender's `id.0.get()`.

7. **combat_reason_decision_as_str** (Type A): assert
   `DecisionReason::CombatReason.as_str() == "combat_reason"` (exact equality).

8. **combat_panel_no_hardcoded_english** (Type A): read
   `scripts/ui/panels/causal_panel.gd` source, extract the
   `"combat_started"` and `"combat_completed"` match arms and the
   `"combat_reason"` inner branch. Scan for quoted string literals of
   alphabetic chars with length ≥ 4. Allowlist: `UI_` prefixed keys,
   snake_case wire discriminators (`combat_started`, `combat_completed`,
   `combat_reason`, `new_value`, `defender_id`, `agent_id`, `kind`,
   `reason`, `tick`). Fail if any non-allowlisted alphabetic literal ≥ 4
   chars is found.

9. **phase8_delta_recall_cue_regression** (Type A): read
   `scripts/ui/agent_renderer.gd`, assert `_recalling_agents` dict,
   `mark_agent_recalling`, `_ingest_memory_recalls`, and
   `RECALL_CUE_FRAMES` are all still present (Phase 8-δ regression guard).

**Test naming**: `harness_p9_delta_combat_*` prefix for all 9 tests.

### Build verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -5
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: all existing tests pass (p3-9γ + p7-δ + p8-δ regressions CLEAN),
9 new p9-δ tests pass, clippy clean.

---

## Section 6: Pipeline Lane

**Lane: `--full`**

Rationale: GDScript UI changes + Rust FFI extension require plan debate to
lock the rendering contract and VLM verification to confirm the combat
indicator appears correctly.

```bash
bash tools/harness/harness_pipeline.sh \
    p9-delta-combat-ui \
    .harness/prompts/p9-delta-combat-ui.md \
    --full
```

---

## Section 7: VLM Verification Checklist

1. **CausalPanel CombatStarted rendering**: combat_started event entry appears
   with `UI_CAUSAL_EVENT_COMBAT_STARTED` label when CausalPanel is open.

2. **CausalPanel CombatReason decision**: agent decision with combat_reason
   shows `UI_CAUSAL_REASON_COMBAT` text.

3. **AgentRenderer combat indicator**: when combat_started fires for an agent,
   a brief scale pulse (red tint or similar) appears for ~0.6s then reverts.
   Must NOT persist permanently.

4. **No regression**: Phase 7-δ green tint and Phase 8-δ blue/scale recall
   pulse still work correctly for their respective trigger states.

---

## Out of Scope

- HP bar / health overlay (requires new FFI).
- Dead-agent removal animation (AgentDied is not a CausalEvent variant).
- `defender_id`-based visual cue on the defender (only attacker cue required).
- `MemoryRecallTrigger::CombatContext` distinct causal panel rendering
  (already handled generically by Phase 8-δ as fallback "memory recalled").
- Any modification to `sim-core`, `sim-systems`, `sim-engine`.
- New `WorldSimNode` `#[func]` methods.
