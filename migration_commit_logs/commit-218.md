# Commit 218 - Localization compile에 key-owner policy 적용

## 커밋 요약
- `localization_compile`이 key-owner 정책(`key_owners.json`)을 읽어 key별 canonical category를 우선 선택하도록 확장.
- 정책 미적용 시 기존 동작을 유지하고, 적용 시 owner hit/miss/override 통계를 제공해 운영 가시성을 강화.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 기본 필드 확장:
    - `key_owners_path` (기본: `key_owners.json`)
    - `max_owner_rule_miss_count` (기본: `null`)
  - owner 정책 로더 추가:
    - `_normalize_owner_category(...)`
    - `_load_key_owners(...)`
    - `owners` 또는 평면 dict 모두 지원.
  - locale 컴파일 로직 개선:
    - 기존 즉시 first-wins 처리 대신 key별 entry 수집 후 canonical 선택 수행.
    - owner 정책이 존재하면 해당 category entry를 우선 채택.
    - owner 통계 수집:
      - `owner_seen`
      - `owner_hits`
      - `owner_misses`
      - `owner_overrides`
  - 회귀 게이트 추가:
    - `max_owner_rule_miss_count` 초과 시 컴파일 실패.
  - 로그 출력에 owner 통계 포함.
- `localization/manifest.json`
  - `key_owners_path: "key_owners.json"` 추가.
  - `max_owner_rule_miss_count: 0` 추가.
- `localization/key_owners.json`
  - audit 제안 기반 canonical owner 정책 파일 추가(248 keys).

## 기능 영향
- 다중 파일 중복 키가 있어도 key별 canonical source를 정책으로 명시 가능해 확장 시 충돌 관리가 쉬워짐.
- owner 정책 품질(`owner_misses`)을 기준으로 CI/검증 게이트를 걸 수 있어 데이터 구조 안정성을 강화.
- 현재 데이터 기준 `owner_hits=248`, `owner_misses=0`으로 정책-데이터 정합 확인.

## 검증
- `python3 tools/localization_compile.py --project-root .` 통과.
  - ko: `owner_seen=248 owner_hits=248 owner_misses=0 owner_overrides=0`
  - en: `owner_seen=248 owner_hits=248 owner_misses=0 owner_overrides=0`
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
