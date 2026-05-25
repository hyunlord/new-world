# V7 Section 14+ Design — Phase 13 Anchor

**Status**: High-level anchor (Sec14-1-a). Detailed sub-stage decomposition
deferred to `.harness/plans/phase13.md` (planning-first dispatch,
Phase 6/7/8/9/10/11/12 precedent).

**Authored**: 2026-05-25. **Author**: governance batch (post-V7 + Phase
12 sprint closure + D Phase A 8-error fix + Pipeline GDScript strict
check infrastructure landing).

**Precedent**: `.harness/audit/section_13_plus_design.md` (Phase 12
anchor — Tier 1 Sprites + Camera Zoom + Agent Sprite Scale Sprint,
commit `6843176f`, 2026-05-24). Structure mirrors that document.
Section 14 supersedes Section 13 only for the Phase 13 anchor;
Section 13's Phase 12 anchor remains authoritative for what the
α + β.1 + β.2 + γ sprint actually delivered.

---

## 0. Provenance & 9th Escalation Disclosure

Ninth successive escalation against the absence of a V7 Master
Direction document. The condition is unchanged across Sections
8/9/10/11/12/13/14: no `/mnt/project/` artefact, no
`*master*direction*` file in-tree, only `AGENTS.md` + `CLAUDE.md` +
`README.md` at repo root. The conceptual anchor referenced in
`rust/crates/sim-core/src/lib.rs:3` is not a written artefact.

Path chosen (Sec14-1-a): `.harness/audit/section_14_plus_design.md`
(this file, new). Filename mirrors Section 13's
`section_13_plus_design.md`.

Governance closure chain — extended (post-D-Phase-B, 2026-05-25):

| Stage | Commit | Scope |
|-------|--------|-------|
| 1  | `a0666b6c` | V7 Foundation Week 1-12 complete |
| 2  | `2e51c167` | CLAUDE.md V7 reset update |
| 3  | `0ed3ec16` | Section 8+ design — Phase 7 anchor |
| 4-9 | (Phase 7-α/β/γ chain) | Social System complete |
| 10 | `f0a60968` | Section 10+ design — Phase 9 anchor |
| 11-14 | (Phase 9-α/β/γ chain) | Combat System complete |
| 15-18 | All δ + governance | Phase 7/8/9 UI integration |
| 19-22 | Issues 12/13/14/15/16 | Pipeline governance patches |
| 23 | `6089976c` | Section 11+ design — Phase 10 anchor |
| 24-30 | Phase 10-α/β/γ + V7 closure | Multi-building Settlement |
| 31 | `919d510f` | Section 12+ design — Phase 11 anchor |
| 32 | `d7c34d78` | Phase 11-α implementation |
| 33 | `450f39cd` | D1 STATE_TINTS retune |
| 34 | `6843176f` | Section 13+ design — Phase 12 anchor |
| 35 | `7f6a6d76` | Phase 12-α (Camera zoom) |
| 36 | `7c203764` | Phase 12-β.1 (TileMapLayer terrain) |
| 37 | `a9402d46` | Phase 12-β.2 A3 (ConstructionSite render) |
| 38 | `1bfdbcbd` | Phase 12-γ (Settlement furniture) |
| 39 | `0238aef2` | D Phase A (8 GDScript runtime errors fix) |
| 40 | `a437c547` | D Phase B (Pipeline GDScript strict check infrastructure) |
| 41 | *(this commit)* | Section 14+ design — Phase 13 anchor (Game-like UI Sprint) |

User-screen evidence carried forward into Phase 13:
- Post-Phase-12 capture (2026-05-25, `1779685795433_image.png`):
  tiled floor surface visible, but agents render too small to
  distinguish, building presence ambiguous, no resource nodes, no
  HUD. User assessment: "초딩도 이해 못 함" — the rendering does not
  read as a simulation to an uninformed observer.
- D Phase A (`0238aef2`) cleared 8 runtime errors the user found in
  Godot Editor; D Phase B (`a437c547`) added a build-time strict
  check so future GDScript parse errors / warnings / FFI binding
  mismatches block the pipeline like a Codex RE-CODE would.

User mandate (2026-05-25, drives this section):
1. **Game-like UI first, backend later.** Visible legibility takes
   precedence over additional simulation depth.
2. **No single-substage dispatches.** Section 14 anchors a complete
   sprint chain; dispatch only after a coherent visible milestone is
   reached.
3. **No mid-sprint user confirmation requests.** Confirmation comes
   at full Phase 13 sprint closure, not per substage.

Prior single-substage commit pattern (Phase 11-α + D1 + Phase 12-α +
β.1 + β.2 + γ + D Phase A + D Phase B = 8 individual landings) is
explicitly retired for Phase 13.

---

## 1. Context

V7 backend remains intact: 12 ECS components + 10 runtime systems +
14 CausalEvent variants + 8 DecisionReason variants + 4
MemoryRecallTrigger variants. The causal chain composes end-to-end.

Phase 12 sprint landed:
- α (`7f6a6d76`): `Camera2D.zoom = (2,2)` default + mouse-wheel +
  smooth tween (0.5×–4.0× range, 1.25× geometric step, 0.15 s ease-out)
- β.1 (`7c203764`): TileMapLayer floor terrain (3 materials × 3
  variants procedurally seeded) + bootstrap cairn building + overlay
  z=10 alpha 0.65
- β.2 A3 (`a9402d46`): ConstructionSite per-entity Sprite2D at z=5
  with `alpha = 0.3 + 0.7 × (progress / required_progress)`
- γ (`1bfdbcbd`): Settlement centroid hearth placeholder at z=4 when
  ≥3 proximate agents meet the formation threshold

D infrastructure landed:
- D Phase A (`0238aef2`): cleared INTEGER_DIVISION × 4 +
  UNUSED_PARAMETER × 3 + FATAL `get_construction_snapshot` (stale
  dylib root cause)
- D Phase B (`a437c547`): `tools/harness/gdscript_strict_check.sh`
  + pipeline Step 2.4 catches parse / warning / FFI mismatch at
  build time

User-screen reality after Phase 12 closure (substrate grep, this
dispatch's Step 0):

- `scripts/ui/agent_renderer.gd:25` declares `SPRITE_SCALE := 0.25`
  → agent renders at `64×72 × 0.25 = 16×18 px`. At Phase 12-α's
  `Camera2D.zoom = (2,2)` that's 32×36 px on a 1920×1080 viewport.
- Default zoom 2× is enough for the VLM to see *something* but too
  low for an uninformed human to recognise individual agents.
- No `ResourceNode` ECS component exists (`grep` of sim-core/src/:
  zero matches). Resource nodes have never been a simulation concept.
- No HUD layer exists. The only UI element is CausalPanel under
  `UI (CanvasLayer)`, and it is hidden by default.
- `scripts/ui/world_renderer.gd:140` places a hardcoded bootstrap
  building at tile `(32, 32)` only. There is no procedural agent /
  building / resource spawning beyond what Phase 6 / Phase 10 emit
  organically over many ticks.

The gap between V7's simulation depth and what a human observer sees
in the first 10 seconds of running the game is the **legibility
deficit** Phase 13 closes.

---

## 2. Phase 13 Anchor — Game-like UI Sprint (V7 Week 26-27)

### 2.1 Choice rationale

User-screen evidence above proves the prior 8 single-substage
landings, while each individually pipeline-passing, did not
compound into a coherent visible milestone. User mandate explicitly
calls for a sprint chain producing a *single user-perceptible step
function* in legibility. Phase 13 is that sprint.

Concrete observable deficits Phase 13 addresses:

1. **Agent sprites too small** — uninformed observer cannot tell
   sprites are humans at all.
2. **Building presence ambiguous** — bootstrap cairn renders as a
   blob; ConstructionSite + Settlement hearth use the same cairn
   texture so they are indistinguishable.
3. **No resource nodes** — substrate gap means the world looks
   uniform; no food / water / wood for agents to interact with.
4. **No visible activity** — agents move (Phase 11-α interpolation)
   but interactions (gathering / talking / building) have no
   distinct visual signature beyond STATE_TINTS palette which is
   sub-resolution.
5. **No HUD** — no time-of-day, no population, no resource counters,
   no era marker. The user has no information layer.
6. **Cold simulation start** — the bootstrap places one building
   then waits for organic ConstructionSite / Settlement formation
   which takes many ticks. First-frame view is sparse.

User axioms touched: **#1 causal traceability**
("UI-visualizable") ← finally fully satisfied; **#3 emotional depth
+ visual milestone** ← primary target.

### 2.2 High-level scope (in / out)

**IN scope (Phase 13)**:
- Agent sprite size + distinguishability (sprite scale and/or
  per-agent texture variation)
- Building visualisation: distinct visible appearance for
  bootstrap building vs ConstructionSite vs completed Settlement
  furniture, sized large enough to read
- Resource nodes (placeholder sprites + minimal GDScript-side
  placement, no new ECS component required for legibility — full
  substrate deferred per the same A3 narrow-honest pattern that
  drove β.2)
- Interaction visualisation (movement already exists via Phase
  11-α; gathering / socialising / building visual cues)
- Basic HUD (time-of-day, agent population, building count, with
  reasonable typography)
- Bootstrap seed (multiple agents + buildings + an immediate
  Settlement so the first observable frames look populated)

**OUT of scope (Phase 14+ defer)**:
- Backend simulation expansion (`BuildingType` enum, `Furniture`
  ECS component, `Wall` ECS component, `ResourceNode` ECS component
  — substrate-addition phases that follow Phase 13's UI baseline)
- Inspector substantial sprint (5-tab agent inspector, full V7
  substrate visualisation)
- Sidebar / Chronicle (settlement history UI)
- Overlay legend / minimap
- Day/night cycle visuals
- Roads, culture zones, military, combat effects
- V4 Phase 3 backend (Health 85부위 + Knowledge + Family)
- Advanced AI extensions
- Stale-dylib guard infrastructure (related to D Phase B but separate)

### 2.3 Sub-stage shape (planning-first dispatch decomposes)

- **Phase 13-α** — Sprite size + agent/building visual distinction.
  Either bump `SPRITE_SCALE` and update Phase 4-γ harness, or
  bump camera default zoom further, or both. Distinct building
  textures for bootstrap vs ConstructionSite vs Settlement.
- **Phase 13-β** — Resource nodes (GDScript-placeholder Sprite2D
  layer at z=3, seeded from world seed) + visible "agent stands
  on resource" cue (no substrate change).
- **Phase 13-γ** — Interaction visualisation: gathering glyph when
  agent is `AgentState::Consuming { target: Food/Water/Sleep }`,
  socialising glyph for `Consuming { Agent }`, building progress
  ring for `Consuming { ConstructionSite }`.
- **Phase 13-δ** — HUD: top-bar or corner panel with time, agent
  count, settlement count, basic resource totals.
- **Phase 13-ε** — Bootstrap seed: spawn 30+ agents distributed
  across the world, place several buildings, prime resources, so
  the first capture frame looks populated and the user-screen
  evidence reads as "a simulation".

Each substage is `--quick` lane (GDScript + new harness files), no
sim-core touch expected. Total estimate: 21-31h across 5 dispatches.

### 2.4 Dependencies

- V7 backend (closed, stable)
- Phase 12-α Camera zoom controls (`7f6a6d76`)
- Phase 12-β.1 TileMapLayer terrain (`7c203764`)
- Phase 12-β.2 A3 ConstructionSite render (`a9402d46`)
- Phase 12-γ Settlement hearth (`1bfdbcbd`)
- D Phase B GDScript strict check (`a437c547`) — Step 2.4 will catch
  any parse/warning/FFI issue introduced by Phase 13 work
- 207 Tier 1 sprite assets (already on disk)
- Godot 4.6, gdext, hecs all stable

### 2.5 Open questions (planning-first dispatch resolves)

The 10 P13Plan-* decisions are in `.harness/plans/phase13.md`:

1. Sprite size increase: `SPRITE_SCALE` bump vs camera zoom default
   bump vs both
2. Agent visual distinguishability mechanism (palette variation,
   per-role overlays, none beyond size)
3. Building distinction mechanism (multiple sprites already on
   disk vs single placeholder per category)
4. Resource node placement (GDScript-side worldgen vs Rust
   ECS-side)
5. Interaction cue rendering (per-agent scale boost vs glyph
   overlay vs text label)
6. HUD position + content + typography
7. Bootstrap seed values (agent count, building positions, initial
   resource distribution)
8. Camera default + initial framing
9. Visual consistency / invariant preservation (if SPRITE_SCALE
   bumped, Phase 4-γ harness rewrite path)
10. α/β/γ/δ/ε exact decomposition + file lists + estimates

---

## 3. Section 15+ Candidates (Deferred)

- **Backend substrate addition phase**: `BuildingType` enum +
  `Furniture` ECS component + `Wall` ECS component +
  `ResourceNode` ECS component. Required to convert Phase 13's
  GDScript-side placeholders into substrate-driven rendering.
- **Inspector substantial sprint**: 5-tab agent inspector with V7
  substrate visualisation (Hunger / Thirst / Sleep / Social,
  AgentState / TargetKind, recent causal events).
- **Sidebar / Chronicle**: settlement chronicle viewer, social
  relationship view, memory recall stream.
- **Overlay legend + minimap**.
- **V4 Phase 3 backend**: Health 85부위 + Knowledge + Family.
- **Stale dylib guard**: detect when `libsim_bridge.dylib` is older
  than its Rust sources at the pipeline level (D Phase B
  documented but did not implement this).
- **Day/night, roads, culture zones, military, combat effects**.
- **Advanced AI**: BT / GOAP / utility curve refinement.

Re-ranking happens after Phase 13 closure.

---

## 4. Single Source of Truth

| Artefact | Authority |
|----------|-----------|
| Phase progress tracking | `.harness/audit/v7_progress.md` |
| Phase 7-12 anchors | `section_8_plus_design.md` … `section_13_plus_design.md` |
| **Phase 13 anchor** | **`section_14_plus_design.md`** (this file) |
| Phase 13 sub-stage decomposition | `.harness/plans/phase13.md` (planning-first dispatch follows this commit) |

---

## 5. Honest Reservations

- V7 Master Direction document remains absent (9th escalation).
- Eight consecutive single-substage commits (Phase 11-α onward)
  individually passed the pipeline but did not compound into a
  recognisable simulation. Phase 13 explicitly retires that pattern
  in favour of a 5-substage sprint chain.
- VLM whole-scene grader cannot discriminate 16-36 px sprite-level
  changes at 1920×1080 (recorded in CLAUDE.md by D Phase B). Phase
  13-α's first decision is whether to make sprites large enough to
  reach perceptual threshold; if yes, the VLM grader becomes
  meaningfully useful again.
- D Phase B's GDScript strict check is a build-time check. Runtime
  FFI binding visible to Godot (the original Phase 12-β.2 FATAL
  cause: stale dylib) is still not automatically detected. If Phase
  13 work induces another dylib-staleness crash the user will catch
  it first, same as before.
- `SPRITE_SCALE = 0.25` is a Phase 4-γ harness invariant. If Phase
  13-α decides to bump it, the `harness_p4_gamma_*` assertions plus
  the four anti-regression guards (`harness_p11_alpha_a*`,
  `harness_p12_alpha_a15`, `harness_p12_beta_a21`,
  `harness_p12_beta2_a12`, `harness_p12_gamma_a11`) must all be
  updated in the same commit. The alternative is to keep
  SPRITE_SCALE locked and rely entirely on camera zoom.
- No `ResourceNode` substrate. Phase 13-β uses GDScript-side
  placeholders; the resulting nodes are visual only and do not
  participate in agent decision-making. That is the substrate gap
  Phase 14+ Section 15+ will need to close before agents can be
  filmed actually gathering from those resources. Phase 13 itself
  is "looks like a simulation"; full substrate-driven gathering is
  a later phase.
- User confirmation comes only after full Phase 13 closure
  (α + β + γ + δ + ε). Per-substage commits will land per
  governance protocol but the integrated visible milestone is
  the user-mandated checkpoint.
- Estimated total: 21-31h across 5 dispatches + 5 pipelines ≈
  3-4 active days plus pipeline elapsed time.
