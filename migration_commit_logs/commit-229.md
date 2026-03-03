# Commit 229 - Key-owner 정책 auto refresh 옵션 추가

## 커밋 요약
- `localization_audit`에 manifest 경로 기준 key-owner 정책 자동 갱신 옵션을 추가.
- `migration_verify`에서 환경변수로 해당 동작을 켜서 검증 중 정책 파일을 재생성할 수 있게 확장.

## 상세 변경
- `tools/localization_audit.py`
  - CLI 옵션 추가:
    - `--refresh-key-owner-policy-auto`
  - 동작:
    - `localization/manifest.json`의 `key_owners_path` 경로를 해석해 생성된 owner policy를 즉시 write.
    - refresh 결과를 로그로 출력.
  - 기존 compare 흐름과 함께 사용 가능 (`refresh -> compare`).
- `tools/migration_verify.sh`
  - 환경변수 추가:
    - `MIGRATION_AUDIT_REFRESH_KEY_OWNER_POLICY` (기본: `false`)
  - `true/false` 유효성 검사 추가.
  - 값이 `true`이면 audit 호출에 `--refresh-key-owner-policy-auto` 전달.

## 기능 영향
- 로컬 개발/데이터 업데이트 시 key-owner 정책 파일을 수동 갱신하지 않고 검증 과정에서 자동 동기화 가능.
- 기본 동작은 기존과 동일(갱신 비활성), CI에서는 drift 검증 게이트 유지.

## 검증
- `python3 tools/localization_audit.py --project-root . --refresh-key-owner-policy-auto --compare-key-owner-policy-auto` 통과.
- `MIGRATION_AUDIT_REFRESH_KEY_OWNER_POLICY=true tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
