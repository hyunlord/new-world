# Commit 063 - personality spec 배열 사전컴파일

## 커밋 요약
- stressor personality spec(`axis/facet/weight/direction`)를 `Array[Dictionary]`에서 Packed 배열로 사전컴파일.
- inject_event 성격 배수 경로를 인덱스 순회 기반으로 최적화.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_load_stressor_defs()`
    - personality spec 저장 필드를 `_p_specs`에서 다음으로 전환:
      - `_p_spec_kinds: PackedByteArray`
      - `_p_spec_ids: PackedStringArray`
      - `_p_spec_axis_stats: PackedStringArray`
      - `_p_spec_weights: PackedFloat32Array`
      - `_p_spec_high: PackedByteArray`
  - `inject_event(...)`
    - `_calc_personality_scale(...)` 호출 인자를 spec 배열 세트로 변경
  - `_calc_personality_scale(...)`
    - 시그니처를 Packed 배열 기반으로 변경
    - spec_count(min of packed array sizes) 기준 인덱스 순회
    - axis/facet 조회 분기 및 weight/high 적용을 배열 값으로 처리
  - `_compile_personality_modifiers(...)`
    - 반환 구조를 spec dictionary array 대신 Packed 배열 묶음으로 변경

## 기능 영향
- personality scale 계산 의미는 동일 유지.
- 이벤트 주입 경로에서 spec dictionary 해석/타입 캐스팅 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
