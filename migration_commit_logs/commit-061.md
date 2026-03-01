# Commit 061 - inject_event 입력 scratch buffer 재사용 확장

## 커밋 요약
- `inject_event` 입력 수집 단계(personality/context)에서 반복 생성되던 PackedArray를 class-level scratch buffer 재사용으로 전환.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - 신규 scratch buffer 추가:
    - `_event_personality_values`
    - `_event_personality_weights`
    - `_event_personality_high`
    - `_event_personality_traits`
    - `_event_active_context_multipliers`
  - `_calc_personality_scale(...)` 변경:
    - values/weights/high/trait 배열을 매 호출 생성하지 않고 `resize(0)` 후 재사용
  - `_collect_active_context_multipliers(...)` 변경:
    - 활성 context multiplier 배열을 scratch buffer로 재사용

## 기능 영향
- personality/context scale 계산 의미는 동일하게 유지.
- 이벤트 주입 경로에서 GDScript 임시 배열 할당/GC 오버헤드를 추가 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
