# Commit 078 - trf 경로 key-id 캐시 적용

## 커밋 요약
- `Locale.trf()` 포맷 문자열 조회를 key-id 캐시 기반으로 최적화.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_trf_key_id_cache` 추가.
  - locale 로드 시 `_trf_key_id_cache.clear()`로 캐시 초기화.
  - `trf(key, params)`에서:
    - key id를 1회 해석 후 캐시에 저장.
    - `ltr_id()`를 우선 사용해 포맷 텍스트 조회.
    - 미지원/빈 결과는 기존 `ltr(key)` fallback 유지.
  - placeholder 치환 동작(`{name}` 형태)은 기존과 동일.

## 기능 영향
- `trf()` 출력 문자열은 기존과 동일.
- 날짜/이벤트 등 반복 포맷 구간에서 key 문자열 lookup 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=473.1`, `checksum=13761358.00000`
