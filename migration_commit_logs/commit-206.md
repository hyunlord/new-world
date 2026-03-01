# Commit 206 - split path 벤치 기본 기준선 검증 강화

## 커밋 요약
- `migration_verify`의 split path 벤치가 기본 반복수(`path=100`)에서는 tuple/xy 체크섬 기준선까지 엄격 검증하도록 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - `run_path_split_and_check(expected_tuple, expected_xy, ...)` 추가.
    - split 벤치 출력의 tuple/xy checksum 파싱
    - 기준선(`35400.00000`, `35400.00000`) 불일치 시 실패
    - tuple/xy 상호 불일치 시 실패
  - `path_split=true` + `path_iters=100`에서는 strict baseline 체크 수행.
  - `path_split=true` + 비기본 반복수에서는 기존 관측 모드 유지.

## 기능 영향
- split 벤치가 기본 프로파일에서 회귀 게이트 역할을 수행.
- 비기본 반복수의 유연한 관측 흐름은 유지.

## 검증
- `MIGRATION_BENCH_PATH_SPLIT=true tools/migration_verify.sh --with-benches` 통과.
  - split checksum 기준선: tuple=`35400.00000`, xy=`35400.00000`
- `MIGRATION_BENCH_PATH_SPLIT=true MIGRATION_BENCH_PATH_ITERS=10 MIGRATION_BENCH_STRESS_ITERS=100 MIGRATION_BENCH_NEEDS_ITERS=100 tools/migration_verify.sh --with-benches` 통과.
  - split checksum 관측: tuple=`3540.00000`, xy=`3540.00000`
