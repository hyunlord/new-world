# Commit 195 - localization_audit 중복 키 충돌 상세 리포트 확장

## 커밋 요약
- `localization_audit.py`가 중복 키 개수뿐 아니라 값 충돌 여부(파일별 값 불일치)까지 추적/출력/JSON 저장할 수 있도록 확장.

## 상세 변경
- `tools/localization_audit.py`
  - `_collect_top_level_entries()` 추가: locale 파일별 key->value 맵 수집.
  - `_find_duplicate_details()` 추가:
    - 중복 키별 소유 파일 목록, 파일별 값, `value_conflict` 여부 계산.
  - 리포트 필드 확장:
    - `duplicate_conflict_count`
    - `duplicate_consistent_count`
    - `duplicate_details`
  - 콘솔 출력 확장:
    - `duplicate_conflicts`, `duplicate_consistent` 요약 추가.
    - 값 충돌 중복 키 상위 20개 출력 추가.
  - 신규 CLI 옵션:
    - `--strict-duplicate-conflicts`: 값 충돌 중복 키가 있으면 non-zero 종료.
    - `--report-json <path>`: 전체 audit JSON 저장.
    - `--duplicate-report-json <path>`: 중복 키 상세 JSON 저장.
  - `_write_json()` 헬퍼 추가.

## 기능 영향
- 기존 `--strict` 판정 기준(parity, keyable group 누락)은 유지.
- 필요 시 중복 키 충돌을 별도 엄격 모드로 게이트 가능.
- 현재 데이터 기준 중복 248개 중 값 충돌 24개를 자동 식별 가능.

## 검증
- `python3 tools/localization_audit.py --project-root .` 실행 확인.
  - `duplicate_keys: 248`
  - `duplicate_conflicts: 24`
- `python3 tools/localization_audit.py --project-root . --strict` 종료코드 `0` 확인.
- `python3 tools/localization_audit.py --project-root . --strict-duplicate-conflicts` 종료코드 `1` 확인.
- `python3 tools/localization_audit.py --project-root . --duplicate-report-json /tmp/worldsim-duplicate-audit.json` 파일 생성 확인.
- `tools/migration_verify.sh --with-benches` 통과.
