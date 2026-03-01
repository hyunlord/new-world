# Commit 262 - verify 리포트에 아티팩트 수정시각(mtime) 추가

## 커밋 요약
- `migration_verify_report.json`에 각 아티팩트의 최종 수정 시각(`artifact_mtime_utc`)을 추가해 파일 시점 추적을 강화.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 헬퍼 `to_json_opt_mtime_utc(path)` 추가:
    - 파일 미설정/미존재 시 `null`
    - 파일 존재 시 UTC ISO-8601 문자열(`YYYY-MM-DDTHH:MM:SSZ`) 반환
  - verify report 직렬화 확장:
    - `artifact_mtime_utc` 객체 추가
    - compile/audit/bench 아티팩트 각각의 수정시각 기록

## 기능 영향
- `artifact_exists` + `artifact_size_bytes` + `artifact_sha256`와 함께 “언제 생성/갱신된 파일인지”를 report 단일 JSON에서 파악 가능.
- 다중 실행 비교 시 최신 산출물 여부 확인이 쉬워짐.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts24 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts24/migration_verify_report.json`에서:
  - `artifact_mtime_utc.compile_report_json` UTC 문자열 확인
  - non-bench 실행 기준 `artifact_mtime_utc.bench_report_json=null` 확인.
