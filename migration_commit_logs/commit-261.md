# Commit 261 - verify 리포트에 실행 시작/종료 시각 추가

## 커밋 요약
- `migration_verify_report.json`에 실행 시작/종료 시각(`started_at_utc`, `finished_at_utc`)을 추가해 시간 추적 메타데이터를 보강.

## 상세 변경
- `tools/migration_verify.sh`
  - 스크립트 시작 시 UTC 타임스탬프(`VERIFY_STARTED_AT_UTC`)를 캡처.
  - verify report 생성 직전 UTC 종료 시각(`verify_finished_at_utc`)을 캡처.
  - report 루트에 신규 필드 추가:
    - `started_at_utc`
    - `finished_at_utc`
  - 기존 `generated_at_utc`는 종료 시각과 동일 값을 유지.

## 기능 영향
- `total_duration_seconds`와 함께 실행 구간(절대 시각)을 직접 확인 가능.
- 배치/CI에서 다중 실행 로그를 시간축으로 정렬하고 추적하기 쉬워짐.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts23 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts23/migration_verify_report.json`에서:
  - `started_at_utc=2026-03-01T15:27:47Z`
  - `finished_at_utc=2026-03-01T15:27:48Z`
  - `total_duration_seconds=1`
  반영 확인.
