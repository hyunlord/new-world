# Commit 172 - movement tick scratch array 재사용

## 커밋 요약
- movement 시스템 tick 루프에서 매번 새로 생성하던 임시 Array를 scratch 버퍼로 재사용하도록 변경해 할당 오버헤드를 줄임.

## 상세 변경
- `scripts/systems/world/movement_system.gd`
  - scratch 필드 추가:
    - `_path_entities_scratch`
    - `_recalc_entities_scratch`
  - `execute_tick(...)`에서
    - 기존 지역 Array 생성 대신 scratch 참조 사용
    - tick 시작 시 `clear()`로 재사용
  - 기존 packed XY recalc 버퍼(`_recalc_from_xy`, `_recalc_to_xy`) 재사용 로직과 함께 사용.

## 기능 영향
- 이동/경로 계산 의미와 결과는 동일.
- tick마다 발생하는 임시 Array 객체 생성 수를 줄여 movement hot path 메모리 churn 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=554.8`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=201.7`, `checksum=38457848.00000`
