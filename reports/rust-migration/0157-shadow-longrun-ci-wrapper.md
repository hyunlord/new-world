# 0157 - shadow longrun CI wrapper

## Commit
- `[rust-r0-257] Add CI wrapper for migration verify shadow longrun`

## 변경 파일
- `tools/migration_verify_shadow_ci.sh`
  - CI/자동화 실행용 래퍼 스크립트 추가.
  - `migration_verify.sh --with-shadow-longrun`를 표준 환경변수 세트로 호출.
  - `GODOT_BIN` 필수 검증, report 경로/프레임 수 기본값 제공.
  - 산출물:
    - shadow report JSON
    - migration verify report JSON

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 런타임 API/시그널/세이브 스키마 변경 없음.
- 운영 스크립트 인터페이스 추가:
  - `tools/migration_verify_shadow_ci.sh`

## 검증 결과
- `bash -n tools/migration_verify_shadow_ci.sh` ✅
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- CI에서 shadow 장기검증을 표준 커맨드 하나로 호출 가능해졌다.
- 실제 longrun 실행은 `GODOT_BIN`이 제공된 환경에서 수행된다.
