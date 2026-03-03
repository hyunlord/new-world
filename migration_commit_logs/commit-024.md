# Commit 024 - Data localization inline 제거 자동화 옵션 추가

## 커밋 요약
- `data_localization_extract`에 key 주입 후 inline 번역 필드(`*_en/*_ko/*_kr`)를 제거하는 옵션을 추가.
- 검증 스크립트 `migration_verify.sh`도 동일 옵션을 지원해 단계적 정리 작업을 반복 가능하게 구성.

## 상세 변경
- `tools/data_localization_extract.py`
  - 신규 옵션:
    - `--strip-inline-fields` (단, `--apply-key-fields`와 함께 사용)
  - 동작:
    - `*_key`가 생성/정합된 그룹에 대해 inline 언어 필드를 제거.
  - 리포트 확장:
    - `summary.changed_file_count`
    - `summary.stripped_field_count`
  - 실행 로그 확장:
    - `apply_key_fields`, `strip_inline_fields`, `changed_files`, `stripped_fields` 출력
- `tools/migration_verify.sh`
  - 옵션 파서 개선:
    - `--apply-key-fields`
    - `--strip-inline-fields` (내부적으로 key-field 적용도 함께 활성화)
  - extraction 단계를 배열 기반 명령 조합으로 변경하여 옵션 조합 처리.
- `data/localization_extraction_map.json`
  - summary에 신규 카운트 필드 반영.

## 기능 영향
- 기존 기본 동작(비파괴 추출)은 그대로 유지.
- 필요 시에만 명시적으로 inline 필드를 제거할 수 있어 데이터 중복 축소를 점진적으로 진행 가능.
- 데이터 로컬라이제이션 운영을 key-first 구조로 수렴시키는 자동화 레일 확보.

## 검증
- `python3 tools/data_localization_extract.py --help` 확인
- 임시 프로젝트 샘플로
  - `python3 tools/data_localization_extract.py --project-root <tmp> --apply-key-fields --strip-inline-fields`
  - 결과: `name_key` 유지 + inline 필드 제거 동작 확인
- `tools/migration_verify.sh` 통과
