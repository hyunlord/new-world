# B4 UI art contract

ART-BATCH round 2, art-only candidates. Game inventory is pinned to `4a44336e7782f2b27615d942b5cd54dbfee89406`; A4 style/tool source is `386248ffbab91993960145f59fbc3636890cea63`. This branch never applies runtime imports.

## Frozen medium

Follow A4 and ADR 0018: fine brown-black/sepia ink, restrained earth wash with base/shadow tones, upper-left light. Standing objects front three-quarter at approximately 20 degrees down; flat symbols plan view. Main:internal stroke target 2:1. Per principal material patch: three broken longitudinal wood marks, three stone/clay facets or cracks, three fiber strands, three hide/fur tufts; metal two rim highlights without chrome. At most one shadow-hatch cluster per object, three short parallel rising 45-degree strokes. These are prompt and visual-review targets, not automatically counted measurements. Essential subject geometry does not count as decoration.

Sepia, bone, ochre, slate and muted sage; copper terracotta, bronze golden brown, iron charcoal. Amber fire is the warmest accent. No dense crosshatch, flat vector substitute, painted text, frames, modern symbols, invented tinting, recoloring or code-redrawn art.

## Files and checks

Icons 64x64 centered (32,32), content within 58x58. Decision vignettes 520x260 centered (260,130). Transparent RGBA, footprint null (UI); world feet/tiling/seams do not apply. Reuse A4 `art/techmap/assets.py` unchanged: real alpha, solid bounds plus fringe, proportional Lanczos, transparent padding only. Its copies here (assets.py, sheets.py, test_assets.py) are byte-identical to the pinned A4 source. No new dependencies.

Compare icons at 32/48/64 on light and dark; narrative at native and exact 2x. Include existing adopted art and A4 references. Structural pass and semantic/era pass plus visual-verdict >=90 (medium/line/light/palette >=4/5) are candidate gates. This is general vision review, not the unavailable historical generator judge. Legacy item style metrics remain diagnostic with failures retained.

## Eras and variants

Paleolithic/neolithic use no metal or dyed cloth. Generic job art uses its earliest permitted material. Metal-specific variants explicitly label bronze/iron. Retained need urgency variants and mood valence use distinct silhouettes, never color alone. Selection/hover/disabled treatments are proposed UI recipes sharing canonical art; they are not additional generated PNGs. Card illustrations use semantic IDs, with livestock birth distinct from human birth. Proposed future types are separated from verified runtime IDs. After all required subjects and variants, regenerate the weakest 10–20 percent and retain before/after evidence.

## Evidence

External evidence root `/Users/rexxa/work/art-batch/B4/`: exact prompts, actual tool output paths, raw PNGs/hashes, reference paths/hashes, tool metadata, all retries and rejected alternatives. Seed/model version null unless provided. Final adopted PNGs and per-file sidecars under `art/style05/ui_b4/<lane>/`, manifests and unapplied import drafts here, boards under `art/boards/ui_b4/`. A selected candidate is not owner approval or game readiness.
