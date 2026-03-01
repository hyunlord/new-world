# Commit 031 - StressSystem 정규화 조회 중복 제거

## 커밋 요약
- `StressSystem`에서 같은 틱에 두 번 계산하던 `NEED_*` 정규화 조회를 1회 조회로 통합.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress(...)`에서
    - `hunger`, `energy`, `social`를 한 번만 조회해 로컬 변수로 보관
  - `_calc_appraisal_scale(...)` 시그니처 확장:
    - 정규화된 need 값을 인자로 전달받아 재조회 제거
  - `_calc_continuous_stressors(...)` 시그니처 변경:
    - `entity` 직접 접근 대신 전달된 need 값 사용

## 기능 영향
- 기존 계산식/결과는 동일.
- 엔티티당 stress 업데이트 시 `StatQuery.get_normalized(...)` 호출 횟수:
  - 기존 6회 → 변경 3회

## 검증
- `tools/migration_verify.sh` 통과
  - rust tests 전체 통과
  - strict localization audit: inline localized fields 0 유지
