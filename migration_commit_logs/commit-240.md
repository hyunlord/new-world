# Commit 240 - Backend smoke dispatch/resolved 일관성 게이트 강화

## 커밋 요약
- `migration_verify` pathfinding backend smoke 검증에 mode별 `cpu/gpu/total` 합산 검증과 `resolved` 대비 dispatch 방향 일관성 검증을 추가해, backend 전환 회귀를 더 엄격히 감지하도록 강화.

## 상세 변경
- `tools/migration_verify.sh`
  - `run_path_backend_smoke_and_check` 확장:
    - mode별(`auto/cpu/gpu`) `cpu=`/`gpu=` dispatch 카운트 파싱 추가.
    - `cpu + gpu == total` 관계를 mode별로 강제 검증.
    - `resolved` 값과 dispatch 방향 일관성 검증 추가:
      - `resolved=cpu`면 `gpu=0` 강제
      - `resolved=gpu`면 `cpu=0` 강제
    - parse 실패/불일치 시 즉시 실패.
  - 기존 검증(모드 라인 존재, configured 일치, checksum 일치, total 기대값 일치, optional resolved 기대값)은 유지.

## 기능 영향
- checksum/total이 우연히 맞더라도 dispatch 카운터가 비정상 분포를 보이는 회귀를 조기에 차단.
- 향후 실제 GPU 경로 도입 시 `resolved` 해석 결과와 실제 dispatch 방향 불일치를 검증 단계에서 즉시 발견 가능.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
  - smoke 결과:
    - `auto/cpu/gpu` 모두 `cpu=10 gpu=0 total=10`
    - checksum `3540.00000` 일치
    - `resolved_auto=cpu`, `resolved_gpu=cpu`
  - 기존 checksum 기준선 유지:
    - pathfind `70800.00000`
    - stress `24032652.00000`
    - needs `38457848.00000`
