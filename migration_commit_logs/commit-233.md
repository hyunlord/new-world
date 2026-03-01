# Commit 233 - Pathfinding dispatch total 검증 게이트 추가

## 커밋 요약
- `migration_verify` pathfinding 벤치 검증에 dispatch 총 호출 수(`total`) 파싱/검증을 추가해, checksum 외 실행 경로 계측까지 회귀 감지 범위를 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - pathfind 일반 벤치:
    - 출력에서 `total=` 값 파싱.
    - 기대값 `path_iters * 2`와 비교(튜플+xy 두 경로 호출).
  - pathfind split 벤치:
    - tuple/xy 각 출력의 `total=` 파싱.
    - 기대값 각 `path_iters`와 비교(구간별 reset 후 단일 경로 호출).
  - parse 실패/불일치 시 즉시 실패.
  - 성공 시 dispatch total 검증 통과 로그 출력.

## 기능 영향
- checksum이 우연히 동일해도 dispatch 경로가 비정상적으로 줄거나 늘어나는 회귀를 조기에 감지 가능.
- backend 계측을 검증 파이프라인에 실질적으로 반영해 GPU 전환 전후의 실행 경로 무결성을 강화.

## 검증
- `MIGRATION_BENCH_PATH_SPLIT=true MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND=cpu tools/migration_verify.sh --with-benches` 통과.
  - 일반 path: dispatch total `200` 검증 통과 (`iters=100`).
  - split path: tuple/xy dispatch total 각각 `100` 검증 통과.
  - checksum 유지:
    - pathfind `70800.00000`
    - split tuple/xy `35400.00000`
    - stress `24032652.00000`
    - needs `38457848.00000`
