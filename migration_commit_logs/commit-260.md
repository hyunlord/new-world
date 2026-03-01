# Commit 260 - verify 리포트 문자열 JSON 이스케이프 보강

## 커밋 요약
- `migration_verify_report.json` 생성 시 문자열 직렬화를 안전하게 처리하도록 JSON escape 경로를 추가.

## 상세 변경
- `tools/migration_verify.sh`
  - `to_json_string(raw)` 헬퍼 추가:
    - `python3 -c 'json.dumps(...)'` 기반으로 문자열을 안전 JSON 문자열로 인코딩.
  - 기존 문자열 직렬화 지점 교체:
    - `to_json_opt_string` -> 안전 인코딩 사용
    - `to_json_opt_path` -> 안전 인코딩 사용
    - `to_json_opt_sha256` -> 안전 인코딩 사용
  - git 메타 직렬화 정리:
    - 수동 따옴표 조립 대신 `to_json_opt_string` 사용으로 일관화.

## 기능 영향
- 경로/브랜치명/버전 문자열에 특수문자(따옴표, 백슬래시 등)가 포함되어도 verify report JSON이 깨지지 않음.
- 문자열 필드 직렬화 방식이 단일 경로로 통합되어 유지보수성 향상.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts22 tools/migration_verify.sh` 통과.
- `python3` JSON 로드 검증:
  - `/tmp/worldsim_audit_artifacts22/migration_verify_report.json` 파싱 성공
  - `git_branch`, `git_head`, `toolchain.cargo` 필드 정상 조회 확인.
