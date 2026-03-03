# Commit 079 - compiled locale sources 기본 제외

## 커밋 요약
- localization compiled 산출물에서 런타임 미사용 `sources` 필드를 기본 제외하도록 컴파일러를 조정.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 기본값에 `include_sources: false` 추가.
  - compile 출력 `meta`에 `include_sources` 상태 기록.
  - `include_sources=true`일 때만 `sources` 필드를 산출물에 포함하도록 변경.
- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - 기본 컴파일 결과에서 `sources` 제거 상태로 재생성.

## 기능 영향
- 런타임 로더(`Locale`)는 `strings`/`keys`만 사용하므로 동작 변화 없음.
- compiled JSON 크기 축소:
  - `en.json`: `572,928` → `399,555` bytes
  - `ko.json`: `583,909` → `410,536` bytes
- 로드 I/O 및 파싱 데이터량이 감소해 locale 로드 효율이 개선.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=468.5`, `checksum=13761358.00000`
