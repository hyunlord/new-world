# STRUCT: one projection

Status: production rule and numeric instrument, **owner verdict pending**.
Source contract: `docs/v8/artstruct/STRUCT_GOAL.md`, Part 0.

## Coordinates and ordering

At 1× a tile is exactly **64×64 px**. Grass, forest floor, dirt, floors,
water, zones and marks are straight top-down. All standing things use the
accepted route I front three-quarter view, approximately **20° down**, with
light from the **upper left**. A standing thing's foot/base is at
`(tile_left + 32, tile_top + 43)`, in the tile's bottom third. This is the
people anchor fixed by HANDOVER, not the centre of an opaque bounding box.
Draw standing objects in ascending **base y, then base x** order; use a stable
ID only as the final tie-breaker.

Sprites may extend above their tile. An extended source canvas declares its
own anchor `(ax, ay)`; paste it at `(world_base_x - ax, world_base_y - ay)`.
The validator receives that declared source-canvas anchor. It never moves,
rescales, crops, or normalizes the image to manufacture a pass.

## Heights at 1×

The goal's ranges stay fixed. Single-number generated-sprite targets use a
declared ±2 px raster tolerance. Code-assembled walls remain exactly 60 px.
Heights count opaque rows inclusively at alpha ≥128.

| Family | Required height / geometry | Alpha-height check band (px) |
|---|---|---:|
| Wall | 16 px cap + 44 px face = 60 px | 60–60 |
| Door | wall height, target 60 px | 58–62 |
| Bedroll | 14 px high; footprint 1×2 tiles | 12–16 |
| Workbench / knapping / pottery / grinding | 30–40 px | 30–40 |
| Drying rack, empty or hide | 48 px | 46–50 |
| Hearth, unlit body | 20 px | 18–22 |
| Hearth, lit or firing | 20 px body with flame above | 20–40 whole silhouette; body needs separate reading |
| Boulder, large or small | 24–36 px | 24–36 |
| Flint outcrop | 20 px | 18–22 |
| Clay bank, full or depleted | 16 px | 14–18 |
| Trees | 2–3 tiles | 128–192 |
| Adult person | 56 px | 54–58 |
| Child | 40 px | 38–42 |
| Deer | 40 px shoulder | 40–60 whole silhouette; shoulder needs separate reading |
| Wolf | 28 px shoulder/body | 28–42 whole silhouette; shoulder needs separate reading |

Additional implementation bands for families without goal-specified heights:
stump 14–24 px, carcass 12–24 px, lean-to 54–66 px, loose item 4–24 px.
These are production parameters, not new owner-approved proportions. The
whole-silhouette bands for flames and animals likewise do not prove their
body/shoulder target: a reading must explicitly check that landmark. Tree
re-angling is only needed if the existing 05 trees read as plan view.

## Walls, roofs and occlusion

Buildings show walls, doors, floor and contents. **Roofs are not drawn.** An
optional debug roof overlay may use **35% alpha**; it is not an owner-board
default. A hut uses a **3×3 log-wall ring** with a door and packed-earth floor.

At base row `b`, the 44 px face occupies rows `b-43..b`; the 16 px cap occupies
`b-59..b-44`. The face thus covers at most the 44 px immediately above and
including its own base row. People immediately north of a south wall are
partly hidden; no extra face or occlusion mask may extend farther. The cap
adds its separate declared 16 px, not an additional tall face. Neighbour
connectivity controls visible faces, while depth sorting controls overlap.

## Projection instrument

`art/compositor/projection_check.py` checks an unchanged candidate against its
declared sprite-canvas anchor and family band:

1. Lowest row containing alpha ≥128 is within **±2 px** of anchor y.
2. Top-to-bottom alpha ≥128 extent is within the table's band.
3. Compute Rec.709 luminance `0.2126 R + 0.7152 G + 0.0722 B`. For each axis,
   average adjacent-pixel luminance differences only where **both pixels are
   opaque**. Screen-coordinate gradient points toward increasing brightness.
   Angle is `atan2(mean_dy, mean_dx)` in degrees modulo 360: right=0°, down=90°,
   left=180°, up=270°. Upper left means both components <−1e−6 and an angle
   strictly between **180° and 270°**. Report both components and the angle.

No samples or gradient magnitude ≤1e−6 is **inconclusive and fails** the
lighting gate. A flat image cannot establish lighting direction. Texture,
ink contours, and asymmetric silhouette may affect this estimate; a numeric
pass is not proof of correct front-three-quarter perspective. The required
family judge and one-line visual reading remain separate checks.

Example:

```sh
uv run art/compositor/projection_check.py candidate.png workbench 32 43
```

JSON includes each result and overall `passed`; exit status 0 means all three
checks passed, 1 means rejected, 2 means invalid CLI family/usage. The Python
API `check_projection(image, ProjectionSpec(family, anchor, height_band=None))`
allows an explicit band for unlisted anatomy; any override is exposed in JSON
as `height_band_override: true` and must be justified in the pass decisions.
An override must never silently relax the goal's specified band.
