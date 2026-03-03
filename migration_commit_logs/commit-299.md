# Commit 299 - MortalitySystem 핵심 hazard 계산 Rust 브리지 이관

## 커밋 요약
- `mortality_system`의 Siler hazard 분해 및 사망 확률(q_annual/q_check) 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `mortality_hazards_and_prob(age_years, ..., is_monthly) -> [f32; 6]`
      - 반환: `[h_infant, h_background, h_senescence, mu_total, q_annual, q_check]`
  - 단위 테스트 추가:
    - `mortality_hazards_and_prob_monthly_check_is_lower_than_annual`
    - `mortality_hazards_and_prob_applies_care_protection_and_dr_reduction`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_mortality_hazards_and_prob(model_inputs, env_inputs, is_monthly)`
  - Godot 인자 수 제한 대응을 위해 `PackedFloat32Array` 기반 입력으로 브리지 시그니처를 구성.

- `scripts/systems/biology/mortality_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_do_mortality_check`에서 모델/환경 입력을 `PackedFloat32Array`로 구성해 Rust-first 호출.
  - Rust 결과가 유효하지 않을 때 기존 GDScript 수식 경로로 fallback 유지.

## 기능 영향
- 사망률 체크의 핵심 연산(매개 hazard + 확률 변환)이 Rust 경로로 이동해 반복 계산 비용을 낮춤.
- 브리지 호출 실패/결과 불일치 시 기존 GDScript 로직으로 안전하게 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `28/56` 적용, 잔여 `28/56`.
