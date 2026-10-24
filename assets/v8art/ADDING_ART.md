# Adding art by data (ADR 0033)

The renderer (`scripts/world/**`) names no item, material, style, look, animal or structure; the gate greps it
(`tools/v8/test_mart2_extensibility.py`). Every family below is added by data: the art line makes the image with its
template and checks, the import brings it in, a binding row connects it to a sim code. No `.gd` change.

## A wall, door or floor style (an era's construction)
1. On the art line follow `art/materials/ADDING_A_STYLE.md` (template prompt, ≤ 3 generations, seam and projection
   checks, a `construction_styles.json` entry with `sim_blueprints`, a data-only commit with its proof).
2. Here: add its textures to `tools/v8/art_import.json` (`material_texture` rows), re-import (README steps 2–6).
   `resolver_parity` regenerates from the art line's own `resolve.py`.
3. Sim side: a blueprint whose name is listed in the style's `sim_blueprints` (`get_space` sends it as
   `wall_styles` / `floor_styles`). If the sim has no such blueprint yet the style still resolves for any material
   of a category whose `kind_defaults` names it.
4. Proof: `tools/v8/fixtures/data_only_wall.json` is the pattern — a material and a blueprint added by data render a
   wall through the unchanged resolver and wall baker (`mart2_ground_test.gd` `data_only_*`).

## A simulation material
1. Art line: a `material_looks.json` entry (`id` = the sim key, `sim_id` = djb2 of the key, `category`, optional
   `tint` and `texture_override`), re-import. A material the art data does not list still draws: the bridge sends its
   category (`wall_categories`), and the category's tint and default style apply.

## A person look (age × sex × top × hair)
1. Art line: route I frames by `TEMPLATE_V3.md` (80 × 112, feet at (40, 104); sleep 112 × 80, (56, 72)), the face
   variant plan and transforms, masks, portraits.
2. Here: add the look to `art_import.json` `people.looks` (and the axis value to `look_grid` if a new top or hair),
   re-import, `tools/v8/art_masks.py`. A new top or hair axis value also goes into
   `rust/crates/sim-bridge/data/appearance.ron` in the same order (`test_art_import.py` checks the two agree).
3. A missing look: list it in `look_grid.fallbacks`.

## An item kind
1. Art line: the item sprite (64 px canvas, anchor recorded). Import it (`item` row).
2. `bindings.json` `items.codes`: `{"code": <ItemKind::code>, "sim": "<Name>", "asset": "<id>"}` (or `assets` for
   one sprite per count bucket 1 / 2–8 / 9+; `many` for 9+; `carcass_of` for a kill's remains). Panels show its icon
   (`scripts/ui/item_icons.gd`).

## A structure, workshop, resource node or tree
1. Art line: a standing sprite in the one projection (STRUCT `PROJECTION.md`: base at (tile_left + 32, tile_top + 43),
   anchor and footprint recorded in `docs/v8/artstruct/assets.json` or the sprite's sidecar). Import it.
2. `bindings.json`: `buildings` / `workshops` / `nodes` / `resources` / `trees.species` / `blueprints` row by sim code.

## An animal
1. Art line: S and E frames per pose (S = 35° toward the viewer's left), anchors in the animal manifest. Import
   (`animals` section of `art_import.json`).
2. `bindings.json` `animals.codes`: species, the pose per `AnimalState` code, `still`, `dead`, `lod_colour`.

## A tunable
`render_params.json` (F12 reloads in a debug build). A new parameter is read with `ArtData.param("section.key",
default)`; keep the default equal to the file's value.
