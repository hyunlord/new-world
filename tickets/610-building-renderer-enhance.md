# T-610: Building Renderer Enhancement

## Priority: Critical
## Status: Open
## Files: building_renderer.gd
## Dispatch: CANDIDATE (single file, self-contained)

## Problem
건물 크기 6-7px로 에이전트(3-5px)와 거의 동일. Stockpile은 외곽선만.
줌 아웃 시 건물과 에이전트 구분 불가.

## Changes

### building_renderer.gd
- 건물 크기: TILE_SIZE * 0.8 (약 13px) — 에이전트보다 확실히 크게
- Stockpile: 갈색 채움 사각형 + 노란 테두리 (Color(0.55, 0.35, 0.15) fill + Color(0.9, 0.7, 0.3) outline)
- Shelter: 주황갈색 삼각형 채움 + 밝은 테두리 (Color(0.7, 0.4, 0.2) fill)
- Campfire: 빨강-주황 원 (radius * 0.4) + 빛 범위 표시 (얇은 원)
- 건설 진행률 바: 크기 증가 (bar_w = building_size, bar_h = 3.0)
- 미건설: alpha = 0.4, 건설 완료: alpha = 1.0

## Done Definition
- 줌 아웃에서 건물이 색상 블록으로 한눈에 보임
- 비축소=사각형, 쉘터=삼각형, 모닥불=원 구분
- 건설 중 건물에 진행률 바 표시
