# Commit 150 - localization append-only key registry 도입

## 커밋 요약
- 로컬라이제이션 컴파일 파이프라인에 append-only key registry를 도입해, 확장 시에도 key ID 순서가 안정적으로 유지되도록 구조를 개선.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 기본값에 아래 항목 추가:
    - `key_registry_path` (기본: `key_registry.json`)
    - `preserve_key_ids` (기본: `true`)
  - key registry 로직 추가:
    - `_load_key_registry(path)`
    - `_build_key_registry(canonical_keys, existing_registry_keys, preserve_key_ids)`
    - `_write_key_registry(path, keys, active_keys)`
  - compile 단계에서:
    - locale 집계 canonical key set 생성
    - 기존 registry를 읽어 append-only 병합
    - `localization/key_registry.json` 갱신
    - compiled locale의 `keys`를 registry 순서로 출력
  - compiled meta 확장:
    - `active_key_count`, `key_registry_path`, `preserve_key_ids`

- `localization/manifest.json`
  - `key_registry_path`, `preserve_key_ids` 항목 추가.

- `localization/key_registry.json` (신규)
  - 현재 전체 key의 안정 ID 맵(`key_to_id`)과 keys 배열을 저장.
  - 키 제거 시에도 ID 보존 가능하도록 `removed_keys`/카운트 메타 포함.

- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - meta에 registry 관련 필드 반영.
  - keys 배열 기준이 canonical 정렬이 아닌 registry 순서로 고정.

## 기능 영향
- 런타임 lookup 결과(`Locale.ltr`)는 기존과 동일.
- 신규 키 추가 시 기존 key ID가 재배열되지 않아, 확장/마이그레이션 시 안정성이 향상.
- 컴파일 산출물에 key registry 메타가 포함되어 빌드/디버그 추적성이 향상.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 79 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=467.1`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=185.4`, `checksum=29743414.00000`
