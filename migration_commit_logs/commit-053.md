# Commit 053 - stress event emotion injection Rust step 도입

## 커밋 요약
- 이벤트 stress 주입 시 감정 레이어(`fast`/`slow`) 반영 수식을 Rust step으로 이관.
- stressor 정의 로드 시 `emotion_inject`를 사전 컴파일해 이벤트 처리 시 문자열 파싱/분기 비용을 줄임.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressEmotionInjectStep { fast, slow }`
  - 신규 함수:
    - `stress_emotion_inject_step(fast_current, slow_current, fast_inject, slow_inject, scale)`
  - 동작:
    - `fast = clamp(current + inject * scale, 0..100)`
    - `slow = clamp(current + inject * scale, -50..100)`
  - 단위 테스트 1개 추가:
    - scale 적용 + clamp 범위 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_emotion_inject_step(...) -> Dictionary{fast, slow}`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_emotion_inject_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_emotion_inject_step(...)`
  - Rust 우선 + fallback 계산 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_EMOTION_ORDER`, `_EMOTION_INDEX` 상수 추가
  - `_update_entity_stress`에서 감정 key 순회를 상수 배열 기반으로 통일
  - `_load_stressor_defs`에서 `emotion_inject`를 `_emo_fast`/`_emo_slow`로 사전 컴파일
  - `inject_event`는 사전 컴파일된 배열을 사용해 `_inject_emotions` 호출
  - `_inject_emotions`를 Rust step 호출 기반으로 재작성
  - `_compile_emotion_inject` helper 추가
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_emotion_inject_step` 호출 추가

## 기능 영향
- 이벤트 감정 주입 경로의 산술/클램프 처리 로직이 Rust 수식으로 일원화.
- 기존 감정 레이어 경계값(FAST 0..100, SLOW -50..100)과 scale 적용 의미를 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (44 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=301.9`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
