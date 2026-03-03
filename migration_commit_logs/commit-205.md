# Commit 205 - migration_verify audit JSON 산출물 옵션 연결

## 커밋 요약
- `migration_verify.sh`가 localization audit 리포트를 JSON 파일로 저장할 수 있도록 `MIGRATION_AUDIT_REPORT_JSON`, `MIGRATION_AUDIT_DUPLICATE_REPORT_JSON` 환경변수 연동을 추가.

## 상세 변경
- `tools/migration_verify.sh`
  - strict audit 단계를 고정 명령에서 동적 `audit_cmd` 배열 실행으로 변경.
  - 환경변수 지원:
    - `MIGRATION_AUDIT_REPORT_JSON` -> `--report-json`
    - `MIGRATION_AUDIT_DUPLICATE_REPORT_JSON` -> `--duplicate-report-json`
  - 환경변수 미설정 시 기존 동작(콘솔 출력 + strict 판정) 유지.

## 기능 영향
- CI/로컬 검증 시 audit 결과를 아티팩트 파일로 남겨 추적/비교가 쉬워짐.
- 기본 경로는 회귀 없이 동일.

## 검증
- `tools/migration_verify.sh --with-benches` 통과.
- `MIGRATION_AUDIT_REPORT_JSON=/tmp/worldsim-audit.json MIGRATION_AUDIT_DUPLICATE_REPORT_JSON=/tmp/worldsim-audit-duplicates.json MIGRATION_BENCH_PATH_ITERS=10 MIGRATION_BENCH_STRESS_ITERS=100 MIGRATION_BENCH_NEEDS_ITERS=100 tools/migration_verify.sh --with-benches` 통과.
  - `/tmp/worldsim-audit.json` 생성 확인
  - `/tmp/worldsim-audit-duplicates.json` 생성 확인
