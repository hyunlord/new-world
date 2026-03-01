# Commit 217 - Localization audit key-owner policy JSON 내보내기 추가

## 커밋 요약
- `localization_audit`에 canonical 소스 제안을 JSON 정책 파일로 내보내는 옵션을 추가해, 중복 키 정리 정책을 기계적으로 재사용할 수 있는 기반을 마련.
- `migration_verify`에서 환경변수로 해당 정책 파일 산출 경로를 연결.

## 상세 변경
- `tools/localization_audit.py`
  - canonical 파일 선택 로직을 `_suggest_canonical_file(files)`로 공통화.
  - `--key-owner-policy-json` CLI 옵션 추가.
  - `duplicate_details`를 기반으로 key별 owner category를 생성하는 `_build_key_owner_policy_payload(report)` 추가.
    - 출력 포맷:
      - `version`
      - `duplicate_report_locale`
      - `owner_key_count`
      - `owners` (`KEY -> category`)
  - Markdown 충돌 리포트도 동일 canonical 선택 로직을 재사용하도록 정리.
- `tools/migration_verify.sh`
  - `MIGRATION_AUDIT_KEY_OWNER_POLICY` 환경변수를 audit 단계 옵션(`--key-owner-policy-json`)으로 전달하도록 추가.

## 기능 영향
- 중복 키 canonical 기준을 사람이 보는 Markdown뿐 아니라 JSON 정책 아티팩트로도 생성 가능.
- 이후 compile 단계에서 해당 정책을 자동 적용(혹은 검증)하는 흐름으로 확장하기 쉬운 구조를 확보.

## 검증
- `python3 tools/localization_audit.py --project-root . --key-owner-policy-json /tmp/worldsim-key-owners.json` 실행 확인.
  - `owner_key_count: 248` 생성 확인.
- `MIGRATION_AUDIT_KEY_OWNER_POLICY=/tmp/worldsim-key-owners-from-verify.json tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
