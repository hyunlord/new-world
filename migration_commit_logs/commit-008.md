# Commit 008 - StatCurve Rust 브리지 실연결

## 커밋 요약
- `sim-systems::stat_curve`(Commit 007)를 Godot 호출 경로에 실제 연결.
- `StatCurve` 계산을 Rust 우선 경로로 실행하고, 네이티브 브리지 미사용 시 기존 GDScript 수학으로 자동 fallback.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `WorldSimBridge`에 stat curve 관련 `#[func]` 메서드 추가:
    - `stat_log_xp_required`
    - `stat_xp_to_level`
    - `stat_scurve_speed`
    - `stat_need_decay`
    - `stat_sigmoid_extreme`
    - `stat_power_influence`
    - `stat_threshold_power`
    - `stat_linear_influence`
    - `stat_step_influence`
    - `stat_step_linear`
  - `PackedInt32Array`/`PackedFloat32Array` 변환 유틸 추가.
- `scripts/core/simulation/sim_bridge.gd`
  - 위 10개 stat 메서드의 GDScript 프록시 추가.
  - 공용 호출 헬퍼 `_call_native_if_exists()` 추가.
- `scripts/core/stats/stat_curve.gd`
  - 각 커브 함수에서 Rust 브리지 메서드 우선 호출.
  - 브리지 결과가 없거나 메서드 미지원이면 기존 GDScript 계산식으로 fallback.
  - 배열 파라미터 전달용 PackedArray 변환 헬퍼 추가.

## 기능 영향
- 기존 게임 동작/수학식은 그대로 유지되며, 네이티브 브리지 사용 시 계산 hot path 일부가 Rust로 실행됨.
- 브리지 미로드 환경에서도 fallback으로 기능 저하 없이 동작.

## 검증
- `cd rust && cargo fmt -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-bridge` 통과
- `cd rust && cargo test -q` 통과
- `cd rust && cargo build -q --release -p sim-bridge` 통과
