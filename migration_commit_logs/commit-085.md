# Commit 085 - locale key-id 캐시 버전 무효화

## 커밋 요약
- locale 재로딩/전환 시 key-id 기반 정적 캐시가 자동 무효화되도록 버전 추적을 도입.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_key_index_version` 추가.
  - `load_locale()`에서 key index 재구성 완료 시 버전 증가.
  - `key_index_version() -> int` API 추가.
- `scripts/core/simulation/game_calendar.gd`
  - `_locale_key_version` 추가.
  - `_ensure_locale_key_cache()`를 통해 locale key index 버전 변경 시 age 라벨 key-id 캐시 초기화.
- `scripts/core/entity/emotion_data.gd`
  - `_intensity_key_cache_version` 추가.
  - `get_intensity_label()`에서 locale key index 버전 변경 감지 시 intensity key-id 캐시 초기화.

## 기능 영향
- 현재 번역 결과 문자열은 기존과 동일.
- locale 전환/재로딩 이후에도 key-id 캐시 오염 없이 안전하게 재해석됨.
- canonical key index 정책과 함께 캐시 안정성이 강화됨.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=512.2`, `checksum=13761358.00000`
