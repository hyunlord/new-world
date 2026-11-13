# C3A paved-road terrain edges

`tiles/` contains16 pavement tiles and16 reciprocal meadow-edge tiles in the existing ground format:64×64 opaque RGBA, anchor(32,43), footprint1×1. N1/E2/S4/W8 bits indicate **same-material neighbours**. Missing bits invoke the existing12px terrain transition. The32 images cover isolated cells, terminal edges, straights, corners, T junctions and four-way junctions; no painted building image substitutes for floor pieces.

The accepted A2 pavement is pinned to `227529c26307bc3a854eb7bf819e88d02af39046`; the shipped meadow to `f182769fe06b1cb60105e4d0bf127b6524277838`. `build.py` reads Git objects directly, normalizes the meadow full-frame to64px and reuses `art/compositor/edges.py::render_tile` without modifying it. No new generation, recolouring, outline painting, mirror blend or new dependency is involved. `tiles/manifest.json` records all source/helper/output hashes, operations, mask directions, geometry and measurements.

```sh
uv run --with pillow --with numpy --with pydantic python -m art.batch_c3a.roads.build <new-output-directory>
uv run --with pytest --with pillow --with numpy --with pydantic python -m pytest art/batch_c3a/roads/test_build.py art/compositor/test_edges.py -q
```

`contact_sheet_1x.png` and2x show all32 tiles. `network_1x.png` uses exported tiles on an actual7×7 road network with bends, T and cross junctions, rather than isolated labels. Visual inspection found readable stone paving and naturally softened meadow borders at1x.

Seam results are deliberately separate:

- Original A2 pavement passed its documented p95-relative seam test. This exporter does not rewrite that decision.
- The older, stricter `pipeline2.ground.measure` wrap diagnostic fails its absolute18px vertical limit: horizontal15.151, vertical20.620; ratios0.995/1.136. Recorded `passed=false`; no smoothing correction or threshold relaxation.
- Actual road-network tile boundaries have longest constant luminance step1px (existing maximum24px), and all12 measured boundaries pass that edge-variation criterion. This checks obvious straight transition seams, not perfect compatibility of all possible neighbours.

Limits: cardinal16 masks do not encode diagonal terrain occupancy. The atlas records world-noise phase(0,0); the live compositor may vary edge noise with world coordinates. Both transition sides must be selected together; simply overlaying a paved-edge tile over unmodified meadow does not reproduce this sample. Game import, selector integration, seasons and climate-specific terrain were not changed.

`art_import.draft.json` is an append-only fragment; pin source_commit to the delivery commit before applying. `bindings.draft.json` is a disabled selector proposal (`runtime_rows_to_append=[]`), not an assertion that a road-edge atlas selector exists in the current renderer. No sim numeric code is invented.
