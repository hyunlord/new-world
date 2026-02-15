# T-620: Resource Overlay Periodic Refresh

## Priority: Medium
## Status: Open
## Files: world_renderer.gd, main.gd

## Problem
자원 오버레이가 초기 렌더에만 적용되고, 채집/재생으로 변한 자원량이 반영 안 됨.
0.15 lerp로 매우 미약하여 바이옴 색상과 거의 구분 불가.

## Changes

### world_renderer.gd
- 자원 오버레이를 별도 Image → ImageTexture로 분리
- `update_resource_overlay(resource_map, world_data)` 메서드 추가
- 자원 밀도별 색상 강도 증가 (lerp 0.15 → 직접 알파 0.1~0.4)
- 식량=노란색, 나무=진한 초록, 돌=밝은 회색

### main.gd
- 100틱마다 `world_renderer.update_resource_overlay()` 호출
- SimulationBus tick 이벤트에 연결하거나 _process에서 체크

## Done Definition
- 자원 밀집 지역이 줌 아웃에서 색상으로 구분됨
- 채집 후 자원 감소가 시각적으로 반영됨
