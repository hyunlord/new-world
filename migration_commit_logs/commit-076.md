# Commit 076 - emotion intensity label key-id 캐시 적용

## 커밋 요약
- `EmotionData.get_intensity_label()`에 locale key id 캐시를 추가해 반복 라벨 조회 경로를 최적화.

## 상세 변경
- `scripts/core/entity/emotion_data.gd`
  - 정적 캐시 `_intensity_key_id_cache` 추가.
  - 강도 라벨 키(`EMO_<ID>_(MILD|BASE|INTENSE)`) 조회 시:
    - 캐시에 key id가 없으면 `Locale.key_id(key)`로 1회 해석 후 저장.
    - key id가 있으면 `Locale.ltr_id(id)` 우선 사용.
    - 미지원/빈 결과는 기존 `Locale.ltr(key)` fallback 유지.

## 기능 영향
- 감정 강도 라벨 결과는 기존과 동일.
- UI에서 반복 호출되는 강도 라벨 조회의 key 문자열 lookup 비용을 줄임.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=459.9`, `checksum=13761358.00000`
