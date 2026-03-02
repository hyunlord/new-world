# 0146 - localization compile fluent preferred

## Commit
- `[rust-r0-246] Switch localization compile pipeline to Fluent-preferred sources`

## 변경 파일
- `tools/localization_compile.py`
  - manifest `source_format`(`json`, `fluent`, `fluent_preferred`) 지원 추가.
  - 기본 소스 포맷을 `fluent_preferred`로 변경.
  - `localization/fluent/<locale>/messages.ftl` 파서/로더 추가.
  - Fluent 소스가 있으면 compiled JSON을 Fluent에서 생성하고, `fluent_preferred`일 때만 JSON 카테고리 fallback 허용.
  - compiled meta/report에 `source_format` 필드 추가.
- `localization/manifest.json`
  - `source_format: "fluent_preferred"` 명시.
- `localization/compiled/en.json`
  - 컴파일 메타데이터를 Fluent 기반 결과로 갱신 (`duplicate_key_count`, `duplicate_conflict_count`, owner-rule 카운터, `source_format`).
- `localization/compiled/ko.json`
  - 컴파일 메타데이터를 Fluent 기반 결과로 갱신 (`duplicate_key_count`, `duplicate_conflict_count`, owner-rule 카운터, `source_format`).
- `reports/rust-migration/README.md`
  - 0146 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 런타임 API/시그널 변경 없음.
- Localization compile 산출물 메타 스키마 확장:
  - 추가: `meta.source_format`

## 검증 결과
- `python3 tools/localization_compile.py --project-root . --report-json reports/rust-migration/data/localization-compile-report.json` ✅
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 런타임(`locale.gd`)에 이어 컴파일 툴체인도 Fluent 우선 경로로 정렬됐다.
- 남은 잔여는 `localization/en|ko/*.json` 원본 데이터를 Fluent 단일 소스로 완전 정리하는 단계다.
