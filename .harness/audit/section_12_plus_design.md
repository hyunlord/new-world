# V7 Section 12+ Design — Phase 11 Anchor

**Status**: High-level anchor (Sec12-1-a). Detailed sub-stage decomposition deferred to
`.harness/plans/phase11.md` (planning-first dispatch, Phase 6/7/8/9/10 precedent).

**Authored**: 2026-05-21. **Author**: governance batch (post-V7 + Phase 10 + All δ + Issue 16 closure).

**Precedent**: `.harness/audit/section_11_plus_design.md` (Phase 10 anchor —
Multi-building Settlement System, commit `6089976c`, 2026-05-21). Structure and tone mirror
that document. Section 12 supersedes Section 11 only for the Phase 11 anchor; Section 11's
Phase 10 anchor remains the authoritative record for what Phase 10 actually delivered
(`40c36d13` settlement chronicle harness).

---

## 0. Provenance & 7th Escalation Disclosure

This is the seventh successive escalation against the absence of a V7 Master Direction
document. Phase 6 planning, Phase 7 planning, Section 8+ design, Section 9+ design,
Section 10+ design, Section 11+ design, and now Section 12+ design have all triggered
the same condition:

- `/mnt/project/` (claude.ai project_files): absent
- `find . -iname "*master*direction*"` in repo: zero matches
- Root `.md` files: `AGENTS.md`, `CLAUDE.md`, `README.md` only
- The "V7 Master Direction" referenced in `rust/crates/sim-core/src/lib.rs:3`
  remains a **conceptual anchor**, not a written artefact.

Path chosen (Sec12-1-a): **`.harness/audit/section_12_plus_design.md`** (this
file, new). Filename mirrors Section 11's `section_11_plus_design.md`. Section 11
explicitly demarcated Section 12 as a *separate future document* (§3, "Section 12
decomposition is NOT specified here"). The modular path honours that demarcation.

Governance closure chain — extended (post-Section-12-anchor, 2026-05-21):

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
| 24 | `bd36219b` | V7 + All δ Final Declaration (Stage 24) |
| 25 | `182476f9` | Issue 16 patch (Stage 25) |
| 26 | `6089976c` | Section 11+ anchor design (Stage 26) |
| 27 | `313cfcd3` | Phase 10-α (Settlement substrate + AgentBorn) |
| 28 | `930bcbcf` | Phase 10-β (SettlementSystem + 8th cascade + Birth) |
| 29 | `40c36d13` | Phase 10-γ (Settlement chronicle harness, V7 Phase 10 complete ★★★★) |
| 30 | `c381eff5` | V7 + Phase 10 Final Declaration (Stage 30) |
| 31 | *(this commit)* | Section 12+ design — Phase 11 anchor (Map Rendering Foundation) |

V7-specific disclosures carried forward:
- 3 ENV-BYPASS chains closed; 4th–6th avoided via Option D Issue 15 fix path
- Issues 12+13+14+15+16 closure chain complete (`00274b57` + `ebbf6ddc` + `ddd5348c` + `182476f9`)
- Issue 16 post-fix verified: 3 consecutive live tests passed (Phase 10-α + β + γ)
- 2× 100/100 PERFECT first-dispatch: Phase 8-α + Phase 10-α 일관 substantial
- **User-mandated direction confirmed: Map Rendering Foundation Sprint** (Sec12-2-a)

---

## 1. Context

V7 (Foundation + Phase 7 + Phase 8 + Phase 9 + Phase 10 + All δ + Issue 16) closure
establishes the following substrate:

**Architecture base** (final — Stage 30, `c381eff5`):
- **12 components**: Position, Agent, Hunger, Thirst, Sleep, AgentState, Construction,
  Social, Relationship, Memory, BodyHealth, Settlement
- **10 runtime systems** (priorities 120-138): Influence (120), Agent (121), Needs (130-136
  parallel), Decision (125), Construction (133), Social (134), Memory (135), Combat (137),
  Settlement (138)
- **14 CausalEvent variants**: AgentSpawned, AgentDied, NeedCritical, ItemConsumed,
  AgentDecision, ConstructionStarted, ConstructionCompleted, BuildingPlaced,
  SocialInteractionStarted, SocialInteractionCompleted, MemoryEncoded, MemoryRecalled,
  CombatStarted, CombatCompleted, AgentBorn, SettlementFormed, SettlementDissolved
  *(note: 14 variants post-Phase 10)*
- **8 DecisionReason variants**: HungerReason, ThirstReason, SleepReason,
  ConstructionReason, SocialReason, MemoryReason, CombatReason, SettlementReason
- **4 MemoryRecallTrigger variants**: HungerContext, SocialContext, ConstructionContext,
  CombatContext
- **Causal chain** (8 reasons → 8 emission chains): all internal reasoning + social
  interaction + memory bias + combat + settlement lifecycle causal-traceable end-to-end
- **All δ UI integration**: SimBridge FFI + CausalPanel (8 DecisionReason variants) +
  AgentRenderer (socializing/recalling/in_combat indicators) + Locale en/ko
- **Chronicle infrastructure**: Phase 5-γ + 6-γ + 7-γ + 8-γ + 9-γ + 10-γ precedent complete
- **Substrate symmetries**: `MEMORY_CAP=32` mirror + `HOSTILITY_BUMP=0.1` mirror +
  `SETTLEMENT_HISTORY_CAP=32` mirror + Phase 7↔9 axis + Phase 8↔9 axis + Phase 8↔10 axis

**Section 8+/9+/10+/11+ §3 candidates ranked (post-Settlement closure)**:
- Combat System ✅ CLOSED (Phase 9)
- Memory System ✅ CLOSED (Phase 8)
- Multi-building Settlement ✅ CLOSED (Phase 10)
- **Map Rendering Foundation** ← Phase 11 (this anchor, user mandate Sec12-2-a) ★ NEW
- Advanced AI (BT/Utility/LLM) ← Phase 12+ (defer)

**★ Critical — UI state honest assessment**:
- Current rendering: `scripts/ui/world_renderer.gd` + `scripts/ui/agent_renderer.gd`
  (direct in `scripts/ui/`, no `renderers/` subdirectory)
- All δ (Phase 7-δ + 8-δ + 9-δ) delivered code-side FFI + CausalPanel + indicator logic
- Visual gap: indicator logic is code-complete but visual density "거의 nothing" vs
  RimWorld/DF target — agent movement is static dots, no state sprite mapping, no resource
  nodes, no build progress bars
- ★ Phase 11 = first UI-focused phase; all prior phases were backend simulation focus
- Axiom #3 emotional depth: settlement growth+decay chronicle verified; *visible* god-game
  density milestone requires Phase 11 Map Rendering Foundation

---

## 2. Phase 11 Anchor — Map Rendering Foundation Sprint (V7 Week 21-23)

### 2.1 Choice rationale (substrate fit + product vision)

**Solvable visual gap — user mandate**:
User mandate: "current UI not game-like" + "Visual target: RimWorld + DF density/feel."
V7 backend substrate is substantial final final final final substantial. The gap is
*rendering*, not simulation. Phase 11 addresses this directly without backend rewrite risk.

**V4 roadmap Phase 1~2 alignment**:
V4 roadmap Phase 1 (~4 weeks) targets: agent movement animation, agent action display,
agent carry display, resource node rendering, day/night visual, terrain enhancement.
Phase 2 targets: building construction progress bars, building effect indicators.
18 of ~28 items remain unimplemented. Phase 11 targets Phase 1 group items first.

**Substrate compositional path** (no substrate rewrite):
- `Position` component already in ECS → agent position snapshot already available via
  SimBridge `collect_agent_snapshot` → smooth interpolation is a GDScript rendering layer
- `AgentState` + `state_tag` (Phase 7-δ) already serialized via SimBridge FFI →
  visual indicator mapping is a GDScript rendering extension over existing snapshot data
- `ConstructionProgress` substrate (Phase 6) → build progress bar rendering is a UI read
  over existing component data
- All δ SimBridge FFI substrate (state_tag + CausalEvent serialization) → direct reuse

**Axiom #3 god-game emotional depth path**:
Agents visibly moving, acting, carrying, and building — not static dots — is the
visual prerequisite for the god-game emotional depth milestone (axiom #3). Songs of Syx,
Caveman2Cosmos, Dwarf Fortress, and RimWorld all establish this foundation before
adding civilisation layers. Phase 11 closes the foundational visual gap.

**Axiom #1 traceability extension** (visible layer):
The causal chain is backend-complete (8 reasons → 8 emission chains). Phase 11 makes
it *visible* — agent state visual indicators confirm what CausalPanel reports. Movement
toward a resource node is visually traceable to a HungerReason decision. Building
construction progress is visually traceable to a ConstructionReason chain.

**Low infrastructure pattern risk**:
Phase 11 is GDScript rendering work (hot-tier pipeline, VLM visual verification path).
No new Rust systems, no new ECS components required for Phase 1 group items. Infrastructure
pattern risk (Issues 12-16) is minimal for GDScript-only work — classify_recode.sh
improvements (Issue 15) and generate_report.sh fix (Issue 16) hold for GDScript pipeline.

**Medium to long production span**:
UI sprint scope is larger than a single backend substrate phase. Sub-stage decomposition
(Phase 11-α/β/γ) should bound each sub-stage to 1-3 days (harness planning-first
dispatch precedent). Planning-first dispatch required before implementation begins.

### 2.2 High-level scope (in / out)

**IN scope (Phase 11)**:
- **에이전트 이동 애니메이션** (Z1, position interpolation between ticks):
  - SimBridge `collect_agent_snapshot` position → `agent_renderer.gd` smooth interpolation
  - Static dot → visible movement path (RimWorld pawn movement baseline)
- **에이전트 작업 표시** (Z1, AgentState state_tag visual indicator):
  - Per-state sprite/color/icon: Idle / Seeking / Consuming / Sleeping / Constructing /
    Socializing / Recalling / InCombat
  - Phase 7-δ + 8-δ + 9-δ `state_tag` FFI substrate direct reuse
- **에이전트 운반 표시** (Z1, carry icon overlay):
  - Small icon above agent when carrying item
  - Carry state detection via AgentState or new carry_tag field (open question)
- **자원 노드 표시** (berry bush, tree, stone deposit):
  - Resource entity component + sprite rendering in `world_renderer.gd`
  - Harvestable nodes visible on map (prerequisite for agent pathing visualization)
- **Optional Phase 1+2 additions** (Sec12-5 — include if sub-stage capacity allows):
  - 건물 건설 진행 표시 (construction progress bar, `ConstructionProgress` component)
  - 낮/밤 시각 표현 (background color cycle, tick-based day length)

**OUT of scope (Phase 12+ defer)**:
- Map rendering Phase 3+ (roads/"desire paths", building 8-category sprites, settlement
  organic growth, resource flow particles)
- Map rendering Phase 4+ (culture zones, armies, combat effects, caravans, walls, weather)
- HUD substantial sprint (time/resource/era/crisis/trend display)
- Inspector substantial sprint (V7 substrate visualization beyond CausalPanel)
- Sidebar/Chronicle substantial sprint
- Influence overlay substantial sprint
- Backend simulation expansion (V4 Phase 3: Health 85-organ + Knowledge + Family)
- Phase 10-δ Settlement UI (optional, deferred from Phase 10)
- Advanced AI decision architecture rewrite

### 2.3 Sub-stage shape (TBD — planning-first dispatch required)

Estimated shape (subject to planning-first revision):
- **Phase 11-α**: 에이전트 이동 애니메이션 + AgentState state_tag visual indicator mapping
  (~3-5 implementation hours; GDScript hot-tier; `agent_renderer.gd` extension)
- **Phase 11-β**: 에이전트 운반 표시 + 자원 노드 component + rendering
  (~3-5 implementation hours; possible minor SimBridge FFI extension for carry state)
- **Phase 11-γ**: Optional Phase 1+2 additions (건물 진행 + 낮/밤) OR chronicle/integration
  harness proving end-to-end visual causal chain
- **Phase 11-δ** (optional): Visual polish + animation tuning + settlement boundary rendering

Planning-first dispatch (`.harness/plans/phase11.md`) must resolve 10 open questions
(§2.5) before sub-stage decomposition is locked.

### 2.4 Dependencies

- V7 (Foundation + Phase 7 + Phase 8 + Phase 9 + Phase 10 + All δ + Issue 16) complete
  (`c381eff5` Stage 30) ✅
- `Position` component + SimBridge `collect_agent_snapshot` snapshot path ✅
- `AgentState` + `state_tag` FFI serialization (Phase 7-δ) ✅
- `ConstructionProgress` component (Phase 6) ✅ (for optional progress bar)
- `scripts/ui/agent_renderer.gd` + `scripts/ui/world_renderer.gd` existing extension base ✅
- Godot 4.6 renderer (MultiMeshInstance2D or Sprite2D path TBD per planning dispatch)

### 2.5 Open questions (planning-first dispatch resolve)

1. **이동 애니메이션 mechanism**: linear position lerp vs Godot tween easing vs
   tick-synchronised step (depends on snapshot frequency + 60 FPS desync handling)
2. **AgentState visual mapping**: sprite swap vs icon overlay vs color tint vs composite
   (sprite + tint); asset availability constraints
3. **운반 carry state detection**: existing `AgentState` variant vs new `carry_tag` FFI
   field vs item-in-hand component approach
4. **자원 노드 component design**: new Rust `Resource` ECS entity + SimBridge snapshot vs
   tile-property approach vs GDScript-side placement only
5. **자원 노드 spawn mechanism**: procedural generation (seed-based) vs manual data placement
   vs RON-driven resource map
6. **자원 노드 sprite assets**: existing asset library coverage (berry/tree/stone);
   asset creation scope if absent
7. **SimBridge FFI Position snapshot**: verify current `collect_agent_snapshot` includes
   `x, y` position fields; if not, FFI extension required (likely minor)
8. **Phase 1+2 optional group inclusion** (Sec12-5): 건물 진행 + 낮/밤 — include in Phase
   11 or defer to Phase 12? Scoping decision for planning-first dispatch
9. **Tick interpolation rate**: simulation runs at 20-30 TPS; rendering at 60 FPS;
   interpolation factor = elapsed_time / tick_duration; Gaffer accumulator already
   in architecture (CLAUDE.md §Day-1 decision 9)
10. **agent sprite assets**: existing `agent_base.png` or equivalent; size/animation
    frame requirements for state-based visual mapping

---

## 3. Section 13+ Candidates (deferred)

`Section 13` will anchor the Phase 12 direction. Candidates (post-Map Rendering
Foundation base):

1. **Advanced AI (BT / Utility / LLM)** — Section 8+ §3 #4. The remaining original
   candidate. After Phase 11 delivers visible rendering, the AI rewrite can consume
   *both* Memory and Settlement priors as bias sources AND be visually verifiable
   (agent behavior changes are observable on the rendered map, not just in CausalPanel).
   Full substrate: Social + Memory + Combat + Settlement + visible rendering = the
   richest possible base for an AI architecture rewrite.
2. **HUD substantial sprint** — time/resource/era/crisis/trend display; prerequisite
   for god-game readability layer
3. **Inspector substantial sprint** — V7 substrate visualization (agent/settlement/
   building detail beyond current CausalPanel scaffold)
4. **Sidebar/Chronicle substantial sprint** — Phase chronicle, settlement history
   timeline, cultural identity layer
5. **Backend simulation expansion** — V4 Phase 3: Health (85-organ body model),
   Knowledge (skill/memory transfer), Family (lineage/inheritance chains)
6. **Other** (user-mandated direction after Phase 11 closure)

Section 13 decomposition is **NOT** specified here. It is the user-mandated direction
chosen after Phase 11 closure, per the established governance pattern.

---

## 4. Single Source of Truth

| Concern | Document |
|---------|----------|
| V7 phase progress tracking + closure declarations | `.harness/audit/v7_progress.md` |
| Phase 7 anchor + scope | `.harness/audit/section_8_plus_design.md` |
| Phase 8 anchor + scope | `.harness/audit/section_9_plus_design.md` |
| Phase 9 anchor + scope | `.harness/audit/section_10_plus_design.md` |
| Phase 10 anchor + scope | `.harness/audit/section_11_plus_design.md` |
| Phase 11 anchor + scope (this layer) | `.harness/audit/section_12_plus_design.md` (this file) |
| Phase 11 sub-stage decomposition | `.harness/plans/phase11.md` (planning-first dispatch, TBD) |
| Phase 6 sub-stage decomposition (existing) | `.harness/plans/phase6.md` |
| Behavioural + architectural guidelines | `CLAUDE.md` |
| V7 roadmap canonical comment block | `rust/crates/sim-core/src/lib.rs:3-11` |

---

## 5. Honest Reservations

1. **Selection is a product judgement, not a derivation.** Map Rendering Foundation is
   the user-mandated direction after Phase 10 closure. It is defensible: the backend
   substrate is substantial and the visual gap is real. It is **not** the only defensible
   choice — Advanced AI is an equally valid next direction if the product vision
   prioritises decision architecture over visual density.

2. **★ First UI-focused phase.** Phases 5-10 were backend simulation work. Phase 11 is
   GDScript rendering work. The harness pipeline infrastructure (Drafter/Challenger/
   Generator/Evaluator/VLM) was built and validated for Rust sim-core/sim-systems/
   sim-engine work. GDScript rendering work routes through `--quick` pipeline tier
   (no planning debate, Visual Verify + Evaluator only). VLM visual verification is the
   primary quality gate — the first time VLM output is not an environmental cost but
   an actual quality signal. Expect higher VLM scrutiny for Phase 11 dispatches.

3. **Asset availability risk.** Phase 11 requires sprite assets for agent states,
   resource nodes, and possibly carry indicators. If assets are absent or insufficient,
   planning-first dispatch must scope to placeholder rendering (colored rectangles /
   circles) for Phase 11-α/β with asset polish deferred to Phase 11-δ or Phase 12.
   Never block simulation logic correctness on asset availability.

4. **GDScript rendering scope creep risk.** UI sprints historically expand: "add
   movement animation" becomes "also fix camera" becomes "also add minimap labels."
   Phase 11 scope is deliberately bounded to Phase 1 group items (§2.2 IN scope).
   Out-of-scope items (HUD, Inspector, Sidebar) are Phase 12+ defer — hard boundary.

5. **SimBridge FFI extension risk.** If `collect_agent_snapshot` does not currently
   include `x, y` position or carry state fields, Phase 11-α will require a minor
   FFI extension. FFI extensions touch `sim-bridge/src/ffi/world_node.rs` (hot-tier
   pipeline). Planning-first dispatch must verify current snapshot schema before
   sub-stage scope is locked (open question §2.5 #7).

6. **Infrastructure pattern monitoring mandatory.** Phase 11 is the first dispatch
   post-Issue 16 fix in a new phase scope. `pipeline_report.md` generation will be
   the 4th live test of the Issue 16 fix. Pattern G (Drafter revision degradation) is
   mitigated but monitoring continues. classify_recode.sh GDScript patterns may need
   tuning if false positives emerge for GDScript-specific idioms.

7. **7th escalation — governance document debt.** The audit trail now contains five
   `section_N_plus_design.md` files (8/9/10/11/12) plus `v7_progress.md` (1536 lines).
   This remains manageable. Consolidation into a proper master direction document is a
   governance question for the user to mandate, not a task to self-assign here.

8. **All δ visual gap honest disclosure.** Phase 7-δ + 8-δ + 9-δ delivered code-side
   indicator logic (state_tag serialization, `mark_agent_recalling`, `mark_agent_in_combat`).
   Visual density "거의 nothing" honest assessment: indicators are code-complete in GDScript
   but require the asset + rendering pipeline Phase 11 will establish. Phase 11-α/β are
   in part retroactive completion of the visual half of the All δ chain.
