# Commit 243 - Bench report JSON 타입 정규화(null/number)

## 커밋 요약
- `migration_verify`가 생성하는 `bench_report.json`의 선택 필드 타입을 정규화해, 값 없음은 빈 문자열 대신 `null`, 수치 필드는 숫자로 출력되도록 개선.

## 상세 변경
- `tools/migration_verify.sh`
  - bench report 생성 전에 JSON 값 정규화 변수 추가:
    - path resolved: string or `null`
    - split tuple/xy checksum: string or `null`
    - split tuple/xy total: number or `null`
    - smoke checksum: string or `null`
    - smoke total_each: number or `null`
    - smoke resolved_auto/resolved_gpu: string or `null`
  - 기존 빈 문자열 출력을 제거하고 JSON 타입 의미를 명확하게 반영.

## 기능 영향
- downstream 파서/대시보드에서 `""` 처리 분기 없이 `null` 기반으로 결측값 처리 가능.
- bench report가 스키마적으로 더 안정되어 회귀 비교 자동화 시 타입 오류 가능성을 줄임.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts5 MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
- `python3 -m json.tool /tmp/worldsim_audit_artifacts5/bench_report.json`로 JSON 유효성 및 타입 확인:
  - `path_split.* = null` (split 비활성 시)
  - `path_smoke.total_each = 10` (숫자)
  - `path.resolved = "cpu"` (문자열)
