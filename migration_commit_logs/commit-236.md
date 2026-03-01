# Commit 236 - Backend smoke 파서 모드 고정/순서 독립 검증 강화

## 커밋 요약
- `migration_verify`의 pathfinding backend smoke 파서를 모드별 라인(`auto/cpu/gpu`) 직접 매칭 방식으로 강화해, 출력 순서 변화에도 안정적으로 `configured/checksum/dispatch/resolved`를 검증하도록 개선.

## 상세 변경
- `tools/migration_verify.sh`
  - `run_path_backend_smoke_and_check` 파서 구조 변경:
    - 기존: 전체 출력에서 checksum/total/mode 라인을 일괄 추출 후 순서 기반 비교.
    - 변경: `mode=auto`, `mode=cpu`, `mode=gpu` 라인을 각각 직접 추출해 모드별 필드를 독립 파싱.
  - 신규/강화 검증:
    - 3개 모드 라인 존재 강제.
    - 각 모드별 checksum 파싱 성공 강제 + 3모드 checksum 동일성 검증.
    - 각 모드별 dispatch total 파싱 성공 강제 + `smoke_iters * 2` 일치 검증.
    - 각 모드의 configured 값이 `auto/cpu/gpu`와 정확히 일치하는지 검증.
    - 기존 resolved 검증(`cpu mode must resolve to cpu`, optional auto/gpu expected) 유지.

## 기능 영향
- smoke 출력 라인 순서/추가 로그 변화가 생겨도 파서가 모드 레이블 기준으로 동작해 검증 안정성이 높아짐.
- mode 설정 자체(`configured`)까지 게이트에 포함되어 backend 모드 전달 회귀를 더 빠르게 탐지 가능.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
  - smoke 검증 통과:
    - `mode=auto/cpu/gpu` 라인 인식
    - checksum `3540.00000` 일치
    - dispatch total `10` 일치
    - `resolved_auto=cpu`, `resolved_gpu=cpu`
  - 기존 checksum 기준선 유지:
    - pathfind `70800.00000`
    - stress `24032652.00000`
    - needs `38457848.00000`
