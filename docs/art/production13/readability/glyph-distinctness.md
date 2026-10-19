# Glyph distinctness — actual 48 px passes; 24/36 failures retained

Strict windowless Godot 4.6 TextServer CPU atlas test. The actual Life normal font is Galmuri11 and bold is Galmuri11-Bold; AA, hinting and subpixel positioning are disabled, oversampling is automatic (0). No fallback font RIDs were used. At capture, Life inherited 24 px. Production settings were not changed by this test.

Both font files have `head.unitsPerEm=1200`; the GCD of every simple outline x/y coordinate is 100. Thus the pixel design grid is **12 px**, despite the family name Galmuri11. Actual24, diagnostic36 and48 are integer 2×,3×,4× scales. The raw `font-design-grid.json` records hashes and coordinate counts.

Every table entry is included: 46 Hangul, 46 lowercase Latin plus 46 initial-capital Latin forms used by display_name. All carry assignments are `(index+1)%46`, so carry cannot introduce an out-of-table syllable. Every same-script pair is checked: 1,035 Hangul and4,186 Latin pairs per weight/size. Rendered masks share a baseline/origin and use shaped advances/offsets. No per-glyph bounding-box normalization. Alpha is thresholded at>=0.5; symmetric-difference bar is>=12 pixels.

| Size | Script | Weight | Min differing pixels | Failing pairs |
|---:|---|---|---:|---:|
| 24 | en | regular | 12 | 0 |
| 24 | ko | regular | 4 | 11 |
| 24 | en | bold | 24 | 0 |
| 24 | ko | bold | 8 | 9 |
| 36 | en | regular | 27 | 0 |
| 36 | ko | regular | 9 | 9 |
| 36 | en | bold | 54 | 0 |
| 36 | ko | bold | 18 | 0 |
| 48 | en | regular | 48 | 0 |
| 48 | ko | regular | 16 | 0 |
| 48 | en | bold | 96 | 0 |
| 48 | ko | bold | 32 | 0 |

The initial actual24 failed. After the prescribed integer-scale search, Life normal_font_size and bold_font_size were overridden to48 in agent_inspector_panel.gd. A fresh test independently read both actual sizes as48 and passed all10,442 pairs with no errors. The CLI returned0, stderr was empty and the completion sentinel reported actual_pass=true. Raw actual-settings evidence is `/Users/rexxa/work/art-production13-evidence/readability/actual48`, with committed summary [glyph-actual48.json](glyph-actual48.json). Historical24/36 failures remain unchanged. The Life content width is332px; larger text increases wrapping/height substantially. Clipping, scrolling, actual UI composition, and human reading remain NOT_VERIFIED. CPU font raster evidence is not framebuffer identity or frame-performance evidence.

Raw evidence: `/Users/rexxa/work/art-production13-evidence/readability/glyph-attempt4`; CLI result `/Users/rexxa/work/art-production13-evidence/glyph-cli-attempt4.json`. Engine exit1 is the expected actual-size failure, with `GLYPH_DISTINCT_P13_COMPLETE actual_pass=false errors=0` and empty stderr. Attempts1–2 preserve harness-development errors; attempt3 preserves24/36 results. Attempt4 is complete24/36/48 evidence.

## Ten closest pairs per script, weight and size

- en / regular / 24px: nu–hu 12; no–ho 12; na–ha 12; ni–hi 12; do–de 24; go–ge 24; Do–De 24; o–e 24; i–I 24; Go–Ge 24
- ko / regular / 24px: 마–미 4; 타–티 4; 나–니 4; 라–리 4; 사–시 4; 하–히 4; 바–비 4; 카–키 4; 아–이 4; 투–루 8
- en / bold / 24px: no–ho 24; na–ha 24; nu–hu 24; ni–hi 24; Bo–Go 28; Bo–Do 28; Ba–Ga 28; Ba–Da 28; ho–bo 28; hu–bu 28
- ko / bold / 24px: 아–이 8; 나–니 8; 마–미 8; 하–히 8; 사–시 8; 바–비 8; 라–리 8; 타–티 8; 카–키 8; 쿠–무 16
- en / regular / 36px: nu–hu 27; no–ho 27; na–ha 27; ni–hi 27; do–de 54; go–ge 54; Do–De 54; o–e 54; i–I 54; Go–Ge 54
- ko / regular / 36px: 마–미 9; 타–티 9; 나–니 9; 라–리 9; 사–시 9; 하–히 9; 바–비 9; 카–키 9; 아–이 9; 투–루 18
- en / bold / 36px: no–ho 54; na–ha 54; nu–hu 54; ni–hi 54; Bo–Go 63; Bo–Do 63; Ba–Ga 63; Ba–Da 63; ho–bo 63; hu–bu 63
- ko / bold / 36px: 아–이 18; 나–니 18; 마–미 18; 하–히 18; 사–시 18; 바–비 18; 라–리 18; 타–티 18; 카–키 18; 쿠–무 36
- en / regular / 48px: nu–hu 48; no–ho 48; na–ha 48; ni–hi 48; do–de 96; go–ge 96; Do–De 96; o–e 96; i–I 96; Go–Ge 96
- ko / regular / 48px: 마–미 16; 타–티 16; 나–니 16; 라–리 16; 사–시 16; 하–히 16; 바–비 16; 카–키 16; 아–이 16; 투–루 32
- en / bold / 48px: no–ho 96; na–ha 96; nu–hu 96; ni–hi 96; Bo–Go 112; Bo–Do 112; Ba–Ga 112; Ba–Da 112; ho–bo 112; hu–bu 112
- ko / bold / 48px: 아–이 32; 나–니 32; 마–미 32; 하–히 32; 사–시 32; 바–비 32; 라–리 32; 타–티 32; 카–키 32; 쿠–무 64

Reproduce with `P13_GLYPH_OUT=<fresh external directory> GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot cli-anything-godot --json -p /Users/rexxa/work/new-world-art-prod script run scripts/test/art/test_glyph_distinct_p13.gd --timeout 240`. Require the completion sentinel, engine returncode and empty stderr; wrapper exit alone is insufficient. The final script additionally records independent actual bold size, requiring a new run after production overrides.
