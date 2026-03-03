# Commit 231 - Owner policy 중복 커버/미사용 게이트 추가

## 커밋 요약
- `localization_compile`에 owner policy 품질 지표를 확장해 중복 키 커버 누락과 미사용 owner 항목을 검출/차단 가능하게 개선.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 설정 확장:
    - `max_owner_unused_count`
    - `max_duplicate_owner_missing_count`
  - owner policy 집계 추가:
    - `duplicate_key_union` (로케일 전체 중복 키 집합)
    - `duplicate_owner_missing_count` (중복 키 중 owner 미지정)
    - `owner_unused_count` (owner 정책에만 있고 실제 key에 없는 항목)
  - 컴파일 로그에 owner-policy 요약 출력:
    - `entries`, `duplicate_keys`, `missing_for_duplicates`, `unused`
  - 산출물 meta 확장:
    - `owner_policy_entry_count`
    - `owner_policy_missing_duplicate_count`
    - `owner_policy_unused_count`
  - 임계치 초과 시 컴파일 실패 게이트 추가.
- `localization/manifest.json`
  - `max_owner_unused_count: 0`
  - `max_duplicate_owner_missing_count: 0`
  - owner policy 품질 게이트를 기본 활성화.
- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - 신규 meta 필드 반영.

## 기능 영향
- owner 정책이 실제 중복 키를 100% 커버하는지 자동 검증 가능.
- 오래된/불필요한 owner 항목 누적을 차단해 정책 파일 품질을 지속적으로 유지.
- 현재 상태 기준:
  - `entries=248`
  - `duplicate_keys=248`
  - `missing_for_duplicates=0`
  - `unused=0`

## 검증
- `python3 tools/localization_compile.py --project-root .` 통과.
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
