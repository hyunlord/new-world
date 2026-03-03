# Commit 025 - data_generated 보존 모드로 strip-inline 후속 검증 안정화

## 커밋 요약
- `data_localization_extract` 기본 동작을 “기존 `data_generated` 보존 + 스캔 결과 덮어쓰기” 방식으로 확장.
- inline 필드를 제거한 뒤에도 후속 검증 실행에서 `data_generated`가 비워지지 않도록 안정화.

## 상세 변경
- `tools/data_localization_extract.py`
  - 기존 생성 파일 로더 추가:
    - `_load_string_dict(path)`
  - `run(...)` 시그니처 확장:
    - `preserve_existing_generated` 인자 추가
  - 기본 동작:
    - `localization/en|ko/data_generated.json` 기존 키를 seed로 로딩
    - 스캔된 키/값은 해당 키만 갱신
  - 신규 옵션:
    - `--no-preserve-existing-generated` (기존 동작처럼 스캔 결과만으로 재생성)
  - 리포트/로그 확장:
    - `summary.preserve_existing_generated`
    - `summary.preserved_generated_key_count`
    - 실행 로그에 `preserved=<count>` 표시
- `data/localization_extraction_map.json`
  - summary에 보존 관련 메타 필드 반영.

## 기능 영향
- `--strip-inline-fields` 적용 이후에도 기본 검증(`migration_verify.sh`) 재실행 시 데이터 번역 키가 유지됨.
- 필요 시 `--no-preserve-existing-generated`로 강제 재생성(클린 리빌드) 가능.

## 검증
- `python3 tools/data_localization_extract.py --help` 확인
- 임시 프로젝트 시나리오 검증:
  - 1차: `--apply-key-fields --strip-inline-fields`
  - 2차: 기본 실행
  - 결과: 2차 실행에서 `entries=0`이어도 `keys`가 유지됨 확인
- `tools/migration_verify.sh` 통과
