# Commit 230 - migration_verify에 resolved backend 기대값 검증 추가

## 커밋 요약
- `migration_verify` pathfinding 벤치 검증에 resolved backend 기대값 체크를 추가해, `auto`/`gpu` 모드의 실제 해석 결과를 CI에서 강제 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND` (허용: `cpu`, `gpu`)
  - 벤치 컨텍스트 로그에 `expected_resolved` 출력 추가.
  - pathfinding 벤치(일반/split) 출력에서 `resolved=...` 파싱 후 기대값과 비교.
    - 불일치 시 즉시 실패.
    - parse 실패 시 실패.
  - checksum 검증 로직은 기존 기준선을 유지.

## 기능 영향
- 현재처럼 GPU feature가 꺼진 환경에서는 `MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND=cpu`로 해석 결과를 고정 검증 가능.
- GPU feature 활성 빌드 시에는 동일 장치로 `expected=gpu` 검증을 바로 적용 가능.

## 검증
- `MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND=cpu tools/migration_verify.sh --with-benches` 통과.
  - pathfind bench에서 `resolved backend ok: cpu` 확인.
  - pathfind checksum `70800.00000`, stress `24032652.00000`, needs `38457848.00000` 유지.
