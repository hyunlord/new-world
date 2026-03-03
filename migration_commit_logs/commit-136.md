# Commit 136 - needs 갈증/체온 소모 수식 Rust 이관

## 커밋 요약
- `needs_system`의 갈증/체온 소모 수식을 Rust `body` 모듈로 이관하고 bridge 경로를 연결해 needs tick 연산 네이티브화를 확장.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `thirst_decay(base_decay, tile_temp, temp_neutral)` 추가.
  - `warmth_decay(base_decay, tile_temp, has_tile_temp, temp_neutral, temp_freezing, temp_cold)` 추가.
  - 테스트 추가:
    - 중립 온도 대비 고온에서 thirst decay 가속
    - warmth decay 온도 밴드(중립/한랭/빙점) 동작 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_thirst_decay(...) -> f32` export 함수 추가.
  - `body_warmth_decay(...) -> f32` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_thirst_decay(...)` wrapper 추가.
  - `body_warmth_decay(...)` wrapper 추가.

- `scripts/systems/psychology/needs_system.gd`
  - 갈증 소모 계산에서 `SimBridge.body_thirst_decay` Rust 우선 호출 + 기존 GDScript fallback 적용.
  - 체온 소모 계산에서 `SimBridge.body_warmth_decay` Rust 우선 호출 + 기존 GDScript fallback 적용.

## 기능 영향
- 갈증/체온 소모 수식 의미와 온도 구간 동작은 기존과 동일.
- bridge 지원 환경에서 needs tick의 갈증/체온 소모 계산이 Rust 경로를 우선 사용.
- bridge 미지원 환경은 기존 GDScript 계산으로 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 69 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=456.6`, `checksum=13761358.00000`
