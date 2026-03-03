# Commit 120 - Chronicle DeceasedRegistry 조회 캐시

## 커밋 요약
- `ChronicleSystem`의 엔티티 이름 조회에서 `DeceasedRegistry` 노드 탐색을 캐시해 반복 트리 탐색 비용을 줄임.

## 상세 변경
- `scripts/systems/record/chronicle_system.gd`
  - `_deceased_registry: Node` 캐시 필드 추가.
  - `_get_deceased_registry()` 헬퍼 추가:
    - 유효한 캐시가 있으면 재사용.
    - 없으면 `get_node_or_null("/root/DeceasedRegistry")`로 1회 갱신.
  - `_get_entity_name()`:
    - 기존 `has_node` + `get_node` 경로를 캐시 헬퍼 호출로 대체.

## 기능 영향
- 이름 조회 결과 의미는 기존과 동일.
- Chronicle 이벤트 누적 시 DeceasedRegistry 조회의 반복 노드 탐색을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=455.0`, `checksum=13761358.00000`
