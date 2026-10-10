# Art pipeline

Requires Python 3.10+, Pillow and NumPy only. This is offline art tooling; it does not modify game assets or game code.

```sh
python3 tools/art_pipeline/pixel_normalize.py \
  --sheet /absolute/path/to/sheet.png \
  --manifest tools/art_pipeline/sheet_v1.json \
  --palette docs/art/D1/palette.hex \
  --output docs/art/D1
python3 -m unittest discover -s tools/art_pipeline
```

The manifest is a complete row-major equal-cell grid. The default maps all sixteen requested objects to their native 16x16, 16x24, 16x32 or 32x32 sizes. Cell boundaries use floor(i*size/count), preserving every pixel even when sheet dimensions do not divide evenly. Neighboring cell sizes can differ by one source pixel. Empty keyed cells and malformed manifests fail explicitly.

RGB Euclidean distance <=60 removes the magenta key, followed by HSV hue within 20 degrees of magenta at saturation >0.45. Existing alpha below 128 is discarded. Each remaining opaque bounding box is fit proportionally into an integer silhouette rectangle. The silhouette reserves an outside outline texel and a transparent top/side margin; its bottom outline may touch the final baseline. Rounding can alter aspect by at most one target texel.

Each target footprint computes an area-weighted mode in 5-bit RGB buckets, using bucket centers and deterministic lowest-code tie breaking. Less than half opaque coverage becomes transparent. Transparent rows/columns left by vanished thin features are trimmed before centering and bottom alignment; validation requires at least one opaque baseline pixel. Surviving texels map to the nearest palette color by CIELAB D65 DeltaE76. An eight-neighbor outside outline uses the minimum-L* palette color, restricted to four-connected transparent background flood-filled from the canvas perimeter so enclosed holes stay transparent. All directions use outlines.

Outputs:

- `sprites/NN_name.png`: native-sized RGBA sprites.
- `sprites_contact.png`: raw cells fitted to 256px previews beside exactly 8x nearest-neighbor normalized sprites. The raw preview is only a comparison view, not a texel-density claim.
- `normalize_report.md` and `.json`: every sprite's dimensions, source opaque count and source edge contact, output opacity and colors, exact palette match percentage, partial-alpha count, forbidden top/side edge count, baseline coverage, encoded-sRGB luma and pass/fail.

Exit 1 means at least one sprite failed validation, including source foreground touching a cell edge. Reports and sprites remain available for diagnosis. Exit 0 validates the mechanical pipeline, not semantic accuracy, animation readiness or visual approval. Shadows are deliberately separate from binary-alpha sprites.

`palette_tools.extract_palette(image, count)` returns median-cut colors, repeatedly merges the closest pair with DeltaE76 <3 using pixel-count-weighted means, and sorts by HSV hue then CIELAB lightness. `save_palette(colors, path)` and `load_palette(path)` use one #RRGGBB value per line. `rgb_to_lab(array)` accepts arrays with RGB in the final dimension.

Tests cover RGB/HSV spill removal, majority coverage, mode rather than averaging, palette membership, reserved outline margins, empty-cell rejection and known D65 white conversion. A full 16-cell procedural fixture was also run through the actual CLI; all sixteen passed with 100% palette membership and zero partial-alpha/forbidden-edge pixels.

## Reproduce this delivery

The exact lossless sources and JSON sidecars remain at `/Users/rexxa/work/art-direction-raw/`. Use `D1/D1_sheet.png`, `D2/D2_sheet_retry1.png` and `D3/D3_sheet_retry1.png` with the respective committed palettes. Original rejected sheets are retained in that raw directory. `docs/art/generation_ledger.json` lists all calls, prompts, references and selected revisions. Committed sheet JPEGs are visual references: compression means they do not reproduce the exact PNG-derived metrics.

After normalizing each direction:

```sh
python3 tools/art_pipeline/presentation.py hud docs/art/D1
python3 tools/art_pipeline/readability.py docs/art/D1
# Repeat those two commands for D2 and D3.
python3 tools/art_pipeline/presentation.py compare docs/art
python3 tools/art_pipeline/audit_art.py --root . --require-source-margins
python3 -m unittest discover -s tools/art_pipeline -p 'test_*.py' -v
```

HUD renders use only palette roles, binary Galmuri font masks, exact 2x nearest expansion and reusable nine-slice panels. The lossless `hud_proof.png` demonstrates source texels; JPEG composites are not lossless pixel proofs. Night changes the clock text but does not tint the UI.

`readability.json` records encoded-sRGB luma differences on a deterministic palette-ground strip at 1x and 4x; it includes the exact ground under opaque pixels as a second measure. It never recolors sprites to manufacture a pass. `audit_art.py` treats readability failures as art comparison warnings, while malformed/native-size/palette/alpha/baseline/path/size failures are fatal. Use `--require-source-margins` to also reject source-cell boundary contact.

## Pass 2 (zero generation)

`python3 tools/art_pipeline/pixel_normalize.py --v2` rebuilds native-size sprites
and per-stage evidence from selected lossless sources. `hybrid_v2.py` creates H;
`scene_v2.py` renders exact-scale scenes, `compare_v2.py` builds the board,
`report_v2.py` updates only the Pass2 Korean assessment, and `audit_v2.py` verifies
format/provenance without hiding content failures. See docs/art/VERIFICATION_V2.md.

The legacy converter remains for reproducing pass1. Do not treat its fixed-box
outputs or old readability strip as direction-selection evidence.
