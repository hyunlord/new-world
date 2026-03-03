# Commit 148 - needs_system 임시 배열 할당 추가 절감

## 커밋 요약
- needs tick 루프에서 Rust 결과를 다룰 때 생성하던 임시 `PackedFloat32Array`를 제거하고 스칼라 변수로 직접 반영하도록 바꿔 per-entity 할당 churn을 축소.

## 상세 변경
- `scripts/systems/psychology/needs_system.gd`
  - base decay 처리:
    - `base_decay_step`/`rust_temp_decay` 임시 배열 생성 제거.
    - Rust 반환 `PackedFloat32Array`를 즉시 스칼라(`rust_hunger_decay`, `rust_thirst_decay` 등)로 디코드.
    - 갈증/체온/안전감 소모 적용 시 `has_rust_base_decay` 플래그로 분기.
  - critical severity 처리:
    - `severity_step` 임시 배열 생성 제거.
    - Rust 반환값을 `rust_sev_thirst/warmth/safety` 스칼라로 디코드 후 재사용.
    - fallback 계산식은 기존과 동일 유지.

## 기능 영향
- needs 감소량 및 stressor severity 결과 의미는 기존과 동일.
- tick 루프의 임시 Packed 배열 할당 수를 줄여 GC/메모리 churn을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 76 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=475.9`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=172.5`, `checksum=29719684.00000`
