# Generated-material autotile rehearsal

Run `python art/pipeline2/autotile.py RAW_DIR OUTPUT_DIR EARTH_PNG GRASS_PNG`.
The raw directory contains `straight_try1_raw.png`, `corner_try1_raw.png`, and
`end_try1_raw.png`. Outputs remain candidates until both judge signals pass.

The 64px connectivity bits are north=1, east=2, south=4, west=8.
Each active bit gets the east half of the generated straight strip, rotated
counterclockwise by N=90, E=0, S=270, W=180 degrees. A generated joint crop
overlays their central join. Adjacent-bit masks use the bottom-left elbow crop
of the corner source; other masks use the bottom terminal crop of the end
source. Mask zero is an isolated terminal-material post. No wall pixels are
redrawn. The exact per-mask recipe, source hashes, crop bounds and operations
are emitted in `mapping.json` and `wall_00.json` through `wall_15.json`.

| Masks | Joint | Arms |
|---|---|---|
| 0 | end | none |
| 1, 2, 4, 8 | end | N, E, S, W respectively |
| 5, 10 | end | NS, EW respectively |
| 3, 6, 9, 12 | corner | NE, ES, NW, SW respectively |
| 7, 11, 13, 14 | corner | NES, NEW, NSW, ESW respectively |
| 15 | corner | NESW |

`floor_00.png` is the supplied packed-earth texture downscaled with Lanczos.
`floor_01.png` composes its top 10 pixels with a strip from the supplied grass
texture. This is a hard crop, not a proven organic grass transition.

Known limitations must remain visible in the report: rotating arms also rotates
baked illumination; the first straight source has upright posts while the
corner/end have horizontal log caps; central overlap joints can show material
and scale changes. A connected 16-mask set is not evidence of style acceptance
or seamless rendering. Judge original sources and working tiles separately,
then review the actual 10x8 Godot enclosure at 1x and 2x.

The first external run is under
`/Users/rexxa/work/arttrial2-data/autotile/derived/`. It includes the atlas,
labelled board, 16/16 actual edge-connectivity checks and 128 legal neighboring
edge comparisons. Those edge comparisons measure RGBA mean absolute difference
only; no acceptance threshold was invented. Numeric results rejected all three
source images and all 16 derived wall tiles. Model review remains pending.
