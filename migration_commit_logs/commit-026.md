# Commit 026 - 코드 경로의 inline locale 필드 직접 참조 제거

## 커밋 요약
- UI/시스템 코드에서 `name_kr/name_en/description_kr/description_en` 직접 참조를 제거하고 key-first 조회로 통일.
- data inline 제거를 진행해도 런타임이 inline 필드에 의존하지 않도록 코드 경로를 정리.

## 상세 변경
- `scripts/systems/psychology/trauma_scar_system.gd`
  - scar 이벤트 payload 생성 시 `name_kr` 직접 조회 제거.
  - `name_key` 기반 localized 이름 해석:
    - `_resolve_scar_name(sdef, scar_id)` 추가
  - 이벤트 payload에 `scar_name`, `scar_name_key`를 추가하고 기존 `scar_name_kr`는 하위호환 유지.
- `scripts/ui/panels/trait_tooltip.gd`
  - trait 이름/설명 표시를 `name_key`/`desc_key` 기반으로 고정.
  - 번역 키 미존재 시:
    - 이름: trait id humanize fallback
    - 설명: 빈 문자열 처리(원시 키 노출 방지)
  - `name_kr/name_en/description_kr/description_en` fallback 제거.
- `scripts/core/entity/emotion_data.gd`
  - 사용되지 않던 `_dyad_labels_kr` 캐시 및 `name_kr` 참조 제거.

## 기능 영향
- key-first locale 구조로의 데이터 정리 시 코드 호환성이 개선됨.
- 툴팁/이벤트 텍스트가 번역 키 미매핑 상황에서도 원시 locale 키를 노출하지 않음.

## 검증
- `tools/migration_verify.sh` 통과
- 직접 검색 확인:
  - `rg -n "name_kr|name_en|description_kr|description_en" scripts`
  - 결과는 `trauma_scar_system.gd`의 이벤트 payload 하위호환 필드(`scar_name_kr`)만 남음
