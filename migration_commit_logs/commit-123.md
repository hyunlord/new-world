# Commit 123 - EntityDetail Life Events 설명 캐시

## 커밋 요약
- EntityDetailPanel의 deceased `life_events` 섹션에서 이벤트 설명 문자열을 프레임마다 재생성하지 않도록 캐시를 추가.

## 상세 변경
- `scripts/ui/panels/entity_detail_panel.gd`
  - `_life_event_desc_cache`, `_life_event_desc_signature` 필드 추가.
  - `_invalidate_life_event_desc_cache()` 추가:
    - entity 전환, deceased 표시 전환, locale 변경 시 캐시 초기화.
  - `_ensure_life_event_desc_cache(entity_id, events)` 추가:
    - 현재 `entity_id + locale + events(first/last/size)` 시그니처 기준으로 캐시 갱신 여부 판단.
  - `_compute_life_event_desc_signature(entity_id, events)` 추가.
  - `_resolve_life_event_description(evt)` 추가:
    - `l10n_key/l10n_params` 기반 `Locale.trf` 처리 + 길이 제한(50) 적용.
  - `life_events` draw 루프:
    - 기존 반복 `Locale.trf` 호출 대신 캐시된 desc 사용.

## 기능 영향
- Entity Detail의 Life Events 텍스트 출력은 기존과 동일.
- 패널 redraw 루프에서 동적 로컬라이징 문자열 재생성을 줄여 UI 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=462.7`, `checksum=13761358.00000`
