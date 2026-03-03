# 0147 - fluent only audit and verify alignment

## Commit
- `[rust-r0-247] Align migration verify/audit pipeline with fluent-only localization`

## 변경 파일
- `localization/manifest.json`
  - `source_format`을 `fluent_preferred`에서 `fluent`로 상향 고정.
- `localization/compiled/en.json`
  - `meta.source_format`를 `fluent`로 갱신.
- `localization/compiled/ko.json`
  - `meta.source_format`를 `fluent`로 갱신.
- `tools/migration_verify.sh`
  - manifest의 `source_format`을 읽어 `json|fluent|fluent_preferred` 검증.
  - `source_format!=json`이고 key-field 적용 옵션이 없으면 `data_localization_extract.py` 단계를 자동 스킵.
  - Fluent 전환 상태에서 불필요한 JSON 추출 의존 제거.
- `tools/localization_audit.py`
  - `source_format` 인지형 감사 모드 추가 (`json` / `compiled`).
  - `fluent|fluent_preferred`에서는 `localization/compiled/<locale>.json` 기준으로 parity/duplicate 집계.
  - 리포트 필드 추가:
    - `localization_source_format`
    - `localization_audit_mode`
  - compiled 모드의 owner policy compare는 기존 `key_owners.json`을 기준으로 비교하도록 정렬.
- `reports/rust-migration/README.md`
  - 0147 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 런타임 API/시그널 변경 없음.
- 감사 리포트 JSON 확장:
  - 추가: `localization_source_format`, `localization_audit_mode`

## 검증 결과
- `bash tools/migration_verify.sh` ✅
  - rust workspace tests 통과
  - data extraction 단계 fluent 모드에서 skip 확인
  - localization compile 통과
  - strict audit 통과 (`source_format=fluent`, `audit_mode=compiled`)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- localization 전환의 검증 체인이 이제 소스 포맷(`fluent`)과 일치한다.
- 남은 로컬라이제이션 잔여는 운영/데이터 정리 관점(레거시 `localization/en|ko/*.json` 아카이브/제거 정책 확정)이다.
