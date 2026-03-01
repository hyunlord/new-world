# Commit 055 - stressor relationship/context modifier 사전 컴파일

## 커밋 요약
- `inject_event` 런타임 경로의 relationship/context dictionary 해석을 로드 단계 사전 컴파일로 치환.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_load_stressor_defs()`
    - `relationship_scaling` 사전 컴파일 결과를 `_r_method`, `_r_min_mult`, `_r_max_mult`로 저장
    - `context_modifiers` 사전 컴파일 결과를 `_c_keys`, `_c_multipliers`로 저장
  - `inject_event(...)`
    - 관계/상황 스케일 계산 시 원본 dictionary 대신 사전 컴파일 필드 사용
  - `_calc_relationship_scale(...)`
    - 시그니처를 `(context, method, min_m, max_m)`로 단순화
  - `_calc_context_scale(...)`
    - 시그니처를 `(context, c_keys, c_multipliers)`로 변경
  - 신규 helper:
    - `_compile_relationship_scaling(...)`
    - `_compile_context_modifiers(...)`

## 기능 영향
- 관계/상황 배수의 수식 의미는 유지하면서 이벤트 실행 시 dictionary 파싱/순회 오버헤드를 완화.
- 기존 Rust 경로(`stress_relationship_scale`, `stress_context_scale`) 호출 구조 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
