# Commit 015 - Skill XP 진행도 단일 브리지 호출 최적화

## 커밋 요약
- `get_skill_xp_info()`의 XP 진행도 계산을 다중 호출/루프 방식에서 단일 Rust 브리지 호출 방식으로 최적화.
- level 기반 XP 진행(`xp_at_level`, `xp_to_next`, `progress_in_level`)을 한 번에 계산해 FFI 왕복 비용을 줄임.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 API: `stat_skill_xp_progress(...) -> VarDictionary`
  - 입력: `level, xp, base_xp, exponent, breakpoints, multipliers, max_level`
  - 출력: `level, max_level, xp_at_level, xp_to_next, progress_in_level`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시: `stat_skill_xp_progress(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 헬퍼: `skill_xp_progress(level, xp, params, max_level)`
  - Rust 브리지 우선 호출, 미가용 시 기존 GDScript 수식 fallback.
- `scripts/core/stats/stat_query.gd`
  - `get_skill_xp_info()`가 `StatCurveScript.skill_xp_progress(...)` 단일 호출 결과를 사용하도록 변경.

## 기능 영향
- 기존 결과값 구조는 유지하면서, XP 진행도 조회 시 per-level 반복 호출을 제거.
- 브리지 사용 환경에서 UI/시스템의 XP 표시 계산 비용 감소.

## 검증
- `cd rust && cargo fmt -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-bridge` 통과
- `cd rust && cargo test -q` 통과
