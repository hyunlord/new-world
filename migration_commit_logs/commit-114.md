# Commit 114 - Development 시스템 포맷 경량화 마무리

## 커밋 요약
- `development` 계열 시스템에서 `params` 전달 패턴을 유지하면서 설명 문자열 생성만 `Locale.trf1/trf2` 경량 경로로 전환.

## 상세 변경
- `scripts/systems/development/parenting_system.gd`
  - `ADULTHOOD_TRANSITION` 설명 생성:
    - `Locale.trf(..., params)` → `Locale.trf1("name", ...)`
  - 기존 chronicle 메타(`key`, `params`) 기록 구조는 유지.

- `scripts/systems/development/child_stress_processor.gd`
  - `SHRP_OVERRIDE` 설명 생성:
    - `Locale.trf(..., params)` → `Locale.trf1("name", ...)`
  - 기존 chronicle 메타(`key`, `params`) 기록 구조는 유지.

- `scripts/systems/development/ace_tracker.gd`
  - `ACE_EVENT_RECORDED` 설명 생성:
    - `Locale.trf(..., params)` → `Locale.trf2("name", "ace_type")`
  - `HEXACO_CAP_MODIFIED` 설명 생성:
    - `Locale.trf(..., params)` → `Locale.trf1("facet", ...)`
  - 기존 chronicle 메타(`key`, `params`) 기록 구조는 유지.

## 기능 영향
- 개발/ACE 관련 Chronicle 이벤트 텍스트와 메타 데이터 구조는 기존과 동일.
- params Dictionary는 로그 메타용으로 유지하면서, 사용자 표시 문자열 생성 시 임시 처리 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=464.0`, `checksum=13761358.00000`
