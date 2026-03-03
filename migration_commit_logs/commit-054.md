# Commit 054 - stressor personality modifier 사전 컴파일

## 커밋 요약
- `inject_event` 런타임 경로에서 personality modifier 키 파싱/분기를 줄이기 위해 stressor 로드 단계 사전 컴파일을 도입.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_load_stressor_defs()`
    - 각 stressor의 `personality_modifiers`를 `_compile_personality_modifiers(...)`로 사전 컴파일
    - 결과를 `_p_specs`, `_p_traits`로 저장
  - `inject_event(...)`
    - 기존 원본 `personality_modifiers` 직접 사용 대신 `_p_specs`, `_p_traits`를 사용
  - `_calc_personality_scale(...)`
    - 시그니처를 `(entity, p_specs, p_traits)`로 변경
    - 런타임에서는 사전 컴파일된 spec만 순회하여 값 수집 후 Rust helper 호출
  - 신규 helper `_compile_personality_modifiers(...)`
    - `*_axis` / facet / traits 구성을 로드 시 표준화

## 기능 영향
- personality scale 결과 의미는 유지하면서 이벤트 처리 시 반복적인 키 문자열 파싱/분기 비용을 줄임.
- 기존 Rust 경로(`StatCurveScript.stress_personality_scale`) 호출 구조는 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
