# T-630: HUD Building Count + Entity Panel Improvements

## Priority: Low
## Status: Open
## Files: hud.gd
## Dispatch: CANDIDATE (single file, self-contained)

## Problem
HUD에 건물 수가 표시되지 않음.
엔티티 선택 패널에 건설 진행률, 경로 정보 없음.

## Changes

### hud.gd — Top Bar
- _pop_label 뒤에 _building_label 추가
- 표시: "Bld:3 Wip:1" (완성 건물 수 / 건설 중 수)
- _process에서 building_manager.get_all_buildings()로 카운트

### hud.gd — Entity Panel
- action_text에 건설 진행률 추가: "build -> Shelter at (120,155) [65%]"
- building_manager.get_building_at()으로 타겟 건물의 build_progress 조회
- 경로 정보: "Path: 3 steps" (cached_path.size() - path_index)

## Done Definition
- HUD 상단에 건물 수 표시
- 엔티티 선택 시 건설 진행률 및 경로 스텝 표시
