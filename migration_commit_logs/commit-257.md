# Commit 257 - verify 리포트에 아티팩트 존재 여부 추가

## 커밋 요약
- `migration_verify_report.json`에 아티팩트별 존재 여부(`artifact_exists`)를 추가해, 경로/해시/크기 외에 파일 존재 상태를 즉시 판별 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 헬퍼:
    - `to_json_opt_exists(path)` 추가
      - 경로 미설정: `null`
      - 파일 존재: `true`
      - 파일 미존재: `false`
  - verify report 확장:
    - `artifact_exists` 객체 추가
      - compile/audit/bench 아티팩트 각각의 존재 여부 출력.

## 기능 영향
- 파서가 아티팩트 상태를 별도 파일시스템 체크 없이 report만으로 판정 가능.
- `artifact_sha256`/`artifact_size_bytes`와 함께 읽으면 “경로 설정됨 + 파일 유무 + 무결성”을 한 번에 판단 가능.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts19 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts19/migration_verify_report.json`에서:
  - `artifact_exists.compile_report_json=true` 등 주요 항목 확인
  - non-bench 실행 기준 `artifact_exists.bench_report_json=null` 확인.
