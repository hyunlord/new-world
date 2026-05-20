# Harness Test Plan Request

## Feature
---
feature: p7-delta-social-ui
phase: 7-δ
lane: --full
seed: 42
---

# P7δ — Social UI Integration

## Section 1: Implementation Intent

V7 Phase 7-δ is the Social UI milestone: the first visual evidence that the
Social System (Phase 7-α/β/γ) is alive to the player. Three surfaces get
updated — CausalPanel, AgentRenderer, and a new RelationshipState debug
overlay — plus the supporting SimBridge extensions and locale keys.

**Planning §δ scope** (locked):
- `CausalPanel` SocialReason / SocialInteraction event rendering with
  localized labels.
- Locale keys: `UI_CAUSAL_REASON_SOCIAL`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED`,
  `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED`, `UI_AGENT_STATE_SOCIALIZING`.
- `AgentRenderer`: tint agents whose `AgentState` is `Consuming { Agent(_) }`
  (i.e. currently socializing) with a distinct colour.
- `RelationshipState` UI overlay: a debug panel listing all pairs with
  `familiarity > 0`.

**User mandate**: "current UI described as 'not game-like'". Phase 7-δ is
Stage 1 of the All chain (7-δ → 8-δ → 9-δ) visualisation path. This
dispatch is scoped to Social only; Phase 8-δ (Memory) and Phase 9-δ (Combat)
are separate dispatches.

**Phase 8-δ + 9-δ scope intact**: do NOT add MemoryReason or CombatReason
rendering here.

## Section 2: What to Build

### 2-A: `rust/crates/sim-bridge/src/ffi/world_node.rs`

#### 2-A-1: Extend `AgentSnapshotRow` with `state_tag: u8`

Add field to `AgentSnapshotRow` struct:
```rust
/// 0 = Idle, 1 = Seeking, 2 = ConsumingAgent (socializing), 3 = ConsumingOther
pub state_tag: u8,
```

Update `collect_agent_snapshot` to query `(Agent, Position, AgentState)`:
```rust
pub fn collect_agent_snapshot(world: &hecs::World) -> Vec<AgentSnapshotRow> {
    let mut rows = Vec::new();
    for (entity, (_, pos, state)) in world.query::<(&Agent, &Position, &AgentState)>().iter() {
        let state_tag: u8 = match state {
            AgentState::Idle => 0,
            AgentState::Seeking { .. } => 1,
            AgentState::Consuming { target: TargetKind::Agent(_) } => 2,
            AgentState::Consuming { .. } => 3,
        };
        rows.push(AgentSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            state_tag,
        });
    }
    rows
}
```

Update `agent_rows_split` to also return `Vec<u8>` for states.

Update `WorldSimNode::get_agent_snapshot()` to add `states: PackedByteArray`
to the returned `VarDictionary`.

#### 2-A-2: Add `get_relationship_snapshot()` `#[func]`

New `#[func]` method:
```rust
/// P7-δ — return all known relationship pairs as a flat array of Dicts.
/// Each dict: { "id_a": i64, "id_b": i64, "familiarity": f64,
///              "hostility": f64 }
/// Only pairs where familiarity > 0 OR hostility > 0 are returned.
#[func]
fn get_relationship_snapshot(&self) -> VarArray {
    collect_relationship_snapshot(&self.engine.resources)
}
```

Pure-Rust collector (Bridge Identity Contract — same pattern as
`collect_tile_causal_history`):
```rust
pub fn collect_relationship_snapshot(resources: &SimResources) -> Vec<RelationshipSnapshotRow> {
    resources.relationships
        .iter()
        .filter(|(_, v)| v.familiarity > 0.0 || v.hostility > 0.0)
        .map(|(k, v)| RelationshipSnapshotRow {
            id_a: k.0.0 as i64,
            id_b: k.1.0 as i64,
            familiarity: v.familiarity,
            hostility: v.hostility,
        })
        .collect()
}
```

`RelationshipSnapshotRow` struct (add near `AgentSnapshotRow`):
```rust
pub struct RelationshipSnapshotRow {
    pub id_a: i64,
    pub id_b: i64,
    pub familiarity: f64,
    pub hostility: f64,
}
```

Check `SimResources` for the actual field name:
```bash
grep -n "relationships\|HashMap.*Relationship" rust/crates/sim-core/src/sim_resources.rs | head
```
and verify `RelationshipState.hostility` exists:
```bash
grep -n "hostility\|familiarity" rust/crates/sim-core/src/components/relationship.rs | head
```

### 2-B: `scripts/ui/panels/causal_panel.gd`

Add three match arms to `_format_event()` (alongside existing
`building_placed`, `stamp_dirty`, `influence_changed`):

```gdscript
"agent_decision":
    var reason: String = ev.get("reason", "")
    if reason == "social_reason":
        kind_label = _ltr("UI_CAUSAL_REASON_SOCIAL")
    else:
        kind_label = _ltr("UI_CAUSAL_EVENT_AGENT_DECISION")
    var agent_id: int = int(ev.get("agent_id", -1))
    extra = " agent=" + str(agent_id)
"social_interaction_started":
    kind_label = _ltr("UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED")
"social_interaction_completed":
    kind_label = _ltr("UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED")
    var fam: float = float(ev.get("familiarity_after", 0.0))
    extra = " fam=" + ("%.2f" % fam)
```

Add `UI_CAUSAL_EVENT_AGENT_DECISION` to locale (generic fallback).

### 2-C: `scripts/ui/agent_renderer.gd`

Extend `_process()` to read `states: PackedByteArray` from snapshot and
apply a distinct tint when `state_tag == 2` (ConsumingAgent = socializing).

Since current `MultiMesh` uses `use_colors = false`, switch to
`use_colors = true` and set per-instance color:
- Default: `Color(1, 1, 1, 0)` (white — no tint change; palette_swap shader
  uses `INSTANCE_CUSTOM.rgb`, not `COLOR`)
- Socializing (state_tag == 2): `Color(0.4, 1.0, 0.6, 0)` (green tint
  applied via `set_instance_color`)

Alternatively, encode state in `custom_data.a` component (currently 0.0)
and update shader to tint on `a > 0.5`. Choose whichever approach compiles
cleanly — the key requirement is that a co-located socializing agent is
visually distinguishable from Idle agents.

Simplest approach (no shader change required):
- Keep `use_colors = false`
- Use `set_instance_custom_data` alpha channel: encode `state_tag` in
  `INSTANCE_CUSTOM.a` (currently unused, always 0.0)
- Update `palette_swap.gdshader` to apply green tint when
  `INSTANCE_CUSTOM.a > 0.5`

OR — if shader change is complex, use `use_colors = true` and
`set_instance_color(i, Color(0.4,1.0,0.6,1))` for state_tag==2, white
otherwise. This requires no shader change.

**Choose the approach that works without breaking existing palette swap
rendering.** Verify `assets/sprites/agent_base.png` and
`shaders/palette_swap.gdshader` are intact.

Snapshot read pattern:
```gdscript
var snap: Dictionary = world_sim.get_agent_snapshot()
var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
var states: PackedByteArray = snap.get("states", PackedByteArray())
```

### 2-D: RelationshipState debug overlay

Add a simple `_relationship_panel` element to `causal_panel.gd` (or as a
sibling Control in the UI scene) that calls
`world_sim.get_relationship_snapshot()` every N ticks and lists pairs:

```
[REL] agent_0 ↔ agent_1  fam=0.30  hos=0.00
[REL] agent_0 ↔ agent_2  fam=0.10  hos=0.10
```

Toggle with `R` key. Title: `_ltr("UI_RELATIONSHIP_PANEL_TITLE")`.

The panel can be a `VBoxContainer` inside a `ColorRect` background, same
style as the existing `CausalPanel`. Limit to top 10 pairs by familiarity.

**New locale keys for this panel**:
- `UI_RELATIONSHIP_PANEL_TITLE` = "Relationships" / "관계"
- `UI_RELATIONSHIP_PAIR_ROW` = "[REL]" / "[관계]"

Since this is a debug panel, position it at bottom-left of screen
(PANEL_MARGIN, screen_height - PANEL_HEIGHT - PANEL_MARGIN).

## Section 3: How to Implement

### 3-A: SimBridge changes (Rust)

**Order of changes**:
1. Add `state_tag: u8` to `AgentSnapshotRow`.
2. Update `collect_agent_snapshot` signature: add `AgentState` to query.
3. Update `agent_rows_split` → `agent_rows_to_parts` (or add `states` Vec).
4. Update `WorldSimNode::get_agent_snapshot` to insert `states` into dict.
5. Add `RelationshipSnapshotRow` struct.
6. Add `collect_relationship_snapshot(resources: &SimResources)` pure-Rust fn.
7. Add `WorldSimNode::get_relationship_snapshot` `#[func]`.

**Required imports** (verify in world_node.rs):
```rust
use sim_core::components::{Agent, AgentState, Position, TargetKind, ...};
use sim_core::sim_resources::SimResources;
use sim_core::components::relationship::{RelationshipKey, RelationshipState};
```

Check the exact import path before writing — use:
```bash
grep -n "^use\|^pub use" rust/crates/sim-bridge/src/ffi/world_node.rs | head -20
grep -rn "pub struct SimResources\|pub relationships" rust/crates/sim-core/src/ | head -5
```

### 3-B: GDScript changes

**`causal_panel.gd`**:
- Add match arms as shown in 2-B. No structural changes needed.

**`agent_renderer.gd`**:
- Add `states` read from snapshot.
- Use `use_colors = true` (change in `_ready`).
- Set `multi_mesh.set_instance_color(i, color)` in the per-agent loop.

**RelationshipState panel** (add to `causal_panel.gd` or separate file):
- If adding to `causal_panel.gd`, add `_rel_panel_visible: bool = false`
  and `_rel_container: VBoxContainer` in `_build_layout`.
- If separate file `scripts/ui/panels/relationship_panel.gd`, follow same
  pattern as `causal_panel.gd`.

### 3-C: Locale

Edit `localization/fluent/en/messages.ftl` and
`localization/fluent/ko/messages.ftl`.

Append at end of each file (following existing `UI_CAUSAL_*` block):

**en**:
```
UI_CAUSAL_REASON_SOCIAL = Social need
UI_CAUSAL_EVENT_AGENT_DECISION = Agent decision
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED = Interaction started
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED = Interaction completed
UI_AGENT_STATE_SOCIALIZING = Socializing
UI_RELATIONSHIP_PANEL_TITLE = Relationships
UI_RELATIONSHIP_PAIR_ROW = [REL]
```

**ko**:
```
UI_CAUSAL_REASON_SOCIAL = 사회적 필요
UI_CAUSAL_EVENT_AGENT_DECISION = 에이전트 결정
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED = 교류 시작
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED = 교류 완료
UI_AGENT_STATE_SOCIALIZING = 교류 중
UI_RELATIONSHIP_PANEL_TITLE = 관계
UI_RELATIONSHIP_PAIR_ROW = [관계]
```

After editing FTL files, compile:
```bash
python3 tools/localization_compile.py
```

Verify compiled output:
```bash
grep "UI_CAUSAL_REASON_SOCIAL\|UI_AGENT_STATE_SOCIALIZING" localization/compiled/en.json
```

## Section 4: Locale Summary

| Key | en | ko |
|-----|----|----|
| `UI_CAUSAL_REASON_SOCIAL` | "Social need" | "사회적 필요" |
| `UI_CAUSAL_EVENT_AGENT_DECISION` | "Agent decision" | "에이전트 결정" |
| `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED` | "Interaction started" | "교류 시작" |
| `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED` | "Interaction completed" | "교류 완료" |
| `UI_AGENT_STATE_SOCIALIZING` | "Socializing" | "교류 중" |
| `UI_RELATIONSHIP_PANEL_TITLE` | "Relationships" | "관계" |
| `UI_RELATIONSHIP_PAIR_ROW` | "[REL]" | "[관계]" |

Keys follow the existing `UI_CAUSAL_*` SCREAMING_SNAKE_CASE convention.
The locale system reads from `localization/compiled/{lang}.json` — the FTL
compile step is **required**.

## Section 5: Verification

### 5-A: Rust gate
```bash
cd rust && cargo build --workspace 2>&1 | tail -5
cd rust && cargo test --workspace 2>&1 | grep "test result"
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: all workspace tests pass (Phase 7-α/β/γ + Phase 8-α/β/γ + Phase
9-α/β/γ harness tests remain green). No new clippy warnings.

### 5-B: Harness tests (sim-test)

Write harness assertions in
`rust/crates/sim-test/tests/harness_p7_delta_social_ui.rs`:

**A1** [Type A]: `get_agent_snapshot()` dict contains `"states"` key.
```rust
let snap = collect_agent_snapshot(&engine.world());
// All rows have state_tag field — verify default Idle agents = 0
for row in &snap { assert_eq!(row.state_tag, 0); }
```

**A2** [Type A]: Idle agent → `state_tag == 0`.

**A3** [Type A]: Agent in `Consuming { Agent(_) }` state → `state_tag == 2`.
(Set up a pair running the social interaction loop to completion, then check
state mid-interaction at `progress < REQUIRED_INTERACTION_PROGRESS`.)

**A4** [Type A]: `collect_relationship_snapshot` returns empty when no
relationships exist.

**A5** [Type A]: After `REQUIRED_INTERACTION_PROGRESS` ticks of social
interaction, `collect_relationship_snapshot` returns ≥1 entry with
`familiarity > 0`.

**A6** [Type A]: Locale compiled JSON contains `UI_CAUSAL_REASON_SOCIAL` key.
(Read `localization/compiled/en.json` from test — or skip if filesystem
access is unreliable in test context.)

Total: ≥5 assertions (A1–A5 minimum; A6 optional).

### 5-C: Locale compile
```bash
python3 tools/localization_compile.py
grep "UI_CAUSAL_REASON_SOCIAL" localization/compiled/en.json localization/compiled/ko.json
```

Both files must contain the new keys.

### 5-D: In-game verification (VLM)

The pipeline VLM will verify:
1. Agents in `Consuming { Agent(_) }` state are visually distinct (tinted)
   from `Idle` agents in the same scene.
2. CausalPanel (Q key) shows social event entries when a social interaction
   has occurred on the selected tile.
3. RelationshipState panel (R key) shows `[REL]` entries after agents have
   completed a social interaction.

The Godot harness test script `scripts/test/harness_visual_verify.gd`
drives the visual verification step.

## Section 6: Pipeline Lane

**Lane**: `--full`

Rationale: GDScript UI changes (`.gd`) trigger hot-tier per hook auto-
detection. SimBridge `.rs` changes also trigger review. `--full` is
appropriate for a UI panel/renderer change with FFI extension.

Expected score: ≥85/100 (UI work, VLM hot-tier visual verification).
Adjusted score ≥90 after VLM env cost if needed.

Hook tier: cold-tier signal A (sim-test `.rs`) + GDScript (hot-tier `.gd`)
→ hook takes the most restrictive tier → `--full`.

## Section 7: In-Game Verification Checklist

For the VLM visual analysis step:

1. **AgentRenderer socializing tint**: Spawn 2 agents with `loneliness > 50`
   at the same tile. After ~4-5 ticks, during the 3-tick interaction window,
   both agents should appear with a distinct tint (green or similar) vs idle
   agents.

2. **CausalPanel social events**: Press Q to open CausalPanel. Click on the
   tile where the social interaction occurred. The panel should show:
   - `[T] Social need  agent=0` (AgentDecision with social_reason)
   - `[T] Interaction started`
   - `[T] Interaction completed  fam=0.10`

3. **RelationshipState panel**: Press R to open relationship debug panel.
   After social interaction completes, panel should list:
   - `[REL] 0 ↔ 1  fam=0.10  hos=0.00`

4. **No regressions**: Idle agents (not socializing) render normally with
   palette-swap colours. Non-social causal events (building_placed, etc.)
   still render correctly in CausalPanel.

## Precedent Files

1. `rust/crates/sim-bridge/src/ffi/world_node.rs` — existing SimBridge pattern
   (Bridge Identity Contract, `collect_*` pure-Rust helpers)
2. `scripts/ui/panels/causal_panel.gd` — existing panel pattern to extend
3. `scripts/ui/agent_renderer.gd` — existing MultiMeshInstance2D driver to extend
4. `rust/crates/sim-test/tests/harness_p7_gamma_social_chronicle.rs` — social
   system setup patterns (co-located agents, loneliness threshold)
5. `rust/crates/sim-test/tests/harness_p9_beta_combat_system.rs` — SimBridge
   collect_* test pattern precedent

## Out of Scope

- Phase 8-δ MemoryReason / MemoryRecalled UI (separate dispatch)
- Phase 9-δ CombatReason / CombatStarted UI (separate dispatch)
- Any changes to sim-core components or sim-systems behavior
- AgentState new variants (AgentState::Socializing NOT added — use existing
  `Consuming { Agent(_) }` check)
- New ECS components

## Acceptance

1. `cargo test --workspace` passes (all 898+ prior tests remain green).
2. `cargo clippy --workspace --all-targets -- -D warnings` clean.
3. `python3 tools/localization_compile.py` succeeds; all 7 new locale keys
   present in both `localization/compiled/en.json` and `ko.json`.
4. Harness tests A1–A5 pass.
5. VLM visual analysis shows socializing tint + CausalPanel social events
   + RelationshipState panel entries (or VISUAL_WARNING with env cost
   applied per §7.1 policy).
6. `cargo test -p sim-test harness_p7_delta -- --nocapture`: all assertions
   pass.

## Your Task
You are the PLANNER. Read the feature description above and produce a test plan.
You do NOT write code. You write a plan that tells the Generator WHAT to test.

## Output Format
Output your test plan directly using this exact structure:

```
---
feature: p7-delta-social-ui
plan_attempt: 1
seed: 42
agent_count: 20
---

## Assertions

### Assertion 1: <name>
- metric: <what to measure>
- threshold: <value>
- type: <A|B|C|D|E>
- rationale: "<why this threshold — cite source for B, cite observed value for C>"
- ticks: <how long to simulate>
- components_read: [<ECS components the test queries>]

### Assertion 2: <name>
...

## Edge Cases
- <edge case 1>: <expected behavior>
- <edge case 2>: <expected behavior>

## NOT in Scope
- <what this test intentionally does NOT check>
```

## Rules
- Every threshold MUST have a Type (A/B/C/D/E) and rationale
- Read .claude/skills/worldsim-harness/evaluation_criteria.md for Type definitions
- Type C thresholds: you MUST state the observed value and margin
- Do NOT include implementation hints, code snippets, or architecture suggestions
- Do NOT suggest HOW to implement — only WHAT to verify

