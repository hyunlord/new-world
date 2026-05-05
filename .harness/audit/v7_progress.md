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
  - [x] T1 (2026-05-04, `77764531`): Cargo deps + sim-core lib.rs `pub mod material` + `MATERIAL_SCHEMA_VERSION=1` (STRUCTURAL-COMMIT, see `.harness/audit/structural_commits.log`)
  - [x] T2-T5 (2026-05-05, `91d4e7c0`): material module 11 files / 2101 LOC — id/category/terrain/error/properties/definition/derivation/explanation/registry/loader/mod (POLICY-GAP-V3.3 authorized, see `.harness/audit/policy_gap.log`; lock violations 0, Evaluator APPROVE, raw 48/100, adjusted 56/90 — block by 3 policy gaps not code defects)
- [⏸] W1.2: Material RON 100+ (T6~T8) — **DEFERRED until governance v3.3 lands**
- [⏸] W1.3: Auto-derivation (folded into W1.1 T2 — already complete in `91d4e7c0`)
- [⏸] W1.4: Material inspector UI — DEFERRED (v3.3 pending)
- [⏸] W1.5: Cause-effect harness 5+ (T9~T10) — DEFERRED (v3.3 pending)
- [⏸] W1.6: 사용자 visual 검증 — DEFERRED (v3.3 pending)

### Governance v3.3 — pending (BLOCKER for W1.2+)
**Reason**: T2-T5 (`91d4e7c0`) revealed 3 systemic policy gaps in v3.2.1 hook governance.
Continuing W1.2~W1.6 with POLICY-GAP authorization on every commit would normalize the
bypass mechanism (same anti-pattern as v6 archive's ENV-BYPASS abuse).

**Identified gaps (from `91d4e7c0` POLICY-GAP authorization)**:
1. Cold-tier Visual Verify scoring — `+8` VLM env cost insufficient for cold-tier intrinsic
   absence (sim-core/sim-data schema work has no UI surface).
2. Attempt penalty discrimination — test-rigor RE-CODE penalised same as lock-violation
   RE-CODE; needs separation.
3. §6 NOT-in-scope FFI false positive — sim-bridge excluded but FFI Verify still FAIL.

**Affected scope (estimated from current Phase 1)**: W1.2~W1.6 (T6~T11), Phase 2~7 all
cold-tier sim-core/sim-data work.

**Next action**: 사용자가 다음 메시지로 v3.3 통합 명령 .md 작성 요청 → Claude.ai
v3.3 ticket → Claude Code dispatch (Step N: hook + pipeline 수정 + retroactive validate
`91d4e7c0` score 재계산).

**Resume condition**: v3.3 lands + retroactive validate `91d4e7c0` passes → W1.2 (T6) 시작.

### 사용자 confirm 기록
*(시스템 완성마다 사용자 명시 confirm 기록)*

## V7 Hard Gates 적용 현황
*(매 시스템마다 5 gates 검증 결과)*
