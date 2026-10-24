# ADR 0015 — Colony art: one code-drawn sheet, layered people, canopy in chunks

## Context
M1 drew the colony from a placeholder sheet (`tools/v8/gen_temp_tiles.py`): one flat 16-px circle per forest tile,
a grey-square hut, a box for the shelter, 16-px people without an outline weight. The owner judged the game too
crude to evaluate (2026-10-21). The palette already matched the chosen mock-up (`docs/images/v8_target_view_b.png`);
the drawing did not. No external art pipeline exists (no ComfyUI, no artist), and the simulation must not change.

## Decision
- **One sheet, painted in code.** `tools/v8/gen_tiles.py` paints every ground tile and sprite into
  `assets/v8/tiles.png` + `tiles.json` (name → rect, plus the people palettes) with seeded painters, under six
  written craft rules (outline, 3–4 values, top-left light, calm clustered ground, people brightest, backed map
  text; `worldsim-godot`). `tools/v8/test_gen_tiles.py` checks the contract (names and sizes, bounds, outline
  colour on every object sprite, greyscale person layers, tunic vs forest-floor luminance, determinism, the
  committed sheet equals a fresh run).
- **People are layers.** Greyscale skin, hair and tunic layers are tinted per person; an unmodulated outline layer
  carries outline, eyes and belt; a halo layer marks people who would be lost (under a crown, dark away from a
  fire). Frames: stand, stride, sleep; a west step flips them.
- **The ground clusters, the canopy overlaps.** The terrain layer picks variants by value noise on the world seed
  and autotiles banks; the tree layer draws 24×24 crowns from y-ordered chunks under the people (z 2 < 5).
- Render-only spreading (sleepers on bedrolls) is allowed; render-only objects that the simulation does not own are
  not (stumps come from tree-set diffs, delivered piles from `site_delivered`).

## Alternatives considered
- Hand-painted or AI-generated art files: no artist or generator in this pass, and binary art without a source
  cannot be regenerated or unit-tested.
- Keep the canopy as a `TileMapLayer`: crowns could not overhang their tile, which is what made the forest dots.
- Separate sprites per tunic colour: 6 × frames × stages sprites instead of one tinted layer.

## Consequences
Every new M2+ object (farm, cave, workshop items) is a painter plus a test row in the same sheet
(`worldsim-godot` procedure 11). The sheet is 512 px wide and grows downward; it stays one texture, so the layers
batch. If real art arrives later, it replaces painters one by one behind the same names.

## Date
2026-10-21

## Source
docs/v8/m1art/ART_GOAL.md §2, §5, §6; docs/v8/m1art/ART_DECISIONS.md D3–D16; docs/decisions/0002-view-b-top-down.md

## Superseded (2026-10-24, M-ART2)
The sheet path is deleted: `tools/v8/gen_tiles.py`, its test and `assets/v8/tiles.*` are gone (last at `29f720aa`). The colony
is drawn from the adopted art at 64 px through manifests and the material resolver (ADR 0018, ADR 0033, ADR 0034). The
craft rules this ADR set (`worldsim-godot`) still apply to anything drawn in code.
