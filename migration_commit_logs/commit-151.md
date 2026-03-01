# Commit 151 - localization compiled payload 중복 키 배열 제거 옵션 도입

## 커밋 요약
- compiled locale 파일에서 중복 `keys` 배열을 분리할 수 있도록 `embed_keys` 옵션을 도입하고, `Locale` 런타임이 `key_registry.json`에서 키 순서를 읽어 인덱스를 재구성하도록 개선.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 옵션 `embed_keys`(기본 `false`) 추가.
  - `embed_keys=false`일 때 compiled locale 출력에서 `keys` 필드 미포함.
  - meta에 `embed_keys`를 기록.

- `localization/manifest.json`
  - `embed_keys: false` 명시.

- `scripts/core/simulation/locale.gd`
  - `key_registry_path` manifest 값을 읽도록 확장.
  - `_load_key_registry_keys()` 추가:
    - `localization/key_registry.json`의 `keys` 배열을 로드/캐시.
  - compiled locale 로드시:
    - `root.keys`가 없으면 registry keys를 사용해 `_rebuild_key_index` 수행.
  - 결과적으로 compiled locale가 `keys`를 포함하지 않아도 안정 key-id 인덱스 유지.

- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - `keys` 배열 제거(중복 제거).
  - meta에 `embed_keys=false` 반영.

## 기능 영향
- 번역 조회(`Locale.ltr`, `trf`, `tr_id`) 의미는 유지.
- locale payload당 중복 키 배열이 제거되어 로컬라이제이션 산출물 크기/중복이 감소.
- key-id 인덱스는 registry 기준으로 유지되어 확장 시 안정성 지속.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 79 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=471.3`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=190.5`, `checksum=29743414.00000`
