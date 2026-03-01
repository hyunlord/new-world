# Commit 033 - Emotion→Stress 기여 계산 Rust 이관

## 커밋 요약
- `StressSystem`의 감정 기여 계산(8감정 가중치 + VA composite)을 Rust로 이관.
- GDScript는 감정값 수집과 breakdown 반영만 수행.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 타입:
    - `EmotionStressContribution`
  - 신규 함수:
    - `stress_emotion_contribution(...)`
      - 임계값(20) 초과분에 가중치 적용
      - `valence/arousal` 기반 `va_composite` 포함
  - 단위 테스트 2개 추가:
    - 중립 입력에서 0 결과
    - 음의 valence + 높은 arousal에서 VA 기여 확인
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_emotion_contribution(...) -> VarDictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_emotion_contribution(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_emotion_contribution(...)`
  - Rust 우선 + 기존 가중치 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_emotion_contribution(...)`를 Rust 결과 기반으로 전환
  - 기존 breakdown 키(`emo_*`, `va_composite`) 유지
  - 중복 상수(`EMOTION_*`, `VA_GAMMA`) 제거

## 기능 영향
- stress 계산 핫패스의 감정 기여 수식이 네이티브 경로로 이동.
- 기존 게임플레이 의미와 breakdown 구조는 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (15 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
