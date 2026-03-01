# Commit 227 - migration_verify에 pathfinding backend 벤치 옵션 추가

## 커밋 요약
- `migration_verify --with-benches`가 pathfinding 벤치 실행 시 backend 모드(`auto|cpu|gpu`)를 환경변수로 제어할 수 있도록 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수 추가:
    - `MIGRATION_BENCH_PATH_BACKEND` (기본값: `auto`)
  - 유효성 검사 추가:
    - `auto`, `cpu`, `gpu` 외 값이면 실패.
  - pathfinding 벤치 실행에 backend 전달:
    - `--bench-pathfind-bridge`
    - `--bench-pathfind-bridge-split`
    - 두 경로 모두 `--backend "${path_backend}"` 인자 전달.
  - 벤치 실행 컨텍스트 출력에 `path_backend` 포함.

## 기능 영향
- CI/로컬 검증에서 pathfinding backend 모드를 고정해 회귀를 확인할 수 있음.
- GPU feature가 꺼진 현재 환경에서는 `gpu` 지정 시에도 resolved가 `cpu`로 폴백되어 checksum 기준선 호환 유지.

## 검증
- `tools/migration_verify.sh --with-benches` 통과 (`path_backend=auto`).
- `MIGRATION_BENCH_PATH_BACKEND=cpu tools/migration_verify.sh --with-benches` 통과.
  - 두 경우 모두 pathfind checksum `70800.00000` 유지.
