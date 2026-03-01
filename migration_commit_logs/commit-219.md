# Commit 219 - Key-owner 정책 동기화 검증(compare) 추가

## 커밋 요약
- `localization_audit`에 생성 정책과 기존 정책 파일을 비교하는 검증 모드를 추가해 owner 정책 드리프트를 자동 감지.
- `migration_verify`에서 환경변수로 해당 비교를 켤 수 있도록 연동.

## 상세 변경
- `tools/localization_audit.py`
  - owner 정책 비교 유틸 추가:
    - `_extract_owner_map(payload)`
    - `_compare_owner_maps(expected, actual)`
  - CLI 옵션 추가:
    - `--compare-key-owner-policy <path>`
  - 비교 동작:
    - generated owners vs existing owners를 비교해 `missing/extra/changed` 집계 출력
    - 하나라도 차이가 있으면 non-zero 종료
    - 차이 키는 최대 10개까지 stderr로 미리보기 출력
- `tools/migration_verify.sh`
  - `MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY` 환경변수를 audit 호출 옵션으로 전달하도록 추가.

## 기능 영향
- 로컬라이제이션 중복 키 canonical 정책이 데이터 변화와 어긋나는 시점을 검증 단계에서 즉시 탐지 가능.
- 정책 파일 수동 수정/자동 생성 이후 drift 여부를 CI에서 일관되게 강제할 수 있음.

## 검증
- `python3 tools/localization_audit.py --project-root . --compare-key-owner-policy localization/key_owners.json` 통과.
  - `missing=0 extra=0 changed=0`
- `MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY=localization/key_owners.json tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
