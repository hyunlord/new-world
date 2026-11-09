# A3 → C2/C3 import handoff draft

These files are data fragments and review aids. They have not been applied to the game or committed on its branch. The adopted asset source is pinned to **`8cb145d8cd0edf3f534b10384f5f95ff56ce562a`** on `v8/art-batch-a3`. A later documentation commit may differ; this asset commit is complete and intentionally remains the import source. Do not run this fragment as a complete importer configuration.

The complete catalogue has **122 base IDs + 7 empty states = 129 logical sprites**. Fifty-five base IDs have three ordinal count images, adding 110 PNGs: **239 proposed PNGs**. Root review deferred `steel_edge` and `iron_spear`: their originals failed independent review and the correction request produced no output. The active draft contains **127 logical sprites / 237 PNGs**, reconciled exactly to root's `selection_final.json` (`ROOT_VISUALLY_ACCEPTED`). The accepted corrections include `iron_plough` attempt 3 and `linen_tunic`, `wool_tunic`, and `dyed_tunic` attempt 2. `art_import.pending.json` retains the two excluded rows without activating them. No substitute identity or image was invented.

## Files and merge targets

| Draft | C2/C3 action |
|---|---|
| `art_import.json` | Append `files` to the existing `tools/v8/art_import.json.files`; change only its `meta_sources.item` as described below; retain the pinned `source_commit` and `source_branch`. Keep every existing config section and existing file row. Do not replace the complete config with this fragment. |
| `../items_manifest.json` | Already present on the **art branch** at `art/style05/batch_a3/items_manifest.json` in the pinned asset commit. Actual importer geometry was verified for all 237 A3 PNGs. This is the source metadata adapter, not an additional imported PNG. |
| `bindings.json` | Append its 53 verified `items.codes` rows to the **bundled** `assets/v8art/bindings.json.items.codes`. Preserve existing rows, bucket limits `[1,8]`, stack layout, fallback, and other families. Recheck codes against C2/C3's registry at application time. |
| `bindings.pending.json` | Review queue only: 69 base identities with `code: null` and `sim: null`, plus 7 unresolved empty-state bindings. Never merge these rows into runtime bindings. Two base identities additionally have deferred artwork. |
| `art_import.pending.json` | Two rejected/deferred item rows and their proposed geometry. Do not append to the importer until later work supplies accepted replacements. |
| `logical_assets.json` | All 129 logical candidates, their source/era, state, exact known identity or null, image IDs, inference flag, and import status. |
| `verification.json` | Counts, pinned evidence, integrity checks, and the explicit limits of fixture and real-data plan tests. |
| `real_import_plan.json` | Actual GitReader plan receipt: all 237 A3 file hashes/sizes/geometry, 53 binding rows, legacy metadata preservation, and combined import budget. |

## Required geometry adapter

The current importer at integration commit `4a44336e7782f2b27615d942b5cd54dbfee89406` reads `config.meta_sources.item`, then indexes `document.assets` by `id`. For each `role: "item"` row, it copies only `canvas_px` → `meta.canvas` and rounded `anchor_px` → `meta.anchor` from that indexed row.

It **does not read adjacent item JSON sidecars**, `source_metadata`, or a per-file `meta` field. It silently imports a new item without geometry when its ID is absent from the one item source manifest. `ArtData.anchor()` then falls back to bottom centre, **(32,63) for a 64px image**, shifting these centred icons from their intended **(32,32)**. This is why simply appending PNG rows is insufficient.

The supplied adapter is the smallest compatible data change: 22 existing ENV1 geometry rows retained exactly, plus one geometry row for each of the 237 active PNG IDs = **259 rows**. All A3 rows declare `canvas_px: [64,64]`, `anchor_px: [32,32]`; each count variant and each empty-state image has its own ID. Legacy 48px rows are intentionally unchanged. The item importer does not propagate footprints; the renderer's default item footprint is 1×1, matching this family. Full prompts, generation records, provenance, and adoption decisions stay in the normal A3 records; this adapter supplies geometry only.

An alternative is to extend the existing ENV1 source manifest with these new geometry rows while retaining its current path. A C2/C3 code change to support per-item sidecars is unnecessary for this batch. Do not switch `meta_sources.item` to an A3-only document that drops old geometry.

## Identity, count, and state rules

The source registry `rust/crates/sim-core/data/colony/items.ron` at the pinned integration commit has 72 stable identities. All 53 non-null catalogue identities matched its numeric code and exact `serde_name`; all 69 null base identities were absent. The active binding fragment contains no null or guessed codes. For example, `wild_grain` binds code 19 / `WildGrain`; the generator's earlier name `grain` is not a new simulation ID.

For a count policy, use `assets: ["<id>", "<id>_count_2_8", "<id>_count_9_plus"]`. The base file is the count-1 image; there is no extra `<id>_count_1.png` in this draft. `stack_look()` selects one sprite per bucket and uses one copy when `assets` is present. Do not also request runtime copies of these already-composed pile images. The inventory icon uses the first asset. The pile images are ordinal quantity indicators, not literal counts of every seed or berry drawn inside a container.

The seven empty states are `honey_empty`, `beer_empty`, `milk_empty`, `fermented_milk_empty`, `fermented_vegetables_empty`, `wine_empty`, and `olive_oil_empty`. Current item selection uses kind and count, with special `many`, `spoiled`, and `carcass_of` handling; it has no general filled/empty selector. Their PNGs can be imported as available art, but they remain unbound until C2/C3 defines the state or a separate verified identity. Never duplicate the parent's code in a second binding row or add an unsupported `empty` key and claim it works.

The 53 rows target bundled `assets/v8art/bindings.json`, not a selected content-pack extension. `ArtData.validate_extension()` currently allows only `code` + `alias_code` item aliases of bundled visuals; direct `asset` or `assets` rows are rejected by that extension schema. Newly authored icons must first become bundled bindings or use a separately reviewed extension capability change.

Current code 17 (`Berries`) already has `berry_basket` artwork, and code 29 (`LegacyFood`) is a compatibility identity. They are outside the 122 missing-art candidates; these drafts do not silently add aliases for either. Four future candidates—`bronze_sword`, `iron_pickaxe`, `iron_chisel`, `iron_shield`—were inferred from bible aggregate counts, not individually named canonical items; they remain explicitly pending.

## Era conflicts retained for integration review

A3 follows the newer ART-BATCH brief: linen/flax textile production, wool textiles, mud brick, and dyed cloth are bronze-era art. This covers `flax_stalks`, `flax_fibre`, `linen_thread`, `linen_cloth`, `hand_spindle`, `linen_tunic`, `linen_wrap`, `dyed_tunic`, `wool`, `wool_yarn`, `wool_tunic`, and `mud_brick`. Bible §4.2 and existing content have earlier flax/linen/tailoring/dyeing or mud-brick nodes. The art era label does not change simulation availability, gates, or IDs; C2/C3 must resolve those content-era differences separately. `flax_seed` remains a neolithic agricultural seed, not a textile. Paleolithic/neolithic artwork contains no metal or dyed cloth. `iron_ore` is labelled iron-chain art even though bronze prospecting mentions three ores.

## Application sequence after root selection

1. Final-selection reconciliation has passed: all 127 accepted logical IDs and their 237 count-expanded PNG IDs match `selection_final.json`; the two withheld IDs are absent from active files and geometry. Preserve this selection through adoption. If the final adoption manifest changes, repeat the set comparison and remove all variants/bindings for any newly rejected base before import.
2. The accepted PNGs at `art/style05/batch_a3/items/<id>.png` and merged geometry adapter are committed at `8cb145d8cd0edf3f534b10384f5f95ff56ce562a`. A real GitReader plan read the complete existing-plus-A3 source set from this commit, confirming all old source assets and manifests remain available. Keep this asset SHA pinned unless intentionally switching to another independently checked commit.
3. In C2/C3, fetch the art branch without merging/checking it out. Append the selected file rows to the existing config, update `source_commit`, `source_branch`, and `meta_sources.item`, and append only still-valid verified bindings. Do not copy raw generations, evidence, or pipeline output folders into `assets/v8art`.
4. Run `python3 tools/v8/art_import.py`, then `python3 tools/v8/art_import.py --check` and `python3 tools/v8/art_import.py --verify`. The importer reads the Git object store, preserves PNG bytes, derives hashes/sizes itself, and removes unlisted files under managed directories. Therefore preserve the full existing configuration rather than applying this fragment as a replacement.
5. Run the importer/mask/extensibility tests listed in `assets/v8art/README.md`, Godot import, and visual checks for 64px/32px UI icons and world pile anchors. Check that final manifest rows have canvas `[64,64]` and anchor `[32,32]` for every active A3 ID, and that count transitions at 1/2/8/9 are correct. The total imported art must remain within the existing 40 MiB budget.
6. Do not treat the empty-state imports or future/null identities as visible gameplay until their actual bindings/selectors are implemented and verified.

## Evidence and limits

Read sources: A3's `sources/ADDING_ART.md`, `sources/art_import.json`, `sources/bindings.json`, `catalogue-research.json`, the content-bible snapshot and C2 goal, plus Git objects for the actual importer, item registry, UI icon consumer, world stack selector, and art registry. Integration snapshot is `4a44336e7782f2b27615d942b5cd54dbfee89406`; art base is `f182769fe06b1cb60105e4d0bf127b6524277838`.

The draft data checks cover count arithmetic, unique image IDs/destinations, no collisions with existing imports/binding codes, exact identity matches, and adapter completeness. The earlier synthetic-PNG contract test proved that the adapter supplies geometry and that omitting it silently loses geometry.

The final test executed the pinned importer's **`plan()` with its actual `GitReader`** against asset commit `8cb145d8cd0edf3f534b10384f5f95ff56ce562a`, using real Git PNG and adapter bytes. All **237** A3 PNGs are 64×64 item files with imported canvas `[64,64]` and anchor `[32,32]`; all **53** active binding rows resolve. Their byte total is **691,146**. The complete existing-plus-A3 plan contains **940 files / 14,846,830 bytes**, below the existing **41,943,040-byte (40 MiB)** budget. All 22 legacy geometry rows are preserved and no destination collides. `real_import_plan.json` records every A3 file's source, destination, hash, size, and geometry.

Only the read-only plan ran. No importer write, `--check`, or `--verify` against game output ran; no game path was written; no Godot import or runtime/count-transition visual test ran. Future identities and the empty-state selector remain unresolved. Root's visual acceptance used visible labels rather than a blind identification study and retained explicit visual exceptions to numeric diagnostics; it does not establish runtime behavior.
