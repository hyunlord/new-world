# T-640: Event Logger Noise Reduction + Better Formatting

## Priority: Medium
## Status: Open
## Files: event_logger.gd
## Dispatch: CANDIDATE (single file, self-contained)

## Problem
resource_gathered가 매 틱 수십 줄 출력 → 유의미한 로그(탄생/사망/건설) 묻힘.
entity_born, entity_died_natural, building_completed 포맷이 기본 fallback.

## Changes

### QUIET_EVENTS 확장
- "resource_gathered", "needs_updated", "auto_eat" 추가

### 채집 요약 (50틱 집계)
- _gather_counts / _gather_totals 딕셔너리로 집계
- 50틱마다 한 줄 요약: "[Tick N] Gathered 47x: Food+28 Wood+52 Stone+12"
- tick 기반 flush (event의 tick을 추적)

### 이벤트 포맷 개선
- entity_born: "[Tick N] + BORN: Name (Pop: N)"
- entity_died_natural: "[Tick N] x DIED: Name age Nd (Pop: N)"
- building_completed: "[Tick N] # BUILT: type at (x,y)"
- entity_starving: "[Tick N] ! STARVING: Name (timer: N/50)"
- job_assigned/job_reassigned: "[Tick N] > Name: job_from -> job_to"

## Done Definition
- resource_gathered 개별 출력 안 됨
- 50틱마다 채집 요약 한 줄 출력
- 탄생/사망/건설 로그가 눈에 띄게 포맷됨
