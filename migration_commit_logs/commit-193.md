# Commit 193 - migration_verify에 선택형 Rust 벤치 체크섬 검증 추가

## 커밋 요약
- `tools/migration_verify.sh`에 `--with-benches` 옵션을 추가해 Rust 수학/경로탐색 벤치 체크섬 회귀를 선택적으로 자동 검증할 수 있게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 새 옵션 `--with-benches` 추가.
  - 옵션 미사용 시 기존 4단계 검증 흐름 유지.
  - 옵션 사용 시 5단계(`rust bench checksum verification`)를 추가로 수행.
  - 내부 `run_bench_and_check` 헬퍼 추가:
    - 벤치 출력의 `checksum=...` 값을 파싱.
    - 기대 체크섬과 불일치 시 즉시 실패(exit 1).
  - 검증 대상 벤치:
    - `pathfind-bridge` (`--iters 100`, expected `70800.00000`)
    - `stress-math` (`--iters 10000`, expected `24032652.00000`)
    - `needs-math` (`--iters 10000`, expected `38457848.00000`)

## 기능 영향
- 기본 마이그레이션 검증 시간/동작은 그대로 유지.
- 필요 시 벤치 체크섬 회귀까지 한 번에 확인 가능.

## 검증
- `tools/migration_verify.sh` 통과.
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum ok: 70800.00000`
  - `stress-math checksum ok: 24032652.00000`
  - `needs-math checksum ok: 38457848.00000`
