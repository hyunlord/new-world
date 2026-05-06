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
- [⏯] W1.2: Material RON 100+ (T6~T8) — **UNBLOCKED** (v3.3.3 land complete)
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
