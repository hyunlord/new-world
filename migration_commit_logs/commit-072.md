# Commit 072 - stress trace 배열 in-place compact

## 커밋 요약
- stress tick에서 trace 유지 필터링 시 매번 새 `Array`를 만들던 경로를 in-place compact 방식으로 전환.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress(...)`
    - `next_traces` 신규 배열 append 방식 제거.
    - `ed.stress_traces`를 로컬 `traces`로 받아 `write_idx` 기반 compact(앞으로 당기기) 방식으로 갱신.
    - 처리 후 `resize(write_idx)`로 비활성 trace tail 제거.

## 기능 영향
- active trace 선별 결과와 trace 업데이트 의미는 기존과 동일.
- trace 수가 많은 상황에서 tick당 배열 할당/복사를 줄여 GC/메모리 churn을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=456.7`, `checksum=13761358.00000`
