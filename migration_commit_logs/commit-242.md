# Commit 242 - Bench 결과 JSON 아티팩트 출력 추가

## 커밋 요약
- `migration_verify --with-benches` 실행 결과를 구조화된 JSON으로 저장할 수 있도록 `MIGRATION_BENCH_REPORT_JSON` 출력 경로를 추가하고, report dir 사용 시 자동 생성되도록 연동.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_BENCH_REPORT_JSON`
  - `MIGRATION_AUDIT_REPORT_DIR`와 함께 사용할 때 기본 경로 자동 할당:
    - `bench_report.json`
  - 벤치 검증 함수 확장:
    - stress/needs checksum을 공용 변수로 보관.
    - split/smoke 검증 결과(체크섬/total/resolved)를 공용 변수로 보관.
  - 벤치 완료 후 JSON 아티팩트 출력:
    - 입력 조건: `path/stress/needs iters`, backend/split/smoke 플래그
    - path 결과: checksum/total/resolved
    - split 결과: tuple/xy checksum/total
    - smoke 결과: checksum/total_each/resolved_auto/resolved_gpu
    - stress/needs checksum
  - 출력 로그:
    - `[migration_verify] bench report written: <path>`

## 기능 영향
- benchmark 게이트를 단순 콘솔 로그가 아니라 파일 기반 지표로 보관/비교 가능.
- CI에서 벤치 체크섬/백엔드 해석 결과를 아티팩트 수집해 회귀 추적 자동화가 쉬워짐.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts4 MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
  - `/tmp/worldsim_audit_artifacts4/bench_report.json` 생성 확인.
  - 주요 값 확인:
    - path checksum `70800.00000`
    - path resolved `cpu`
    - smoke checksum `3540.00000`
    - stress checksum `24032652.00000`
    - needs checksum `38457848.00000`
