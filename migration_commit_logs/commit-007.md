# Commit 007 - Stat Curve 수학 모듈 Rust 이관(1차)

## 커밋 요약
- GDScript `stat_curve.gd`의 핵심 수학 커브를 Rust 순수 함수 모듈로 이관.
- 향후 `StatQuery`/브리지 연동을 위한 기반 함수와 단위 테스트를 추가.

## 상세 변경
- `rust/crates/sim-systems/src/lib.rs`
  - `stat_curve` 모듈 export 추가.
- `rust/crates/sim-systems/src/stat_curve.rs` (신규)
  - 구현 함수:
    - `log_xp_required`
    - `xp_to_level`
    - `scurve_speed`
    - `need_decay`
    - `sigmoid_extreme`
    - `power_influence`
    - `threshold_power`
    - `linear_influence`
    - `step_influence`
    - `step_linear`
  - 단위 테스트 5개 추가

## 기능 영향
- 아직 GDScript/Bridge 호출 경로에 직접 연결되지는 않음.
- 수학 로직을 Rust side에 선이관해 후속 연결(StatQuery Rust 전환) 위험을 낮춤.

## 검증
- `cargo test -q -p sim-systems` 통과
- `cargo test -q` (workspace 전체) 통과
