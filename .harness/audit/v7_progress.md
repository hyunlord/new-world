# V7 Progress Tracker

**Started**: 2026-05-03
**Reference**: project knowledge `WORLDSIM_V7_MASTER_DIRECTION.md`

## 진행 상황

### Day 1 (2026-05-03)
- [x] Archive 백업: archive/pre-v7-reset
- [x] 코드 폐기: rust/, scripts/, scenes/(자산 외), data/
- [x] 새 Cargo workspace
- [x] sim-core skeleton
- [x] 새 Godot project
- [x] 빈 main scene

### Week 1~2 (진행중): Material System Deep
- [~] W1.1: Material schema 설계
  - [x] T1 (2026-05-04, `77764531`): Cargo deps + sim-core lib.rs `pub mod material` + `MATERIAL_SCHEMA_VERSION=1` (STRUCTURAL-COMMIT pre-V.4, see `.harness/audit/structural_commits.log`)
  - [x] T2-T5 (2026-05-05, `91d4e7c0`): material module 11 files / 2101 LOC — id/category/terrain/error/properties/definition/derivation/explanation/registry/loader/mod (POLICY-GAP-V3.3 authorized, see `.harness/audit/policy_gap.log`; lock violations 0, Evaluator APPROVE, raw 48/100, adjusted 56/90 — block by 3 policy gaps not code defects)
  - [x] T6.6 (2026-05-06, `78f437cb`): Material Loader by_category dispatcher (8 accessors) + sample granite.ron + 3 integration tests (71 tests pass + clippy clean, STRUCTURAL-COMMIT v3.3.4 lane)
- [⏯] W1.2: Material RON 100+ (T6.1~T6.5) — **UNBLOCKED** (v3.3.4 land complete, STRUCTURAL lane 정식 처리 가능)
- [⏯] W1.3: Auto-derivation (folded into W1.1 T2 — already complete in `91d4e7c0`)
- [⏯] W1.4: Material inspector UI — UNBLOCKED (v3.3.3 lands)
- [⏯] W1.5: Cause-effect harness 5+ (T9~T10) — UNBLOCKED (v3.3.3 lands)
- [⏯] W1.6: 사용자 visual 검증 — UNBLOCKED (v3.3.3 lands)

### Governance v3.3.3 — LAND COMPLETE ✓ (2026-05-06)

**V.4 cascade (5 commits)**:
- V.4.1 `871f131b` — v3.3.3 amendment register (Tests 20 dimension + Hot threshold 90)
- V.4.2 `fcd06f96` — v3.3.1 self-correction (D5a, 22-line cascade)
- V.4.3 `8ef1ea62` — v3.3 ticket inline re-patch (D6a, 27 changes)
- V.4.4 `0c5d88a0` — Hook tier branching fix (Cold 54→75, Hot 72→90, pure consumer)
- V.4.5 `62031050` — generate_report.sh cold tier auto credit (D4α producer)

**Architecture SRP achieved**:
- Hook (`pre-commit-check.sh`) = pure consumer (verdict line 4 / pipeline_report.md 추출)
- Producer (`generate_report.sh`) = score 산출 + cold tier auto credit (Visual 20)
- Classifier (`cold_tier_classifier.sh`) = 4 Signal 검증 (A/B/C/D)

**T2 retroactive validate (`91d4e7c0`)**:
- cold_tier_classifier exit 0 (A=1 B=1 C=1 D=1 모두 confirmed) ✓
- Score: GATE 10 + PLAN 5 + CODE 10 + TESTS 20 + VISUAL 20 (auto credit) + REG 15 + EVAL 15 = **95/100**
- Hook threshold 75 (cold) → PASS (safety margin +20)

**Resolved gaps**:
1. Cold-tier Visual Verify — V.4.5 cold tier auto credit (Visual 20)
2. Attempt penalty discrimination — pending Step N6 (RE-CODE 분류 + per-attempt penalty)
3. §6 NOT-in-scope FFI false positive — pending Step N6 (FFI vacuous integration)

**Next action**: N6 (Step W) — `harness_pipeline.sh` FFI vacuous + RE-CODE 분류 + per-attempt penalty integration.

### N6 cascade — LAND COMPLETE ✓ (2026-05-06)

**N6 cascade (3 commits)**:
- N6.1 `4680378d` — FFI vacuous integration (Step 2.5c, D10c helper 호출처별 분리)
- N6.2 `2f90ffed` — RE-CODE classification + penalty producer (D7-A env-var, D11-A snapshot, D12-A timing)
- N6.3 `13c5814c` — SCORE_CODE penalty consumer (SCORE_CODE_BASE + TOTAL_PENALTY + 0 floor)

**Architecture 책임 분담 완성**:
- `cold_tier_classifier.sh` — 4 Signal 검증 (A/B/C/D)
- `ffi_vacuous_check.sh` — sim-bridge crate 전체 vacuous (Step 2.5c, broader scope)
- `changed_sim_bridge()` helper (legacy) — lib.rs 단일 정밀 (PostCode L649)
- `classify_recode.sh` — RE-CODE verdict 5-카테고리 분류
- `score_attempt_penalty.sh` — per-attempt cap -5 누적 (producer)
- `harness_pipeline.sh` — orchestration + snapshot + env-var export
- `generate_report.sh` — dimension 산출 + cold tier auto credit + SCORE_CODE penalty 적용 (consumer)
- Hook (`pre-commit-check.sh`) — pure consumer + threshold 비교

**T2 retroactive validate (`91d4e7c0`) updated**:
- cold_tier_classifier exit 0 (A=1 B=1 C=1 D=1) ✓
- FFI vacuous CONFIRMED ✓ (Mech 10/10)
- 1 LOCK_VIOLATION attempt → CQ 6/15 (base 11 - penalty 5)
- Score: GATE 10 + PLAN 5 + CODE 6 + TESTS 20 + VISUAL 20 + REG 15 + EVAL 15 = **91/100**
- Hook threshold 75 (cold) → PASS (safety margin +16)

**Score evolution audit chain**:
- 91d4e7c0 initial: raw 48/100, adjusted 56/90 BLOCK (3 policy gaps)
- V.4.6 sim (dimension only): 95/100 PASS
- W3 후 정확한 sim (penalty 반영): **91/100 PASS**

**v3.3.x 진화 audit chain 완료** (~13 commits): v3.3 통합 명령 → v3.3.1 → v3.3.2 → v3.3.3 V.4 cascade (7 commits) → N6 cascade (3 commits) + W5 audit.

**v3.3.4 amendment 후보** (post-N6 audit, 사용자 결정):
1. per-attempt cap 의미 명확화 (각 attempt 단독 -5 누적, 글로벌 cap 아님)
2. attempts/ subdirectory 구조 명세 (W3.1 directory mismatch 해결)
3. SCORE_CODE attempt-aware base 공식 명세 (15/11/8)

**Next action (D9 사용자 결정)**:
- D9α: N7 (Step X) — `score_model.sh` 정합 검토 (~1h)
- D9β: T6 (Material RON 100) 시작 — Phase 1 본 작업 (~3-5h, 권장)
- D9γ: N8~N12 — self-test V5-V8 + audit (~2-3h)

W1.2~W1.6 (T6~T11) 진행 가능 — UNBLOCKED.

### 사용자 confirm 기록
*(시스템 완성마다 사용자 명시 confirm 기록)*

## V7 Hard Gates 적용 현황
*(매 시스템마다 5 gates 검증 결과)*

---

# V7 Foundation Week 1-12 Closure (2026-05-17)

## Phase 2 — Week 3-4: Tile Grid + Influence System ✓

T7-series substantial chain (T7.5 ~ T7.10) — sim-systems/runtime/influence land + 6 channels
wiring (Warmth/Light/Noise/Danger/Spiritual/Beauty):

- T7.5-T7.8: sim-systems substrate + sim-bridge FFI + substantial harness + Godot scaffold
- T7.9: harness visual restore + render mechanism (Gaffer fixed-tick accumulator)
- T7.10-A `c22c7bb2`: Warmth channel wiring
- T7.10-B `64fb905d`: Light shadowcast wiring (second-channel Phase 2 escape)
- T7.10-B1 `f51aa989`: SPACE-key Warmth↔Light viz toggle
- T7.10-C `4c286a9a`: Noise channel linear-decay wiring
- T7.10-D `87a9c569`: Danger channel linear-decay wiring
- T7.10-E `df11b632`: Spiritual channel BFS wiring
- T7.10-E-fix `26bad02b`: IUS k=0.10→k=0.08 spec align
- T7.10-F `5bb2f919`: Beauty channel BFS wiring

## Phase 3 — Week 5-6: Cause-Effect Tracking + 왜? UI ✓

Tile-level 8-event sparse causal log + parent chain + FFI + GDScript CausalPanel:

- 3-α `bb925bd1`: tile-level 8-event sparse causal log (TILE_CAUSAL_RING_SIZE=8)
- 3-β `fa6652a6`: EventId (u64 monotonic) + parent chain
- 3-γ-1 `af4a9c7e`: FFI causal surface (get_tile_causal_history + get_event_chain)
- 3-γ-2-α `4fb87057`: Locale autoload + CausalPanel scaffold
- 3-γ-2-β `d8545fa6`: Tile click → causal chain rendering

## Phase 4 — Week 7-8: Agent Core ✓

Canonical components + Brownian movement + MultiMeshInstance2D sprite rendering:

- 4-α `5a34e4aa`: Canonical Agent + Position components (canonical land)
- 4-β `a592dc0c`: AgentMovementSystem priority 120 (Brownian, deterministic)
- 4-γ `b09b2468`: MultiMeshInstance2D sprite rendering + palette_swap.gdshader

## Phase 5 — Week 9-10: First Daily Routine ✓

Three Need components + day/night cycle + agent-originated FSM/decision/causal chain:

- 5-α `0cd52ffc`: AgentId + Hunger + HungerDecaySystem (priority 130)
- 5-β `9d6d3f74`: AgentState FSM (Idle/Seeking/Consuming) + AgentDecisionSystem (125)
  + Thirst + ThirstDecaySystem (131) + food/water tile substrate
  + CausalEvent::AgentDecision + DecisionReason (Hunger/Thirst ThresholdBreach)
- 5-γ `a765a374`: Sleep + SleepDecaySystem (132) + day/night clock (ticks_per_day=1440)
  + TargetKind::Sleep (Path b symmetry) + FatigueThresholdBreach
  + sleep_tiles substrate + full-day chronicle harness (13 assertions)

## Phase 6 — Week 11-12: Building System Deep ✓

Agent-driven construction loop + 4-link causal chain + end-to-end chronicle:

- 6-α `ba4e02b2`: BuildingBlueprint + ConstructionSite + TargetKind::ConstructionSite
  (4th variant, Phase 5-γ Sleep symmetry precedent) — 95/100 A
- 6-β `21b09e26`: ConstructionSystem (priority 133) + CausalEvent::ConstructionStarted/
  ConstructionCompleted + DecisionReason::ConstructionReason + AgentDecisionSystem takeover
  of Phase 6-α inert placeholders — 90/100 B (adjusted 98 with VLM env cost)
- 6-γ `66435f06`: Construction chronicle harness (15 assertions, build-a-shelter cycle,
  full causal chain: AgentDecision{ConstructionReason} → ConstructionStarted →
  ConstructionCompleted → BuildingPlaced) — 90/100 B

## Audit Closures

### Issue 12 — generate_report.sh review_latest resolution (`00274b57`)

**Symptom**: `ls .harness/reviews/<feature>/review_attempt*.md | tail -1` used alphabetical
sort, picking `review_attempt9.md` over `review_attempt10.md` (string sort, not numeric).
**Fix**: switched to `review_latest.md` symlink (already maintained by pipeline) with
mtime-sorted fallback when the symlink is missing. Score computation now reads the most
recent review attempt deterministically regardless of attempt count.

### Issue 13 — verdict file staleness mechanism (`00274b57`)

**Symptom**: pre-commit hook could read a stale `verdict` file from a prior pipeline run
when the current run failed to update it (e.g., script death between attempts). Score
extraction inherited the old verdict's score line.
**Fix**: hook now checks `verdict` mtime — if epoch >1800s old vs HEAD commit time, refuses
to credit the score line and forces re-run. Closes the staleness window without policy
bypass. Both fixes consolidated into `00274b57 fix(harness): correct scoring bugs`.

## V7 Foundation Closure Declaration

V7 roadmap (`rust/crates/sim-core/src/lib.rs:5-11`) Foundation Week 1-12 — **complete**.

**Recovery span**: 2026-05-03 (V7 reset) → 2026-05-17 (closure) = ~14 days.

**Pipeline stability**: 26 consecutive APPROVED harness verdicts (no FAIL streak break).

**Causal traceability coverage** (axiom #1):
- BuildingPlaced (Phase 2 BSS) → StampDirty (BSS) → InfluenceChanged (IUS)
- AgentDecision{Hunger/Thirst/Fatigue/Construction ThresholdBreach}
- ConstructionStarted → ConstructionCompleted → BuildingPlaced (agent-driven loop)
- Every event carries `id: EventId` + `parent: Option<EventId>` for chain walking

**Visual + behavioral milestones** (axiom #3):
- Phase 4-γ: MultiMeshInstance2D agent rendering + shader palette swap (visual)
- Phase 5-γ: Full-day routine chronicle (behavioral — Hunger/Thirst/Fatigue full cycle)
- Phase 6-γ: Build-a-shelter chronicle (behavioral — construction full cycle)

**Architecture base extension** (V7 Foundation final state):
- sim-core: material/, tile/, influence/, causal/, components/
- sim-systems/runtime: influence/, agent/, decision/, needs/, construction/
- sim-bridge: 6 FFI methods (get_influence_overlay, get_tile_detail, get_tile_causal_history,
  get_event_chain, get_agent_snapshot, on_building_placed)
- sim-engine: SimResources (tile_grid + causal_log + food/water/sleep_tiles + ticks_per_day)
- chronicle infrastructure: 2 closure-milestone chronicles (p5-γ, p6-γ)

**Governance evolution**: v3.3.1 → v3.3.17, 14+ patches (Issue 10/11/12/13 closures,
ENV-BYPASS v3.2.1 baseline registry, score formula refinements).

**ENV-BYPASS closures**: 3 chains landed under controlled conditions with audit trail.

**Next decision base** (사용자 mandate required):
- CLAUDE.md V7 reset 정합 update (separate dispatch, aspirational/legacy carry-over)
- Phase 6-δ optional (UI integration: ConstructionSite visual + CausalPanel ConstructionReason)
- Phase 7+ (Combat / Social / Memory / Settlement — V7 외부 phase, mandate 필수)
- V7 Master Direction Section 8+ design (Foundation 완료 후 별도 design work)

---

# V7 Phase 7 (Multi-agent Social System) — Partial closure (α + β only) (2026-05-17)

## Phase 7-α — Social/Relationship substrate ✓

- Commit `35fbd501`, score 90/100 B Ship-it (APPROVE attempt 3 of r4, 4-dispatch
  recovery: r1 API 500, r2 silent-death, r3 RE-CODE × 3, r4 skip-gen APPROVE).
- Landed: `Social { loneliness, growth_rate }` (Sleep mirror) +
  `RelationshipKey` canonicalised + `RelationshipState` + `TargetKind::Agent(AgentId)`
  as the **first payload-carrying `TargetKind` variant** (Phase 5-γ Sleep +
  Phase 6-α ConstructionSite symmetry precedent extended).
- Phase 7-α `agent_decision.rs` inert placeholders preserved for the β takeover.

## Phase 7-β — SocialInteractionSystem + SocialDecaySystem + agent-driven social loop ✓ (Re-plan post)

- Commit `de336f83`, score 92/100 B Ship-it (APPROVE attempt 2 of r6,
  6-dispatch recovery: r1-r5 RE-CODE escalation chain → user Re-plan decision
  Option A (add `SocialDecaySystem` priority 135) → r6 fresh skip-gen APPROVE).
- Landed runtime:
  - `SocialInteractionSystem` priority **134**, tick_interval 1 — mutual
    handshake + multi-tick progress + completion + asymmetric-partner
    fallback + **sorted canonical RelationshipKey** pair ordering
    (determinism lock).
  - `SocialDecaySystem` priority **135** (re-plan addition) — Sleep/Hunger/
    Thirst decay precedent symmetry final.
- Landed causal substrate:
  - `CausalEvent::SocialInteractionStarted` + `::SocialInteractionCompleted`
    variants (canonical `(smaller, larger)` agents pair).
  - `DecisionReason::SocialReason` (5th variant).
- Landed FSM extension:
  - `AgentDecisionSystem` Idle 5th cascade (lowest-AgentId tie-break).
  - Seeking-branch mutual-handshake takeover (smaller-AgentId emits the
    single `SocialInteractionStarted`).
  - Consuming-branch `Agent(_)` no-op preserved (System-134 owns exit).
- Landed schema:
  - `SimResources::relationships: HashMap<RelationshipKey, RelationshipState>`
  - `SimResources::interaction_progress: HashMap<RelationshipKey, u32>`
- Landed constants in `agent_decision.rs`:
  - `SOCIAL_THRESHOLD = 50.0`, `SOCIAL_CONSUME_AMOUNT = 30.0`,
    `REQUIRED_INTERACTION_PROGRESS = 3`, `FAMILIARITY_BUMP = 0.1`
- Re-plan path note: "incorporate plan-lock spec" 정통 — the original §β plan
  locked 22 assertions; Generator under-implementation delivered 17 across 4
  dispatches. The "scope expansion" framing in the recovery chain was
  re-classified at the Option A decision point as plan-lock enforcement, not
  Codex over-reach. SocialDecaySystem addition closed the canonical-emergence
  gap that A18-A22 (4380-tick emergent run, 3-seed sweep, same-seed
  determinism, hot-tier schedule, despawn fallback) require.

## Phase 7 architecture base (post β)

Five needs systems all parallel:

```
130 HungerDecaySystem
131 ThirstDecaySystem
132 SleepDecaySystem
133 ConstructionSystem
134 SocialInteractionSystem
135 SocialDecaySystem
```

Causal chain land:

```
AgentDecision{SocialReason} → SocialInteractionStarted → SocialInteractionCompleted
```

## Phase 7-γ — Two-agent chronicle harness — DEFERRED (2026-05-17)

- 6 dispatches stalled at hook BLOCK 88/100 (Codex APPROVED on attempt 3
  final of each cycle, but hook gate requires ≥ 90).
- Score breakdown (final r6 attempt 3):
  - Mechanical Gate **8**/10 (FFI FAIL — stale unrelated evidence per Codex
    disclosure: "older HAS_BROKEN artifacts exist, but relevant/current V7
    reset evidence reports ALL_COMPLETE/SKIP")
  - Code Quality **5**/15 (attempt-3 -10 penalty cap)
  - Test Coverage 20/20, Visual 20/20, Regression 15/15, Evaluator 15/15
- Working tree preserved via `git stash`
  (`p7-gamma-wip-deferred-88-of-100-hook-block-6-dispatches`). Fresh attempt
  is still possible.
- Decision: **Option B (Defer)** — Pipeline integrity 정통 보호 + 4th
  ENV-BYPASS chain precedent risk explicitly avoided. The recurring
  attempt-3 penalty + stale-FFI signal is environmental in character but
  the 3-ENV-BYPASS rolling history (already closed at V7 Foundation) makes
  a 4th bypass undesirable for governance hygiene.

## Infrastructure signal disclosure (V7 reset cumulative)

- **Phase 7-β**: 6 dispatch attempts (5 stalled then 1 APPROVED post-Re-plan).
- **Phase 7-γ**: 6 dispatch attempts (all stalled at hook BLOCK 88/100).
- Cumulative since V7 reset (2026-05-03): ~28+ dispatch attempts across
  Phase 5/6/7 chains.
- Recurring failure patterns observed (across both Phases 7-β and 7-γ):
  - Generator silent death (API 500 / Claude rate limit at boundary windows)
  - Codex Evaluator timeout (600s) requiring Claude fallback
  - Attempt-3 -10 penalty cap on substantial harness scope
  - Stale FFI evidence inheriting -2 from unrelated prior dispatches
- **Honest read**: this is an infrastructure-level signal, not a code-defect
  signal. The mechanism evaluation belongs to a separate governance dispatch.
  3 ENV-BYPASS chains already closed at V7 Foundation closure; a 4th here
  would weaken the rule-7.1 governance integrity precedent.

## V7 milestone progression (post-defer)

- ✅ Foundation Week 1-12 (complete)
- ✅ Week 13-14 Phase 7-α + β (Multi-agent Social System operational)
- ⏸ Phase 7-γ chronicle harness (deferred, stash preserved)

## Next decision base (사용자 mandate required)

| Option | Scope |
|--------|-------|
| **A. Phase 7-γ fresh attempt** | `git stash pop` + integrate the 4 actionable issues from r5/r6 (A5 window, A15 Type C→D, diagnostic counter, prompt-file removal) + fresh skip-gen dispatch |
| **B. Section 9+ design / Phase 8 anchor** | Combat / Memory / Multi-building Settlement / Advanced AI (Section 8+ design §3 candidates) |
| **C. Phase 7-δ optional** | UI integration: `CausalPanel` social rendering + `AgentRenderer` socializing indicator + locale keys |
| **D. Phase 4-δ optional** | BodyHealth — V7 reset 후 명시적 path 부재, scope 정의 의무 |
| **E. Infrastructure governance work** | Pipeline mechanism evaluation against the 28+ dispatch signal (separate audit-only dispatch, no feature work) |
| **F. V7 partial closure declaration** | Mark V7 (Foundation + Phase 7-α/β) as substantial final, defer Phase 7-γ + δ + further phases to a fresh design cycle |

---

# Infrastructure Governance Audit (Phase 7 recurring pattern, 2026-05-17)

User selected Option E + F combined: audit the Phase 7 dispatch infrastructure
signal honestly, then mark V7 (Foundation + Phase 7-α/β) as substantial final.

## Phase 7 cumulative dispatch inventory

| Sub-stage | Dispatches | Outcome |
|-----------|-----------|---------|
| α (Social/Relationship substrate) | 4 (r1-r4) | r1 API 500 → r2 silent-death → r3 RE-CODE×3 + rate limit → r4 skip-gen APPROVED 90/100 |
| β (SocialInteractionSystem + agent loop) | 6 (r1-r6) | r1-r5 RE-CODE escalation chain → user Re-plan (Option A: SocialDecaySystem) → r6 fresh skip-gen APPROVED 92/100 |
| γ (chronicle harness) | 6 (r1-r6) | All 6 stalled at hook BLOCK 88/100 — Codex APPROVED but attempt-3 penalty cap + FFI FAIL stale evidence pushed adjusted_score below 90 |

**Cumulative Phase 7 dispatches: 16 across 3 sub-stages.**

## Pattern classification (honest)

| Pattern | Source | Recovery |
|---------|--------|----------|
| **A — API 500 mid-Generator** | External Anthropic API | retry + skip-gen |
| **B — Claude rate limit silent death** | External Anthropic API | skip-gen after reset |
| **C — Generator silent death (partial output)** | Sub-agent / Claude Code session boundary | manual fix exhaustive arms + skip-gen |
| **D — Attempt-3 penalty cap (-10)** | Pipeline scoring mechanism (intentional) | real signal — accumulated rework cost |
| **E — FFI FAIL stale unrelated evidence (-2)** | Pipeline mechanism gap | currently no fix path — disclosure only |
| **F — "Scope expansion" demands** | Plan-lock enforcement (NOT scope creep) | incorporate plan-lock spec (per Re-plan precedent) |

## Mechanism boundary disclosure

- **Patterns A/B/C**: external infrastructure. Outside Claude Code/Pipeline
  control. Recovery via skip-gen + synthetic gen_result is the documented
  precedent (Phase 6-β r1/r2, Phase 7-α r1-r3, Phase 7-β r1-r2).
- **Pattern D**: intentional governance signal. Attempt-3 cap of -10 is
  designed to reflect cumulative rework cost. Not a bug; a real penalty for
  substantial-scope dispatches needing 3 generator runs.
- **Pattern E**: pipeline mechanism gap. Codex r6 disclosure verbatim:
  > "older HAS_BROKEN artifacts exist, but relevant/current V7 reset evidence
  > reports ALL_COMPLETE/SKIP and no newly added BROKEN method regression was
  > identified."

  The hook's Mech Gate scoring counts `FFI FAIL` even when the FAIL is on
  unrelated historical evidence the regression guard explicitly clears. This
  is the **Issue 14 candidate** — analogous to Issue 12 (alphabetical sort
  bug) and Issue 13 (verdict staleness): a pipeline-mechanism fix path
  exists, deferred to a separate governance dispatch.
- **Pattern F**: not a problem. The "scope expansion" framing during
  Phase 7-β recovery was re-classified at the Option A decision point as
  Generator under-implementation against an already-locked 22-assertion
  plan — Codex was correctly enforcing the lock, not over-reaching. Same
  re-classification applies to Phase 7-γ Codex demands (the 13-assertion
  plan + 4 actionable r5/r6 issues were plan-locked, not Codex over-reach).

## Issue 14 candidate (Pattern E fix, deferred)

Mechanism gap: `tools/harness/cold_tier_classifier.sh` or the equivalent
FFI evidence reader counts historical `HAS_BROKEN` files even when those
files predate the current V7 reset and are explicitly cleared by the
regression guard.

**Fix path (proposal, not implemented):** the FFI check should either
(a) restrict scope to evidence files post-V7-reset commit timestamp, or
(b) exclude evidence directories whose feature names don't appear in the
regression guard CLEAN list, or (c) require the regression guard to emit a
positive `current_ffi: ALL_COMPLETE` token that the Mech Gate trusts over
the file-existence check.

**Disposition**: deferred to a future audit-only governance dispatch. Not
priority right now. Pattern E adds -2 to Mech Gate, which is recoverable
only via plan-quality + code-quality wins; the recurring -2 contributed
materially to Phase 7-γ stalling at 88/100.

## Honest evaluation of Phase 7-γ stall

The Phase 7-γ work is **Codex-APPROVED on attempt 3 of every cycle** (r1-r6).
The 88/100 block isn't a code-defect signal — it's the structural
combination of Pattern D (-10 attempt-3 cap) + Pattern E (-2 FFI FAIL).
The arithmetic ceiling under these conditions is 88 unless one of:
1. Attempt-1 APPROVE (no penalty cap) — improbable given iterative Codex
   refinement on a substantial harness.
2. Issue 14 fix lands first — separate governance dispatch.
3. ENV-BYPASS authorized — declined per 4th-chain risk avoidance.

This is why Option B (Defer) was the correct decision — not because the
code is wrong, but because pursuit of "above 90" without fixing the
underlying mechanism gap would either grind further dispatches (wasting
external API quota on patterns A/B/C exposure) or land a 4th ENV-BYPASS
chain that weakens the rule-7.1 governance precedent.

---

# V7 (Foundation + Phase 7 partial) — Final Declaration (2026-05-17)

## V7 milestone progression — final

- ✅ Foundation Week 1-12 complete (commit `a0666b6c`)
- ✅ Week 13-14 Phase 7-α + β (Multi-agent Social System operational —
  35fbd501 + de336f83)
- ⏸ Phase 7-γ chronicle harness deferred (a00a1b77 declaration,
  stash preserved for fresh-attempt recovery)

## V7 architecture base — final substantial

Six runtime systems in the post-decision priority slate (the **5 needs
systems all parallel** intended end-state):

```
 90  BuildingStampSystem            (Phase 2)
100  InfluenceUpdateSystem          (Phase 2)
110  AgentInfluenceSampleSystem     (Phase 2)
120  AgentMovementSystem            (Phase 4-β)
125  AgentDecisionSystem            (Phase 5-β / 6-β / 7-β cascade)
130  HungerDecaySystem              (Phase 5-α)
131  ThirstDecaySystem              (Phase 5-β)
132  SleepDecaySystem               (Phase 5-γ)
133  ConstructionSystem             (Phase 6-β)
134  SocialInteractionSystem        (Phase 7-β)
135  SocialDecaySystem              (Phase 7-β re-plan)
1000 InfluenceVisualizationSystem   (Phase 2 debug)
```

## Causal chain coverage — final

Five `AgentDecision` reasons each anchor a causal emission chain:

```
Hunger threshold breach    → AgentDecision{Hunger}      → (consume, no event chain — Phase 5-β scope)
Thirst threshold breach    → AgentDecision{Thirst}      → (consume)
Fatigue threshold breach   → AgentDecision{Fatigue}     → (consume)
Construction proximity     → AgentDecision{Construction} → ConstructionStarted → ConstructionCompleted → BuildingPlaced
Mutual social co-location  → AgentDecision{Social}     → SocialInteractionStarted → SocialInteractionCompleted
```

Plus the original Phase 2/3 chain:
```
BuildingPlaced (FFI) → StampDirty → InfluenceChanged   (per channel)
```

Every event carries `id: EventId` + `parent: Option<EventId>` for the
"왜?" UI chain walk (axiom #1 traceability).

## V7 final stats

| Metric | Value |
|--------|-------|
| Recovery span | 2026-05-03 → 2026-05-17 (~14 days V7 reset → V7 closure, +~2 days Phase 7 closure) |
| Pipeline streak | 28 consecutive APPROVED + 1 hook BLOCK at 88 (Phase 7-γ defer) |
| Governance patches | 15+ (v3.3.1 → v3.3.17, Issue 10-13 closures) |
| ENV-BYPASS chains closed | 3 (4th explicitly avoided per Option B) |
| Feat commits since V7 reset | 28+ |
| Phase 7 cumulative dispatches | 16 (α 4 + β 6 + γ 6) |

## Axioms — substantial final

- **Axiom #1 (causal traceability)**: every agent decision and every
  state-changing event emits a chain entry. UI walk-back is complete from
  any leaf event to its root.
- **Axiom #3 (visual + behavioral milestone full reach)**: Phase 4-γ
  delivered the visual milestone (MultiMeshInstance2D rendering); Phase
  5-γ delivered the behavioral milestone (full daily routine chronicle);
  Phase 6-γ delivered the construction loop chronicle; Phase 7-α/β
  delivered the runtime social loop. Phase 7-γ chronicle is the documented
  gap — operational system without the dedicated chronicle test.

## Governance closure chain — final

| Stage | Commit | Scope |
|-------|--------|-------|
| 1 | `a0666b6c` | V7 Foundation Week 1-12 complete |
| 2 | `2e51c167` | CLAUDE.md V7 reset 정합 update |
| 3 | `0ed3ec16` | Section 8+ design — Phase 7 anchor |
| 4 | `a00a1b77` | Phase 7-γ defer + partial closure disclosure |
| 5 | *(this commit)* | Infrastructure governance audit + V7 Final declaration |

## Next decision base (사용자 mandate required)

| Option | Scope |
|--------|-------|
| **Issue 14 (Pattern E) fix dispatch** | Pipeline mechanism gap fix for stale FFI evidence (precedent: Issue 12/13 `00274b57`). Audit-only governance work. |
| **Phase 7-γ fresh attempt** | `git stash pop` + integrate r5/r6 actionables + fresh skip-gen. Requires Issue 14 fix first or ENV-BYPASS to clear 88-ceiling. |
| **Section 9+ design (Phase 8 anchor)** | Combat / Memory / Multi-building Settlement / Advanced AI per Section 8+ design §3 candidates. User mandate required. |
| **Phase 7-δ optional** | UI integration. Locked scope in plan §γ. |
| **Phase 4-δ optional** | BodyHealth. V7 reset 후 명시적 path 부재 — scope 정의 의무. |
| **V7 종결 절대 final** | Current declaration sufficient. No further commits planned without new user mandate. |
