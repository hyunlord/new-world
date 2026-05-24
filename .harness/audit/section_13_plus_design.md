# V7 Section 13+ Design — Phase 12 Anchor

**Status**: High-level anchor (Sec13-1-a). Detailed sub-stage decomposition deferred to
`.harness/plans/phase12.md` (planning-first dispatch, Phase 6/7/8/9/10/11 precedent).

**Authored**: 2026-05-24. **Author**: governance batch (post-V7 + Phase 11-α + D1 closure).

**Precedent**: `.harness/audit/section_12_plus_design.md` (Phase 11 anchor — Map
Rendering Foundation Sprint, commit `919d510f`, 2026-05-21). Structure and tone mirror
that document. Section 13 supersedes Section 12 only for the Phase 12 anchor;
Section 12's Phase 11 anchor remains the authoritative record for what Phase 11-α
actually delivered (`d7c34d78` + `450f39cd` D1 tint retune).

---

## 0. Provenance & 8th Escalation Disclosure

This is the eighth successive escalation against the absence of a V7 Master
Direction document. Phase 6 planning, Phase 7 planning, Section 8+ design,
Section 9+ design, Section 10+ design, Section 11+ design, Section 12+ design,
and now Section 13+ design have all triggered the same condition:

- `/mnt/project/` (claude.ai project_files): absent
- `find . -iname "*master*direction*"` in repo: zero matches
- Root `.md` files: `AGENTS.md`, `CLAUDE.md`, `README.md` only
- The "V7 Master Direction" referenced in `rust/crates/sim-core/src/lib.rs:3`
  remains a **conceptual anchor**, not a written artefact

Path chosen (Sec13-1-a): **`.harness/audit/section_13_plus_design.md`** (this
file, new). Filename mirrors Section 12's `section_12_plus_design.md`. Section 12
explicitly demarcated Section 13 as a *separate future document* implicitly via
its §3 "Section 13+ candidates (deferred)" list. The modular path honours that
demarcation.

Governance closure chain — extended (post-Phase-11-α + D1, 2026-05-24):

| Stage | Commit | Scope |
|-------|--------|-------|
| 1  | `a0666b6c` | V7 Foundation Week 1-12 complete |
| 2  | `2e51c167` | CLAUDE.md V7 reset 정합 update |
| 3  | `0ed3ec16` | Section 8+ design — Phase 7 anchor (Multi-agent Social System) |
| 4  | `f1c12f9d` | Phase 7-γ chronicle harness implementation (V7 Phase 7 complete) |
| 5  | `c924770d` | Phase 7-γ closure declaration + v7_progress.md status reflection |
| 6  | `67c9a49d` | Section 9+ design — Phase 8 anchor (Memory System) |
| 7  | `a6ce6d9d` | Phase 8-α + β closure declaration + governance chain update |
| 8  | `0660f4ea` | Phase 8-γ implementation (chronicle harness, 16-assertion lifecycle) |
| 9  | `7da81c0b` | Phase 8-γ closure declaration (V7 Phase 8 complete ★) |
| 10 | `f0a60968` | Section 10+ design — Phase 9 anchor (Combat System) |
| 11 | `58976d1f` | Phase 9-α (BodyHealth substrate) |
| 12 | `4fb2e16e` | Phase 9-β (Combat System runtime) |
| 13 | `86ec5fff` | Phase 9-γ (Combat chronicle harness, V7 Phase 9 complete ★) |
| 14 | `447b1ba2` | Phase 9 closure declaration |
| 15 | `813e2d06` | Phase 7-δ Social UI |
| 16 | `7ce33a33` | Phase 8-δ Memory UI |
| 17 | `4f0ed817` | Phase 9-δ Combat UI (★ message mislabeled — see v7_progress.md §governance gaps) |
| 18 | `bd36219b` | All δ complete declaration + 4f0ed817 audit |
| 19 | `00274b57` | Issues 12+13 fix (harness pipeline governance) |
| 20 | `ebbf6ddc` | Issue 14 fix (cold tier auto credit) |
| 21 | `ddd5348c` | Issue 15 fix (Drafter agent revision degradation Pattern G) |
| 22 | `182476f9` | Issue 16 fix (pipeline_report.md mechanism gap closure) |
| 23 | `6089976c` | Section 11+ design — Phase 10 anchor (Multi-building Settlement) |
| 24 | `bd36219b` | V7 + All δ Final Declaration |
| 25 | `182476f9` | Issue 16 patch |
| 26 | `6089976c` | Section 11+ anchor design |
| 27 | `313cfcd3` | Phase 10-α (Settlement substrate + AgentBorn) |
| 28 | `930bcbcf` | Phase 10-β (SettlementSystem + 8th cascade + Birth) |
| 29 | `40c36d13` | Phase 10-γ (Settlement chronicle harness, V7 Phase 10 complete ★★★★) |
| 30 | `c381eff5` | V7 + Phase 10 Final Declaration |
| 31 | `919d510f` | Section 12+ design — Phase 11 anchor (Map Rendering Foundation) |
| 32 | `d7c34d78` | Phase 11-α implementation (agent movement interpolation + state tint substrate) |
| 33 | `450f39cd` | D1 STATE_TINTS color retune (Phase 11-α visible-delta fix) |
| 34 | *(this commit)* | Section 13+ design — Phase 12 anchor (Tier 1 Sprites + Camera Zoom + Agent Sprite Scale) |

V7-specific disclosures carried forward:
- 3 ENV-BYPASS chains closed; 4th–6th avoided via Option D Issue 15 fix path
- Issues 12+13+14+15+16 closure chain complete
- Issue 16 post-fix verified: 3 consecutive live tests passed (Phase 10-α + β + γ)
- 2× 100/100 PERFECT first-dispatch: Phase 8-α + Phase 10-α
- Phase 11-α (`d7c34d78`) APPROVE on Codex but `visual:WARNING (env)` — no
  runtime rendering proof was captured during evaluator review
- D1 (`450f39cd`) Score 88/100 B grade, APPROVE on attempt 1 of attempt-2 plan;
  STATE_TINTS retune (cool-blue Idle + saturated 1/2/3) committed but on-screen
  visible-delta verification is still user-pending (16×18 px sprite at 1920×1080
  is sub-resolution for the standard VLM whole-scene capture)
- **User mandate (2026-05-24): "최소한 게임같이" + "단편 dispatch 회피"** —
  drives Phase 12 anchor below

---

## 1. Context

V7 architecture base remains intact and continues to compose:

- 12 ECS components (Phase 7+8+9+10 cumulative) + 10 runtime systems
- 14 CausalEvent variants + 8 DecisionReason variants + 4 MemoryRecallTrigger
- Full Causal chain (8 reasons → 8 emission chains)
- Phase 11-α landed substrate: Gaffer accumulator (position interpolation),
  `MultiMesh.use_colors = true`, 4-entry STATE_TINTS palette,
  `palette_swap.gdshader` `modulate × palette` composition
- D1 landed: STATE_TINTS retuned to cool-blue / saturated yellow / pink / green
  + harness A20/A21/A22 anti-revert guards

★ Critical user-screen evidence (2026-05-21 → 2026-05-24, post-D1):
the screen still looks dominated by **grey background + black rectangles +
green dots + diamond influence stamp + γ-2-β CausalPanel**. The Phase 11-α
+ D1 tint change does land in the MultiMesh upload (`set_instance_color`
verified), but each agent renders at `SPRITE_SCALE = 0.25` × 64×72 px =
**16×18 px on a 1920×1080 viewport**. At that scale, palette-swapped agents
modulated by a tint are sub-resolution to the human eye at default camera
zoom, and indistinguishable from the prior identity-white appearance to the
pipeline VLM's whole-scene grading.

Substrate grep verification (this dispatch's Step 0 — files read live,
not from memory):

- `assets/sprites/` inventory: **205 PNG files** across:
  - `walls/{material}/[1-3].png` — 21 files, 7 materials × 3 variants, 16×16
  - `floors/{material}.png` — 9 files
  - `furniture/{type}.png` — 108 files
  - `buildings/{type}/[N].png` — 64 files (gathering_marker 16, cairn 8, …), 32×32
  - `agent_base.png` (64×72 RGBA, 4×3 palette layout — Phase 4-γ)
  - `palette_lut.png` (palette swap LUT — Phase 4-γ)
  - `wildlife/` — 3 files (bonus, not in prior inventory)

- `scripts/ui/world_renderer.gd`: renders **only one thing** — an influence
  channel overlay (Warmth / Light / Noise / Danger / Spiritual / Beauty)
  via a single `Sprite2D` whose `Image` is rebuilt every frame from
  `WorldSim.get_influence_overlay(channel)`. There is **no TileMap, no
  per-tile sprite, no terrain rendering**. The black-rectangle background
  the user sees is the GRID `64×64 L8` Image at `Sprite2D.scale = (16, 16)`
  = 1024×1024 px placed at `position = (960, 540)`. The 205 wall/floor/
  furniture/building sprites are **not referenced** by any scene script.

- `scenes/main.tscn`: Camera2D node exists at `position = (960, 540)`
  with `zoom = Vector2(1, 1)`. No script attached, no zoom input handler,
  no bounds, no smooth interpolation. The camera is effectively a
  fixed-frame wide shot of the entire bootstrap area.

- `scripts/ui/agent_renderer.gd`: agent sprite scale derived from
  `SPRITE_SCALE = 0.25` and `SPRITE_W/H = 64/72` → **16×18 px on screen**.
  The MultiMesh per-instance transform multiplies SPRITE_SCALE by the
  recall/combat cue scale boost (1.25× / 1.3×) when active, so even the
  boosted sprite tops out at ~20×22 px.

- DGX Spark ComfyUI generation pipeline remains the path for any Tier 2/3
  sprite expansion (resource nodes, agent variants, environment props).
  Phase 12 deliberately stays within the existing Tier 1 inventory.

---

## 2. Phase 12 Anchor — Tier 1 Sprites Integration + Camera Zoom + Agent Sprite Scale Sprint (V7 Week 24-25)

### 2.1 Choice rationale

The user-screen evidence above proves that the gap between V7's
*simulation* completeness (12 components, 10 systems, full causal chain)
and what a human observer *sees* is not a backend gap — it is a
**rendering integration gap**. Three concrete deficits:

1. **No terrain rendering**: the 21 wall + 9 floor sprites generated for
   V7 sit unused; the world appears as a grey/black silhouette of the
   influence overlay alone.
2. **No camera zoom**: a fixed 1920×1080 view of a 64×64 tile grid (=
   1024×1024 px world) gives no opportunity to see individual sprites at
   their authored detail; conversely, no overview-out is available either.
3. **Sub-resolution agents**: D1 proved the STATE_TINTS palette is
   plumbed end-to-end through Rust ECS → FFI → MultiMesh → shader, but
   at 16×18 px the resulting hue change is below the visual delta
   threshold for both the user and the standard pipeline VLM.

Phase 12 addresses all three in one sprint, deliberately combined per
the user mandate "단편 dispatch 회피" (avoid one-shot dispatches that
each look like nothing). The resulting sprint reaches the **first
"minimally game-like" milestone** since V7 reset — a windowed Godot
session should show tiled terrain, zoom-able camera, and agents whose
state tint is actually visible.

Compositional substrate: Phase 6 Construction + Phase 10 Settlement
already track `BuildingId` + `ConstructionState` per building. Phase 12
maps those types to the 64 building sprites without backend changes.

User axioms touched: **#1 causal traceability**
("UI-visualizable")  ← partially satisfied; **#3 emotional depth + visual
milestone** ← primary target.

### 2.2 High-level scope (in / out)

**IN scope (Phase 12)**:
- Camera zoom controls (Camera2D zoom, mouse-wheel input, bounds, smooth
  interpolation)
- Agent sprite scale tuning (resolve sub-resolution path — either
  `SPRITE_SCALE` increase or camera-default zoom-in, or both)
- Tier 1 sprites integration:
  - Walls 21 sprites — per-tile or per-segment wall rendering
  - Floors 9 sprites — terrain background rendering (replaces grey)
  - Buildings 64 sprites — ConstructionState → sprite assignment for
    buildings tracked by Phase 6/10
  - Furniture 108 sprites — Settlement integration if substrate exists
    (Phase 10 Settlement → Settlement.furniture[] list inspection); else
    deferred to Phase 12-γ stretch goal
- `WorldRenderer` extension (tile layer + sprite assignment pipeline)
  *without* breaking the existing influence-overlay layer (overlays must
  remain toggleable / blendable)

**OUT of scope (Phase 13+ defer)**:
- Resource nodes (berry / tree / stone): require DGX Spark sprite
  generation; Tier 1 inventory does not include them
- HUD (시간/자원/시대/위기): Section 14+ candidate
- Inspector (V7 substrate visualization): Section 14+ candidate
- Sidebar / Chronicle UI: Section 14+ candidate
- Day/night cycle visuals: Section 14+ candidate
- Roads, desire-line paths: Section 14+ candidate
- Backend simulation expansion (V4 Phase 3: Health 85부위 + Knowledge +
  Family): orthogonal track
- Advanced AI (Section 8+ §3 #4): orthogonal track

### 2.3 Sub-stage shape (planning-first dispatch decomposes)

- **Phase 12-α** — Camera zoom + agent sprite scale fix (~3-5h,
  `--quick` lane). Pure GDScript: Camera2D zoom controls + smooth
  interpolation + bounds, optional SPRITE_SCALE bump. First dispatch:
  produces immediate visible delta (zoom-in shows D1 STATE_TINTS clearly).
- **Phase 12-β** — Tier 1 sprite integration: walls + floors + buildings
  (~5-7h, `--quick` or `--full` lane depending on whether
  `WorldRenderer` refactor crosses sim-* boundaries — Step 0 grep
  indicates renderer is GDScript-only so `--quick` likely applies).
  TileMap or per-tile Sprite2D for walls/floors; ConstructionState →
  sprite mapping for buildings via Phase 6/10 substrate.
- **Phase 12-γ** — Furniture sprites + Settlement integration (~3-5h).
  Predicated on Phase 10 Settlement substrate exposing a furniture list;
  if absent, this stage drops to a per-building stub or is fully
  deferred to a future Settlement-furniture phase.
- **Phase 12-δ (optional)** — polish + integration harness + runtime
  visual evidence (the harness gap D1 honestly disclosed).

### 2.4 Dependencies

- V7 (Foundation + Phase 7 + Phase 8 + Phase 9 + Phase 10 + Phase 11-α
  + D1) closed; substrate stable
- 205 Tier 1 sprite assets present and dimensioned (Step 0 grep)
- Phase 6 Construction substrate (`BuildingId`, ConstructionState)
- Phase 10 Settlement substrate (Settlement records, agent membership)
- Godot 4.6 TileMap / TileMapLayer API available
- Camera2D node already in `main.tscn` — only zoom + input handler needed

### 2.5 Open questions (planning-first dispatch resolves)

The 10 P12Plan-* decisions are left to `.harness/plans/phase12.md`:

1. Tier 1 integration mechanism: TileMap vs per-tile Sprite2D vs
   MultiMeshInstance2D
2. Sprite placement: procedural worldgen vs manual scene placement
3. Camera zoom control surface (mouse-wheel only / +/- keys / both /
   pinch gesture)
4. Agent sprite scale resolution: SPRITE_SCALE bump vs camera default
   zoom-in vs both
5. agent_base.png reuse path (existing 64×72 4×3 layout preserved)
6. WorldRenderer extension shape (TileMapLayer subnode vs new node
   class)
7. Building sprite assignment: ConstructionState → sprite enum mapping
8. Furniture sprite placement: Settlement integration path or stub
9. Resource node sprites: confirm Phase 13 deferral or include placeholder
10. α/β/γ/δ sub-stage decomposition with concrete file lists and
    estimates

---

## 3. Section 14+ Candidates (Deferred)

Items recognised as user-visible improvements but explicitly out of
Phase 12 scope. Ordering is informational, not a commitment.

- **HUD substantial sprint**: time-of-day, resource counters, era marker,
  crisis indicator. First-order visible meta layer.
- **Inspector substantial sprint**: V7 substrate visualization
  (Hunger/Thirst/Sleep/Social bars, AgentState/TargetKind labels, recent
  causal events). Reuses backend substrate; rebuild the legacy
  `entity_detail_panel_v4.gd` against current FFI.
- **Sidebar / Chronicle substantial sprint**: settlement chronicle (Phase
  10-γ), social relationship view, memory recall stream. Extends γ-2-β
  CausalPanel into a docked sidebar.
- **Overlay substantial sprint**: complete the food / danger / warmth /
  social / knowledge / resource overlay legend with toggleable layer
  visibility and minimap.
- **Resource nodes (DGX Spark + Phase 13)**: berry / tree / stone /
  water source sprite generation pipeline + worldgen placement +
  rendering.
- **Visual depth**: roads / culture zones / military / combat effects
  (carry-over from Section 8+ §3 #5).
- **V4 Phase 3 backend**: Health 85부위 + Knowledge + Family
  (orthogonal track, not blocked by Phase 12).
- **Advanced AI**: Section 8+ §3 #4 remaining items (Behavior Trees /
  GOAP / utility curve refinement).

Re-ranking happens after Phase 12 closure and post-12 user feedback.

---

## 4. Single Source of Truth

| Artefact | Authority |
|----------|-----------|
| Phase progress tracking | `.harness/audit/v7_progress.md` |
| Phase 7 anchor | `section_8_plus_design.md` |
| Phase 8 anchor | `section_9_plus_design.md` |
| Phase 9 anchor | `section_10_plus_design.md` |
| Phase 10 anchor | `section_11_plus_design.md` |
| Phase 11 anchor | `section_12_plus_design.md` |
| **Phase 12 anchor** | **`section_13_plus_design.md`** (this file) |
| Phase 12 sub-stage decomposition | `.harness/plans/phase12.md` (planning-first dispatch, follows immediately after this commit) |

---

## 5. Honest Reservations

- V7 Master Direction document remains absent (8th escalation,
  unchanged from §0)
- D1 (`450f39cd`) sub-resolution disclosure: pipeline pass proved
  STATE_TINTS shape + plumbing but did **not** prove on-screen visible
  delta; that proof is what Phase 12-α targets first
- Phase 11-α + D1 = two single-sub-stage dispatches; the gap between
  "code lands" and "user sees a game-like change" remains the central
  governance honesty
- Phase 12 is a **substantial scope sprint** (~11-17h across 3-4
  dispatches). The user mandate "단편 dispatch 회피" explicitly accepts
  the larger commitment in exchange for a coherent visible milestone
- Tier 1 integration mechanism choice (TileMap vs per-tile Sprite2D vs
  MultiMesh) is non-trivial; defer to `phase12.md` Step 0 evidence
- Furniture integration depends on Settlement substrate having a
  furniture concept; if absent, Phase 12-γ becomes a stub or fully
  defers
- Pipeline VLM evidence vs user-screen evidence gap remains
  unresolved at the harness level (the standard
  `harness_visual_verify.gd` does not exercise AgentRenderer or per-tile
  rendering at human-eye scale); Phase 12-δ may close that gap with a
  feature-specific runtime visual harness, but the precedent from C-1
  shows that pipeline orchestration of feature-specific harnesses is
  itself a separate problem
- Visible-delta confirmation after Phase 12 closure remains a
  user-driven step (windowed Godot run), exactly as it was for D1
