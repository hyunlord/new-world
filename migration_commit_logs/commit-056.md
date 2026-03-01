# Commit 056 - personality trait 매칭 경로 최적화

## 커밋 요약
- `_calc_personality_scale`의 trait multiplier 적용 시 반복 선형 탐색을 제거하고, trait id 맵 기반 조회로 전환.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_personality_scale(...)`
    - 호출 초기에 `_build_trait_id_map(entity)`로 trait id 맵을 1회 생성
    - trait multiplier 적용 시 `_entity_has_trait` 반복 호출 대신 `trait_id_map.has(...)` 사용
    - axis 조회용 stat 문자열(`axis_stat`)을 사전 컴파일 필드에서 읽도록 조정
  - `_compile_personality_modifiers(...)`
    - spec에 `axis_stat` 필드 추가
  - `_entity_has_trait(...)` 제거
  - 신규 helper `_build_trait_id_map(...)` 추가

## 기능 영향
- 성격 스케일 계산 결과 의미는 유지하면서 trait 매칭 루프의 반복 탐색 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
