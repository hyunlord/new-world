# Commit 273 - sim-data 확장 (species/mortality/developmental/attachment)

## 커밋 요약
- Rust `sim-data`에 species/mortality/developmental/attachment 로더를 추가해 데이터 이식 범위를 추가 확장.

## 상세 변경
- `rust/crates/sim-data/src/lib.rs`
  - 신규 모듈 wiring:
    - `species`
    - `mortality`
    - `developmental_stages`
    - `attachment_config`
  - `DataBundle` 확장 필드:
    - `species`
    - `mortality`
    - `developmental_stages`
    - `attachment`
  - `load_all`이 신규 4개 데이터셋까지 로드하도록 확장.

- 신규 로더 파일 추가
  - `rust/crates/sim-data/src/species.rs`
    - `data/species/*/species_definition.json` 로딩
  - `rust/crates/sim-data/src/mortality.rs`
    - `data/species/*/mortality/siler_parameters.json` 로딩
  - `rust/crates/sim-data/src/developmental_stages.rs`
    - `data/developmental_stages.json` 로딩
  - `rust/crates/sim-data/src/attachment_config.rs`
    - `data/attachment_config.json` 로딩

- `rust/crates/sim-test/src/main.rs`
  - 데이터 로드 로그에 `species`, `mortality_profiles`, `developmental_stages` 카운트를 추가.

- `rust/tests/data_loading_test.rs`
  - 신규 로더 결과에 대한 통합 검증 확장:
    - species/mortality/developmental non-empty
    - attachment 설정값 기본 유효성(`determination_window_days > 0`)

## 기능 영향
- Rust 쪽에서 인간 종 정의, 사망 모델 파라미터, 발달 단계, 애착 설정을 직접 파싱 가능.
- 향후 심리/발달/생물학 시스템 이식 시 GDScript 데이터 의존을 추가로 축소할 준비가 완료됨.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS` 확인).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**:
  - 이전: `7/9` 완료
  - 현재: `8/9` 완료 (species, mortality 추가)
  - 잔여: `1/9` (building 계열 로더/데이터 경로 정리)
- **전체 전환(엔진/30+ 시스템/Godot runtime 분리 포함)**:
  - 여전히 하이브리드 단계, 메인 tick 오케스트레이션은 GDScript.
