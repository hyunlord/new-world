# Commit 152 - localization compile 변경 감지 쓰기 추가

## 커밋 요약
- localization 컴파일러에 “내용 동일 시 파일 미갱신” 경로를 추가해 반복 검증/개발 중 불필요한 파일 rewrite와 git noise를 줄임.

## 상세 변경
- `tools/localization_compile.py`
  - `_write_json_if_changed(path, data)` 추가:
    - JSON 렌더링 결과가 기존 파일과 동일하면 쓰기 생략
    - 다를 때만 파일 갱신
  - key registry 쓰기와 compiled locale 출력 쓰기를 `_write_json_if_changed`로 전환.
  - compile 로그에 `updated=0|1` 상태 출력 추가.

## 기능 영향
- localization 산출물 내용이 바뀌지 않으면 파일 timestamp/내용이 유지됨.
- CI/로컬 검증 반복 시 변경 없는 compiled 파일 재작성 방지.
- 번역 결과와 런타임 동작은 기존과 동일.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 79 tests)
  - localization compile: `updated=0` 확인
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=472.1`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=186.5`, `checksum=29743414.00000`
