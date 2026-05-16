# V7 Section 8+ Design — Phase 7 Anchor

**Status**: High-level anchor (Sec8-4-a). Detailed sub-stage decomposition deferred to
`.harness/plans/phase7.md` (planning-first dispatch, Phase 6 precedent).

**Authored**: 2026-05-17. **Author**: governance batch (post-V7 Foundation closure).

---

## 0. Provenance & Escalation Disclosure

This is the third successive escalation against the absence of a V7 Master Direction
document. Phase 6 planning, Phase 7 planning, and now Section 8+ design have all
triggered the same condition:

- `/mnt/project/` (claude.ai project_files): absent
- `find . -iname "*master*direction*"` in repo: zero matches
- root `.md` files: `AGENTS.md`, `CLAUDE.md`, `README.md` only

The "V7 Master Direction" referenced in `rust/crates/sim-core/src/lib.rs:3`
("V7 Master Direction Section 7: Foundation systems.") is a **conceptual anchor**,
not a written artefact. Foundation Week 1-12 was defined in-code by the
`lib.rs:5-11` comment block; Section 8+ has no equivalent.

Path chosen (Sec8-1-b): **`.harness/audit/section_8_plus_design.md`** (this file),
consistent with the `v7_progress.md` governance-audit precedent. Retroactively writing
a full Section 1-7 master document would be substantial scope (~2-4h) that duplicates
information already captured by `v7_progress.md` + `CLAUDE.md` + `lib.rs:5-11`. The
governance audit path is the proportionate choice.

---

## 1. Context

- V7 Foundation Week 1-12 complete (closure declaration `a0666b6c`, 2026-05-17).
- CLAUDE.md V7-aligned (`2e51c167`, A-1~A-13 prereq section retired).
- 26 consecutive APPROVED harness verdicts.
- Architecture base: Material + Tile + Influence + CausalEvent + Agent + Needs +
  Decision + Construction + chronicle infrastructure.
- Recovery span: 2026-05-03 → 2026-05-17 (~14 days).

V7 Foundation closed the **infrastructure** of god-game simulation:
- Agents exist, move, have needs, decide, consume, construct.
- Every action is causally traceable end-to-end (axiom #1).
- Visual + behavioural chronicles prove the loop (axiom #3).

What V7 Foundation deliberately did **not** include:
- Agents do not **interact with each other**. No agent-to-agent edges in the
  causal graph.
- No HP / damage / combat / death-by-other-agent.
- No memory / learning / skill progression (despite leftover A-13 hint).
- No multi-building settlement coordination beyond single-site construction.
- No advanced AI (behavior trees, utility AI, LLM integration).

These gaps are the candidate anchors for Section 8+.

---

## 2. Section 8 — Phase 7 Anchor: Multi-agent Social System

### 2.1 Choice rationale

The "missing piece that makes the most natural next sequel after First Daily Routine"
is **agent-to-agent interaction**. Justification:

1. **Substrate fit**: `AgentDecisionSystem` (priority 125), `CausalEvent::AgentDecision`,
   `DecisionReason`, `AgentState` FSM, `TargetKind` enum are all extension-ready. A new
   `TargetKind::Agent(AgentId)` variant follows the Phase 5-γ Sleep precedent / Phase 6-α
   ConstructionSite precedent exactly.
2. **Causal continuity**: the chain `AgentDecision{SocialReason} →
   SocialInteractionStarted → SocialInteractionCompleted` mirrors the Phase 6 build-a-
   shelter chain shape. Reuse of the chronicle harness pattern is direct.
3. **Visual milestone fit (axiom #3)**: two or more agents converging on a tile and
   "doing something together" is the next visual milestone after `p4-γ`
   (agents appear) and `p6-γ` (agent constructs). Visual ambition is incremental.
4. **God-game vision fit**: Caveman2Cosmos / RimWorld / Songs of Syx all converge on
   the moment "agents notice each other and form relationships" as the
   non-trivial behavioral step that turns the sandbox into a society.
5. **Lower coupling than Combat**: Combat presupposes BodyHealth (currently absent —
   A-11 leftover) and a death/spawn lifecycle. Social interaction can be modelled
   with the existing agent set as a pure additive surface, then Combat layers on top
   in Section 9.

### 2.2 High-level scope

**In scope** (Phase 7):
- Agent-to-agent proximity / co-location detection.
- A `Social` component (analogue of `Hunger`/`Thirst`/`Sleep`): tracks an agent's
  per-tick social need or saturation (loneliness vs satiety). Drives FSM entry into
  `Seeking { TargetKind::Agent(other_id) }`.
- A new system at priority **134** (after `ConstructionSystem`=133):
  `SocialInteractionSystem` handles progress of co-located interacting agents
  (mutual handshake → exchange tick → complete).
- New `CausalEvent` variants: `SocialInteractionStarted`,
  `SocialInteractionCompleted`. Parent-chained to `AgentDecision{SocialReason}`.
- New `DecisionReason::SocialReason` variant (5th, after `ConstructionReason`).
- A `Relationship` model — Phase 7 ships the **minimum viable shape only**:
  a sparse `HashMap<(AgentId, AgentId), RelationshipState>` on `SimResources`,
  where `RelationshipState` is a single scalar (`familiarity: f64` in `[0.0, 1.0]`)
  incremented on each successful interaction. Rich relationship types
  (friend / rival / family / etc.) are deferred to Section 9+ unless a Phase 7-δ
  expansion is mandated.
- A chronicle harness (Phase 7-γ): two agents on a small grid, both with
  `Social` needs growing, eventually interact, `familiarity` increments,
  chronicle log demonstrates the 3-link causal chain.

**Out of scope** (deferred to Section 9+):
- HP / damage / combat / death-by-agent.
- Memory / event recall / persistent episodic memory.
- Multi-building settlement coordination.
- Speech / dialogue / language.
- LLM integration of any kind.
- Behavior trees / utility AI rewrite.
- Family / genealogy enrichment beyond the `familiarity` scalar.

### 2.3 Sub-stage decomposition (TBD — planning-first dispatch)

Detailed decomposition lives in `.harness/plans/phase7.md` after a planning-first
dispatch (Phase 6 precedent). The expected shape (subject to that planning round):

- **Phase 7-α**: `Social` component + `TargetKind::Agent(AgentId)` variant +
  `DecisionReason::SocialReason` + serde + harness with ≥12 assertions.
  Zero runtime system change. Pure data substrate. (Phase 6-α precedent.)
- **Phase 7-β**: `SocialInteractionSystem` (priority 134) +
  `SocialInteractionStarted/Completed` causal variants + `AgentDecisionSystem`
  cascade extension (4th condition: co-located agent with mutual social breach) +
  `RelationshipState` sparse map on `SimResources` + harness with ≥12 assertions.
  (Phase 6-β precedent.)
- **Phase 7-γ**: End-to-end chronicle harness, two agents, full interaction cycle,
  ≥13 assertions, closure milestone. (Phase 6-γ precedent.)
- **Phase 7-δ** (optional, user mandate base): Visual milestone — Godot scene
  showing two agents converging + interaction indicator + CausalPanel `SocialReason`
  label. Out of scope until V7 visual ambition is reassessed.

### 2.4 Dependencies

- V7 Foundation Week 1-12 complete ✓ (`a0666b6c`).
- V7 architecture base 정통 활용:
  - `AgentState` FSM (Phase 5-β/γ)
  - `AgentDecisionSystem` priority 125 (Phase 5-β/γ)
  - `CausalEvent` + `EventId` chain (Phase 3-α/β)
  - `Agent { id: AgentId }` (Phase 5-α)
  - `Position` (Phase 4-α)
  - `SimResources::issue_event_id` (Phase 3-β)
- **No prerequisite on BodyHealth / Combat / Memory** — Phase 7 is additive over
  the existing agent set.

### 2.5 Open questions (resolved at planning-first dispatch time)

1. **Social need shape**: scalar `loneliness` that grows over time (mirror of
   `Hunger.value`) vs an "interaction count budget per simulated day" (cap-and-
   reset on time_of_day cycle). The simpler scalar mirroring Phase 5 needs is the
   default unless evidence dictates otherwise.
2. **Interaction trigger**: same-tile co-location only (Phase 5/6 precedent) vs
   adjacent-tile (Chebyshev radius 1). Same-tile default.
3. **Interaction resolution**: instant (one tick) vs N-tick progress like
   construction. **Construction-style progress** is the default — preserves the
   `Seeking → Consuming → Idle` FSM shape and the 3-event causal chain
   (Started → Completed → Decision-of-other-agent).
4. **Mutual consent semantics**: both agents must be in `Seeking { Agent(other) }`
   simultaneously vs one agent's `Consuming` is sufficient. Default: **both must
   be in Seeking** (symmetric mutual handshake), avoiding "harassment" gameplay
   semantics for free.
5. **`familiarity` accumulation rule**: `+= 0.1` per completed interaction
   (saturating at 1.0). Default; revisable.
6. **Cascade ordering vs needs**: where does `SocialReason` sit in the
   `AgentDecisionSystem` cascade ladder? Default: **lowest** (after Construction),
   mirroring the Phase 6-β decision that Needs always win.

These six questions are the natural blockers for Phase 7-α's planning-first dispatch.

---

## 3. Section 9+ (deferred)

`Section 9` will anchor the **second** Phase 7-and-beyond direction. Candidates,
ranked by substrate-fit (most natural sequel first):

1. **Combat System** — once Social lands, agent-to-agent interaction has a positive
   axis (friendship) and the natural symmetric extension is the negative axis
   (conflict). Requires BodyHealth (A-11 leftover, ~Phase 4-δ scope).
2. **Memory System** — agents recall prior interactions, biasing future
   `AgentDecision` cascades. Causal substrate (Phase 3-β `EventId` chain) is
   directly reusable.
3. **Multi-building Settlement** — natural Phase 6 extension. Multi-site
   coordination, population growth, settlement-level decisions.
4. **Advanced AI (BT / Utility / LLM)** — full architecture replacement. Substantial
   scope; sensible only after the cascade above lands.

Section 9 decomposition is **NOT** specified here. It is the user-mandated direction
chosen after Phase 7 closure, per the established governance pattern.

---

## 4. Single source of truth

| Concern | Document |
|---------|----------|
| Phase 1-6 progress tracking + closure declaration | `.harness/audit/v7_progress.md` |
| Phase 7+ anchor + scope (this layer) | `.harness/audit/section_8_plus_design.md` (this file) |
| Phase 7 sub-stage decomposition | `.harness/plans/phase7.md` (planning-first dispatch, TBD) |
| Phase 6 sub-stage decomposition (existing) | `.harness/plans/phase6.md` |
| Behavioural + architectural guidelines | `CLAUDE.md` |
| V7 roadmap canonical comment block | `rust/crates/sim-core/src/lib.rs:5-11` |

---

## 5. Honest reservations

1. The "Multi-agent Social System first" recommendation is a **product judgement**,
   not a derivation from a master document. It is the most natural substrate-fit
   sequel; it is **not** the only defensible choice. If the user's god-game vision
   prioritises Combat (visceral feedback loop) or Multi-building Settlement
   (civilisation-scale visualisation) over Social, the recommendation should be
   overridden without reservation.
2. The Phase 7-δ visual milestone is left **unscoped** in this anchor. V7 chose to
   prove the simulation loop without binding to a heavy visual debt. If the user
   wants Phase 7 to carry a visual milestone, that decision should be made at
   the `phase7.md` planning-first dispatch, not retroactively.
3. The `RelationshipState` scalar (`familiarity: f64`) is intentionally minimal.
   Rich relationship semantics (kinship, rivalry, dependency, etc.) are valuable
   but each is a Section 9+ scope unit. Phase 7 ships the minimum that closes
   the loop.
4. The "interaction = mutual handshake (both Seeking)" default avoids one-sided
   "harassment" semantics by construction. That choice is opinionated. Some
   simulations (RimWorld) prefer one-sided initiation. Phase 7-α's planning
   dispatch should confirm or override this.
