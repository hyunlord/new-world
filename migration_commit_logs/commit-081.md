# Commit 081 - locale 간 canonical key index 고정

## 커밋 요약
- localization compile이 모든 locale에 대해 동일한 canonical key 집합/순서를 사용하도록 조정.

## 상세 변경
- `tools/localization_compile.py`
  - locale별 1차 compile 결과를 먼저 수집한 뒤, 지원 locale 전체 union으로 canonical key 목록 생성.
  - 각 locale 출력 시 canonical key 목록을 공통 `keys`로 사용.
  - locale별 누락 키는 `en` fallback(또는 최종적으로 key 자체)으로 채워 key 집합 정합성 유지.
  - meta에 `missing_key_fill_count` 추가.
  - compile 로그에 `filled=<count>` 출력 추가.
- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - `missing_key_fill_count` 메타 포함 형태로 재생성.

## 기능 영향
- 현재 데이터에서는 `filled=0`으로 기존 문자열 결과와 동일.
- locale 전환 시에도 key-id 순서가 locale 간 고정되어 ID 캐시 정합성이 강화.
- 향후 특정 locale에만 키가 추가되더라도 canonical index 체계가 유지됨.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
  - localization compile 로그: `filled=0` 확인
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=496.9`, `checksum=13761358.00000`
