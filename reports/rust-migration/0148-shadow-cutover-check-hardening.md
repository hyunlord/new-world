# 0148 - shadow cutover check hardening

## Commit
- `[rust-r0-248] Harden shadow cutover gate checker and verify hook`

## 변경 파일
- `tools/rust_shadow_cutover_check.py`
  - `--required-min-frames` 옵션 추가 (기본: report의 `min_frames_for_cutover` 사용).
  - report 필드 해석 강화:
    - `max_work_delta` / `allowed_max_work_delta` 우선 사용
    - 하위 호환 alias (`max_event_delta`, `allowed_max_event_delta`) fallback 유지
  - 게이트별 판정 출력 추가:
    - frame/tick/work/mismatch/all gate
    - `remaining_frames_for_gate`
  - 종료 조건 강화:
    - `approved_for_cutover=true` 이면서 gate 전체 통과 시에만 0 반환.
- `tools/migration_verify.sh`
  - 선택적 shadow 게이트 검증 훅 추가:
    - `MIGRATION_SHADOW_REPORT_JSON` 설정 시 `rust_shadow_cutover_check.py` 실행
    - `MIGRATION_SHADOW_REQUIRED_MIN_FRAMES`로 최소 프레임 강제값 전달 가능
  - 기본 실행 경로(환경변수 미설정)는 기존과 동일.
- `reports/rust-migration/README.md`
  - 0148 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 런타임 API/시그널 변경 없음.
- 도구 인터페이스 변경:
  - `tools/rust_shadow_cutover_check.py --required-min-frames <n>`
  - `migration_verify.sh` 환경변수:
    - `MIGRATION_SHADOW_REPORT_JSON`
    - `MIGRATION_SHADOW_REQUIRED_MIN_FRAMES`

## 검증 결과
- `bash tools/migration_verify.sh` ✅
  - rust workspace tests 통과
  - localization compile/audit 통과
  - 기본 경로에서 shadow optional hook 미설정 시 회귀 없음 확인

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 장기 shadow 검증(10,000 frame 기준)을 CI/자동화 파이프라인에서 강제할 수 있는 훅을 확보했다.
- 남은 잔여는 실제 headless shadow 장기 런 실행(job) 연결 및 결과 리포트 누적이다.
