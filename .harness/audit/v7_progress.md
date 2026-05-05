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
- [ ] W1.2: Material RON 100+
- [ ] W1.3: Auto-derivation (folded into W1.1 T2)
- [ ] W1.4: Material inspector UI
- [ ] W1.5: Cause-effect harness 5+
- [ ] W1.6: 사용자 visual 검증

### 사용자 confirm 기록
*(시스템 완성마다 사용자 명시 confirm 기록)*

## V7 Hard Gates 적용 현황
*(매 시스템마다 5 gates 검증 결과)*
