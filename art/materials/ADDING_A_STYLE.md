# Adding a construction style or simulation material

The three versioned JSON documents are inputs. `resolve.py` has no named styles or
materials, and `walls.py` chooses geometry from neighbours, never from a style name.
Do not add a branch to either module to introduce an era.

## A construction style, in six steps

1. **Template.** Copy the relevant `prompts/wall.md`, `floor.md`, or `door.md` recipe.
   Fill `{style_description}`, `{material}`, and `{era}`. Reserve a stable style id.
   For walls, request the seven pieces below with matching scale and upper-left
   lighting. Request N–S independently; never rotate the E–W source.
2. **Generate.** Preserve originals, exact prompts, attachments, hashes, tool ids,
   and every attempt outside the repository in the pass data directory. At most
   three attempts per texture. Normalize whole generated texture bounds to the
   target dimensions; never repaint a seam, clone pixels or replace the source
   with procedural texture. Save selected assets under `art/materials/textures/ID/`.
3. **Seam and projection checks.** Run the STRUCT RGB join statistic (four repeats;
   each join MAD ≤ 1.15 × interior-column p95) on face/E–W and on the *measurement
   view* of N–S transposed to columns. The source image stays unrotated. Floor
   checks both axes. Nonrepeating ends, doors and junctions record seam N/A with
   reason; use `art.compositor.projection_check` for standing doors and assembled
   wall silhouettes. Keep failed results and retry count; do not relabel failures.
4. **JSON.** Add a `construction_styles.json` entry: kind, era, complete texture
   paths, default tint, top parameters, face height, end/corner references, prompt
   template id, simulation blueprint names. Change category defaults only when
   that category should use this style for an unknown blueprint. New named
   material entries belong in `material_looks.json`; overrides are optional.
   Validate with `python3 -m pytest art/materials/test_resolve.py -q`.
5. **Render layouts.** Run `python3 -m art.materials.preview STYLE MATERIAL OUTDIR`.
   Inspect E–W, N–S, corner, T, cross, isolated, open-door room, floor and both door
   images. Test an unlisted material with the simulation category. Every image
   must come through the generic resolver/assembly. Preserve native-scale output.
6. **Owner strip and data-only commit.** Add the same room to the board's data
   list and rebuild the owner strip. Commit only data, generated assets and prose;
   never a `.py` file in the style-addition commit. Run
   `python3 -m art.materials.check_data_commit COMMIT STYLE_ID` and record its
   actual JSON as `docs/v8/artpolish2/DATA_ONLY_PROOF.json`. Then
   `python3 -m pytest art/materials/test_data_commit.py -q` must run the published
   proof test without a skip. The owner still decides adoption.

| Wall texture key | File | Native size | Repeating axis |
|---|---|---:|---|
| face | face.png | 256×44 | x |
| top_ew | top_ew.png | 256×64 | x |
| top_ns | top_ns.png | 64×256 | y, independently generated |
| end | end.png | 24×44 | none; face only |
| junction | junction.png | 64×64 | none |
| door_closed | door_closed.png | 64×60 | none |
| door_open | door_open.png | 64×60 | none; preserve aperture alpha |

A floor style has `floor.png` 256×256. A door style has the two 64×60 door images.
The schema fixes face height at 44 px, matching STRUCT's y+43 base anchor. Full
N–S top connections are 64 px deep; an exposed south edge shows 16 px of top.
`width_px=44` is centred transverse to the run, retaining that projection; the
visible 16 px E–W band becomes 11 px. A narrow junction contains only the arms
whose neighbour exists. `side_shadow` is a soft alpha shadow of that top mask.

## A material, in the same six steps

1. Take its canonical `id`, `MaterialCategory`, and raw `MaterialId` from the
   simulation registry. The source key is not the translated display name.
2. Choose a tested Lab tint relative to each style's neutral source. If its grain
   truly differs, generate a complete or partial per-style texture override using
   that style's template; no new rendering code is needed.
3. Check any new override's seams/projection as above; inspect tinted native images
   for clipping. A tint-only addition needs deterministic Lab/alpha tests and visual
   inspection, not a fictional new-generation record.
4. Add the material entry with `id`, `sim_id`, `category`, `tint:{dL,da,db}` and
   optional `texture_override:{STYLE:{face:PATH,...}}`. Missing override keys inherit
   the style set. A material without a tint inherits its category tint.
5. Render all affected styles and an unknown-material fallback. The category is
   supplied separately for a material absent from this snapshot.
6. Add comparisons to the owner strip and commit the JSON/assets. Run the schema,
   resolver and unchanged generic-code tests. Record owner acceptance separately.

## M-ART2 loading contract

- JSON `schema_version` is exactly `1`; reject unknown versions and extra keys.
  The exported `*.schema.json` validates field types/ranges, required non-null
  per-kind texture roles, wall end/corner references, and canonical relative PNG
  paths. Negative fixtures run against the independent `jsonschema` validator.
  JSON Schema alone is **not** the complete loader contract: Godot must additionally
  check unique style/material/category ids and raw material ids; unambiguous
  blueprints per kind; existing, kind-correct defaults; known material categories
  and override-style references; equal `ends`/`textures.end` and
  `corner`/`textures.junction`; category wall-default consistency; actual image
  dimensions and file presence. These relational/file checks are enforced by the
  Python parser/loader and remain mandatory semantic checks in a Godot port.
- `material_looks.json` snapshots 105 source keys/categories at
  `origin/v8/main` commit `cb58a801d656291a04a5ebfb3be9805845ee6433`, from
  `rust/crates/sim-core/data/material/**`. It also contains raw `sim_id` values.
- `MaterialId` is **not an array index**. At that pin,
  `rust/crates/sim-core/src/material/id.rs` defines djb2 over UTF-8 source bytes:
  start at 5381; for each byte `h = (h * 33 + byte) mod 2^32`. Thus `oak` is
  193501184. Use these integers as dictionary keys in Godot, or compute the same
  hash at load time. The schema rejects duplicate raw ids. Do not hash the display
  name, localize the key, or depend on registry iteration order.
- The pure Python `resolve(catalog, style_id, material_id, category=None, kind='wall')`
  accepts either canonical key string or raw u32. It returns `material_key` when
  known. Blueprint names also resolve through `sim_blueprints` in the JSON.
- Godot should map its wall blueprint and `TileGrid.wall_material` u32 to these
  documents. For an unseen material, the bridge must also expose its registry
  category (or an equivalent category lookup). This pass does **not** add that
  bridge/runtime read. Normalize Rust categories `stone/wood/animal/mineral/plant`
  to the corresponding JSON enum labels; collapse any `Mod(u8)` to category `Mod`
  for fallback while retaining the raw id. Do not guess category from an id.
- Style fallback: exact style → matching blueprint → category kind default →
  document kind default. Tint fallback: material → category → style. Unknown
  category plus known style retains the style default. Overrides merge onto the
  style texture set before loading. Paths are canonical repository-relative PNG
  filenames, never empty, whitespace-containing, absolute, directory-like,
  redundant-separator or dot-segment paths. Backslashes, drive/URI colons and
  control characters are rejected. The file loader also rejects a valid-looking
  path whose resolved symlink leaves the repository root. Add `res://` only at the
  Godot resource boundary.
- Tint uses D65 CIELAB: `L*=0..100`, signed a*/b*. Alpha is unchanged. Top treatment
  then applies periodic Gaussian blur, saturation and value; exposed boundary ink
  is composed last. Python RGB/Lab conversion and deterministic tests are provided
  in `tone.py`. No game code is installed or changed by this art pass.

- Wall occupancy and door openings are global across material batches. A material
  group chooses textures for its draw subset; it must not decide connectivity
  from that subset. Pass `occupancy=all_wall_cells` and `openings=all_door_cells`
  to the Python assembly. Inset-top ink follows the actual silhouette, including
  exposed arm edges; connected tile boundaries receive no seam ink.
