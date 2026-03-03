# Commit 237 - Localization owner-policy Markdown 리포트 추가

## 커밋 요약
- `localization_audit`에 owner-policy 품질 전용 Markdown 리포트 출력을 추가하고, `migration_verify`에서 환경변수로 해당 아티팩트를 생성할 수 있도록 연동.

## 상세 변경
- `tools/localization_audit.py`
  - 신규 함수 ` _build_owner_policy_markdown(report)` 추가:
    - owner policy 경로/엔트리/누락/미사용 카운트 요약 출력
    - 이슈가 없으면 `No owner policy coverage issues found.` 출력
    - 이슈가 있으면 아래 섹션으로 키 목록 표 출력
      - `Missing Owner For Duplicate Keys`
      - `Unused Owner Keys`
  - 신규 CLI 옵션 추가:
    - `--owner-policy-markdown <path>`
  - 옵션이 주어지면 owner-policy markdown 파일을 write하도록 main 경로 확장.

- `tools/migration_verify.sh`
  - 신규 환경변수 추가:
    - `MIGRATION_AUDIT_OWNER_POLICY_MARKDOWN`
  - audit 명령 구성 시 값이 있으면 `--owner-policy-markdown` 인자를 전달하도록 연동.
  - compare override 분기(`MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY`)에서도 동일 전달을 보장.

## 기능 영향
- duplicate conflict 리포트와 별개로 owner policy 커버리지 상태를 독립 아티팩트로 생성 가능.
- CI/운영 환경에서 owner-policy drift/누락을 사람이 바로 확인 가능한 형태로 보관하기 쉬워짐.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_OWNER_POLICY_MARKDOWN=/tmp/worldsim_owner_policy_report.md tools/migration_verify.sh` 통과.
  - `/tmp/worldsim_owner_policy_report.md` 생성 확인.
  - 현재 상태 요약:
    - `owner_policy_entries=248`
    - `missing_for_duplicates=0`
    - `owner_unused=0`
    - `No owner policy coverage issues found.`
