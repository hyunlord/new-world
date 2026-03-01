# Commit 254 - verify 리포트에 아티팩트 크기(bytes) 추가

## 커밋 요약
- `migration_verify_report.json`에 아티팩트별 파일 크기(`artifact_size_bytes`)를 추가해, 해시와 함께 용량 변화까지 추적 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 헬퍼:
    - `to_json_opt_size_bytes(path)` 추가
      - 파일이 없으면 `null`
      - 있으면 바이트 단위 파일 크기(정수) 반환
  - verify report 확장:
    - `artifact_size_bytes` 객체 추가
      - compile/audit 관련 산출물 + bench report size 기록
      - 미생성 항목은 `null`

## 기능 영향
- 실행 간 산출물 크기 변화를 정량적으로 비교할 수 있어 이상 징후 감지가 쉬워짐.
- `artifact_sha256`와 함께 사용하면 내용 변경과 크기 변경을 동시에 확인 가능.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts16 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts16/migration_verify_report.json`에서:
  - `artifact_size_bytes.compile_report_json` 등 주요 항목이 숫자로 기록됨
  - non-bench 실행 기준 `artifact_size_bytes.bench_report_json = null`
  확인.
