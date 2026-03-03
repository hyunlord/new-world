# Commit 211 - localization duplicate conflict Markdown 리포트 추가

## 커밋 요약
- `localization_audit.py`에 중복 충돌 보고서를 Markdown으로 출력하는 기능을 추가하고, `migration_verify`에서 환경변수로 연동.

## 상세 변경
- `tools/localization_audit.py`
  - 텍스트 출력 헬퍼 `_write_text` 추가.
  - 값 샘플 포맷터 `_format_value_sample` 추가(JSON 축약 + 파이프 escape).
  - Markdown 생성기 `_build_duplicate_conflict_markdown(report)` 추가:
    - locale/충돌 개수 요약
    - conflict key별 파일/샘플 값 표 생성
  - 신규 CLI 옵션 `--duplicate-conflict-markdown <path>` 추가.
- `tools/migration_verify.sh`
  - 환경변수 `MIGRATION_AUDIT_CONFLICT_MARKDOWN` 추가 연동.
  - 설정 시 strict audit 단계에서 Markdown 리포트 파일 생성.

## 기능 영향
- localization 충돌 정리 작업 시 JSON 외에 사람 읽기 쉬운 Markdown 산출물 확보 가능.
- 기존 strict 판정/기본 검증 흐름은 유지.

## 검증
- `tools/migration_verify.sh --with-benches` 통과.
- `MIGRATION_AUDIT_CONFLICT_MARKDOWN=/tmp/worldsim-duplicate-conflicts.md MIGRATION_BENCH_PATH_ITERS=10 MIGRATION_BENCH_STRESS_ITERS=100 MIGRATION_BENCH_NEEDS_ITERS=100 tools/migration_verify.sh --with-benches` 통과.
  - `/tmp/worldsim-duplicate-conflicts.md` 생성 확인.
  - 상위 표 형태 충돌 리포트 출력 확인.
