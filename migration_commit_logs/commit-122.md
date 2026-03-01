# Commit 122 - ChroniclePanel 이벤트 설명 캐시 도입

## 커밋 요약
- ChroniclePanel의 이벤트 설명 문자열을 프레임마다 재계산하지 않도록 로컬 캐시를 추가.

## 상세 변경
- `scripts/ui/panels/chronicle_panel.gd`
  - `_desc_cache`, `_desc_cache_signature` 필드 추가.
  - `_invalidate_desc_cache()` 추가:
    - locale 변경/필터 변경 시 캐시 무효화.
  - `_ensure_desc_cache(events)` 추가:
    - 현재 이벤트 집합 서명(`locale/filter/size/first/last`) 기준으로 캐시 재생성 여부 결정.
  - `_compute_desc_cache_signature(events)` 추가:
    - 이벤트 리스트의 안정 시그니처 생성.
  - `_resolve_event_description(evt)` 추가:
    - 기존 `l10n_key/l10n_params/cause_id` 처리 + 길이 제한(55) 로직을 캐시 생성 단계로 이동.
  - `_draw()`에서 이벤트 루프 진입 전 `_ensure_desc_cache(events)` 호출 후,
    - 각 이벤트는 캐시된 설명 문자열을 우선 사용.

## 기능 영향
- Chronicle 패널 표시 결과는 기존과 동일.
- 패널 가시 상태에서 `queue_redraw()`가 반복되는 상황에서 `Locale.trf`/문자열 조립 호출을 줄여 렌더 경로의 CPU 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=450.0`, `checksum=13761358.00000`
