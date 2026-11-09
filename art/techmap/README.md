# A4 tech-map, era and chapter candidates

This family is produced on `v8/art-batch-a4` from the adopted art-line source. It is not imported into the game.

- [STYLE.md](STYLE.md): fixed medium, mark budgets, 2:1 contour ratio, canvases, alpha and visual gates.
- [catalogue.json](catalogue.json): 167 concrete knowledge IDs, four eras and ten chapter illustrations including legacy `neolithic_village`.
- [coverage.json](coverage.json): exact selected/missing counts; incomplete production is explicit.
- [manifest.json](manifest.json): selected images, prompts, raw/generated provenance, normalization and independent review evidence.
- [art_import.draft.json](art_import.draft.json), [bindings.draft.json](bindings.draft.json): unapplied rows. Pin the final remote-verified art SHA before importing.
- [state_variants.json](state_variants.json): stable/endangered/unknown/inert UI treatments over canonical icons.
- [ADR.md](ADR.md): decisions and integration boundaries.

Raw attempts, rejected candidates, numeric diagnostics and full review sheets remain under `/Users/rexxa/work/art-batch/A4/`. The final Korean report is `REPORT.md` there. Curated family contact sheets live in `art/boards/techmap/`.

## Mechanical checks

```sh
uv run art/techmap/assets.py icon OUTPUT_DIR CHECK_DIR RECORD.json
uv run art/techmap/assets.py banner OUTPUT_DIR CHECK_DIR RECORD.json
uv run art/techmap/assets.py chapter OUTPUT_DIR CHECK_DIR RECORD.json
uv run art/techmap/sheets.py icon SHEET.png NORMALIZED.png
uv run --with pillow --with pydantic --with pytest python -m pytest art/techmap/test_assets.py -q
```

Multiple records/images may follow the last argument. Checks do not adopt an asset. Each selected image also requires an independent visual verdict at least90 and medium/line/light/palette each at least4. Legacy world-item numeric calibration is retained unchanged as a diagnostic, including failures.

## C2/C3 integration

The current UI has264x88 tech cards and520px chapter text width but no bitmap consumers for these families. Add explicit generic import rows, the `ui` managed directory and the metadata manifest. Then implement consumers for `ui_knowledge`, `ui_eras` and `ui_chapters`, preserve center anchors, apply state treatments and capture the actual game. Generic copying alone does not create an image slot or ingest UI anchor metadata. Resolve chapter6 by semantic ID: legacy `neolithic_village` and planned `first_copper` are separate assets. Localized titles remain UI text.

No seasons, count buckets, filled/empty or depletion variants apply to knowledge/header/chapter art.
