# Commit 019 - Rust/Localization 통합 검증 스크립트 추가

## 커밋 요약
- Rust 마이그레이션 + localization 전환 검증을 한 번에 실행하는 통합 스크립트를 추가.
- 반복 검증 절차를 표준화해 커밋 단위 안정성 확인을 빠르게 수행 가능.

## 상세 변경
- `tools/migration_verify.sh` (신규)
  - 단계:
    1) `cd rust && cargo test -q`
    2) `tools/data_localization_extract.py`
    3) `tools/localization_compile.py`
    4) `tools/localization_audit.py --strict`
  - 옵션:
    - `--apply-key-fields` 전달 시 extraction 단계에서 원본 data에 `*_key` 자동 주입

## 검증
- `tools/migration_verify.sh` 실행 성공
- 실행 결과:
  - Rust 테스트 전체 통과
  - extraction/compile/audit(strict) 전체 통과
  - strict audit: `inline_keyable_without_key = 0`
