# A4 tech-map, era and chapter candidates

The completed family contains 167 knowledge icons, four era banners and ten chapter illustrations, selected from 199 generated attempts. It is produced on `v8/art-batch-a4` from the adopted art-line source and is not imported into the game.

- [STYLE.md](STYLE.md): fixed medium, mark budgets, 2:1 contour ratio, canvases, alpha and visual gates.
- [catalogue.json](catalogue.json): 167 concrete knowledge IDs, four eras and ten chapter illustrations including legacy `neolithic_village`.
- [coverage.json](coverage.json): complete 167-node, four-era and ten-chapter coverage.
- [manifest.json](manifest.json): selected images, prompts, raw/generated provenance, normalization and independent review evidence.
- [art_import.draft.json](art_import.draft.json), [bindings.draft.json](bindings.draft.json): unapplied rows pinned to the immutable all-assets commit. Preserve `source_commit` when importing; the branch head can contain later handoff documentation.
- Icon sheets: [bronze](../boards/techmap/bronze_icons.png), [neolithic](../boards/techmap/neolithic_icons.png), [iron](../boards/techmap/iron_icons.png), [paleolithic](../boards/techmap/paleolithic_icons.png).
- [state_variants.json](state_variants.json): stable/endangered/unknown/inert UI treatments over canonical icons.
- [ADR.md](ADR.md): decisions and integration boundaries.

Raw attempts, rejected candidates, numeric diagnostics and full review sheets remain under `/Users/rexxa/work/art-batch/A4/`. The final Korean report is `REPORT.md` there. Curated family contact sheets live in `art/boards/techmap/`: [era banners](../boards/techmap/era_banners.png), [chapters 1-5](../boards/techmap/chapters_1_5.png), and [chapters 6-9 plus legacy](../boards/techmap/chapters_6_9_legacy.png), alongside each era's icon sheet.

## Mechanical checks

```sh
uv run art/techmap/assets.py icon OUTPUT_DIR CHECK_DIR RECORD.json
uv run art/techmap/assets.py banner OUTPUT_DIR CHECK_DIR RECORD.json
uv run art/techmap/assets.py chapter OUTPUT_DIR CHECK_DIR RECORD.json
uv run art/techmap/sheets.py icon SHEET.png NORMALIZED.png
uv run --with pillow --with pydantic --with pytest python -m pytest art/techmap/test_assets.py -q
```

Multiple records/images may follow the last argument. Checks do not adopt an asset. Each selected image also requires an independent visual verdict at least 90 and medium/line/light/palette each at least 4. Legacy world-item numeric calibration is retained unchanged as a diagnostic, including failures.

## C2/C3 integration

The current UI has 264x88 tech cards and 520px chapter text width but no bitmap consumers for these families. Add explicit generic import rows, the `ui` managed directory and the metadata manifest. Then implement consumers for `ui_knowledge`, `ui_eras` and `ui_chapters`, preserve center anchors, apply state treatments and capture the actual game. Generic copying alone does not create an image slot or ingest UI anchor metadata. Resolve chapter 6 by semantic ID: legacy `neolithic_village` and planned `first_copper` are separate assets. Localized titles remain UI text.

No seasons, count buckets, filled/empty or depletion variants apply to knowledge/header/chapter art.
