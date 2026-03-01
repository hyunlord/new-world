# Commit 263 - verify 리포트에 아티팩트 집계 지표 추가

## 커밋 요약
- `migration_verify_report.json`에 아티팩트 집계 요약(`artifact_counts`)을 추가해 기대/존재/누락 상태를 즉시 확인 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 헬퍼 `count_artifact_presence(path)` 추가:
    - 경로가 설정된 아티팩트만 기대치(`expected`)로 카운트
    - 실제 파일 존재 시 `present` 증가
  - verify report 생성 시 집계 계산:
    - `artifact_expected_count`
    - `artifact_present_count`
    - `artifact_missing_count`
  - report 루트에 `artifact_counts` 객체 추가:
    - `expected`
    - `present`
    - `missing`

## 기능 영향
- 파서/대시보드가 `artifact_exists` 전체를 스캔하지 않고도 산출물 상태를 빠르게 요약 판단 가능.
- `MIGRATION_AUDIT_REPORT_DIR` 사용 시 non-bench/with-benches 실행 간 기대 아티팩트 수 차이를 한 필드로 비교 가능.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts25 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts25/migration_verify_report.json`에서:
  - `artifact_counts.expected=7`
  - `artifact_counts.present=7`
  - `artifact_counts.missing=0`
  확인.
