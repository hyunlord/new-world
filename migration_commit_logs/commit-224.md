# Commit 224 - Compiled localization meta에 owner 규칙 통계 포함

## 커밋 요약
- `localization_compile` 산출물(`localization/compiled/*.json`)의 `meta`에 owner policy 관련 통계를 추가해, runtime/툴링에서 정책 적용 상태를 직접 확인 가능하도록 확장.

## 상세 변경
- `tools/localization_compile.py`
  - `meta` 필드 확장:
    - `key_owners_path`
    - `owner_rule_seen_count`
    - `owner_rule_hit_count`
    - `owner_rule_miss_count`
    - `owner_rule_override_count`
- `localization/compiled/ko.json`, `localization/compiled/en.json`
  - 위 신규 meta 필드 반영으로 산출물 갱신.

## 기능 영향
- 컴파일 로그를 보지 않아도 산출물 자체에서 owner 정책 적용률/누락 여부를 확인 가능.
- 빌드 아티팩트 기반 QA/배포 점검 시 정책 신뢰도를 빠르게 체크할 수 있음.

## 검증
- `python3 tools/localization_compile.py --project-root .` 통과.
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
