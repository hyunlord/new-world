# 0149 - shadow longrun automation entrypoint

## Commit
- `[rust-r0-249] Add configurable shadow smoke runtime override and longrun verifier`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - 런타임 모드 오버라이드 훅 추가:
    - `set_runtime_mode_override(mode: String)`
    - `_resolve_runtime_mode()`
    - `_is_supported_runtime_mode(mode: String)`
  - `_init_rust_runtime()`가 `GameConfig.SIM_RUNTIME_MODE` 고정 대신 override-aware 모드 해석 사용.
  - shadow reporter setup 블록 들여쓰기 정합성 수정.
- `tools/rust_shadow_smoke.gd`
  - 사용자 인자 파싱 추가 (`OS.get_cmdline_user_args()`):
    - `--seed=`
    - `--frames=`
    - `--delta=`
    - `--runtime-mode=`
    - `--report-path=`
  - 기본 런타임 모드를 `rust_shadow`로 설정하여 shadow report 생성 경로를 명시적으로 검증.
  - 실행 구성 출력(`SHADOW_SMOKE_CONFIG`) 및 report path 출력(`SHADOW_REPORT_PATH`) 유지.
- `tools/rust_shadow_longrun_verify.sh` (신규)
  - Godot headless smoke 실행 + `rust_shadow_cutover_check.py` 후속 검증을 한 번에 수행.
  - 기본값:
    - `SHADOW_FRAMES=10000`
    - `SHADOW_RUNTIME_MODE=rust_shadow`
  - `GODOT_BIN` 필수.
- `reports/rust-migration/README.md`
  - 0149 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 런타임 시그널 변경 없음.
- SimulationEngine 공개 메서드 추가:
  - `set_runtime_mode_override(mode: String) -> void`
- 새 자동화 도구:
  - `tools/rust_shadow_longrun_verify.sh`

## 검증 결과
- `bash -n tools/rust_shadow_longrun_verify.sh` ✅
- `bash tools/migration_verify.sh` ✅
  - rust workspace tests 통과
  - localization compile/audit 통과

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 이제 headless에서 10,000-frame shadow 장기 검증을 반복 실행할 수 있는 진입점이 생겼다.
- 남은 잔여는 실제 CI/job에 `GODOT_BIN`을 연결해 정기 실행/리포트 누적하는 운영 단계다.
