# V7 Section 11+ Design — Phase 10 Anchor

**Status**: High-level anchor (Sec11-1-a). Detailed sub-stage decomposition deferred to
`.harness/plans/phase10.md` (planning-first dispatch, Phase 6/7/8/9 precedent).

**Authored**: 2026-05-21. **Author**: governance batch (post-V7 All δ + Issue 16 closure).

**Precedent**: `.harness/audit/section_10_plus_design.md` (Phase 9 anchor —
Combat System, commit `f0a60968`, 2026-05-21). Structure and tone mirror that document.
Section 11 supersedes Section 10 only for the Phase 10 anchor; Section 10's Phase 9
anchor remains the authoritative record for what Phase 9 actually delivered
(`86ec5fff` chronicle harness).

---

## 0. Provenance & 6th Escalation Disclosure

This is the sixth successive escalation against the absence of a V7 Master Direction
document. Phase 6 planning, Phase 7 planning, Section 8+ design, Section 9+ design,
Section 10+ design, and now Section 11+ design have all triggered the same condition:

- `/mnt/project/` (claude.ai project_files): absent
- `find . -iname "*master*direction*"` in repo: zero matches
- Root `.md` files: `AGENTS.md`, `CLAUDE.md`, `README.md` only
- The "V7 Master Direction" referenced in `rust/crates/sim-core/src/lib.rs:3`
  remains a **conceptual anchor**, not a written artefact.

Path chosen (Sec11-1-a): **`.harness/audit/section_11_plus_design.md`** (this
file, new). Filename mirrors Section 10's `section_10_plus_design.md`. Section 10
explicitly demarcated Section 11 as a *separate future document* (§3, "Section 11
decomposition is NOT specified here"). The modular path honours that demarcation.

Governance closure chain — extended (post-Section-11-anchor, 2026-05-21):

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
| 23 | *(this commit)* | Section 11+ design — Phase 10 anchor (Multi-building Settlement) |

V7-specific disclosures carried forward:
- 5 ENV-BYPASS chains avoided (3 closed + 4th + 5th avoided via Option D Issue 15 fix path)
- Issues 12+13+14+15+16 closure chain: `00274b57` + `ebbf6ddc` + `ddd5348c` + `182476f9`
- Section 10+ §3 candidates verbatim: *"Section 11 decomposition is NOT specified here.
  It is the user-mandated direction chosen after Phase 9 closure, per the established
  governance pattern."*
- **User-mandated direction confirmed: Multi-building Settlement System** (Sec11-2-a)

---

## 1. Context

V7 All δ + Issue 16 closure establishes the following substrate:

**Architecture base** (final, as of `182476f9`):
- **11 components**: Position, Agent, Hunger, Thirst, Sleep, AgentState, Construction,
  Social, Relationship, Memory, BodyHealth
- **9 runtime systems** (priorities 120-137): Influence (120), Agent (121), Needs (130-136
  parallel), Decision (125), Construction (133), Social (134), Memory (135), Combat (137)
- **11 CausalEvent variants**: AgentSpawned, AgentDied, NeedCritical, ItemConsumed,
  AgentDecision, ConstructionStarted, ConstructionCompleted, BuildingPlaced,
  SocialInteractionStarted, SocialInteractionCompleted, MemoryEncoded, MemoryRecalled,
  CombatStarted, CombatCompleted *(note: count reflects final Phase 9 additions)*
- **7 DecisionReason variants**: HungerReason, ThirstReason, SleepReason,
  ConstructionReason, SocialReason, MemoryReason, CombatReason
- **4 MemoryRecallTrigger variants**: HungerContext, SocialContext, ConstructionContext,
  CombatContext
- **Causal chain** (full end-to-end): `SocialInteractionCompleted{negative}` →
  `MemoryEncoded` → `MemoryRecalled{CombatContext}` → `AgentDecision{CombatReason}` →
  `CombatStarted` → `CombatCompleted` (anti-recursion safe)
- **All δ UI integration**: SimBridge FFI + CausalPanel (7 DecisionReason variants) +
  AgentRenderer (socializing/recalling/in_combat indicators) + Locale en/ko
- **Chronicle infrastructure**: Phase 5-γ + 6-γ + 7-γ + 8-γ + 9-γ precedent

**Section 8+/9+/10+ §3 candidates ranked (post-Combat closure)**:
- Combat System ✅ CLOSED (Phase 9)
- Memory System ✅ CLOSED (Phase 8)
- **Multi-building Settlement** ← Phase 10 (this anchor, user mandate Sec11-2-a)
- Advanced AI (BT/Utility/LLM) ← Phase 11+ (defer)

---

## 2. Phase 10 Anchor — Multi-building Settlement System

### 2.1 Choice rationale (substrate fit + product vision)

**Phase 6 Construction substrate direct extension**:
The existing Construction substrate (`BuildingPlaced`, `ConstructionStarted`,
`ConstructionCompleted` CausalEvent variants; `ConstructionSystem` priority 133;
`Construction` component; `Blueprint` data) is an extension-ready base. Adding a
Settlement aggregation layer over Buildings follows the Phase 5/6/7/8/9 incremental
additive pattern: one new component, one new system, new CausalEvent variants — not
a substrate rewrite.

**Memory + Combat compositional path**:
- *Memory of locations*: Phase 8 episodic memory substrate is directly reusable.
  Agents can encode `BuildingPlaced` events as location memories, biasing future
  `ConstructionReason` decisions toward familiar sites. "This hill is where my
  family built the first shelter" is a two-system composition (Construction + Memory)
  already latent in the substrate.
- *Community combat history*: Phase 9 `CombatStarted/Completed` events become
  settlement-level chronicle entries. Deaths at a site → historical events the
  settlement remembers. Grudges, founding myths, defensive decisions — all are
  Memory-over-Combat compositions that Phase 10 can expose.
- *Axiom #1 extension*: the causal chain `AgentDecision{SettlementReason}` →
  `SettlementFormed` → `[member agents/buildings]` would be the next link in the
  already-seven-reason chain, traceable end-to-end through CausalPanel.

**Population lifecycle + episodic inheritance**:
Phase 9 delivers entity despawn on `AgentDied`. Phase 10 extends this to
population-level statistics: births (new `AgentSpawned` events with parent context),
deaths (existing `AgentDied` with settlement context), and the emergent lifecycle of
a settlement as its population grows, peaks, and declines. Episodic inheritance —
memories passed from parent to child — is a natural Phase 10-β extension over the
Phase 8 Memory substrate.

**Macro god-game milestone**:
Phases 5-9 established the agent-level loop (needs → decisions → actions → chronicle).
Phase 10 establishes the *settlement-level* loop — the next zoom level. This is the
Songs of Syx / Caveman2Cosmos / Dwarf Fortress civilisation-emergence layer: clusters
of agents form persistent communities, those communities have histories, and the god
player observes and intervenes at the community level. The visual milestone for Phase
10-δ (optional) would be a labelled settlement boundary with population counter and
chronicle — a leap in the god-game density axis (axiom #3).

**Medium prerequisite cost**:
Settlement requires: a new `Settlement` component (or resource entry), a new
`SettlementSystem`, and new CausalEvent/DecisionReason variants. It does **not**
require a rewrite of the agent decision architecture (unlike Advanced AI), a new
physics layer, or a new data format. The substrate-fit cost is additive over
Phase 6 Construction and Phase 8/9 Memory/Combat — the same pattern as Phase 7,
8, and 9 before it.

**Infrastructure pattern risk**:
Phase 7-β + 8-β + 9-β all triggered recurring infrastructure patterns (Issues
12-16). Phase 10 = Settlement = substrate *extension* (not rewrite). Patterns A/B/C/D
are closed or mitigated. Pattern G is monitored. The additive path minimises new
surfaces where the patterns can re-fire.

**Product vision alignment**:
- *Caveman2Cosmos*: cultural progression through settlement-level tech and social
  organisation — Phase 10 is the first civilisation-scale milestone.
- *Songs of Syx*: settlement growth, population management, community identity —
  Phase 10 builds the same spatial and social aggregation layer.
- *Dwarf Fortress*: fortress-level emergent stories (the history tab, the community
  identity) arise from exactly this substrate: individual agents + buildings +
  population lifecycle + community memory.
- *WorldSim god-game vision*: the player as god observes and intervenes at *both*
  agent and settlement level. Phase 10 delivers the second observation level.

### 2.2 High-level scope

**IN scope (Phase 10)**:

- **Settlement component** (Settlement aggregation entity or SimResources entry):
  - `settlement_id` (unique identifier, type alias on `u32` or new newtype)
  - `member_agents: HashSet<AgentId>` (living member agents)
  - `member_buildings: HashSet<BuildingId>` (owned buildings)
  - `population_stats`: birth count, death count, current population
  - `founded_at: u64` (tick timestamp)
  - `community_history: Vec<EventId>` (ring-buffered significant events,
    mirror of agent `Memory.events` — cap TBD, ~32 events)
- **SettlementSystem** (priority TBD, candidate 138 = after CombatSystem 137):
  - Formation: detect agent clusters meeting threshold (proximity + shared buildings)
    → emit `CausalEvent::SettlementFormed`
  - Membership updates: agent join/leave on movement or death
  - Community history ingestion: CombatCompleted + BuildingPlaced + AgentDied events
    → append to `community_history` where relevant
  - Dissolution: population depletion below threshold → emit
    `CausalEvent::SettlementDissolved`
- **Population lifecycle**:
  - Birth: new `AgentSpawned` event with `parent_agent: Option<AgentId>` context
    (Phase 9 entity spawn substrate extension)
  - Death: existing `AgentDied` + settlement membership removal
  - Population-level statistics update on each birth/death
- **New CausalEvent variants**:
  - `SettlementFormed { settlement_id, founder_agents: Vec<AgentId>, tick: u64 }`
  - `SettlementDissolved { settlement_id, cause: DissolutionCause, final_pop: u32 }`
  - (12th + 13th variants in CausalEvent enum)
- **New DecisionReason variant**:
  - `SettlementReason` (8th variant) — agent decision driven by settlement
    membership, loyalty, or migration imperative
- **AgentDecisionSystem 8th cascade** (after CombatReason):
  - Settlement-aware decision: agent migrates toward / away from settlement
    based on population pressure or loyalty
- **Chronicle harness (Phase 10-γ)**:
  - Two or more agents on a grid, construction completed, proximity threshold met
  - Settlement forms, community history accumulates, population lifecycle events fire
  - Settlement dissolves (all members die or migrate)
  - ≥12 assertions proving the formation→lifecycle→dissolution cycle

**OUT of scope (Phase 11+ defer)**:

- Advanced AI (BT/Utility/LLM) — Section 8+ §3 #4 (remaining candidate,
  defer to Phase 11+ post-Settlement)
- Cross-settlement diplomacy / trade / war (substantial scope expansion)
- Religion / culture / language mechanics
- Tech tree / civilisation progression
- Wildlife / non-agent entities
- Phase 10-δ UI integration (SettlementPanel, community history display —
  optional, user-gated per All δ precedent)
- Sub-settlement district structure
- Named settlements / dynastic inheritance (Phase 11+ enrichment)

### 2.3 Sub-stage shape (TBD — planning-first dispatch)

Detailed decomposition lives in `.harness/plans/phase10.md` after a planning-first
dispatch (Phase 6/7/8/9 precedent). The expected shape:

- **Phase 10-α**: `Settlement` component/resource struct + `SettlementId` newtype +
  `CausalEvent::SettlementFormed/Dissolved` variants + `DecisionReason::SettlementReason`
  + serde + population stats stub + harness with ≥12 assertions. Zero runtime system
  change. Pure data substrate. (Phase 6-α / 8-α precedent.)
- **Phase 10-β**: `SettlementSystem` (priority 138 candidate) + formation/dissolution
  logic + membership tracking + community history ingestion (CombatCompleted +
  BuildingPlaced + AgentDied hooks) + population lifecycle (birth events) +
  `AgentDecisionSystem` 8th cascade + harness with ≥12 assertions. (Phase 6-β / 8-β
  precedent.)
- **Phase 10-γ**: End-to-end chronicle harness — formation, lifecycle, dissolution
  full cycle; ≥13 assertions; closure milestone. (Phase 6-γ / 7-γ / 8-γ / 9-γ
  precedent.)
- **Phase 10-δ** (optional, user mandate base): Visual milestone — Godot scene
  showing settlement boundary overlay, population counter HUD element, CausalPanel
  `SettlementReason` label, community history chronicle panel. Out of scope until
  V7 visual ambition is reassessed post-Phase-10-γ.

### 2.4 Dependencies

- V7 Foundation + Phase 7 + Phase 8 + Phase 9 + All δ + Issue 16 closure ✓
  (substrate complete: `182476f9` + `bd36219b`)
- **Phase 6 Construction substrate** (directly reused):
  - `ConstructionSystem` priority 133
  - `CausalEvent::BuildingPlaced` (community history ingestion source)
  - `Blueprint` + `BuildingId` data types
- **Phase 8 Memory substrate** (compositionally reused):
  - `Memory` component + `MEMORY_CAP` constant
  - `MemoryEncoded` / `MemoryRecalled` CausalEvent variants
  - `DecisionReason::MemoryReason` cascade logic
- **Phase 9 Combat substrate** (compositionally reused):
  - `BodyHealth` component + `AgentDied` event (population lifecycle trigger)
  - `CombatCompleted` CausalEvent (community history ingestion source)
  - Entity despawn pattern (Phase 9-β) → population death tracking
- **No prerequisite on Advanced AI / BT / LLM** — Phase 10 is additive over
  the existing agent decision cascade, extending it by one reason variant.

### 2.5 Open questions (TBD — planning-first dispatch resolve)

1. **Settlement representation**: entity-based aggregation (a new hecs `Entity` with
   `Settlement` component) vs hash-based grouping (`HashMap<SettlementId, SettlementData>`
   on `SimResources`) vs spatial tile grouping (influence grid zone). Default candidate:
   **SimResources HashMap** (mirrors `RelationshipState` sparse map precedent from
   Phase 7-α; avoids ECS archetype churn for a non-agent entity type).
2. **Formation trigger**: agent proximity + shared building density threshold vs
   explicit agent decision (`SettlementReason` cascade fires first). Default candidate:
   **automatic formation** (SettlementSystem detects cluster, emits `SettlementFormed`
   without requiring explicit agent intent — simpler, avoids chicken-and-egg with the
   8th cascade decision).
3. **Dissolution mechanism**: population depletion below threshold (e.g. ≤1 living
   member) vs time-without-construction-activity vs explicit migration event. Default
   candidate: **population ≤ 0** (deterministic, testable).
4. **SettlementSystem priority**: 138 candidate (after CombatSystem 137). Confirm
   no ordering dependency with CombatSystem or ConstructionSystem at planning time.
5. **Population lifecycle — Birth mechanism**: new `AgentSpawned` event with
   `parent_agent: Option<AgentId>` context vs a separate `BirthSystem`. Default
   candidate: **extend existing `AgentSpawned` with parent context** (additive,
   no new system slot required for Phase 10-α).
6. **Community history representation**: per-settlement `Vec<EventId>` (ring-capped,
   ~32 events, mirror of `Memory.events`) vs a separate `CommunityMemory` resource.
   Default candidate: **per-settlement ring buffer** (structural symmetry with
   agent `Memory` component — same `MEMORY_CAP` constant reusable).
7. **CausalEvent::SettlementFormed/Dissolved signatures**: `SettlementFormed` should
   carry `settlement_id`, `founder_agents`, `tick`. `SettlementDissolved` should carry
   `settlement_id`, `cause: DissolutionCause` enum, `final_pop`. Confirm at planning.
8. **AgentDecisionSystem 8th cascade — settlement-aware mechanism**: migration
   toward settlement (population pull) vs migration away (overcrowding) vs loyalty
   lock (refuse to leave). Default candidate: **migration pull only** for Phase 10
   (simplest testable assertion; overcrowding/loyalty are Phase 11+ enrichments).

---

## 3. Section 12+ Candidates (deferred)

`Section 12` will anchor the Phase 11 direction. Candidates (post-Settlement base):

1. **Advanced AI (BT / Utility / LLM)** — Section 8+ §3 #4. The remaining original
   candidate. After Phase 10 delivers settlement-level aggregation, the AI rewrite
   can consume *both* Memory and Settlement priors as bias sources. A settlement-aware
   behavior tree (agent weighs settlement loyalty in decision) or a utility AI that
   scores actions against settlement-level goals becomes feasible. Choosing this after
   Settlement means the AI rewrite inherits the full Social + Memory + Combat +
   Settlement substrate.
2. **Other** (user-mandated direction after Phase 10 closure):
   - Cross-settlement diplomacy / trade (requires two settlements — Phase 10 provides)
   - Religion / culture / language mechanics (emergent from settlement identity)
   - Wildlife / non-agent combat (Phase 9 OUT-OF-SCOPE item deferred here)
   - Civilisation tech tree (Caveman2Cosmos Phase 2 analogue)

Section 12 decomposition is **NOT** specified here. It is the user-mandated direction
chosen after Phase 10 closure, per the established governance pattern.

---

## 4. Single Source of Truth

| Concern | Document |
|---------|----------|
| V7 phase progress tracking + closure declarations | `.harness/audit/v7_progress.md` |
| Phase 7 anchor + scope | `.harness/audit/section_8_plus_design.md` |
| Phase 8 anchor + scope | `.harness/audit/section_9_plus_design.md` |
| Phase 9 anchor + scope | `.harness/audit/section_10_plus_design.md` |
| Phase 10 anchor + scope (this layer) | `.harness/audit/section_11_plus_design.md` (this file) |
| Phase 10 sub-stage decomposition | `.harness/plans/phase10.md` (planning-first dispatch, TBD) |
| Phase 6 sub-stage decomposition (existing) | `.harness/plans/phase6.md` |
| Behavioural + architectural guidelines | `CLAUDE.md` |
| V7 roadmap canonical comment block | `rust/crates/sim-core/src/lib.rs:3-11` |

---

## 5. Honest Reservations

1. **Selection is a product judgement, not a derivation.** Multi-building Settlement
   is the Section 8+ §3 #3 candidate ranked by substrate-fit after Combat closure.
   It is the user-mandated direction and it is defensible on architecture grounds.
   It is **not** the only defensible choice. If the product vision shifts toward
   Advanced AI (decision architecture rewrite) or another direction, this anchor
   should be superseded without reservation.

2. **Settlement scope risk.** Multi-building Settlement introduces a new *aggregation
   layer* — the first time Phase X delivers a structure that supervenes over individual
   agents and buildings. Aggregation layers historically add complexity faster than
   expected (see Dwarf Fortress fortress-mode scope). Phase 10 deliberately scopes
   to the minimum viable settlement loop (formation → lifecycle → dissolution +
   chronicle) and defers diplomacy, religion, tech tree, and UI to Phase 11+.

3. **Population lifecycle Birth mechanism.** Births introduce new agents mid-simulation
   — the first time agents appear other than at world spawn. The `AgentSpawned` event
   extension must preserve the `parent_agent: Option<AgentId>` chain for causal
   traceability (axiom #1). If Birth introduces complexity that disrupts the Phase 5-9
   spawn assumptions, it should be deferred to Phase 10-β (after the substrate
   Phase 10-α lands) rather than included in the α scope.

4. **Infrastructure pattern monitoring is mandatory.** Issues 12-16 closed Patterns
   A/B/C/D/G in the harness pipeline. Phase 10 will trigger new pipeline dispatch
   cycles. The classify_recode.sh and generate_report.sh fixes from Issues 15-16
   should be verified against Phase 10-α pipeline runs to confirm they hold. Issue 16
   post-fix verification (`pipeline_report.md` generation) is a Phase 10-α dispatch
   natural checkpoint.

5. **6th escalation — governance document debt.** Each successive section design
   document adds ~250-350 lines to the governance audit trail. If Phase 11+ follows
   the same pattern, the audit trail will contain five `section_N_plus_design.md`
   files (8/9/10/11/12) plus `v7_progress.md` (1300+ lines). This is manageable
   but will eventually require consolidation into a proper master direction document.
   That consolidation is itself a governance question for the user to mandate, not
   a task to self-assign here.

6. **Issue 16 fix integration.** The `generate_report.sh` duration_sec sanitization
   fix (`182476f9`) is in place. Phase 10-α planning dispatch + implementation
   pipeline will be the first live test of the fix under real run conditions. Monitor
   for recurrence of the `[[: 0\n0: syntax error` in pipeline stderr.
