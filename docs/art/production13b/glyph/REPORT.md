# Art Pass 13b design-grid glyph validation

**PASS: machine glyph distinctness only.** Normal and bold Life sizes are24px (2×12). Windowed layout/scroll, actual input and fresh fact reading are separate leader-owned checks and are not established by this atlas result.

## Method and invocation

Attempt1, exit0; Godot4.6 stable, headless CPU TextServer ICU/HarfBuzz/Graphite. Both scripts and both weights are rendered independently at12px and24px. Binary alpha threshold0.5, common origin/baseline, no crop normalization. Missing glyph, unexpected font fallback and empty masks remain errors. All46 Korean entries and46 Latin entries plus46 initial-capital forms are included; carry/suffix closure is preserved.

```sh
P13_GLYPH_OUT=/Users/rexxa/work/new-world-art-prod/docs/art/production13b/glyph/attempt1 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path /Users/rexxa/work/new-world-art-prod --script res://scripts/test/art/test_glyph_distinct_p13.gd
```

Raw log: `/Users/rexxa/work/art-production13b-evidence/glyph-attempt1.log`. Result: [glyph-result.json](attempt1/glyph-result.json). Historical production13 24/36/48 results were not modified. The existing gate entrypoint forwards explicitly to the new helper; the helper name avoids duplicate test discovery.

## Closest pairs

| Script | Weight | All pairs | Failed design pairs | Minimum design pixels | Corresponding screen pixels |
|---|---|---:|---:|---:|---:|
| en | regular | 4186 | 0 | 3 | 12 |
| ko | regular | 1035 | 0 | 1 | 4 |
| en | bold | 4186 | 0 | 6 | 24 |
| ko | bold | 1035 | 0 | 2 | 8 |

### en regular

| Left | Right | Design pixels (12px) | Screen pixels (24px) |
|---|---|---:|---:|
| na | ha | 3 | 12 |
| ni | hi | 3 | 12 |
| no | ho | 3 | 12 |
| nu | hu | 3 | 12 |
| Do | De | 6 | 24 |
| Go | Ge | 6 | 24 |
| do | de | 6 | 24 |
| go | ge | 6 | 24 |
| i | I | 6 | 24 |
| o | e | 6 | 24 |

### ko regular

| Left | Right | Design pixels (12px) | Screen pixels (24px) |
|---|---|---:|---:|
| 나 | 니 | 1 | 4 |
| 라 | 리 | 1 | 4 |
| 마 | 미 | 1 | 4 |
| 바 | 비 | 1 | 4 |
| 사 | 시 | 1 | 4 |
| 아 | 이 | 1 | 4 |
| 카 | 키 | 1 | 4 |
| 타 | 티 | 1 | 4 |
| 하 | 히 | 1 | 4 |
| 이 | 야 | 2 | 8 |

### en bold

| Left | Right | Design pixels (12px) | Screen pixels (24px) |
|---|---|---:|---:|
| na | ha | 6 | 24 |
| ni | hi | 6 | 24 |
| no | ho | 6 | 24 |
| nu | hu | 6 | 24 |
| Ba | Da | 7 | 28 |
| Ba | Ga | 7 | 28 |
| Bo | Do | 7 | 28 |
| Bo | Go | 7 | 28 |
| ha | ba | 7 | 28 |
| hi | bi | 7 | 28 |

### ko bold

| Left | Right | Design pixels (12px) | Screen pixels (24px) |
|---|---|---:|---:|
| 나 | 니 | 2 | 8 |
| 라 | 리 | 2 | 8 |
| 마 | 미 | 2 | 8 |
| 바 | 비 | 2 | 8 |
| 사 | 시 | 2 | 8 |
| 아 | 이 | 2 | 8 |
| 카 | 키 | 2 | 8 |
| 타 | 티 | 2 | 8 |
| 하 | 히 | 2 | 8 |
| 모 | 도 | 4 | 16 |

## Font and evidence identity

Font settings and SHA256 are recorded in the result JSON. Font assets and PersonNames tables are unchanged. All10442 pair comparisons passed; no missing/fallback/empty-mask errors. Screen values are measured from separately rendered24px masks, not inferred by multiplying design values.

All atlas and pair files for this invocation:

- `en-bold-12-binary-atlas.png` SHA256 `79d1b30c5735d3d172bd0bd9bd36f1eccd8395f291c3cb803c529dd92f242449`
- `en-bold-24-binary-atlas.png` SHA256 `b2fa5ba0437c92cd4223f71c97d013cc8a50e776250966427702551e27b9e54d`
- `en-bold-design12-screen24-all-pairs.json` SHA256 `5b4035ffb7b5e8ccd065388b92ab225d4f88ef9256b609e9bcacfe4bbfdd19c8`
- `en-regular-12-binary-atlas.png` SHA256 `833cf5bb46b828059f9ab6db02acd1fa63fbc4dceb64daf294bef64e3b746095`
- `en-regular-24-binary-atlas.png` SHA256 `37e718f16fee70c06c59b972d91267a87f26d252f1b184f2aab3a571344163de`
- `en-regular-design12-screen24-all-pairs.json` SHA256 `d3ebb885bfa23c6d71d649384cd3ee64f23db51c8eb98366ec1b7b0c452876a4`
- `glyph-result.json` SHA256 `ad8c1eb01544296a92d6aa98c8c797c598d98a48914bb39f6c98cb729c0d52a9`
- `ko-bold-12-binary-atlas.png` SHA256 `e522d73c17b8bf064ef5314d67a93ef1b602d056a1eed6acaa3746743957daca`
- `ko-bold-24-binary-atlas.png` SHA256 `6c470fc4a2b9a8a5081fe1af26ac97bc872fc005b46fc8441234382a640cc566`
- `ko-bold-design12-screen24-all-pairs.json` SHA256 `79ac86b9ace53bafac6a3ad266d5c1539c4b879813e1ed6f8225a7fc173e8d9b`
- `ko-regular-12-binary-atlas.png` SHA256 `1cdd2dcf4fc8f64bf15b8e5d716483e632461db9103bd5a67118b4818af0e28e`
- `ko-regular-24-binary-atlas.png` SHA256 `18c9d0c33e3e4a0c27ceb650ecb55b5d80df6f3cf85b49c04f8a064e2f4d9b49`
- `ko-regular-design12-screen24-all-pairs.json` SHA256 `7818bccb2f9597c96d98c62303977ab24d8f784f8c7d05ca35a4febddde14112`
