# ENV1 wall autotiles and seams

**PARTIAL:32/32 connectivity cases pass;246/256compatible edge comparisons are flagged.** Both materials have all16neighbour cases, but seamless acceptance is not claimed.

Bits: N=1,E=2,S=4,W=8. Filenames `art/style05/env1/built/wall_{log|stone}_{00..15}.png`, each64×64. Four generated sources are independent straightstrip/corner pairs. Logstraight try3/logcorner try3 and stonestraight try1/stonecorner try3 feed the final derivatives. No mirror-blend.

The source strip is cropped to its visible vertical band then scaled64×16 and placed at y24. The complete generated L corner is cropped and geometrically reduced; no painted seam repair. N/E/S/W terminals use cropped generated arms, straight masks5/10use rotations, corners preserve the whole generated elbow, T/X combine the elbow and remaining generated arms. Crop rectangles and exact recipes are in every tile sidecar. Quarter rotations rotate the baked illumination.

The log retry repaired the most obvious width/material mismatch: the final elbow is a broad bark spine rather than thin railing poles. The rounded bend is still stylized and the rope/texture changes remain visible. Stonecorners now share mottled/lichen material but have stronger face depth/shadow. T/X internal seams and tone/texture changes are visible at2x. `autotile_board.png` shows allcases plus representative10→10,6→10,10→12,7→13 joins.

## Measurement

All compatible E→W and S→N pairs are compared separately per material,128pairs each. On visible edge pixels (either alpha≥128), compare premultiplied RGB MAE; flag whenMAE>18oralpha-support mismatch>2pixels. This is a conservative diagnostic adapted from the existing ground edge limit; it is not a substitute for visual review. Every flagged pair is listed below. All256records, including10unflagged pairs, are in `built_autotile.json`.

| Material | Axis | Piece pair | Visible RGB MAE | Alpha gap px |
|---|---|---|---:|---:|
| log | E-W | 02 → 08 | 38.128 | 0 |
| log | E-W | 02 → 09 | 43.214 | 1 |
| log | E-W | 02 → 11 | 38.128 | 0 |
| log | E-W | 02 → 12 | 37.157 | 0 |
| log | E-W | 02 → 13 | 37.157 | 0 |
| log | E-W | 02 → 14 | 38.128 | 0 |
| log | E-W | 02 → 15 | 38.128 | 0 |
| log | E-W | 03 → 08 | 37.157 | 0 |
| log | E-W | 03 → 09 | 40.264 | 1 |
| log | E-W | 03 → 10 | 22.441 | 0 |
| log | E-W | 03 → 11 | 37.157 | 0 |
| log | E-W | 03 → 12 | 43.215 | 0 |
| log | E-W | 03 → 13 | 43.215 | 0 |
| log | E-W | 03 → 14 | 37.157 | 0 |
| log | E-W | 03 → 15 | 37.157 | 0 |
| log | E-W | 06 → 08 | 43.214 | 1 |
| log | E-W | 06 → 09 | 47.642 | 2 |
| log | E-W | 06 → 10 | 27.047 | 1 |
| log | E-W | 06 → 11 | 43.214 | 1 |
| log | E-W | 06 → 12 | 40.264 | 1 |
| log | E-W | 06 → 13 | 40.264 | 1 |
| log | E-W | 06 → 14 | 43.214 | 1 |
| log | E-W | 06 → 15 | 43.214 | 1 |
| log | E-W | 07 → 08 | 37.157 | 0 |
| log | E-W | 07 → 09 | 40.264 | 1 |
| log | E-W | 07 → 10 | 22.441 | 0 |
| log | E-W | 07 → 11 | 37.157 | 0 |
| log | E-W | 07 → 12 | 43.215 | 0 |
| log | E-W | 07 → 13 | 43.215 | 0 |
| log | E-W | 07 → 14 | 37.157 | 0 |
| log | E-W | 07 → 15 | 37.157 | 0 |
| log | E-W | 10 → 08 | 38.128 | 0 |
| log | E-W | 10 → 09 | 43.214 | 1 |
| log | E-W | 10 → 11 | 38.128 | 0 |
| log | E-W | 10 → 12 | 37.157 | 0 |
| log | E-W | 10 → 13 | 37.157 | 0 |
| log | E-W | 10 → 14 | 38.128 | 0 |
| log | E-W | 10 → 15 | 38.128 | 0 |
| log | E-W | 11 → 08 | 37.157 | 0 |
| log | E-W | 11 → 09 | 40.264 | 1 |
| log | E-W | 11 → 10 | 22.441 | 0 |
| log | E-W | 11 → 11 | 37.157 | 0 |
| log | E-W | 11 → 12 | 43.215 | 0 |
| log | E-W | 11 → 13 | 43.215 | 0 |
| log | E-W | 11 → 14 | 37.157 | 0 |
| log | E-W | 11 → 15 | 37.157 | 0 |
| log | E-W | 14 → 08 | 43.214 | 1 |
| log | E-W | 14 → 09 | 47.642 | 2 |
| log | E-W | 14 → 10 | 27.047 | 1 |
| log | E-W | 14 → 11 | 43.214 | 1 |
| log | E-W | 14 → 12 | 40.264 | 1 |
| log | E-W | 14 → 13 | 40.264 | 1 |
| log | E-W | 14 → 14 | 43.214 | 1 |
| log | E-W | 14 → 15 | 43.214 | 1 |
| log | E-W | 15 → 08 | 37.157 | 0 |
| log | E-W | 15 → 09 | 40.264 | 1 |
| log | E-W | 15 → 10 | 22.441 | 0 |
| log | E-W | 15 → 11 | 37.157 | 0 |
| log | E-W | 15 → 12 | 43.215 | 0 |
| log | E-W | 15 → 13 | 43.215 | 0 |
| log | E-W | 15 → 14 | 37.157 | 0 |
| log | E-W | 15 → 15 | 37.157 | 0 |
| log | S-N | 04 → 01 | 38.128 | 0 |
| log | S-N | 04 → 03 | 43.214 | 1 |
| log | S-N | 04 → 05 | 38.128 | 0 |
| log | S-N | 04 → 07 | 43.214 | 1 |
| log | S-N | 04 → 09 | 37.157 | 0 |
| log | S-N | 04 → 11 | 43.214 | 1 |
| log | S-N | 04 → 13 | 38.128 | 0 |
| log | S-N | 04 → 15 | 43.214 | 1 |
| log | S-N | 05 → 03 | 27.047 | 1 |
| log | S-N | 05 → 07 | 27.047 | 1 |
| log | S-N | 05 → 09 | 22.441 | 0 |
| log | S-N | 05 → 11 | 27.047 | 1 |
| log | S-N | 05 → 15 | 27.047 | 1 |
| log | S-N | 06 → 01 | 37.157 | 0 |
| log | S-N | 06 → 03 | 40.264 | 1 |
| log | S-N | 06 → 05 | 37.157 | 0 |
| log | S-N | 06 → 07 | 40.264 | 1 |
| log | S-N | 06 → 09 | 43.215 | 0 |
| log | S-N | 06 → 11 | 40.264 | 1 |
| log | S-N | 06 → 13 | 37.157 | 0 |
| log | S-N | 06 → 15 | 40.264 | 1 |
| log | S-N | 07 → 01 | 38.128 | 0 |
| log | S-N | 07 → 03 | 43.214 | 1 |
| log | S-N | 07 → 05 | 38.128 | 0 |
| log | S-N | 07 → 07 | 43.214 | 1 |
| log | S-N | 07 → 09 | 37.157 | 0 |
| log | S-N | 07 → 11 | 43.214 | 1 |
| log | S-N | 07 → 13 | 38.128 | 0 |
| log | S-N | 07 → 15 | 43.214 | 1 |
| log | S-N | 12 → 01 | 43.214 | 1 |
| log | S-N | 12 → 03 | 47.642 | 2 |
| log | S-N | 12 → 05 | 43.214 | 1 |
| log | S-N | 12 → 07 | 47.642 | 2 |
| log | S-N | 12 → 09 | 40.264 | 1 |
| log | S-N | 12 → 11 | 47.642 | 2 |
| log | S-N | 12 → 13 | 43.214 | 1 |
| log | S-N | 12 → 15 | 47.642 | 2 |
| log | S-N | 13 → 01 | 43.214 | 1 |
| log | S-N | 13 → 03 | 47.642 | 2 |
| log | S-N | 13 → 05 | 43.214 | 1 |
| log | S-N | 13 → 07 | 47.642 | 2 |
| log | S-N | 13 → 09 | 40.264 | 1 |
| log | S-N | 13 → 11 | 47.642 | 2 |
| log | S-N | 13 → 13 | 43.214 | 1 |
| log | S-N | 13 → 15 | 47.642 | 2 |
| log | S-N | 14 → 01 | 37.157 | 0 |
| log | S-N | 14 → 03 | 40.264 | 1 |
| log | S-N | 14 → 05 | 37.157 | 0 |
| log | S-N | 14 → 07 | 40.264 | 1 |
| log | S-N | 14 → 09 | 43.215 | 0 |
| log | S-N | 14 → 11 | 40.264 | 1 |
| log | S-N | 14 → 13 | 37.157 | 0 |
| log | S-N | 14 → 15 | 40.264 | 1 |
| log | S-N | 15 → 01 | 38.128 | 0 |
| log | S-N | 15 → 03 | 43.214 | 1 |
| log | S-N | 15 → 05 | 38.128 | 0 |
| log | S-N | 15 → 07 | 43.214 | 1 |
| log | S-N | 15 → 09 | 37.157 | 0 |
| log | S-N | 15 → 11 | 43.214 | 1 |
| log | S-N | 15 → 13 | 38.128 | 0 |
| log | S-N | 15 → 15 | 43.214 | 1 |
| stone | E-W | 02 → 08 | 57.386 | 0 |
| stone | E-W | 02 → 09 | 53.595 | 3 |
| stone | E-W | 02 → 11 | 57.386 | 0 |
| stone | E-W | 02 → 12 | 47.120 | 2 |
| stone | E-W | 02 → 13 | 47.120 | 2 |
| stone | E-W | 02 → 14 | 57.386 | 0 |
| stone | E-W | 02 → 15 | 57.386 | 0 |
| stone | E-W | 03 → 08 | 47.120 | 2 |
| stone | E-W | 03 → 09 | 60.387 | 3 |
| stone | E-W | 03 → 10 | 45.750 | 2 |
| stone | E-W | 03 → 11 | 47.120 | 2 |
| stone | E-W | 03 → 12 | 55.544 | 4 |
| stone | E-W | 03 → 13 | 55.544 | 4 |
| stone | E-W | 03 → 14 | 47.120 | 2 |
| stone | E-W | 03 → 15 | 47.120 | 2 |
| stone | E-W | 06 → 08 | 53.595 | 3 |
| stone | E-W | 06 → 09 | 78.283 | 2 |
| stone | E-W | 06 → 10 | 52.591 | 3 |
| stone | E-W | 06 → 11 | 53.595 | 3 |
| stone | E-W | 06 → 12 | 60.388 | 3 |
| stone | E-W | 06 → 13 | 60.388 | 3 |
| stone | E-W | 06 → 14 | 53.595 | 3 |
| stone | E-W | 06 → 15 | 53.595 | 3 |
| stone | E-W | 07 → 08 | 47.120 | 2 |
| stone | E-W | 07 → 09 | 60.387 | 3 |
| stone | E-W | 07 → 10 | 45.750 | 2 |
| stone | E-W | 07 → 11 | 47.120 | 2 |
| stone | E-W | 07 → 12 | 55.544 | 4 |
| stone | E-W | 07 → 13 | 55.544 | 4 |
| stone | E-W | 07 → 14 | 47.120 | 2 |
| stone | E-W | 07 → 15 | 47.120 | 2 |
| stone | E-W | 10 → 08 | 57.386 | 0 |
| stone | E-W | 10 → 09 | 53.595 | 3 |
| stone | E-W | 10 → 11 | 57.386 | 0 |
| stone | E-W | 10 → 12 | 47.120 | 2 |
| stone | E-W | 10 → 13 | 47.120 | 2 |
| stone | E-W | 10 → 14 | 57.386 | 0 |
| stone | E-W | 10 → 15 | 57.386 | 0 |
| stone | E-W | 11 → 08 | 47.120 | 2 |
| stone | E-W | 11 → 09 | 60.387 | 3 |
| stone | E-W | 11 → 10 | 45.750 | 2 |
| stone | E-W | 11 → 11 | 47.120 | 2 |
| stone | E-W | 11 → 12 | 55.544 | 4 |
| stone | E-W | 11 → 13 | 55.544 | 4 |
| stone | E-W | 11 → 14 | 47.120 | 2 |
| stone | E-W | 11 → 15 | 47.120 | 2 |
| stone | E-W | 14 → 08 | 53.595 | 3 |
| stone | E-W | 14 → 09 | 78.283 | 2 |
| stone | E-W | 14 → 10 | 52.591 | 3 |
| stone | E-W | 14 → 11 | 53.595 | 3 |
| stone | E-W | 14 → 12 | 60.388 | 3 |
| stone | E-W | 14 → 13 | 60.388 | 3 |
| stone | E-W | 14 → 14 | 53.595 | 3 |
| stone | E-W | 14 → 15 | 53.595 | 3 |
| stone | E-W | 15 → 08 | 47.120 | 2 |
| stone | E-W | 15 → 09 | 60.387 | 3 |
| stone | E-W | 15 → 10 | 45.750 | 2 |
| stone | E-W | 15 → 11 | 47.120 | 2 |
| stone | E-W | 15 → 12 | 55.544 | 4 |
| stone | E-W | 15 → 13 | 55.544 | 4 |
| stone | E-W | 15 → 14 | 47.120 | 2 |
| stone | E-W | 15 → 15 | 47.120 | 2 |
| stone | S-N | 04 → 01 | 57.386 | 0 |
| stone | S-N | 04 → 03 | 53.595 | 3 |
| stone | S-N | 04 → 05 | 57.386 | 0 |
| stone | S-N | 04 → 07 | 53.595 | 3 |
| stone | S-N | 04 → 09 | 47.120 | 2 |
| stone | S-N | 04 → 11 | 53.595 | 3 |
| stone | S-N | 04 → 13 | 57.386 | 0 |
| stone | S-N | 04 → 15 | 53.595 | 3 |
| stone | S-N | 05 → 03 | 52.591 | 3 |
| stone | S-N | 05 → 07 | 52.591 | 3 |
| stone | S-N | 05 → 09 | 45.750 | 2 |
| stone | S-N | 05 → 11 | 52.591 | 3 |
| stone | S-N | 05 → 15 | 52.591 | 3 |
| stone | S-N | 06 → 01 | 47.120 | 2 |
| stone | S-N | 06 → 03 | 60.388 | 3 |
| stone | S-N | 06 → 05 | 47.120 | 2 |
| stone | S-N | 06 → 07 | 60.388 | 3 |
| stone | S-N | 06 → 09 | 55.544 | 4 |
| stone | S-N | 06 → 11 | 60.388 | 3 |
| stone | S-N | 06 → 13 | 47.120 | 2 |
| stone | S-N | 06 → 15 | 60.388 | 3 |
| stone | S-N | 07 → 01 | 57.386 | 0 |
| stone | S-N | 07 → 03 | 53.595 | 3 |
| stone | S-N | 07 → 05 | 57.386 | 0 |
| stone | S-N | 07 → 07 | 53.595 | 3 |
| stone | S-N | 07 → 09 | 47.120 | 2 |
| stone | S-N | 07 → 11 | 53.595 | 3 |
| stone | S-N | 07 → 13 | 57.386 | 0 |
| stone | S-N | 07 → 15 | 53.595 | 3 |
| stone | S-N | 12 → 01 | 53.595 | 3 |
| stone | S-N | 12 → 03 | 78.283 | 2 |
| stone | S-N | 12 → 05 | 53.595 | 3 |
| stone | S-N | 12 → 07 | 78.283 | 2 |
| stone | S-N | 12 → 09 | 60.387 | 3 |
| stone | S-N | 12 → 11 | 78.283 | 2 |
| stone | S-N | 12 → 13 | 53.595 | 3 |
| stone | S-N | 12 → 15 | 78.283 | 2 |
| stone | S-N | 13 → 01 | 53.595 | 3 |
| stone | S-N | 13 → 03 | 78.283 | 2 |
| stone | S-N | 13 → 05 | 53.595 | 3 |
| stone | S-N | 13 → 07 | 78.283 | 2 |
| stone | S-N | 13 → 09 | 60.387 | 3 |
| stone | S-N | 13 → 11 | 78.283 | 2 |
| stone | S-N | 13 → 13 | 53.595 | 3 |
| stone | S-N | 13 → 15 | 78.283 | 2 |
| stone | S-N | 14 → 01 | 47.120 | 2 |
| stone | S-N | 14 → 03 | 60.388 | 3 |
| stone | S-N | 14 → 05 | 47.120 | 2 |
| stone | S-N | 14 → 07 | 60.388 | 3 |
| stone | S-N | 14 → 09 | 55.544 | 4 |
| stone | S-N | 14 → 11 | 60.388 | 3 |
| stone | S-N | 14 → 13 | 47.120 | 2 |
| stone | S-N | 14 → 15 | 60.388 | 3 |
| stone | S-N | 15 → 01 | 57.386 | 0 |
| stone | S-N | 15 → 03 | 53.595 | 3 |
| stone | S-N | 15 → 05 | 57.386 | 0 |
| stone | S-N | 15 → 07 | 53.595 | 3 |
| stone | S-N | 15 → 09 | 47.120 | 2 |
| stone | S-N | 15 → 11 | 53.595 | 3 |
| stone | S-N | 15 → 13 | 57.386 | 0 |
| stone | S-N | 15 → 15 | 53.595 | 3 |

## Ground wrap seams

Floors and snow require both axes; ford requires north/south along the river; hill/dugface require left/right along the edge. Vertical jumps across a ledge are expected and are not counted as its horizontal tiling requirement. The preserved two-axis measurement remains in the manifest.

| Tile | Required axes | H MAE / ratio | V MAE / ratio | Required-wrap result |
|---|---|---|---|---|
| floor_packed_earth | horizontal,vertical | 7.828 / 1.215 | 6.188 / 0.885 | PASS |
| floor_dug_rock | horizontal,vertical | 10.354 / 1.046 | 9.932 / 0.903 | PASS |
| ford | vertical | 14.771 / 0.968 | 16.745 / 0.977 | PASS |
| hill_edge | horizontal | 6.781 / 0.559 | 51.484 / 3.973 | PASS |
| dug_face | horizontal | 21.698 / 1.894 | 54.333 / 4.236 | FAIL |
| snow_overlay | horizontal,vertical | 32.094 / 1.242 | 28.688 / 0.961 | FAIL |

The3×3repeat board deliberately shows repeated ledges in both directions to expose seams; runtime should repeat ledges only along their horizontal axis and rotate geometrically for other orientations. Packed/dug floors andford pass the required edge metrics, but repeating crack/gravel motifs remain visible. Dugface and snow retain their failed seams.
