# Commit 250 - verify 메타 리포트에 git 컨텍스트 추가

## 커밋 요약
- `migration_verify_report.json`에 git 컨텍스트(`git_branch`, `git_head`, `git_dirty`)를 추가해 실행 결과를 정확한 코드 상태와 연결할 수 있도록 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - verify report 생성 시 git 메타 수집:
    - `git rev-parse --abbrev-ref HEAD` -> branch
    - `git rev-parse HEAD` -> commit SHA
    - `git status --porcelain` -> dirty 여부
  - 수집값을 `migration_verify_report.json`에 추가:
    - `git_branch` (string or null)
    - `git_head` (string or null)
    - `git_dirty` (boolean)
  - git 정보 조회 실패 시 null 안전 처리 유지.

## 기능 영향
- 검증 결과 아티팩트를 특정 커밋/브랜치 상태와 직접 매핑할 수 있어 회귀 원인 추적이 빨라짐.
- 워킹트리 변경이 섞인 상태(`git_dirty=true`)인지도 보고서에서 즉시 확인 가능.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts12 tools/migration_verify.sh` 통과.
  - `/tmp/worldsim_audit_artifacts12/migration_verify_report.json`에서
    - `git_branch="lead/main"`
    - `git_head=<SHA>`
    - `git_dirty=true`
    확인.
