# Commit 067 - stress breakdown 감정 키 상수화

## 커밋 요약
- stress tick breakdown에서 감정 키 문자열 포맷팅(`"emo_%s"`)을 제거하고 상수 키 배열을 사용하도록 최적화.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - 상수 추가:
    - `_EMOTION_BREAKDOWN_KEYS`
  - `_update_entity_stress(...)`
    - 감정 breakdown 누적 시 문자열 포맷 대신 `_EMOTION_BREAKDOWN_KEYS[i]` 사용

## 기능 영향
- breakdown 키 의미는 동일(`emo_fear`, `emo_anger`, ...).
- tick 루프 내 문자열 포맷팅 비용을 소폭 절감.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
