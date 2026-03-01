# Commit 286 - TitleSystem 판정 수식 Rust 브리지 이관

## 커밋 요약
- `title_system`의 연령 기반/스킬 기반 칭호 판정 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `title_is_elder(age_years, elder_min_age_years) -> bool`
    - `title_skill_tier(level, expert_level, master_level) -> i32`
  - 단위 테스트 추가:
    - elder threshold 판정 검증
    - expert/master tier 우선순위 판정 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_title_is_elder(...)`
    - `body_title_skill_tier(...)`

- `scripts/systems/social/title_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_evaluate_age_titles`를 Rust-first 판정으로 전환(fallback 유지).
  - `_evaluate_skill_titles`를 Rust-first tier 판정으로 전환(fallback 유지).

## 기능 영향
- 칭호 부여/회수의 기준 판정식이 Rust 경로로 이동해 반복 판정 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 경로 유지로 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `15/56` 적용, 잔여 `41/56`.
