# Commit 068 - stress trace breakdown key 캐시

## 커밋 요약
- stress trace breakdown 키(`trace_<source_id>`)를 trace 항목에 캐시해 tick 루프의 반복 문자열 포맷팅을 줄임.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `inject_stress_event(...)`, `inject_event(...)`
    - 신규 trace append 시 `breakdown_key` 필드 저장
  - `_update_entity_stress(...)`
    - trace 처리 시 `breakdown_key` 우선 사용
    - 기존 저장 데이터(키 없음)는 1회 생성 후 trace에 저장해 재사용

## 기능 영향
- trace breakdown 키 의미는 기존과 동일 유지.
- trace가 유지되는 동안 키 문자열 재생성 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
