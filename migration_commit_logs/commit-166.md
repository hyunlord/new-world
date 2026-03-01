# Commit 166 - stress rebound queue packed 캐시 적용

## 커밋 요약
- `stress_system`의 rebound queue 처리 경로를 packed cache(`PackedFloat32Array`/`PackedInt32Array`) 기반으로 확장해 tick당 딕셔너리 파싱 비용을 줄임.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - rebound queue 메타 키 상수 추가:
    - `_REBOUND_QUEUE_META_KEY`
    - `_REBOUND_AMOUNTS_META_KEY`
    - `_REBOUND_DELAYS_META_KEY`
  - `schedule_rebound(...)`
    - 기존 `rebound_queue`(Array of Dictionary) 유지.
    - 동시에 packed cache(`rebound_queue_amounts`, `rebound_queue_delays`)를 append하도록 확장.
  - `_process_rebound_queue(...)`
    - packed cache가 유효하면 이를 직접 사용.
    - packed cache가 없거나 깨졌으면 기존 `rebound_queue`를 1회 파싱해 fallback.
    - 처리 후 `rebound_queue`와 packed cache를 동기화해 저장.

## 기능 영향
- rebound 동작 의미(지연 감소, total_rebound 반영)는 유지.
- 기존 save/runtime 호환성을 위해 `rebound_queue` 메타 구조를 계속 유지.
- packed cache가 존재하는 런타임에서는 queue 파싱 오버헤드를 줄여 stress tick 경로 효율 개선.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=506.4`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=325.9`, `checksum=38457848.00000`
