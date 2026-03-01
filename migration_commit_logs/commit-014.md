# Commit 014 - StatQuery XP 계산을 Rust-backed StatCurve로 통일

## 커밋 요약
- `StatQuery`의 LOG_DIMINISHING XP 계산 루프를 기존 수식 직접 계산에서 `StatCurveScript` 호출로 전환.
- `StatCurveScript`는 이미 Rust 브리지 우선 경로를 사용하므로, XP 레벨 계산 hot path를 Rust 실행 경로에 연결.

## 상세 변경
- `scripts/core/stats/stat_query.gd`
  - `get_skill_xp_info()`
    - `xp_at_level` 계산을 `StatCurveScript.log_xp_required()` 누적으로 변경.
    - `xp_to_next` 계산도 동일 함수 호출로 변경.
  - `_compute_level_from_xp()`
    - 수동 루프/수식 제거.
    - `StatCurveScript.xp_to_level(total_xp, params, max_level)` 직접 호출.
  - 더 이상 사용하지 않는 `_get_breakpoint_multiplier()` 제거.

## 기능 영향
- 수학 로직 중복 제거로 유지보수성 향상.
- XP 관련 곡선 수식의 단일 소스가 `StatCurve`로 정리되어 회귀 위험 감소.
- 브리지 사용 환경에서는 XP 레벨 변환이 Rust 경로를 사용해 성능 개선 기대.

## 검증
- `cd rust && cargo test -q` 통과
