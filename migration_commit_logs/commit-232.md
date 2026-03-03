# Commit 232 - Localization audit에 owner policy 요약 지표 추가

## 커밋 요약
- `localization_audit` 리포트에 owner policy 품질 지표를 포함해, audit 단계만으로도 정책 커버 상태를 확인할 수 있게 확장.

## 상세 변경
- `tools/localization_audit.py`
  - run_audit에 owner policy 통계 추가:
    - `owner_policy_entry_count`
    - `owner_policy_missing_duplicate_count`
    - `owner_policy_unused_count`
    - `owner_policy_missing_duplicate_keys`
    - `owner_policy_unused_keys`
    - `owner_policy_path`
  - 통계 산출 방식:
    - 중복 키 union 대비 owner 미지정 수 집계
    - 전체 localization key union 대비 owner 미사용 키 집계
  - 콘솔 출력 확장:
    - 상단 요약에 owner policy 지표 3종 표시
    - 누락/미사용 키가 있을 때 상위 목록 출력

## 기능 영향
- compile 로그를 보지 않아도 audit에서 owner 정책 품질(커버 누락/미사용)을 즉시 파악 가능.
- `migration_verify` strict audit 출력에서 owner 정책 상태를 함께 확인할 수 있어 운영 가시성이 개선.

## 검증
- `python3 tools/localization_audit.py --project-root . --compare-key-owner-policy-auto` 통과.
  - owner_policy_entries=248
  - owner_policy_missing_duplicates=0
  - owner_policy_unused=0
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
