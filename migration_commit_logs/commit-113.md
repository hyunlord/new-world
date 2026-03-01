# Commit 113 - Psychology/Development 이벤트 포맷 경량화

## 커밋 요약
- 심리/발달 시스템의 고정 placeholder 이벤트 설명 포맷을 `Locale.trf1/trf2` 경량 경로로 전환.

## 상세 변경
- `scripts/systems/psychology/contagion_system.gd`
  - `CONTAGION_SPIRAL_WARNING`:
    - `trf` + Dictionary → `trf2("stress", "valence")`

- `scripts/systems/development/attachment_system.gd`
  - `ATTACHMENT_FORMED`:
    - `trf` + Dictionary → `trf2("name", "type")`

- `scripts/systems/psychology/coping_system.gd`
  - `COPING_ACQUIRED`:
    - `trf` + Dictionary → `trf1("name")`
  - `COPING_UPGRADED`:
    - `trf` + Dictionary → `trf1("name")`

## 기능 영향
- Chronicle 이벤트 설명 문자열 출력 의미는 기존과 동일.
- 이벤트 로그 생성 경로의 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=452.5`, `checksum=13761358.00000`
