# Commit 167 - localization missing-fill 회귀 게이트 추가

## 커밋 요약
- localization 컴파일 단계에 `missing fill` 회귀 게이트를 추가해, 언어 확장 중 누락 번역 키 증가를 자동 차단.

## 상세 변경
- `tools/localization_compile.py`
  - `DEFAULT_MANIFEST`에 `max_missing_key_fill_count` 옵션 추가.
  - manifest에서 `max_missing_key_fill_count` 파싱/검증 로직 추가.
  - locale 컴파일 루프에서 `missing_filled_count`의 최대값(`max_locale_missing_filled`) 추적 추가.
  - 컴파일 종료 시 `max_locale_missing_filled > max_missing_key_fill_count`이면 실패하도록 회귀 게이트 추가.

- `localization/manifest.json`
  - `max_missing_key_fill_count: 0` 추가.
  - 현재 기준에서 locale 누락 채움(`filled`)이 증가하면 컴파일 단계에서 즉시 감지되도록 설정.

## 기능 영향
- 런타임 번역 동작은 변경 없음.
- localization 확장 시 번역 누락이 늘어나는 회귀를 CI/검증 단계에서 조기 차단 가능.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=736.0`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=566.6`, `checksum=38457848.00000`
