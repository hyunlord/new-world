# ADR 0002 — View B: front top-down, one person = one tile

## Context
The V7 art line drew people 1.5 tiles tall in a bright 3/4 view; at the planned map size people were too large and hard to tell apart. Two mockups were compared: A (keep the bright 3/4 look, smaller) and B (front top-down, dark earthy tones).

## Decision
View B: a front top-down view in dark, earthy tones, Dwarf Fortress mood; one person fills one tile (32×32 screen pixels at the default zoom) with a shadow under the feet; 45–60 tiles across one screen.

## Alternatives considered
View A (bright 3/4, smaller sprites) — rejected by the owner on 2026-10-20.

## Consequences
Terrain, walls, floors, furniture and people are redrawn to the new standard (VISION §6.3 order). The V7 atlas and renderers stay only until M1 replaces them; tile layers and batched drawing are required from the start (the renderer, not the simulation, was the bottleneck at 2,000 people).

## Date
2026-10-20

## Source
docs/VISION.md §1, §6
