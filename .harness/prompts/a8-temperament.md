# A-8: Temperament Pipeline — Action Bias 확장 + Shift Rules 구현

## Implementation Intent
1. `temperament_action_bias()` 30개 ActionType 전부 TCI 4축 affinity 매핑
2. `check_shift_rules()` stub → 실제 shift 적용 (9 event_key)
3. `TemperamentShiftRuntimeSystem` 이벤트 확장 (event_store 읽기)
4. SimEventType → event_key 매핑 (tags 기반)

## Verification
- bias != 0.0인 action 20개 이상
- NS/HA/RD/P 방향성 검증
- Shift 작동 시 expressed 축 변화
