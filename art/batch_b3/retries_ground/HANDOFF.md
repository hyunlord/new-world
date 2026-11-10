# B3 retries_ground handoff

Candidates are exported for review, not runtime-imported. Frozen A2 numeric gates are unchanged. Root independently reviewed accepted candidates at native size; external model judge remains MISSING.

| Round-1 rejected asset | B3 attempts | Result / replacement |
|---|---:|---|
| ditch_dry | 1 | art/style05/struct/b3/retries_ground/ditch_dry.png |
| flax_growing | 3 | All three fail frozen edge density. |
| flax_harvested | 3 | Try2 numeric pass but seed capsules and long leaves wrong for harvested state; try3 wash-flatness fails. |
| flax_ripe | 3 | All three fail frozen edge density; final raw border contact. |
| irrigation_channel_dry | 1 | art/style05/struct/b3/retries_ground/irrigation_channel_dry.png |
| terrace_empty | 3 | art/style05/struct/b3/retries_ground/terrace_empty.png |

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
