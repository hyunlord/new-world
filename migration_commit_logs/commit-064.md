# Commit 064 - emotion inject 없는 이벤트 fast path 분기

## 커밋 요약
- `emotion_inject`가 실질적으로 없는 stressor 이벤트는 emotion 결합 step을 건너뛰고 scale step만 실행하도록 분기.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_compile_emotion_inject(...)`
    - `has_values` 플래그 계산/반환 추가 (`raw_val != 0` 항목 존재 여부)
  - `_load_stressor_defs()`
    - `_emo_has_values` 필드 저장
  - `inject_event(...)`
    - `_emo_has_values`가 `true`일 때만:
      - `_fill_event_emotion_current(...)`
      - `stress_event_inject_step(...)`
      - `_apply_event_emotion_layers(...)`
    - `false`일 때는 `stress_event_scale_step(...)`만 호출

## 기능 영향
- emotion inject 값이 없는 이벤트의 최종 stress 주입 의미는 유지.
- 해당 케이스에서 불필요한 emotion snapshot/결합 step 호출을 제거해 이벤트 경로 오버헤드를 절감.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
