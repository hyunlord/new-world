# Commit 204 - migration_verify에 선택형 split path 벤치 관측 추가

## 커밋 요약
- `migration_verify.sh --with-benches`에 `MIGRATION_BENCH_PATH_SPLIT` 옵션을 추가해 `sim-test --bench-pathfind-bridge-split`를 선택적으로 실행하고 tuple/XY checksum 정합성을 확인하도록 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_BENCH_PATH_SPLIT` (`true|false`, 기본 `false`)
  - 벤치 설정 출력에 split 상태 포함.
  - `run_path_split_observe` 헬퍼 추가:
    - split 벤치 출력에서 tuple/XY checksum 2개 파싱
    - checksum 누락 시 실패
    - tuple/XY checksum 불일치 시 실패
  - `path_split=true`일 때 split 벤치를 `path_iters`와 동일 반복 수로 실행.
  - macOS 기본 bash 3.x 호환을 위해 `mapfile` 사용 없이 `sed` 기반 파싱 적용.

## 기능 영향
- 기본 실행(`split=false`)은 기존과 동일.
- 필요 시 path tuple/XY 경로를 검증 파이프라인에서 즉시 비교 관측 가능.

## 검증
- `tools/migration_verify.sh --with-benches` 통과 (기본 split=false).
- `MIGRATION_BENCH_PATH_SPLIT=true MIGRATION_BENCH_PATH_ITERS=10 MIGRATION_BENCH_STRESS_ITERS=100 MIGRATION_BENCH_NEEDS_ITERS=100 tools/migration_verify.sh --with-benches` 통과.
  - split 체크섬: tuple=`3540.00000`, xy=`3540.00000`
