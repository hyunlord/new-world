# Commit 017 - Locale.tr_data 경고 오버헤드 완화

## 커밋 요약
- `Locale.tr_data()`의 deprecated 경고를 매 호출 출력에서 1회 출력으로 변경.
- key-first 조회(`name_key`, `desc_key`, `*_key`) 경로는 유지.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_tr_data_warned: bool` 상태 변수 추가.
  - `tr_data()` 내부 `push_warning(...)`를 최초 1회만 실행하도록 변경.

## 기능 영향
- 기존 동작/결과는 동일.
- `tr_data()` 반복 호출 구간에서 warning/log 오버헤드 감소.
- 콘솔 로그 노이즈 감소로 디버깅 가독성 향상.

## 검증
- `python3 tools/localization_audit.py --project-root . --strict` 통과(exit=0)
