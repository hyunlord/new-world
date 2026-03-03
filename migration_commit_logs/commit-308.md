# Commit 308 - StatSync 파생 스탯 계산 Rust 브리지 이관

## 커밋 요약
- `stat_sync_system`의 `_compute_derived` 핵심 파생 스탯 계산 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `stat_sync_derived_scores(inputs: &[f32]) -> [f32; 8]`
      - 입력: HEXACO/감정/가치/지능/외형/연령 스칼라 29개
      - 출력: `charisma, intimidation, allure, trustworthiness, creativity, wisdom, popularity, risk_tolerance`
  - 단위 테스트 추가:
    - `stat_sync_derived_scores_returns_expected_shape`
    - `stat_sync_derived_scores_handles_short_input`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_stat_sync_derived_scores(inputs: PackedFloat32Array) -> PackedFloat32Array`

- `scripts/systems/record/stat_sync_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_compute_derived`에서 입력 스칼라를 `PackedFloat32Array`로 구성해 Rust-first 호출.
  - 브리지 실패/결과 불일치 시 기존 GDScript 계산 경로 fallback 유지.

## 기능 영향
- 매 tick 반복되는 파생 스탯 계산의 핵심 수식이 Rust 경로로 이동.
- 결과 저장(StatQuery.set_value) 및 기존 동기화 흐름은 동일 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 실측 기준)**: `41/56` 적용, 잔여 `15/56`.
