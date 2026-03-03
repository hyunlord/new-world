# Commit 175 - movement greedy fallback 무할당 분기화

## 커밋 요약
- movement greedy fallback에서 매 호출 생성되던 후보 배열을 제거하고 직접 분기 호출로 전환해 할당 churn을 줄임.

## 상세 변경
- `scripts/systems/world/movement_system.gd`
  - `_move_toward_target(...)` 변경:
    - 기존 `candidates: Array[Vector2i]` 생성 + 순회 로직 제거.
    - 기존 우선순위(대각선 → x축 → y축)를 동일하게 유지한 직접 분기 호출로 대체.
  - `_try_move_candidate(entity, tick, candidate)` 헬퍼 추가:
    - walkable 체크 + 이동 + `entity_moved` 이벤트 emit를 공통화.
    - 성공 여부(`bool`) 반환으로 분기 단순화.

## 기능 영향
- greedy fallback 이동 의미/우선순위는 동일.
- fallback 이동 경로에서 임시 배열 생성이 사라져 tick당 메모리 churn 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=402.8`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=143.1`, `checksum=38457848.00000`
