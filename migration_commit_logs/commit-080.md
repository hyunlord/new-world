# Commit 080 - tr_id 결과 캐시 추가

## 커밋 요약
- `Locale.tr_id()`에 최종 결과 캐시를 추가해 반복 호출 시 문자열 조합/대문자 변환/조회 비용을 줄임.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_tr_id_result_cache` 추가.
  - locale 로드 시 `_tr_id_result_cache.clear()`로 캐시 초기화.
  - `tr_id(prefix, id)`에서:
    - `prefix + "\n" + id` 키 기반으로 최종 번역 문자열 캐시를 우선 조회.
    - 캐시 miss 시 기존 key-id 경로로 계산 후 결과를 캐시에 저장.
    - 번역 미존재 fallback(`id` 반환)도 캐시에 저장해 반복 miss 비용 축소.

## 기능 영향
- `tr_id()`의 반환 의미는 기존과 동일.
- HUD/리스트/툴팁 등 동일 ID를 반복 조회하는 UI 구간의 호출 오버헤드를 추가 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=467.1`, `checksum=13761358.00000`
