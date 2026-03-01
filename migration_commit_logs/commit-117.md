# Commit 117 - Locale.trf 빈 파라미터 fast-path 추가

## 커밋 요약
- `Locale.trf`에 `params.is_empty()` 조기 반환 경로를 추가해 placeholder가 없는 호출의 치환 루프를 생략.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `trf(key, params)`에서 번역 텍스트를 결정한 직후:
    - `if params.is_empty(): return text` 추가.
  - 기존 key-id 캐시 및 fallback 로직은 유지.

## 기능 영향
- 출력 텍스트 의미는 기존과 동일.
- 동적 로컬라이징 호출 중 빈 파라미터 케이스에서 루프/문자열 치환 오버헤드를 줄임.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=449.6`, `checksum=13761358.00000`
