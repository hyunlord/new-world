# T-600: Population Growth Fix + Diagnostic Logging

## Priority: Critical
## Status: Open
## Files: population_system.gd, behavior_system.gd

## Problem
Pop=30에서 성장 멈춤. 원인:
1. `built_shelters * 6 <= alive_count` — 쉘터 5개 × 6 = 30 ≤ 30 → 성장 차단
2. behavior_system의 `_should_place_building()`도 `shelters.size() * 6 < alive_count` — 30 < 30 = false → 새 쉘터 건설 안 함

## Changes

### population_system.gd
- `_check_births()`: 쉘터 카운트를 built + under_construction 모두 포함
- 비교 연산자 `<=` → `<` 로 변경 (경계값에서 성장 허용)
- 500틱마다 인구 상태 진단 로그 출력
- 조건 불충족 시 사유를 이벤트로 emit

### behavior_system.gd
- `_should_place_building()`: 선제적 건설 트리거 (쉘터 용량이 pop+6 이하면 건설)
- `_try_place_building()`: 동일한 선제적 로직 적용

## Done Definition
- Pop이 30을 돌파하여 80+ 까지 성장
- 500틱마다 "[Pop] pop=N food=N shelters=N" 진단 로그
