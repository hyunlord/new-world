# Commit 214 - duplicate conflict Markdown에 canonical 추천 컬럼 추가

## 커밋 요약
- `localization_audit`의 Markdown 충돌 리포트에 `Canonical (Suggested)` 컬럼을 추가해 충돌 키 통합 시 우선 기준 파일을 즉시 파악할 수 있게 개선.

## 상세 변경
- `tools/localization_audit.py`
  - Markdown 표 컬럼 확장:
    - `Key`
    - `Canonical (Suggested)`
    - `Files`
    - `Sample Values`
  - canonical 추천 규칙:
    - 우선순위 `ui.json` > `game.json` > `events.json`
    - 미매칭 시 첫 번째 파일

## 기능 영향
- 충돌 보고서 가독성/실행 가능성 향상.
- 실제 정리 작업 시 어떤 파일 값을 기준으로 통일할지 빠르게 결정 가능.

## 검증
- `MIGRATION_AUDIT_CONFLICT_MARKDOWN=/tmp/worldsim-duplicate-conflicts.md MIGRATION_BENCH_PATH_ITERS=10 MIGRATION_BENCH_STRESS_ITERS=100 MIGRATION_BENCH_NEEDS_ITERS=100 tools/migration_verify.sh --with-benches` 통과.
  - 생성된 Markdown에 `Canonical (Suggested)` 컬럼 및 추천 값 확인.
