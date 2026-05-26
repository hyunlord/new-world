# V7 Section 15+ Design — Phase 14 Anchor

**Status**: High-level anchor (Sec15-1-a). Detailed sub-stage decomposition
deferred to `.harness/plans/phase14.md` (planning-first dispatch).

**Authored**: 2026-05-26. **Author**: governance batch (post-Phase 13
sprint closure + user honest reassessment).

**Precedent**: `.harness/audit/section_14_plus_design.md` (Phase 13
anchor — Game-like UI Sprint, commit `f2efcd9b`). Structure mirrors
that document. Section 15 supersedes Section 14 only for the
Phase 14 anchor; Section 14's Phase 13 anchor remains the
authoritative record of what the α/β/γ/δ/ε chain actually
delivered.

---

## 0. Provenance & 10th Escalation Disclosure

Tenth successive escalation against the absence of a V7 Master
Direction document. The condition is unchanged across Sections
8/9/10/11/12/13/14/15.

Path chosen (Sec15-1-a): `.harness/audit/section_15_plus_design.md`
(this file, new).

Governance closure chain — extended (post-Phase-13-ε, 2026-05-26):

| Stage | Commit | Scope |
|-------|--------|-------|
| 1-30 | … | Foundation + Phase 7/8/9/10 chains + V7 closure |
| 31 | `919d510f` | Section 12+ — Phase 11 anchor |
| 32 | `d7c34d78` | Phase 11-α |
| 33 | `450f39cd` | D1 STATE_TINTS retune |
| 34 | `6843176f` | Section 13+ — Phase 12 anchor |
| 35-38 | Phase 12 α/β.1/β.2 A3/γ | Tier 1 sprites + camera + ConstructionSite + Settlement |
| 39 | `0238aef2` | D Phase A (8 GDScript errors) |
| 40 | `a437c547` | D Phase B (Pipeline GDScript strict check) |
| 41 | `f2efcd9b` | Section 14+ — Phase 13 anchor |
| 42 | `0714c891` | Phase 13-α camera zoom 3.0× + 3 building sprites |
| 43 | `98751086` | Phase 13-β resource placeholders |
| 44 | `df0041f6` | Phase 13-γ STATE_SCALE_BOOST |
| 45 | `91041f3f` | Phase 13-δ HUD top-bar |
| 46 | `f357e20b` | Phase 13-ε 3 bootstrap buildings |
| 47 | *(this commit)* | Section 15+ design — Phase 14 anchor (RimWorld-like Visual Overhaul Sprint) |

**Phase 13 sprint honest reassessment (2026-05-26)**:
- α + β + γ + δ + ε all landed pipeline-APPROVED.
- BUT user mandate "다른 게임 베껴라" (RimWorld + DF + Songs of
  Syx + others) arrived mid-sprint and was NOT incorporated into
  the substages already in flight. Autopilot autonomously
  completed δ + ε before the mandate could re-anchor the work.
- BUT user windowed verify step (per phase13.md §6) was skipped
  because Auto mode persisted through the sprint chain.
- User assessment of Phase 13 result: still does not read as a
  game. Agents move but it is not clear why. Sprite distinction
  is present but is not "RimWorld-like". The single-substage
  pattern (now nine consecutive: Phase 11-α + D1 + Phase 12-α +
  β.1 + β.2 A3 + γ + D-A + D-B + each Phase 13 sub-stage as a
  separate landing) compounded into shipped code but did not
  compound into a game-like milestone.

**User mandate (2026-05-25, drives this section)**:
1. **Copy other games.** Reference titles ordered by relevance:
   RimWorld (1st), Dwarf Fortress, Songs of Syx, then Banished,
   Factorio, Stronghold, Caesar III, Tropico, Prison Architect,
   Oxygen Not Included.
2. **Substantial sprint chain, not single-substage**. Phase 14 is
   a 6-substage chain (α → ζ).
3. **No mid-sprint confirmation**. Confirmation only at full
   Phase 14 closure (post-ζ).
4. **Phase 13 result stays.** No revert. Phase 14 builds on (or
   replaces parts of) Phase 13 outputs.

The 8-9 consecutive single-substage commits (Phase 11-α through
Phase 13-ε) are explicitly archived as the failed pattern;
Phase 14 retires it for the second time, with a longer chain and
explicit game-reference targets per substage.

---

## 1. Context

V7 backend remains intact (12 ECS components + 10 runtime systems
+ 14 CausalEvent variants + 8 DecisionReason variants + 4
MemoryRecallTrigger).

Phase 12 + Phase 13 visual state (live grep, 2026-05-26):
- `assets/sprites/agent_base.png`: 64×72 RGBA, 4×3 palette layout
  (Phase 4-γ). Already a human-like figure — Phase 14 can keep or
  replace it.
- `assets/sprites/wildlife/`: 3 sprites (bear, boar, wolf), unused
  in current renderer. Available as threat-indicator assets.
- `agent_renderer.gd`: 417 lines (after Phase 13-γ STATE_SCALE_BOOST).
- `world_renderer.gd`: 344 lines (after Phase 13-ε 3 buildings).
- `panels/hud_topbar.gd`: 103 lines (Phase 13-δ).
- Camera default zoom: 3.0× (Phase 13-α).
- Tile size: 16 px. Agent on-screen: 48-62 px at zoom 3.0× (Phase
  13-γ active-state boost).

Backend extractable per-agent fields (sim-core/src/components/):
- `Agent.id: AgentId` (u64 monotonic)
- `Hunger.value: f32`, `Thirst.value: f64`, `Sleep.fatigue: f64`
- `AgentState` enum (Idle / Seeking / Consuming + TargetKind)
- `Position.x, y: u32`
- `Social.value: f32` (Phase 7)
- `Memory` (Phase 8) — bounded ring
- `Body health`, `Combat` (Phase 9)
- Settlement membership (`Settlement.member_agents`)

Existing FFI collectors:
- `collect_agent_snapshot` (entity_bits, x, y, state_tag, agent_id)
- `collect_construction_snapshot` (entity_bits, x, y, progress,
  required_progress)
- `collect_settlement_snapshot` (entity_bits, settlement_id,
  centroid_x, centroid_y, member_count)
- `collect_relationship_snapshot` (Phase 7-δ)
- `collect_tile_causal_history`, `collect_event_chain`

**Gap for Phase 14-γ (click inspector)**: no per-agent-detail
FFI exists. A new `collect_agent_detail(agent_id) -> Dict` is
needed to surface Hunger / Thirst / Sleep / AgentState into the
GDScript inspector. This is a small, single-method FFI extension
(~50-80 lines Rust) modelled on existing collectors.

---

## 2. Phase 14 Anchor — RimWorld-like Visual Overhaul Sprint (V7 Week 28-29)

### 2.1 Choice rationale

Phase 13 commits were each individually correct but did not
compound. The honest signal: the small-step approach to "make
it game-like" cannot reach a perceptual milestone — what is
needed is a deliberate copy of a working reference game's
overall visual language, not local optimisations.

Reference titles, ordered by relevance:

1. **RimWorld** (primary) — clear humanoid sprites, per-job
   colour overlays, head-icons for current activity, hover/click
   inspector, persistent HUD.
2. **Dwarf Fortress** — race/job colour distinctions, status
   glyphs, hover info.
3. **Songs of Syx** — large populations, zoom-level adaptive
   rendering, resource flow visualisation.
4. **Banished** — village density, building variety, season cues.
5. **Factorio** — explicit resource flow lines, distinct icons.
6. **Stronghold** — job-specific routes, activity hand-off.
7. **Caesar III / Tropico** — city-camera, citizen-level activity,
   persistent HUD.
8. **Prison Architect** — per-inmate alerts, activity overlay.
9. **Oxygen Not Included** — Duplicant click info, resource +
   atmosphere visualisation.

User axioms touched: **#1 causal traceability**
(UI-visualizable) ← finally addressed via click inspector;
**#3 emotional depth + visual milestone** ← primary target.

### 2.2 High-level scope (in / out)

**IN scope (Phase 14)**:
- Agent sprite overhaul (RimWorld + DF style — clear humanoid
  shape, optional per-role colour overlay)
- Activity head-icon (RimWorld style — small glyph above agent
  indicating Hunger / Thirst / Sleep / Construction / Idle)
- Resource sprite variety (RimWorld + Banished — 5+ types:
  wood, stone, berry, water, food)
- Building sprite variety (Banished + Songs of Syx — add 4+ to
  the current campfire/cairn/hearth set)
- Click inspector (RimWorld + ONI — click agent or building,
  side panel shows current activity + needs)
- HUD persistent expansion (RimWorld + Tropico — time/speed +
  population categories + resource totals + notifications)
- Activity-route visualisation (Stronghold + Factorio — movement
  trails for active agents, gather/build effect cues)
- Zoom-level adaptive rendering (Songs of Syx — zoom > 2 shows
  individual icons + names; zoom < 1 shows settlement clusters)

**OUT of scope (Section 16+ defer)**:
- Backend substrate addition (`BuildingType` / `Wall` /
  `Furniture` / `ResourceNode` ECS components)
- Inspector full 5-tab (Phase 14 ships click-info baseline only)
- Sidebar / Chronicle UI
- Day/night cycle visuals
- V4 Phase 3 backend (Health 85-부위 + Knowledge + Family)
- Advanced AI extensions
- Sprite asset generation pipeline (DGX Spark ComfyUI integration
  — only invoked ad-hoc if a specific asset is missing)

### 2.3 Sub-stage shape (planning-first dispatch decomposes)

Each substage explicitly names the reference game(s) it borrows
from.

- **Phase 14-α** — Agent sprite overhaul (RimWorld + DF). Clearer
  humanoid figure (reuse `agent_base.png` if it already reads as
  human; otherwise specify a DGX Spark replacement), per-role
  colour overlay, head-icon for current activity. 5-8h, `--quick`
  if no new asset needed.
- **Phase 14-β** — Resource + building sprite variety (RimWorld +
  Banished + Songs of Syx). 5+ resource types replacing the single
  storage_pit placeholder from Phase 13-β. 4+ building variants
  beyond the current campfire/cairn/hearth set. 5-8h, `--quick`.
- **Phase 14-γ** — Click inspector (RimWorld + ONI). New Rust
  `collect_agent_detail(agent_id)` FFI returning Hunger / Thirst
  / Sleep / AgentState / current target. Same for ConstructionSite
  and Settlement. GDScript side panel mounted under existing UI
  CanvasLayer. 8-12h, `--full` (Rust FFI extension).
- **Phase 14-δ** — HUD persistent expansion (RimWorld + Tropico).
  Extend Phase 13-δ HudTopbar from 4 counters to time-of-day +
  speed-control + population breakdown + resource totals +
  notification area. 5-8h, `--quick`.
- **Phase 14-ε** — Activity route + gather/build cues (Stronghold
  + Factorio). Trail rendering for non-Idle agents; particle/glyph
  effects for ConstructionSite progress + Consuming(Resource).
  5-8h, `--quick`.
- **Phase 14-ζ** — Zoom-level adaptive rendering (Songs of Syx).
  Thresholded sprite swap: at zoom > 2 individual sprites + icons;
  at zoom < 1 settlement-cluster heatmap. 3-5h, `--quick`.

Each substage is `--quick` except γ (Rust FFI extension is
`--full`). Total: 31-49h across 6 dispatches.

### 2.4 Dependencies

- V7 backend (closed, stable)
- Phase 12 sprint (camera + terrain + ConstructionSite +
  Settlement — Phase 14 reuses ConstructionSite + Settlement
  FFIs for the click inspector)
- Phase 13 sprint (HUD scaffold + state-scale boost reused;
  placeholder resources from β replaced by variety in 14-β;
  3-building bootstrap from ε reused as visible anchor while
  14-β diversifies the texture pool)
- D Phase B GDScript strict check (`a437c547`) — runs on every
  Phase 14 dispatch
- 207 Tier 1 sprite assets (some Phase 14 needs may exceed this;
  DGX Spark ComfyUI generation reserved for missing-asset cases
  only, not pre-emptive)

### 2.5 Open questions (planning-first dispatch resolves)

The 10 P14Plan-* decisions are in `.harness/plans/phase14.md`.
Two require explicit user confirmation:

- **P14Plan-1 — Agent sprite source**: reuse `agent_base.png`
  (already a 64×72 humanoid) vs DGX Spark replacement
- **P14Plan-5 — Inspector FFI extension scope**: which fields
  surface in the new `collect_agent_detail`

Other P14Plan-* decisions can be locked at planning time.

---

## 3. Section 16+ Candidates (Deferred)

- **Backend substrate addition phase**: `BuildingType` +
  `Furniture` + `Wall` + `ResourceNode` ECS components
- **Inspector full 5-tab**: V7 substrate visualization (Hunger/
  Thirst/Sleep/Social + AgentState + recent causal events +
  relationships + memory ring)
- **Sidebar / Chronicle UI**: settlement history viewer
- **Stale dylib guard**: detect dylib mtime < Rust source mtime
- **Day/night, roads, culture zones, military, combat effects**
- **V4 Phase 3 backend**: Health 85부위 + Knowledge + Family
- **Sprite asset generation pipeline (DGX Spark ComfyUI)**:
  ad-hoc invoked during Phase 14 if needed; full integration
  is Section 16+

Re-ranking happens after Phase 14 closure.

---

## 4. Single Source of Truth

| Artefact | Authority |
|----------|-----------|
| Phase progress tracking | `.harness/audit/v7_progress.md` |
| Phase 7-13 anchors | `section_8_plus_design.md` … `section_14_plus_design.md` |
| **Phase 14 anchor** | **`section_15_plus_design.md`** (this file) |
| Phase 14 sub-stage decomposition | `.harness/plans/phase14.md` (planning-first dispatch follows this commit) |

---

## 5. Honest Reservations

- Nine consecutive single-substage commits (Phase 11-α through
  Phase 13-ε, including D Phase A/B) pipeline-passed individually
  but did NOT compound into a recognisable game. Phase 14
  explicitly retires that pattern with a 6-substage chain anchored
  on reference-game targets.
- Phase 13's user-verify step was skipped because Auto mode
  persisted through the sprint chain. Phase 14's mandate is
  similar ("no mid-sprint confirmation, full closure only") but
  Auto-mode risk is acknowledged: if Auto mode triggers premature
  closure declaration, the user retains the explicit right to
  pause via direct intervention.
- VLM whole-scene grader sub-resolution problem (CLAUDE.md, D
  Phase B doc) — at zoom 3.0× sprites are 48-62 px which crosses
  the threshold for the agent layer, but per-sprite head-icons
  (Phase 14-α) and trail glyphs (Phase 14-ε) will likely remain
  sub-resolution. Per-substage VISUAL_WARNING is environmental,
  not blocking.
- D Phase B's GDScript strict check is build-time only. Runtime
  FFI dylib staleness is still not auto-detected. Phase 14-γ's
  new FFI is the substage most exposed to this; user windowed
  verify after γ implicitly tests the dylib rebuild path.
- `agent_base.png` is already a humanoid figure. If the user's
  RimWorld reference implies a different sprite style, P14Plan-1
  asks whether to keep or replace. DGX Spark ComfyUI generation
  is the replacement path; this dispatch does NOT pre-emptively
  generate.
- 4 of 6 substages need NO new sprite assets (α reuses
  agent_base; β picks from existing 207 Tier 1; γ is FFI + UI;
  δ is HUD; ε is procedural effects; ζ is GDScript logic).
- Phase 14-γ Rust FFI extension is the only `--full` lane
  substage. Other 5 are `--quick`.
- User confirmation comes ONLY after full Phase 14 closure (post-ζ).
  Per-substage commits will land per pipeline protocol; the
  integrated visible milestone is the user-mandated checkpoint.
- Total estimated 31-49h across 6 dispatches + pipelines ≈ 5-7
  active days plus pipeline elapsed time.
- "Sprite asset shortage" risk is bounded — DGX Spark ComfyUI
  generation can be invoked ad-hoc if a specific need arises,
  but the plan deliberately starts from existing 207 assets to
  keep `--quick` lane for 5 of 6 substages.
