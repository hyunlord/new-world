# Commit 115 - 동적 로컬라이징 파라미터 복사 최소화

## 커밋 요약
- 동적 `l10n_params` 렌더 경로에서 불필요한 `Dictionary.duplicate()`를 제거하고, 실제 수정이 필요한 경우에만 복사하도록 변경.

## 상세 변경
- `scripts/ui/panels/chronicle_panel.gd`
  - `l10n_params`를 기본 참조로 사용.
  - `cause_id`가 있는 경우에만 `l10n_params_with_cause`를 `duplicate()` 후 `"cause"`를 주입해 `Locale.trf` 호출.
  - `cause_id`가 없으면 복사 없이 원본 파라미터로 바로 `Locale.trf` 호출.

- `scripts/ui/panels/entity_detail_panel.gd`
  - 개인 이벤트 렌더 시 `l10n_params`를 매 프레임 `duplicate()`하지 않고 그대로 `Locale.trf`에 전달.

## 기능 영향
- Chronicle/Entity Detail의 이벤트 설명 텍스트 출력은 기존과 동일.
- 이벤트 목록 렌더 루프에서 불필요한 Dictionary 복사를 줄여 메모리 churn과 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=460.9`, `checksum=13761358.00000`
