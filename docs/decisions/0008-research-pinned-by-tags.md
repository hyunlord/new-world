# ADR 0008 — Research is pinned by tags; V8 owes research nothing

## Context
The research track (the frozen `game_v19` world, the world-model series WM-1…WM-6) shared the repository and constrained every change to the simulation.

## Decision
The V7 lines are pinned by annotated tags `v7-final-sim` (354e4a13, revitalize Pass 21) and `v7-final-art` (b1ca032f, art pass 13b); world-model work continues on `codex/worldmodel-2026-10`. The V8 line keeps no research code and no obligation to keep research results reproducible; deletions happen only on the V8 line.

## Alternatives considered
Keep research code in the V8 tree behind features (rejected: it shaped every change for the game).

## Consequences
V8-0 deleted the world-model recorder, ablation scenes and the `game_v9…v25` chain; `docs/HISTORY.md` and `docs/archive/INDEX.md` say where each lives.

## Date
2026-10-20

## Source
docs/VISION.md §5; docs/v8/V8_DIRECTION.md §11.2
