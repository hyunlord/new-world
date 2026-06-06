# Fix Inspector Panel Overlap (Visualization C)

## Section 1: Implementation Intent

**Problem:** When the player clicks an agent, `agent_inspector_panel.gd`
(right-anchored, full-height Control, 280 px wide, `offset_top = 0.0`)
renders starting at the very top of the viewport — directly on top of
`hud_status_panel.gd`, which is also right-anchored at the top-right
(320 px wide, `offset_top = HUD_MARGIN(12)`, `offset_bottom = 192`).
Both panels share the top-right corner, so the inspector's
Agent/Pos/State/Target text overlaps the status panel's Day/Year +
resource counters + "Agent born" notification, producing unreadable
jumbled text (confirmed in the user's windowed screenshot).

**Approach:** Pure layout fix. Push the inspector's top edge down so it
begins *below* the status panel. The status panel occupies y ∈ [12, 192]
(`HUD_MARGIN .. HUD_MARGIN + PANEL_HEIGHT`). Setting the inspector's
`offset_top` to `204.0` (= `PANEL_HEIGHT 180` + `HUD_MARGIN 12` × 2)
leaves a 12 px gap below the status panel's bottom edge (192). The
status panel keeps a fixed height (180), so it cannot grow into the
inspector's region — the separation is stable.

**Tradeoff:** The inspector loses 204 px of vertical space at the top.
This is acceptable: the inspector has only 4 text rows + 3 progress bars
(≈ 160 px of content) and the viewport is ≥ 1080 px tall, so there is
ample room below y=204. No content is clipped.

This is display-position only. No simulation logic, no FFI, no data
changes. Rust is untouched (no dylib rebuild needed).

## Section 2: What to Build

**File changed (GDScript only):**
- `scripts/ui/panels/agent_inspector_panel.gd`
  - Add a module-level const documenting the derivation:
    ```gdscript
    # Top edge sits below hud_status_panel.gd (top-right, 320×180 from
    # y=HUD_MARGIN(12) to y=HUD_MARGIN+PANEL_HEIGHT=192) so the two
    # right-anchored panels never overlap. 180 + 12*2 = 204 → 12 px gap.
    const INSPECTOR_TOP_OFFSET := 204.0
    ```
  - In `_ready()`, change `offset_top = 0.0` → `offset_top = INSPECTOR_TOP_OFFSET`.
  - `anchor_top` stays `0.0`; `anchor_bottom`, `offset_bottom`,
    `offset_left`, `offset_right`, `PANEL_WIDTH` all UNCHANGED.

**New regression harness (Rust, sim-test):**
- `rust/crates/sim-test/tests/harness_fix_inspector_overlap.rs`

**Scope boundary — DO NOT:**
- Touch `hud_status_panel.gd`, `hud_topbar.gd` (A17 hash lock), or any
  other panel/renderer.
- Add/rename FFI functions or SimBridge methods.
- Change the inspector's width, anchors (other than reading offset_top),
  content, fields, ProgressBars, or `display_agent` logic.
- Add locale keys (none needed — no new user-visible text).
- Modify any Rust simulation code.

## Section 3: How to Implement

1. Open `scripts/ui/panels/agent_inspector_panel.gd`.
2. After the existing `const PANEL_WIDTH := 280.0` line, add the
   `INSPECTOR_TOP_OFFSET := 204.0` const with the explanatory comment
   above.
3. In `_ready()`, locate the right-anchor block:
   ```gdscript
   anchor_top = 0.0
   anchor_bottom = 1.0
   offset_left = -PANEL_WIDTH
   offset_right = 0.0
   offset_top = 0.0      # ← change this line
   offset_bottom = 0.0
   ```
   Change only `offset_top = 0.0` → `offset_top = INSPECTOR_TOP_OFFSET`,
   and update the preceding comment to note the panel now starts below
   the status panel.
4. Leave everything else (bg ColorRect, VBoxContainer, labels, bars)
   exactly as-is — they are children laid out relative to the panel rect,
   so they shift down with the panel automatically.

**Regression harness assertions** (`harness_fix_inspector_overlap.rs`,
reads `scripts/ui/panels/agent_inspector_panel.gd` and
`scripts/ui/panels/hud_status_panel.gd` as text). The assertions must be
**structurally rigorous** — substring matches that a malformed source or a
child-node assignment could satisfy are NOT acceptable:

- A1 (Type A): count **module-level** (column-0) `const INSPECTOR_TOP_OFFSET`
  declarations — must be **exactly 1** — then parse its literal and assert
  it equals exactly `204.0`.
- A2 (Type A, structurally scoped): extract the `_ready()` body and assert a
  **bare panel-rect** `offset_top = INSPECTOR_TOP_OFFSET` assignment exists
  (identifier with no `obj.` prefix, so `_vbox.offset_top` is excluded);
  AND assert no bare panel-rect `offset_top = 0.0` remains, whitespace-
  insensitively.
- A3 (Type D — no-overlap invariant): parse `HUD_MARGIN` and `PANEL_HEIGHT`
  from `hud_status_panel.gd`; assert
  `INSPECTOR_TOP_OFFSET >= HUD_MARGIN + PANEL_HEIGHT` (i.e. 204 ≥ 192) so the
  inspector top is at or below the status panel bottom. Survives future
  status-panel resizes.
- A4 (Type A — regression, structurally scoped): parse `PANEL_WIDTH` and
  assert it equals exactly `280.0`; AND assert the `_ready()` body keeps a
  **bare panel-rect** `anchor_right = 1.0` (excluding `bg.anchor_right` /
  `_vbox.anchor_right`).

Use plain text parsing (read file, parse numeric literals, scope to the
`_ready()` body for bare-assignment checks). No engine launch required.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | `agent_inspector_panel.gd` offset_top + const | 🔴 DIRECT | — |
| T2 | `harness_fix_inspector_overlap.rs` regression test | 🟢 DISPATCH | T1 |

DIRECT for T1: the production change is a single const + one assignment
(<10 lines), below the dispatch threshold and tightly coupled to the
exact existing layout block.

## Section 5: Localization Checklist

No new localization keys. (Layout-only change; no user-visible text added.)

## Section 6: Verification & Notion

```bash
# Rust workspace must stay green (harness compiles, nothing else affected)
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail

# New regression harness passes
cd rust && cargo test -p sim-test --test harness_fix_inspector_overlap -- --nocapture

# GDScript parse check
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --check-only \
  --script scripts/ui/panels/agent_inspector_panel.gd
```

Expected: workspace tests pass; `harness_fix_inspector_overlap` A1–A4
green; GDScript parses clean.

Windowed confirmation (manual, after merge): launch the game, click an
agent — the inspector now appears below the status panel; Day/Year +
resources (top) and Agent/Pos/State (below) are both readable, no
overlap.

Notion: update the V7 visualization tracking page (C complete; D next).
