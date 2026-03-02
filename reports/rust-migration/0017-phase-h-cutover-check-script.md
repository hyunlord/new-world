# 0017 - Phase H cutover readiness check script

## Summary
Shadow 리포트(`latest.json`)를 기준으로 컷오버 승인 여부를 자동 판정하는 스크립트를 추가했다. CI/로컬에서 즉시 exit code로 승인 상태를 확인할 수 있다.

## Files Changed
- `tools/rust_shadow_cutover_check.py`
  - `--report` 경로 입력으로 shadow report JSON 로드
  - 핵심 지표 출력
    - `approved_for_cutover`
    - `frames`, `mismatch_frames`, `mismatch_ratio`
    - `max_tick_delta`, `max_event_delta`, 허용 임계치
  - 종료 코드
    - `0`: 승인됨 (`approved_for_cutover=true`)
    - `1`: 미승인
    - `2`: 파일/파싱 오류

## API / Signal / Schema Changes
- 없음 (운영 도구 추가)

## Verification
- `python3 tools/rust_shadow_cutover_check.py --help` : PASS
- `cd rust && cargo check -p sim-bridge` : PASS (직전 커밋에서 확인)
- `godot --headless --check-only` : 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 98% complete / 2% remaining
- Current: 99% complete / 1% remaining
- Delta: +1%

## Notes
- 남은 1%는 실제 shadow 리포트 수집 후 `approved_for_cutover=true` 검증과 default mode 전환 승인 절차다.
