# 0150 - shadow longrun hook in migration verify

## Commit
- `[rust-r0-250] Add optional shadow longrun stage to migration verify`

## 변경 파일
- `tools/migration_verify.sh`
  - CLI 옵션 추가:
    - `--with-shadow-longrun`
  - shadow longrun 실행 단계 추가(옵션):
    - `tools/rust_shadow_longrun_verify.sh` 호출
    - `GODOT_BIN` 또는 `MIGRATION_SHADOW_GODOT_BIN` 필수
    - 환경변수:
      - `MIGRATION_SHADOW_LONGRUN_SEED` (default `20260302`)
      - `MIGRATION_SHADOW_LONGRUN_FRAMES` (default `10000`)
      - `MIGRATION_SHADOW_LONGRUN_DELTA` (default `0.1`)
      - `MIGRATION_SHADOW_LONGRUN_RUNTIME_MODE` (default `rust_shadow`)
      - `MIGRATION_SHADOW_LONGRUN_REPORT_PATH` (optional)
  - 기존 `MIGRATION_SHADOW_REPORT_JSON` 기반 cutover check와 연동:
    - longrun report path가 지정되면 shadow cutover check 입력으로 자동 연결
  - verify report JSON 확장:
    - `with_shadow_longrun`
    - `config.shadow.*`
    - `timings_seconds.shadow_cutover_check`
    - `timings_seconds.shadow_longrun`
    - `artifacts.shadow_report_json`
    - `verification_status.shadow_cutover_check_executed`
    - `verification_status.shadow_report_present_when_checked`
    - `verification_status.shadow_longrun_executed`
    - artifact hash/size/mtime/exists에 `shadow_report_json` 추가

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 런타임 API/시그널 변경 없음.
- 검증 스크립트 인터페이스 변경:
  - `tools/migration_verify.sh --with-shadow-longrun`
  - 관련 `MIGRATION_SHADOW_LONGRUN_*` 환경변수

## 검증 결과
- `bash -n tools/migration_verify.sh` ✅
- `bash tools/migration_verify.sh` ✅
- `MIGRATION_VERIFY_REPORT_JSON=/tmp/worldsim-migration-verify.json bash tools/migration_verify.sh` ✅
- `python3 -m json.tool /tmp/worldsim-migration-verify.json` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- migration_verify에서 longrun shadow 검증을 직접 트리거할 수 있게 되어 운영 자동화 연결 비용이 줄었다.
- 남은 잔여는 CI 실행 환경에 `GODOT_BIN`을 고정해 정기 스케줄로 장기 검증을 붙이는 단계다.
