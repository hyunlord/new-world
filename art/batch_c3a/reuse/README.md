# C3A pinned reuse package

This package reuses 35 existing PNGs without copying their blobs into this branch:
14 A3 item PNGs, one B3 spear, eight A2 outcrop/workshop PNGs, five A2 road tiles,
and seven B3 physical construction stages. All retain their original canvas, alpha,
anchor and footprint. `inventory.json` records SHA256 and era for each row.

`art_import.draft.json` is an additive `files` fragment for the importer at
`87c67b1de93894bc5959e6ad619b7769d5eb3f68`. Each row has its own immutable `from`
and `meta_source`; retain all unrelated global `meta_sources` and files. Source
commits may predate their later reporting/draft commits. Do not replace the full
integration config with this file.

`bindings.draft.json` contains symbolic proposals, not executable runtime config.
Resolve numeric codes against the C3 registry, then copy only the supported binding
fields. `IronOre` code75 is documented by the bronze inventory but the art is iron
**era**. `WroughtIron` points to the existing hammered-bar ID `iron_ingot`, including
its count variants; an additional renamed image would duplicate the same bytes.
The existing data-driven `ArtData.item_scale` defaults remain authoritative:
0.45 +0.15 per bucket, capped0.85. Explanatory `world_scale:null` must be omitted
when making runtime rows, not copied as a numeric override. Structures use native
64px tile geometry; ground tiles are64×64 with anchor(32,43).

`construction.draft.json` preserves the existing bloomery/forge physical states,
warehouse/stone-house layered lifecycle recipes, and paved-road whole-cell stage
recipes. Room previews are review images, never a single painted runtime building.
Forge foundation remains a capped B3 gap. C3A's wall-pieces package is separate.

The three `reuse_*_native_2x.png` sheets show these exact pinned sprites at native
size and nearest-neighbour2×. They support a C3A review and do not silently upgrade
an old numeric/style verdict. Existing CompositeBow, iron hoe and iron plough are
held for regeneration because their frozen numeric style checks failed.

Verification: `/Users/rexxa/work/art-batch/C3A/checks/reuse_git_geometry.json`.
Reproduction source: `/Users/rexxa/work/art-batch/C3A/build_reuse.py`.
The check reads each pinned Git blob, compares its SHA256, decodes RGBA/canvas,
checks alpha occupancy and compares the anchor against the actual pinned metadata.

Not checked here: game import, lifecycle selection or in-game scale captures.
