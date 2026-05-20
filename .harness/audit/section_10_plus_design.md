# V7 Section 10+ Design — Phase 9 Anchor

**Status**: High-level anchor (Sec10-4-a). Detailed sub-stage decomposition deferred to
`.harness/plans/phase9.md` (planning-first dispatch, Phase 6/7/8 precedent).

**Authored**: 2026-05-20. **Author**: governance batch (post-V7 Phase 8 closure).

---

## 0. Provenance & Escalation Disclosure

This is the fifth successive escalation against the absence of a V7 Master Direction
document. Phase 6 planning, Phase 7 planning, Section 8+ design, Section 9+ design, and
now Section 10+ design have all triggered the same condition:

- `/mnt/project/` (claude.ai project_files): absent
- `find . -iname "*master*direction*"` in repo: zero matches
- root `.md` files: `AGENTS.md`, `CLAUDE.md`, `README.md` only

The "V7 Master Direction" referenced in `rust/crates/sim-core/src/lib.rs:3` is a
**conceptual anchor**, not a written artefact. The governance audit path
(`.harness/audit/section_N_plus_design.md`) is the established proportionate choice.

**Milestone context:**
- V7 Foundation Week 1-12 complete (`a0666b6c`, 2026-05-17)
- V7 Phase 7 (Multi-agent Social System) complete (`f1c12f9d` + `c924770d`, 2026-05-17)
- V7 Phase 8 (Memory System) complete (`0660f4ea` + `7da81c0b`, 2026-05-20)
- Issues 12+13+14+15 closure (`00274b57` + `ebbf6ddc` + `ddd5348c`)
- 5 ENV-BYPASS chains avoided (3 closed + 4th + 5th avoided per Option D)
- Section 8+ design (`0ed3ec16`) §3 + Section 9+ design (`67c9a49d`) §3 candidates
  ranked path base for this anchor decision

---

## 1. Context

V7 Phase 8 closed the **episodic memory loop** of god-game simulation:

- Agents recall prior interactions, biasing future decisions via the cascade-flip mechanism.
- Per-agent `Memory { entries: [MemoryEntry; 32] }` stores encoded events with valence,
  salience, and reinforcement counts.
- `MemorySystem` 4-phase tick (recalled-set collection → selective decay → encoding →
  reinforcement) is live at priority 136.
- Every memory recall is causally traceable: `MemoryRecalled{CascadeBias}` →
  `AgentDecision{MemoryReason, parent: recalled.id}`.

**V7 architecture base final substantial:**

```
10 ECS components:
  Position, Agent, Hunger, Thirst, Sleep, AgentState,
  BuildingBlueprint/ConstructionSite, Social, Relationship, Memory

Runtime systems (priority-ordered):
   90  BuildingStampSystem
  100  InfluenceUpdateSystem
  110  AgentInfluenceSampleSystem
  120  AgentMovementSystem
  125  AgentDecisionSystem (6-arm cascade: Hunger/Thirst/Fatigue/Construction/Social/Memory)
  130  HungerDecaySystem
  131  ThirstDecaySystem
  132  SleepDecaySystem
  133  ConstructionSystem
  134  SocialInteractionSystem
  135  SocialDecaySystem
  136  MemorySystem

9 CausalEvent variants:
  BuildingPlaced, StampDirty, InfluenceChanged,
  AgentDecision, ConstructionStarted, ConstructionCompleted,
  SocialInteractionStarted, SocialInteractionCompleted, MemoryRecalled

6 DecisionReason variants:
  HungerThresholdBreach, ThirstThresholdBreach, FatigueThresholdBreach,
  ConstructionReason, SocialReason, MemoryReason
```

**Causal chain (6 reasons → 6 emission chains):**
```
Hunger/Thirst/Fatigue threshold breach → AgentDecision → consume
Construction proximity                 → AgentDecision → ConstructionStarted → ConstructionCompleted → BuildingPlaced
Mutual social co-location              → AgentDecision → SocialInteractionStarted → SocialInteractionCompleted
Memory cascade-flip                    → MemoryRecalled → AgentDecision{MemoryReason} (anti-recursion safe)
```

**Section 9+ §3 candidates** (post-Memory base, ranked by Memory-compositional
substrate-fit):
1. **Combat System** — Memory of conflict → grudge/revenge/rivalry mechanics (←this)
2. **Multi-building Settlement** — Memory of locations → settlement history/inheritance
3. **Advanced AI (BT/Utility/LLM)** — Memory as bias source for redesigned decision arch

---

## 2. Phase 9 Anchor — Combat System (V7 Week 17-18)

### 2.1 Choice rationale

**Memory substrate compositional fit #1:**

The Memory substrate opened by Phase 8 maps most directly onto Combat:
- A negative `SocialInteractionCompleted` event → `MemoryEntry{low valence, high salience}`
  already encodes prior conflict in the agent's memory.
- `MemoryRecallTrigger::CombatContext` (new variant) triggers recall when a co-located
  agent-proximity event fires — the agent *remembers* the prior conflict before deciding
  to engage.
- The cascade-flip mechanism (Phase 8-β, `BIAS_FLIP_THRESHOLD`) activates:
  `MemoryRecalled{CombatContext}` → `AgentDecision{CombatReason}` → `CombatStarted`.

Full memory-driven causal chain:
```
SocialInteractionCompleted{negative outcome} → MemoryEntry{low valence}
  → [later tick] MemoryRecalled{CombatContext, parent: SocialCompleted.id}
  → AgentDecision{CombatReason, parent: MemoryRecalled.id}
  → CombatStarted → CombatCompleted
  → [optionally] MemoryEntry{combat outcome, low valence + high salience}
```
Every node carries `id + parent` — axiom #1 (causal traceability) extends through combat.

**Social ↔ Combat symmetric axis:**

Phase 7 established the *positive* agent-to-agent interaction axis (friendship /
`SocialInteractionSystem`). Phase 9 establishes the *negative* axis (conflict /
`CombatSystem`). The structural symmetry is natural: both start from co-location
proximity, both emit `Started → Completed` event pairs, both extend the
`AgentDecisionSystem` cascade.

**Axiom #3 (emotional depth) visual milestone:**

Visceral combat — agents fighting because they *remember* past grievances — is the
god-game emotional depth milestone that Dwarf Fortress, RimWorld, and Songs of Syx all
deliver. The Phase 8-γ chronicle proved memory drives decisions; Phase 9-γ will prove
memory drives *conflict*.

**BodyHealth integration path:**

`BodyHealth` (A-11 leftover, scope undefined post-V7-reset) is the Phase 9-α prereq
sub-stage. Combat without HP tracking reduces to a state-change event with no damage
model. Phase 9 is the natural slot to integrate BodyHealth as the substrate Phase 9-β
CombatSystem consumes.

### 2.2 High-level scope

**IN scope (Phase 9):**

- `BodyHealth` component (Phase 9-α prereq):
  - HP tracking, damage application, death detection
  - Scope decision (simple total HP vs per-body-part): deferred to `phase9.md` §α
- `CombatSystem` (priority TBD, candidate: 137):
  - Agent-vs-agent proximity trigger → aggressor/target selection
  - Damage application using `BodyHealth`
  - Death handling (entity despawn + possible `Corpse` component — planning §β)
- `CausalEvent::CombatStarted` + `CausalEvent::CombatCompleted` (11th + 12th variants)
- `DecisionReason::CombatReason` (7th variant)
- `MemoryRecallTrigger::CombatContext` (Phase 8 enum extension, new variant)
- `AgentDecisionSystem` 7th cascade arm: Memory-of-conflict → Combat trigger bias
- `RelationshipState` hostility field (negative familiarity axis — planning §β scope)
- Chronicle harness: combat cycle evidence + memory-driven combat trigger assertion

**OUT of scope (Phase 10+ defer):**

- Wildlife / non-agent enemies (wildlife system is a separate Phase)
- Combat tactics / positioning / fleeing (V7 simplicity-first; Phase 10+ layering)
- Death consequences at settlement level (population decline, grief — Phase 10+)
- Combat equipment / weapons / armor (material system integration — Phase 10+)
- Phase 9-δ UI integration (CombatPanel, BodyHealth inspector — optional, user-gated)

### 2.3 Sub-stage shape (TBD — `phase9.md` planning-first dispatch)

| Phase | Scope | Precedent |
|-------|-------|-----------|
| Phase 9-α | `BodyHealth` component + constants (Phase 4-δ substrate integration) | Phase 6-α / 7-α / 8-α data-only substrate |
| Phase 9-β | `CombatSystem` + `CausalEvent` variants + `DecisionReason` + 7th cascade arm | Phase 6-β / 7-β / 8-β system + cascade |
| Phase 9-γ | Chronicle harness (combat cycle + memory-driven trigger evidence) | Phase 6-γ / 7-γ / 8-γ chronicle |
| Phase 9-δ | UI integration (CombatPanel, BodyHealth renderer) — optional, user-gated | Phase 7-δ / 8-δ precedent |

Estimates and verification criteria are deferred to `phase9.md`.

### 2.4 Dependencies

- V7 Foundation Week 1-12 complete (`a0666b6c`)
- V7 Phase 7 (Multi-agent Social System) complete (`f1c12f9d`)
- V7 Phase 8 (Memory System) complete (`0660f4ea`)
  - `MemorySystem` 4-phase tick + `REINFORCEMENT_BOOST` live
  - `MemoryRecallTrigger` enum available for extension
  - Cascade-flip mechanism (`BIAS_FLIP_THRESHOLD`) available for 7th arm
- `Relationship` component (Phase 7-α, `familiarity: f64`) — hostility extension base

**No dependency** on Combat prerequisites outside BodyHealth. Phase 9-α delivers
BodyHealth as its own sub-stage before Phase 9-β requires it.

### 2.5 Open questions (resolve at `phase9.md` planning-first dispatch)

1. **BodyHealth spec**: simple total HP (`f64`) vs per-body-part damage model?
2. **Combat trigger**: mutual proximity only, or aggressor-initiated from Memory bias?
3. **Death mechanism**: entity despawn immediately, or `Corpse` component + decay tick?
4. **CombatSystem priority**: 137 (after MemorySystem 136) or different slot?
5. **Combat damage formula**: deterministic (strength stat?) vs variance (RNG seed)?
   Does RelationshipState hostility amplify damage?
6. **Memory-of-conflict trigger condition**: negative valence threshold? recency weight?
7. **`MemoryRecallTrigger::CombatContext` signature**: what fields does it carry?
8. **RelationshipState hostility**: separate `hostility: f64` field, or negative
   familiarity (allow `familiarity` to go below 0.0)?

---

## 3. Section 11+ Candidates (deferred)

`Section 11` will anchor the Phase 10 direction. Candidates (post-Combat base):

1. **Multi-building Settlement** — Section 8+ §3 #3. Community combat history,
   settlement-of-origin memory, population lifecycle + episodic inheritance.
   Combat deaths create historical events that communities remember.
2. **Advanced AI (BT / Utility / LLM)** — Section 8+ §3 #4. A combat-aware,
   memory-biased decision architecture. Choosing after Combat means the AI
   rewrite inherits both Social *and* Combat priors already in the memory system.

Section 11 decomposition is **NOT** specified here. It is the user-mandated direction
chosen after Phase 9 closure, per the established governance pattern.

---

## 4. Single Source of Truth

| Concern | Document |
|---------|----------|
| V7 progress tracking + closure declarations | `.harness/audit/v7_progress.md` |
| Phase 7 anchor (Multi-agent Social System) | `.harness/audit/section_8_plus_design.md` |
| Phase 7 sub-stage decomposition | `.harness/plans/phase7.md` |
| Phase 8 anchor (Memory System) | `.harness/audit/section_9_plus_design.md` |
| Phase 8 sub-stage decomposition | `.harness/plans/phase8.md` |
| **Phase 9 anchor (Combat System, this document)** | **`.harness/audit/section_10_plus_design.md`** |
| Phase 9 sub-stage decomposition | `.harness/plans/phase9.md` (planning-first dispatch — next) |
| Phase 6 sub-stage decomposition | `.harness/plans/phase6.md` |
| Infrastructure governance audit | `.harness/audit/v7_progress.md` (§ Issues + §Governance) |
| Behavioural + architectural guidelines | `CLAUDE.md` |
| V7 roadmap canonical comment block | `rust/crates/sim-core/src/lib.rs:5-11` |

Section 10 supersedes Sections 8 and 9 only for the Phase 9 anchor. Sections 8 and 9
remain the authoritative records for Phase 7 and Phase 8 scope respectively.

---

## 5. Honest Reservations

1. **Choice is a product judgement, not a derivation.** Section 8+ §3 ranked Combat #1
   and Memory #2 by *substrate-fit*. The user-mandated choice of Combat for Phase 9 is
   defensible on memory-compositional grounds (§2.1), but it is a vision choice, not
   forced by the architecture. If the user later prioritises Multi-building Settlement
   (civilisation-scale visualisation) or Advanced AI (decision architecture rewrite) over
   Combat, the recommendation should be overridden without reservation.

2. **BodyHealth scope is substantial.** A-11 (BodyHealth, leftover from pre-V7-reset)
   had undefined scope at V7 reset. Phase 9-α integrates it — but the spec decisions
   (simple HP vs per-body-part, death mechanic, respawn or permanent death) carry
   significant downstream consequences. The `phase9.md` planning-first dispatch must
   resolve §2.5 open questions 1–3 before Phase 9-α begins.

3. **Phase 9-β is the heaviest expected sub-stage.** Phase 7-β (6 code attempts,
   5 RE-CODE rounds before APPROVE) and Phase 8-β (6 code attempts, Issues 12–15 path)
   set the precedent. Phase 9-β (CombatSystem + 7th cascade arm + BodyHealth integration)
   is structurally similar in complexity. Infrastructure pattern monitoring (Patterns
   A–D) is mandatory.

4. **Memory-driven combat trigger semantics are opinionated.** The `CombatContext`
   trigger threshold is a design choice, not derivable from the memory model alone.
   An agent should not attack on every recalled negative event — the bias must be
   conditioned on proximity, hostility magnitude, and possibly a cooldown. §2.5
   open questions 6–8 must be resolved at planning time.

5. **Wildlife combat is explicitly OUT of scope.** The wildlife system (Phase P-* series,
   landed earlier) exists in the codebase, but agent-vs-wildlife combat mechanics are
   not Phase 9 scope. Mixing agent-vs-agent and agent-vs-wildlife combat in one phase
   risks scope explosion. Phase 10+ is the correct slot for wildlife combat integration.

6. **Infrastructure pattern monitoring is mandatory.** Issues 12–15 closed Patterns
   C–G; Patterns A (API rate limit), B (Codex timeout), C (Generator silent death),
   D (attempt-3 penalty cap) remain active. Pipeline streak 31+ is healthy but not
   immunity. `pipeline_report.md` absent mechanism (Issue 16 후보) is an unresolved
   governance gap.
