# Commit 121 - ChroniclePanel 레지스트리 조회 경량화

## 커밋 요약
- ChroniclePanel 이벤트 렌더 루프에서 `DeceasedRegistry` 조회를 1회 캐시로 전환하고, 날짜 분기 블록 정렬을 정리.

## 상세 변경
- `scripts/ui/panels/chronicle_panel.gd`
  - 이벤트 루프 시작 전에 `deceased_registry`를 `get_node_or_null("/root/DeceasedRegistry")`로 1회 조회.
  - related entity 이름 조회 시 매 반복 `get_node_or_null` 대신 캐시된 `deceased_registry` 재사용.
  - 날짜 문자열 분기(`if tick / elif hour / else short_date`) 블록 정렬을 정리해 fallback 경로를 명확히 유지.

## 기능 영향
- Chronicle 패널의 이벤트 표시/클릭 영역 동작은 기존과 동일.
- 이벤트 렌더 루프에서 반복 노드 탐색을 줄여 UI 갱신 경로 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=466.1`, `checksum=13761358.00000`
