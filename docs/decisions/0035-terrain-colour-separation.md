# ADR 0035 — Keep terrain classes readable independently of texture contrast

Status: accepted for implementation (M-ART2 D19, 2026-10-24).

## Context

The owner/planner found the M-ART2 comparison too grey: water disappeared and forest, grass and bare earth lost
their identities at reduced zoom. The initial shared land/water tone ceiling did not constrain class contrast.
Standing trees bypassed that ceiling and exactly retained their adopted source's yellow/olive palette.

## Decision

Amend ADR 0034's tone stage: land textures use the registered ENV1 maxima (mean luminance 0.38, saturation 0.30;
texture standard deviation at most 0.25 of the person reference). Only ground images enter that cap.

Water has its own data-defined blue target and small texture variance, with a luminance floor allowing at least
0.10 difference from land under daytime and configured full-night modulation. Rendered checks also require
circular HSV hue separation of 0.12 turns. At ½× and ¼×, data-defined terrain colours and weights are applied by
the ground chunk shader after downsampling, excluding standing sprites and crisp raised caps. Winter attenuates
the map tint. All controls live in `assets/v8art/render_params.json` and follow F12 invalidation.

## Alternatives considered

- Raising a common land/water cap alone: does not guarantee water contrast or class hue separation.
- A whole-screen colour treatment: would affect people, walls and trees and break the ground-only boundary.
- Changing imported source PNGs: would violate the existing provenance contract and cannot explain a renderer
  leak when source and rendered tree pixels are identical.

## Consequences

- Water is deliberately lighter blue than before; calmness comes from its limited texture contrast.
- Tiny per-chunk palette textures and materials add rendering work; earlier FPS measurements are historical,
  and the next full gate/performance run must measure the changed renderer.
- Pixel tests require a windowed GPU backend. Headless chunk tests retain their existing scope.
- Original goal, prior gate, source images and pre-D19 captures remain preserved.

## Date

2026-10-24

## Source

Owner/planner comparison review; `docs/v8/m_art2/MART2_DECISIONS.md` D19;
`assets/v8art/render_params.json`; `tools/v8/mart2_ground_test.gd`.
