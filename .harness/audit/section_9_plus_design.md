# V7 Section 9+ Design — Phase 8 Anchor

**Status**: High-level anchor (Sec9-4-a). Detailed sub-stage decomposition deferred to
`.harness/plans/phase8.md` (planning-first dispatch, Phase 6 + 7 precedent).

**Authored**: 2026-05-18. **Author**: governance batch (post-V7 Phase 7 closure).

**Precedent**: `.harness/audit/section_8_plus_design.md` (Phase 7 anchor —
Multi-agent Social System, commit `0ed3ec16`, 2026-05-17). Structure and tone
mirror that document. Section 9 supersedes Section 8 only for the Phase 8
anchor; Section 8's Phase 7 anchor remains the authoritative record for what
Phase 7 actually delivered (`f1c12f9d` chronicle harness).

---

## 0. Provenance & 4th escalation disclosure

This is the fourth successive escalation against the absence of a V7 Master
Direction document. Phase 6 planning, Phase 7 planning, Section 8+ design, and
now Section 9+ design have all triggered the same condition:

- `/mnt/project/` (claude.ai project_files): absent
- `find . -iname "*master*direction*"` in repo: zero matches
- Root `.md` files: `AGENTS.md`, `CLAUDE.md`, `README.md` only
- The "V7 Master Direction" referenced in `rust/crates/sim-core/src/lib.rs:3`
  remains a **conceptual anchor**, not a written artefact.

Path chosen (Sec9-1-b): **`.harness/audit/section_9_plus_design.md`** (this
file, new). Filename mirrors Section 8's `section_8_plus_design.md`. The
Section 8 document explicitly demarcated Section 9 as a *separate future
document* (§3, "Section 9 decomposition is NOT specified here"). The modular
path honours that demarcation; an append would have buried Section 9 inside a
file whose section number doesn't match its subject.

Governance closure chain — extended (post-Section-9-anchor, 2026-05-18):

| Stage | Commit | Scope |
|-------|--------|-------|
| 1 | `a0666b6c` | V7 Foundation Week 1-12 complete |
| 2 | `2e51c167` | CLAUDE.md V7 reset 정합 update |
| 3 | `0ed3ec16` | Section 8+ design — Phase 7 anchor (Multi-agent Social System) |
| 4 | `a00a1b77` | Phase 7-γ defer + partial closure disclosure |
| 5 | `d4c050e2` | Infrastructure governance audit + V7 partial-closure final |
| 6 | `ebbf6ddc` | Issue 14 fix — Pattern E closure (FFI SKIP token) |
| 7 | `f1c12f9d` | Phase 7-γ chronicle harness implementation (V7 Phase 7 complete) |
| 8 | `c924770d` | V7 Phase 7 closure declaration |
| 9 | *(this commit)* | Section 9+ design — Phase 8 anchor (Memory System) |

---

## 1. Context

- V7 Foundation Week 1-12 complete ✓ (`a0666b6c`, 2026-05-17).
- V7 Phase 7 (Multi-agent Social System) complete ✓ (`f1c12f9d` + `c924770d`,
  2026-05-17). 459 new harness tests, score 90/100 at cold-tier threshold 75.
- Infrastructure governance Issues 12+13+14 closed ✓ (`00274b57` +
  `ebbf6ddc`). Pipeline mechanism gap chain (verdict staleness, gate partial
  credit, non-APPROVE scoring, FFI SKIP token) audited and corrected.
- 28+ dispatches since V7 reset (2026-05-03). Patterns A–F classified in
  `v7_progress.md`; Pattern E closed via Issue 14, Patterns A/B/C remain
  environmental signal under continued monitoring.
- 29 consecutive APPROVED harness verdicts (V7 reset → Phase 7-γ closure).

V7 Foundation closed the **infrastructure** of god-game simulation. Phase 7
closed the **social substrate** (agent-to-agent interaction). Together they
deliver:

- Agents exist, move, have needs, decide, consume, construct, **and interact
  with each other**. Multi-agent causal chain operational
  (`AgentDecision{SocialReason} → SocialInteractionStarted →
  SocialInteractionCompleted`).
- Every action is causally traceable end-to-end (axiom #1).
- Visual + behavioural + multi-agent chronicles prove the loop (axiom #3).

What Phase 7 deliberately did **not** include (remaining Section 8+ §3
candidates):

- HP / damage / combat / death-by-other-agent (Section 8 §3 candidate #1).
- **Memory / event recall / persistent episodic memory** (Section 8 §3
  candidate #2 — this document's anchor).
- Multi-building settlement coordination beyond single-site construction
  (Section 8 §3 candidate #3).
- Advanced AI (behavior trees, utility AI, LLM integration — Section 8 §3
  candidate #4).
- Speech / dialogue / language.
- Family / genealogy enrichment beyond the `familiarity` scalar.

These remain the candidate anchors for Section 9 and beyond.

---

## 2. Section 9 — Phase 8 Anchor: Memory System

### 2.1 Choice rationale

Section 8 §3 ranked Memory System as **candidate #2 by substrate-fit** behind
Combat. The user-mandated choice for Phase 8 is **Memory System**, not Combat.
The rationale below documents why that mandate is defensible — not as a
re-derivation of the Section 8 ranking, but as the product-vision reasoning
specific to Phase 8's place in the V7 → V8+ arc.

1. **Substrate fit (lowest prereq cost of all four candidates)**: Phase 3-β
   delivered the `EventId` + parent-chain causal log. Every `CausalEvent`
   that an agent participates in (`AgentDecision`, `SocialInteractionStarted`,
   `SocialInteractionCompleted`, future `ConstructionStarted` references with
   that agent as the actor, etc.) is already a candidate memory anchor.
   Memory System reads that substrate; it does **not** require a new
   foundational layer the way Combat requires BodyHealth (A-11 leftover).
2. **Causal continuity (axiom #1 extension)**: today's causal chain ends at
   an agent's `AgentDecision`. With Memory, the chain extends further:
   `MemoryRecalled → AgentDecision{MemoryReason}` makes the agent's
   **internal** reasoning trace observable, not just its external actions.
   Walking the "왜?" chain from a current decision back through a recalled
   past event is the natural completion of axiom #1's traceability promise.
3. **Visual milestone fit (axiom #3, deferred)**: visual surfacing of memory
   (recall indicator on an agent, memory-biased decision label in CausalPanel)
   is Phase 8-δ optional, not in-scope for the foundational dispatch. The
   simulation-level milestone is "an agent's later decision is provably
   biased by an earlier event" — observable via the chronicle harness
   without UI.
4. **God-game vision fit**: in Caveman2Cosmos / Songs of Syx / RimWorld /
   Dwarf Fortress, the moment a colonist "remembers" something (a friend
   died, an enemy struck them, a hunt failed at this tile) is the moment the
   simulation starts feeling like a society of persons rather than a Petri
   dish. Memory System is the **emotional-depth milestone** of the V7+
   foundation.
5. **Compositional base for Sections 10+ candidates**: each remaining
   Section 8 §3 candidate gets richer when Memory exists:
   - **Combat** (Section 8 §3 #1): memory of conflict turns one-off
     skirmishes into recurring rivalries; revenge / forgiveness mechanics
     become possible.
   - **Multi-building Settlement** (Section 8 §3 #3): memory of locations
     turns "where do I want to build?" from a per-tick reachability search
     into a "this hill is where my family lived" narrative.
   - **Advanced AI** (Section 8 §3 #4): memory is the substrate any
     behaviour-tree / utility-AI / LLM upgrade integrates over. Building it
     first means the AI rewrite has a real bias source to consume.
   Taking Memory before any of (#1, #3, #4) lets all three be Memory-aware
   when they land, rather than retrofitting memory into them.
6. **Risk-mitigation read**: Phase 7's 6-dispatch γ stall (resolved at
   `f1c12f9d`) exposed how fragile substantial harness scope can be under
   recurring infrastructure patterns A/B/C. Choosing the **lowest
   substrate-cost candidate** for Phase 8 reduces the surface area where
   those patterns can re-fire. Combat (BodyHealth prereq) and Multi-building
   Settlement (aggregation layer) both expand substantial substrate;
   Advanced AI rewrites it. Memory adds one storage component, one system,
   one CausalEvent variant, one DecisionReason variant — the smallest
   possible delta that still ships a milestone.

### 2.2 High-level scope

**In scope** (Phase 8):
- A `Memory` component on `Agent`: bounded ring of episodic entries.
  Each entry references an `EventId` from the causal log, plus per-agent
  metadata not in the global log (emotional valence at the time of
  encoding, an encoding-tick timestamp for decay math, an
  importance/salience scalar).
- A `MemorySystem` (priority candidate **136**, immediately after
  `SocialDecaySystem` at 135 — see §2.5 open question 3): encodes new
  memories from this tick's causal events relevant to the agent;
  decays / forgets old or low-salience entries.
- A new `CausalEvent::MemoryRecalled` variant. Parent-chained to the
  recalled entry's original `EventId`, and consumed by
  `AgentDecisionSystem` as the parent for any `AgentDecision{MemoryReason}`.
- A new `DecisionReason::MemoryReason` variant (6th, after
  `SocialReason`). Mirrors the Phase 5/6/7 cascade-extension precedent.
- A bias hook in `AgentDecisionSystem`: when a memory exists that argues
  for or against the cascade's natural choice (e.g. a strongly negative
  memory of a prior `SocialInteractionCompleted` with a specific partner
  damps Social cascade preference for that partner), the cascade applies
  the bias and emits the `MemoryRecalled → AgentDecision{MemoryReason}`
  causal pair.
- A memory decay mechanism: simplest first — linear time decay scaled by
  reinforcement count. Aging-out entries below a salience floor are
  pruned (or compressed; see §2.5 open question 2).
- A chronicle harness (Phase 8-γ): one or two agents, an early event
  (e.g. a failed social interaction or a specific tile visit), a delay,
  a later decision provably biased by recall of that early event.
  ≥13 plan-locked assertions per Phase 5-γ / 6-γ / 7-γ chronicle
  precedent.

**Out of scope** (deferred to Section 10+ / Phase 9+):
- HP / damage / combat / death-by-agent (Section 8 §3 #1 — Phase 9
  candidate once Memory lands).
- Multi-building settlement coordination (Section 8 §3 #3).
- Speech / dialogue / language.
- Family / genealogy enrichment beyond the `familiarity` scalar +
  per-target memory entries.
- Semantic memory (concept categorisation, type-of-thing memory) — Phase 8
  ships episodic memory only.
- Long-term consolidation (sleep-driven episodic-to-semantic transfer).
- Memory visualisation (UI integration — Phase 8-δ optional, gated on V7
  visual-ambition reassessment).
- Behavior trees / utility AI / LLM rewrite (Section 8 §3 #4).

### 2.3 Sub-stage decomposition (TBD — planning-first dispatch)

Detailed decomposition lives in `.harness/plans/phase8.md` after a
planning-first dispatch (Phase 6 + 7 precedent). The expected shape
(subject to that planning round):

- **Phase 8-α**: `Memory` component (storage struct + entry struct + serde)
  + bounded ring or `Vec` cap policy + harness with ≥12 assertions on the
  *data substrate only*. Zero runtime system change. (Phase 5-α / 6-α / 7-α
  precedent.)
- **Phase 8-β**: `MemorySystem` (priority 136 candidate) + encoding logic
  (which causal events become memories for which agent) + decay loop +
  `CausalEvent::MemoryRecalled` variant + `DecisionReason::MemoryReason`
  variant + `AgentDecisionSystem` bias cascade extension + harness with
  ≥12 assertions. (Phase 5-β / 6-β / 7-β precedent.)
- **Phase 8-γ**: End-to-end chronicle harness, agent(s), early
  memory-anchoring event, delay, later bias-driven decision, ≥13
  assertions. Closure milestone. (Phase 5-γ / 6-γ / 7-γ precedent.)
- **Phase 8-δ** (optional, user mandate base): Visual milestone — agent
  memory indicator + CausalPanel `MemoryReason` / `MemoryRecalled` labels
  + locale keys. Out of scope until V7 visual ambition is reassessed.

### 2.4 Dependencies

- V7 Foundation Week 1-12 complete ✓ (`a0666b6c`).
- V7 Phase 7 (Multi-agent Social System) complete ✓ (`f1c12f9d`).
- V7 architecture base 정통 활용 (Phase 8 reads, does not replace):
  - `Agent { id: AgentId }` (Phase 5-α)
  - `Position` (Phase 4-α)
  - `AgentState` FSM (Phase 5-β/γ)
  - `AgentDecisionSystem` priority 125 (Phase 5-β/γ, extension target)
  - `CausalEvent` + `EventId` chain (Phase 3-α/β — **primary substrate**)
  - `SimResources::issue_event_id` (Phase 3-β)
  - `Social` / `RelationshipState` (Phase 7-α — referenced by negative
    SocialInteraction memories)
- **No prerequisite on BodyHealth / Combat / Multi-building Settlement /
  AI rewrite** — Phase 8 is additive over the existing agent set, like
  Phase 7 was.

### 2.5 Open questions (resolved at planning-first dispatch time)

1. **Memory entry struct fields**: minimum viable set vs richer
   per-target schema. Default candidate: `MemoryEntry { event_id: EventId,
   encoded_tick: u64, valence: f64, salience: f64,
   reinforcement_count: u32 }`. Resolve at `phase8.md` planning, citing
   what `AgentDecisionSystem` actually consumes.
2. **Decay mechanism**: pure linear time decay (`salience -= rate * dt`)
   vs combined linear + reinforcement-boost (`salience -= rate * dt;
   on recall, salience += boost`). Default: combined. Resolve with the
   chronicle's required tick budget — if Phase 8-γ needs >100 ticks of
   delay between encoding and recall, decay rate has to support that.
3. **`MemorySystem` priority**: 136 (immediately after `SocialDecaySystem`
   = 135) is the natural slot. Alternatives: 124 (just before
   `AgentDecisionSystem` 125, so memory state is fresh when the cascade
   runs), or 126/127 (just after decision, so memories of *this tick's*
   decision are encoded in time). Default 136 (after needs decay,
   before viz). Resolve at planning with an explicit tick-order trace.
4. **`AgentDecisionSystem` bias semantics**: binary gate (memory
   eligibility check) vs weighted scoring (memory shifts cascade
   weights). Default: weighted scoring on the existing cascade order,
   not a separate cascade pass. `MemoryReason` is emitted when the
   weight shift flips the cascade's natural choice. Resolve at
   planning.
5. **Memory capacity policy**: hard per-agent cap with FIFO eviction vs
   salience-floor-only with unbounded growth vs hybrid (cap + evict by
   salience). Default: hybrid, cap = 32 entries (round-robin like
   Phase 3-β's per-tile 32-event ring buffer — symmetry with existing
   substrate). Resolve at planning.
6. **`CausalEvent::MemoryRecalled` signature**: minimum useful fields.
   Default candidate: `MemoryRecalled { id: EventId, parent:
   Option<EventId>, agent: AgentId, recalled_event: EventId,
   triggered_by: MemoryRecallTrigger }`. The `triggered_by` enum lives
   in this phase or expands later (Phase 9-β with combat-triggered
   recall, etc.). Resolve at planning.

These six questions are the natural blockers for Phase 8-α's planning-first
dispatch.

---

## 3. Section 10+ (deferred)

`Section 10` will anchor the next Phase-9-and-beyond direction. Candidates,
ranked by Memory-compositional substrate-fit (most natural sequel first,
given Phase 8 lands first):

1. **Combat System** — Section 8 §3 #1. With Memory, combat gains
   per-agent rivalry / grudge / revenge mechanics; one-off skirmishes
   become recurring conflicts. Still requires BodyHealth (A-11 leftover,
   ~Phase 4-δ scope) as a sub-stage prereq.
2. **Multi-building Settlement** — Section 8 §3 #3. Multi-site
   coordination + settlement-level decisions become memory-rich
   (settlement-of-origin, where my family lived, where the famine
   happened). Population-growth lifecycle integrates with episodic memory
   inheritance questions.
3. **Advanced AI (BT / Utility / LLM)** — Section 8 §3 #4. The AI
   rewrite consumes Memory as its bias source. Choosing this last means
   it integrates over a substrate that already exists, rather than
   inventing one as part of the rewrite.

Section 10 decomposition is **NOT** specified here. It is the
user-mandated direction chosen after Phase 8 closure, per the established
governance pattern. The Combat → Memory composition is the most likely
candidate but is **not pre-committed**.

---

## 4. Single source of truth

| Concern | Document |
|---------|----------|
| Phase 1-6 progress + V7 Foundation closure declaration | `.harness/audit/v7_progress.md` |
| Phase 7 anchor (Multi-agent Social System) | `.harness/audit/section_8_plus_design.md` |
| Phase 7 sub-stage decomposition | `.harness/plans/phase7.md` (planning-first dispatch — landed) |
| **Phase 8 anchor (Memory System, this document)** | **`.harness/audit/section_9_plus_design.md`** |
| Phase 8 sub-stage decomposition | `.harness/plans/phase8.md` (planning-first dispatch, TBD — next dispatch) |
| Phase 6 sub-stage decomposition | `.harness/plans/phase6.md` (existing) |
| Infrastructure governance audit (Issues 12+13+14, dispatch patterns A–F) | `.harness/audit/v7_progress.md` (Infrastructure Governance Audit section + Issue 14 closure section) |
| Behavioural + architectural guidelines | `CLAUDE.md` |
| V7 roadmap canonical comment block | `rust/crates/sim-core/src/lib.rs:5-11` |

Section 9 supersedes Section 8 only for the Phase 8 anchor. Section 8's
Phase 7 anchor remains the authoritative record for Phase 7's scope; Phase
7's actual delivery is recorded in `v7_progress.md`'s closure declaration
and in the `f1c12f9d` commit body.

---

## 5. Honest reservations

1. **Choice is a product judgement, not a derivation.** Section 8 §3
   ranked Combat #1 and Memory #2 by *substrate-fit*. The user-mandated
   choice of Memory over Combat is defensible on lowest-prereq-cost and
   axiom-#1-extension grounds (§2.1), but it is a vision choice, not
   forced by the architecture. If the user later prioritises a visceral
   combat loop (Songs of Syx / RimWorld feedback signal) over emotional
   depth, the recommendation should be overridden without reservation.
2. **Phase 8-δ visual milestone is left unscoped.** Same posture as
   Phase 7-δ. V7 chose to prove simulation loops without heavy visual
   debt; that decision should be reassessed at the `phase8.md`
   planning-first dispatch, not retroactively.
3. **Memory schema is intentionally minimal.** Episodic memory only.
   Semantic memory (categorical, "this *type* of thing happened"),
   long-term consolidation, autobiographical narrative — all are
   valuable but each is a Phase 9+ scope unit. Phase 8 ships the
   minimum that closes the bias loop and produces a chronicle.
4. **The bias-cascade semantics (open question 4) is opinionated.**
   Weighted scoring vs binary gate vs separate cascade pass are all
   defensible. The default in §2.5 is "weighted scoring on the existing
   cascade" because it preserves the Phase 5/6/7 cascade-ordering
   precedent. A redesign of the cascade ordering itself is out of
   scope here and belongs to Section 8 §3 #4 (Advanced AI) when that
   lands.
5. **Infrastructure pattern monitoring is mandatory.** Issue 14 closed
   Pattern E; Patterns A (API 500), B (Codex timeout), C (Generator
   silent death), and D (attempt-3 -10 cap, intentional) all remain
   live signals across Phase 8's dispatches. The `v7_progress.md`
   Infrastructure Governance Audit section is the authoritative tracker
   for these. Any Phase 8 dispatch that hits A/B/C should be reported
   verbatim, not normalised into the score. Pattern D remains
   intentional governance and should not be "fixed."
6. **V7 Master Direction document remains absent.** Fourth escalation
   disclosure (§0). This document is not a substitute; it is a
   governance audit artefact for the single Phase 8 anchor decision.
   Phases 9+ will need either the same per-section governance artefact
   or, eventually, an actual top-level direction document. That choice
   is itself a Section 10+ governance question, not a Phase 8 question.
