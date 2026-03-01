# Commit 073 - localization stable key index 도입

## 커밋 요약
- localization compiled 산출물에 stable key index(`keys`)를 추가하고 런타임 `Locale`에 key↔id API를 도입.

## 상세 변경
- `tools/localization_compile.py`
  - locale 컴파일 결과에 정렬된 key 목록(`keys`) 포함.
  - meta에 `key_count` 추가.
- `scripts/core/simulation/locale.gd`
  - `_key_to_id`, `_id_to_value` 캐시 추가.
  - 신규 API 추가:
    - `has_key(key: String) -> bool`
    - `key_id(key: String) -> int`
    - `ltr_id(id: int) -> String`
  - compiled 로드 시 `keys`가 있으면 해당 순서를 사용해 stable id 인덱스 구성.
  - legacy category 로드 fallback 시 `_flat_strings` 기준 정렬 인덱스를 구성해 호환 유지.
- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - `keys` 배열과 `meta.key_count` 필드 포함 형태로 재생성.

## 기능 영향
- 기존 `Locale.ltr()` 호출 경로는 그대로 유지.
- 향후 Rust/GDExtension 브리지에서 문자열 key 대신 정수 id 기반 조회를 사용할 기반이 마련됨.
- locale 확장 시 key 목록/개수 추적이 쉬워져 운영 안정성과 검증성이 향상.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=459.8`, `checksum=13761358.00000`
