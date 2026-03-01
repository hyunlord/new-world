# Commit 058 - event emotion injection scratch buffer 재사용

## 커밋 요약
- `inject_event` 감정 주입 경로에서 매 호출마다 생성되던 current fast/slow PackedArray를 scratch buffer 재사용 방식으로 전환.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - 신규 scratch buffer:
    - `_event_fast_current`, `_event_slow_current`
  - `_inject_emotions(...)` 변경:
    - 기존 `PackedFloat32Array` 신규 생성 대신 class-level scratch resize + 값 갱신 방식 사용
    - `StatCurveScript.stress_emotion_inject_step(...)` 호출/결과 반영 로직은 동일 유지

## 기능 영향
- 감정 주입 수식/클램프 의미는 동일하게 유지.
- 이벤트 주입 빈도가 높은 구간에서 GDScript 임시 배열 할당/GC 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
