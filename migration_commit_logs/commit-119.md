# Commit 119 - Chronicle params 정수값 직접 전달

## 커밋 요약
- Chronicle `l10n.params` 생성 시 수치값을 `str(...)`로 선변환하지 않고 원본 정수/숫자로 직접 저장하도록 정리.

## 상세 변경
- `scripts/systems/world/tech_maintenance_system.gd`
  - `TECH_STABILIZED_FMT`, `TECH_FALLBACK_FMT`, `TECH_LOST_FMT`의 `settlement` params:
    - `str(settlement.id)` → `settlement.id`

- `scripts/systems/world/tech_propagation_system.gd`
  - `TOAST_TEACHING_COMPLETED`:
    - `student`, `teacher`, `level` 선 문자열 변환 제거
  - `TOAST_TEACHING_ABANDONED`:
    - `student`, `teacher` 선 문자열 변환 제거
  - `TOAST_TEACHING_STARTED`:
    - `teacher`, `student` 선 문자열 변환 제거
  - `TOAST_TECH_IMPORTED_*`:
    - `settlement`, `source`, `carrier` 선 문자열 변환 제거

## 기능 영향
- 이벤트 설명 렌더 결과는 동일 (`Locale.trf`에서 문자열 치환 시 변환 수행).
- 이벤트 생성 시 불필요한 문자열 할당을 줄여 Chronicle params 생성 경로의 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.7`, `checksum=13761358.00000`
