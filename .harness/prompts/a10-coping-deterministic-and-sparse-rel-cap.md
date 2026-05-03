# a10-coping-deterministic-and-sparse-rel-cap

## Section 1: Implementation Intent

A-10 "Misc" bundle의 4개 항목 중 2개 처리:

1. **coping.rs HashMap → BTreeMap** (TODO v3.1 해소, 결정성 회복)
   - `strategy_cooldowns`, `usage_counts`, `proficiency` 3개 HashMap → BTreeMap
   - `CopingStrategyId`에 `PartialOrd, Ord` derive 추가
   - 같은 seed → 같은 결과 보장 (BTreeMap iteration 순서 결정적)

2. **SPARSE_REL_CAP 상수 도입 + Social::add_edge_capped helper**
   - `config::SPARSE_REL_CAP = 100` (Dunbar layer 3, 10K agent 인프라)
   - `Social::add_edge_capped(edge, cap)` - trust 기반 eviction
   - 기존 `SOCIAL_EDGE_CAP=50` + `enforce_edge_cap` (familiarity 기반)은 유지

NetworkId/LodTier는 정의 모호하여 별도 피처로 분리.

## Section 2: What to Build

### 변경 파일

1. `rust/crates/sim-core/src/enums.rs` - CopingStrategyId에 `PartialOrd, Ord` 추가
2. `rust/crates/sim-core/src/components/coping.rs` - HashMap→BTreeMap (3 fields)
3. `rust/crates/sim-core/src/config.rs` - SPARSE_REL_CAP = 100 추가
4. `rust/crates/sim-core/src/components/social.rs` - add_edge_capped helper 추가
5. `rust/crates/sim-test/src/main.rs` - harness 5개 추가

### Scope boundary
- ❌ 71개 HashMap 전부 변환 (coping 3개만)
- ❌ NetworkId/LodTier 구현
- ❌ 기존 SOCIAL_EDGE_CAP / enforce_edge_cap 변경
- ❌ production push 사이트 변경 (all 6 are test code)

## Section 3: How to Implement

이미 구현 완료:
- CopingStrategyId: `PartialOrd, Ord` 추가
- coping.rs: BTreeMap 변환 완료
- config.rs: SPARSE_REL_CAP = 100 추가
- social.rs: add_edge_capped helper 추가
- sim-test: 5개 harness 테스트 추가

## Section 4: Dispatch Plan

| 티켓 | 파일 | 모드 |
|------|------|:----:|
| T1: Ord derive + config 상수 | enums.rs, config.rs | 🔴 DIRECT |
| T2: coping BTreeMap | coping.rs | 🔴 DIRECT |
| T3: add_edge_capped | social.rs | 🔴 DIRECT |
| T4: harness 5개 | sim-test/main.rs | 🔴 DIRECT |

## Section 5: Localization Checklist

해당 없음 - 내부 구조 변경만.

## Section 6: Verification

Gate: `cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: 0 failed, clippy clean

Harness:
- harness_a10_coping_uses_btreemap: BTreeMap type check + sorted order
- harness_a10_sparse_rel_cap_enforced: len ≤ SPARSE_REL_CAP
- harness_a10_sparse_rel_cap_evicts_weakest: 0.1 evicted, 0.8 kept
- harness_a10_weak_edge_rejected_when_full: 약한 edge reject
- harness_a10_production_simulation_no_unbounded_edges: 2000 ticks, max edges ≤ SOCIAL_EDGE_CAP
