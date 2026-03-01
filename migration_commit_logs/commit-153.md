# Commit 153 - localization duplicate key 회귀 가드 추가

## 커밋 요약
- localization manifest에 중복 키 상한선을 추가하고, 컴파일 단계에서 로케일별 중복 키가 기준선을 넘으면 실패하도록 회귀 가드를 도입.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 옵션 `max_duplicate_key_count`(기본 `null`) 추가.
  - manifest의 `max_duplicate_key_count`를 읽어 정수로 검증.
  - 로케일별 duplicate key 개수 중 최대값(`max_locale_duplicates`)을 추적.
  - `max_locale_duplicates > max_duplicate_key_count`면 non-zero 종료.
  - 오류 메시지를 `max_locale_duplicates / max_allowed` 형식으로 출력.

- `localization/manifest.json`
  - `max_duplicate_key_count: 248` 추가 (현재 기준선 고정).

## 기능 영향
- 현재 상태(로케일별 중복 248)는 통과.
- 이후 신규 작업에서 중복 키가 248을 초과하면 compile 단계에서 즉시 감지/실패.
- 기존 번역 로딩/조회 동작은 변경 없음.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 79 tests)
  - localization compile `updated=0`, duplicates 유지
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=447.7`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=231.8`, `checksum=29743414.00000`
