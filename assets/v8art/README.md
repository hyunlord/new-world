# assets/v8art — the adopted art, imported

Everything the colony renderer draws (ADR 0034) comes from this folder. Nothing here is edited by hand except the
three files marked *hand-written*.

| path | what | made by |
|---|---|---|
| `manifest.json` | every imported file: id, role, path, source path and commit, SHA-256, bytes, pixel size, anchor / canvas / footprint; the 36 logical looks (age × sex × top × hair, the bridge's `look_ids` order) with the two fallbacks; the people and animal frame layouts; the tone reference | `tools/v8/art_import.py` |
| `people/frames/<look>/<S\|E>/<pose>.png` | route I frames (34 looks × 2 facings × 5 poses) | import |
| `people/portraits/<look>/<v1..v6>.png` | POLISH2 portraits, 128 px | import |
| `people/face_variants.json`, `people/variant_transforms.json`, `people/top_colours.json`, `people/route_i_manifest.json` | face variant plan, per-variant Lab shifts, LOD marker colours, route I manifest | import |
| `people/masks/<look>/<facing>/<pose>.png`, `people/masks/MASKS.json`, `people/masks/skin_regions.json` | the fixed skin (R) and hair (G) masks and their regions | `tools/v8/art_masks.py` |
| `materials/*.json`, `materials/textures/<style>/*.png` | the material system: construction styles, simulation materials, category defaults, their JSON Schemas, wall/door/floor textures | import |
| `ground/`, `structures/`, `items/`, `animals/` | ground textures, water, snow; standing structures, ford stones, rock face; items; wolf and deer frames | import |
| `flora/species.json`, `flora/habit_defaults.json` (+ schemas) | the flora catalogue (M-ART2.1, ART-TREES): per species its habit, frames by size (foliage variants and bare winter frames), stump, four season records (CIELAB shift, frame, snow cap), map canopy, log material; habit defaults and fallbacks — read by `scripts/world/art/flora_resolver.gd` | import |
| `flora/sprites/`, `flora/masks/`, `flora/textures/canopy_mass.png` | tree, stump and bush frames (manifest id = file stem), their canopy masks (id = stem + `_mask`), the ¼× canopy-mass texture | import |
| `flora/tests/test_alder.json` | ART-TREES' data-only species (not in the catalogue; `mart21_trees_test.gd` adds it by data) | import |
| `bindings.json` | *hand-written*: sim codes → art ids (terrain, space, tones, buildings, workshops, nodes, resources, trees, items, animals, blueprints, order icons, people) | this repo |
| `render_params.json` | *hand-written*: every renderer tunable; F12 reloads it in a debug build | this repo |
| `LICENSE.md` | origin: all made in this project | import |

## Re-import from a newer art commit

1. Fetch the art branch (`git fetch origin v8/art-…`); never merge it, never check it out here.
2. Edit `tools/v8/art_import.json`: `source_commit` (the full hash), and the explicit lists for anything the owner
   newly adopted (looks, items, structures, textures, animals). Every adopted file is named; nothing is globbed.
3. `python3 tools/v8/art_import.py` — reads the files from git objects at that commit, writes them unchanged,
   rewrites `manifest.json` and `LICENSE.md`, removes files no longer listed. A second run must change nothing
   (`python3 tools/v8/art_import.py --check` exits 0).
4. If the people frames or the art line's masks changed: `python3 tools/v8/art_masks.py` (then `--check`).
5. `python3 tools/v8/art_import.py --verify` (gate step `art_manifest`), `python3 -m unittest
   tools/v8/test_art_import.py tools/v8/test_art_masks.py tools/v8/test_mart2_extensibility.py`.
6. If the material or flora documents changed: `python3 tools/v8/resolver_parity.py --generate` (runs that commit's
   `resolve.py` and `resolve_flora.py`), then the gate step `resolver_parity`. The flora files are not listed one by
   one: the import copies every path the catalogue names (`flora` section of the config).
7. Bindings: a new kind needs a `bindings.json` row (and nothing in `scripts/world/**`); an id the bindings name that
   the manifest lacks fails `--verify`.
8. Re-import in Godot (`--import`, twice on a fresh tree), then `tools/v8/mart2_ground_test.gd`,
   `tools/v8/mart2_people_test.gd` and a windowed look (`capture.gd --mart2-set`).
9. Stay within the 40 MB budget (`budget_bytes` in the config; `--verify` checks it).

How to add a kind by data: [ADDING_ART.md](ADDING_ART.md). A new tree species is a catalogue record on the art line
(`art/flora/ADDING_A_SPECIES.md` there) plus a `bindings.json` `trees.species` weight here; no code changes.
