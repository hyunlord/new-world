# Commit 221 - Owner policy drift 검증을 migration_verify 기본 게이트로 승격

## 커밋 요약
- `localization_audit`에 manifest 기반 key-owner 정책 자동 비교 모드를 추가.
- `migration_verify`가 기본적으로 owner policy drift를 검증하도록 변경해, 별도 환경변수 없이 정책 일관성을 강제.

## 상세 변경
- `tools/localization_audit.py`
  - manifest 로딩/경로 해석 유틸 추가:
    - `_load_manifest_dict(project_root)`
    - `_resolve_manifest_key_owner_policy_path(project_root)`
  - CLI 옵션 추가:
    - `--compare-key-owner-policy-auto`
      - `localization/manifest.json`의 `key_owners_path`를 자동 대상 경로로 사용.
  - 기존 비교 로직을 확장해:
    - 명시 경로(`--compare-key-owner-policy`) 또는 auto 경로 중 하나를 선택
    - 차이(`missing/extra/changed`)가 있으면 non-zero 종료
- `tools/migration_verify.sh`
  - audit 기본 호출에 `--compare-key-owner-policy-auto`를 포함.
  - `MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY`가 설정된 경우, auto 대신 명시 경로 비교로 override.

## 기능 영향
- key-owner 정책 파일이 누락/불일치 상태면 검증 파이프라인이 즉시 실패해 drift를 조기 차단.
- 운영자는 기본 경로를 그대로 사용하거나, 필요 시 환경변수로 비교 대상을 바꿔 검증 가능.

## 검증
- `python3 tools/localization_audit.py --project-root . --compare-key-owner-policy-auto` 통과.
  - `missing=0 extra=0 changed=0`
- `tools/migration_verify.sh --with-benches` 통과(환경변수 없이).
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
