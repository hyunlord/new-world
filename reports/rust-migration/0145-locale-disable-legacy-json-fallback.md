# 0145 - locale disable legacy json fallback

## Commit
- `[rust-r0-245] Disable legacy per-category locale JSON fallback path`

## 변경 파일
- `scripts/core/simulation/locale.gd`
  - `USE_FLUENT_RUNTIME_DEFAULT`를 `true`로 변경해 manifest 없이도 Fluent 런타임 우선 동작.
  - `load_locale()`에서 `fluent -> compiled` 실패 시 실행되던 `localization/<locale>/*.json` 카테고리 순회 로딩 경로 제거.
  - fluent/compiled 모두 실패하면 경고를 남기고 빈 인덱스를 유지하도록 정리.
  - `_categories`는 manifest 호환 입력 유지 목적(런타임 로딩 미사용)으로 주석 명확화.
- `reports/rust-migration/README.md`
  - 0145 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 API 시그니처 변경 없음.
- 로컬라이제이션 런타임 로딩 정책 변경:
  - 제거: 카테고리별 레거시 JSON 직접 로딩 fallback
  - 유지: Rust Fluent 런타임 + compiled JSON fallback

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 로컬라이제이션은 이제 런타임에서 Fluent/compiled 자산만 사용한다.
- 남은 로컬라이제이션 전환 잔여는 “데이터 원본/툴 체인에서 레거시 JSON 소스 제거”다.
