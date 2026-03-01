# Commit 140 - needs base decay에 safety 소모 통합

## 커밋 요약
- `needs_base_decay_step` 결과에 safety decay를 포함해 needs 기본 소모(배고픔/에너지/사회/갈증/체온/안전감)를 한 번에 반영하도록 통합.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `needs_base_decay_step` 시그니처에 `safety_decay_rate` 추가.
  - 반환 배열을 6원소로 확장:
    - `[hunger_decay, energy_decay, social_decay, thirst_decay, warmth_decay, safety_decay]`
  - `needs_expansion_enabled`가 false이면 safety/thirst/warmth decay는 0.0 처리.
  - 테스트(`base_decay_step_matches_manual_formula`)를 safety 항목까지 검증하도록 확장.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_needs_base_decay_step` scalar decode 인덱스를 safety 포함 구조로 조정.
  - 입력 순서: hunger/energy/social + safety + thirst + warmth + temperature params.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_base_decay_step(...)` wrapper 인자에 `safety_decay_rate` 추가.
  - scalar packed 인코딩 순서에 safety 항목 추가.

- `scripts/systems/psychology/needs_system.gd`
  - base decay scratch scalar 길이를 13→14로 확장하고 safety rate를 포함해 전달.
  - base decay 성공 시 safety를 `base_decay_step[5]`로 반영.
  - base decay fallback 시에만 기존 `SAFETY_DECAY_RATE` 직접 감산 유지.
  - 별도 safety 감산 블록 제거로 중복 경로 정리.

## 기능 영향
- 안전감 소모 의미는 기존과 동일(확장 활성 시에만 소모).
- bridge 지원 환경에서 needs 기본 소모 6축이 단일 base step 결과로 반영.
- bridge 미지원 환경은 기존 fallback과 동일 동작.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 71 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=449.5`, `checksum=13761358.00000`
