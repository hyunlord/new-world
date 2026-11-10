# B3 retries_struct handoff

Candidates are exported for review, not runtime-imported. Frozen A2 numeric gates are unchanged. Root independently reviewed accepted candidates at native size; external model judge remains MISSING.

| Round-1 rejected asset | B3 attempts | Result / replacement |
|---|---:|---|
| aqueduct_ns | 3 | Try3 numeric pass but opaque width ~79px exceeds exact64px declared width; capped geometry reject. |
| loom_empty | 2 | art/style05/struct/b3/retries_struct/loom_empty.png |
| mine_entrance_abandoned | 1 | art/style05/struct/b3/retries_struct/mine_entrance_abandoned.png |
| mine_entrance_active | 3 | All three fail frozen wash-flatness; no built substitute. |
| olive_press_empty | 1 | art/style05/struct/b3/retries_struct/olive_press_empty.png |
| palisade_corner | 3 | Try3 wash-flatness fails; prior attempts wrong north arm geometry. |
| palisade_ns | 3 | Try3 numeric pass but nearest pole too tall and smooth vector wash; visual reject. |
| pen_fence_corner | 3 | All three fail frozen wash-flatness. |
| pen_fence_ns | 3 | Try2 front post ~45px vs exact40px nominal; final call returned HTTP400 moderation_blocked with no image. Geometry not waived. |
| potter_kiln_unlit | 2 | art/style05/struct/b3/retries_struct/potter_kiln_unlit.png |
| timber_bridge | 1 | art/style05/struct/b3/retries_struct/timber_bridge.png |

Self-review regenerated the weakest early candidates: loom, unlit kiln, aqueduct and terrace; all final failures remain rejected. The NS pen fence final tool call produced no image (HTTP400 output moderation_blocked), recorded in external evidence. Aqueduct NS is rejected despite numeric success because its ~79px width violates the exact64px plan. No geometry gate was waived.

Provenance: all 38 generated images retained in `/Users/rexxa/work/art-batch/B3/retries_{struct,ground}/raw`, with prompts, reference hashes, normalization operations and numeric verdicts. Structural generation timestamps inherited from A2 were corrected in export sidecars using B3 receipts; unknown timestamps remain null. Historical external normalized checks retain their original stale timestamp; do not use it as B3 generation time. Seeds are null because the tool did not expose them.

Contacts: `contact_1x.png` contains game-size assets and separately labeled reference thumbnails; `contact_2x.png` is its exact nearest-neighbor enlargement.

Import draft: nearest filtering, mipmaps disabled, paste origin = world base minus manifest anchor. Ground tiles are opaque full-frame textures; no runtime code, catalog, bindings, or Godot import was changed. Ground lifecycle recipes are in `../retries_ground/terraincoverage.json`; sources are exact A2 tiles pinned at `227529c26307bc3a854eb7bf819e88d02af39046`.

Quick verification:

```sh
uv run --with pillow python /Users/rexxa/work/art-batch/B3/retries_struct/verify_retries.py
```

Expected: `PASS: 8 candidates, 9 explicit gaps, 38 preserved outputs, 20 terrain state rows; no runtime import.`

Ground opacity and seam recomputation (read-only):

```sh
uv run --with pillow --with numpy python /Users/rexxa/work/art-batch/B3/retries_ground/verify_ground.py
```

Across both lanes: 39 requests, 38 image outputs, one blocked no-image pen_fence_ns final request. Separate import, symbolic bindings and geometry adapter drafts are included; source commit must be pinned by root.
