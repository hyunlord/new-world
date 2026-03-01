# Commit 062 - personality trait modifier 배열 사전컴파일

## 커밋 요약
- stressor personality trait modifier를 Dictionary에서 Packed 배열(ID/배수)로 사전컴파일.
- inject_event 성격 배수 계산 시 trait map/trait modifier 해석 오버헤드를 추가로 축소.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_load_stressor_defs()`
    - personality 컴파일 결과 저장 필드를 `_p_traits`에서 다음으로 변경:
      - `_p_trait_ids: PackedStringArray`
      - `_p_trait_multipliers: PackedFloat32Array`
  - `inject_event(...)`
    - `_calc_personality_scale(...)` 호출 인자를 trait Dictionary 대신 trait ID/배수 배열로 변경
  - `_calc_personality_scale(...)`
    - 시그니처를 `(entity, p_specs, p_trait_ids, p_trait_multipliers)`로 변경
    - trait 적용 루프를 인덱스 기반 Packed 배열 순회로 전환
  - `_compile_personality_modifiers(...)`
    - 반환값에서 trait 정보를 Dictionary 대신 Packed 배열로 생성
  - `_build_trait_id_map(...)`
    - 로컬 Dictionary 신규 생성 대신 `_event_trait_id_map.clear()` 기반 scratch 재사용

## 기능 영향
- personality scale 계산 의미는 유지.
- 이벤트 경로에서 trait modifier dictionary 순회/변환 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
