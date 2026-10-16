# Decisions and provenance

- Started 2026-10-10 (Asia/Seoul). Fresh clone: `/Users/rexxa/work/new-world-art`; base `origin/lead/main`; branch `codex/art-direction-2026-10`. The explicit art brief overrides the repository's reset/push-to-lead workflow. No harness hooks installed.
- Fonts: npm `galmuri@2.40.3`, https://registry.npmjs.org/galmuri/-/galmuri-2.40.3.tgz. Galmuri11, Galmuri11-Bold, Galmuri9 preserved with upstream OFL.md, verified to name SIL OPEN FONT LICENSE Version 1.1.
- Image smoke test succeeded with built-in image_gen tool. Requested model per brief: gpt-image-2; tool does not expose or return a model identifier, so model identity is not independently verified. Requested 1024 square; actual size recorded in raw JSON sidecar. Opaque magenta background; no transparency requested.
- Raw images remain outside git under `/Users/rexxa/work/art-direction-raw/`; originals in the session CODEX_HOME generated_images directory are retained. Prompt, reference path, size and timestamp are recorded per output.
- All three directions retain 1-texel outside outlines. Top and side edges keep transparent margins; bottom outline may touch the baseline, resolving the foot-on-lower-edge requirement without clipping outlines.
- PNG normalized sprites are exact palette members. JPEG target frames are concept references, not native-grid production assets; compression and generated pixel density are explicitly outside sprite palette validation.
- User-approved Pillow, NumPy and Python standard library suffice; no extra runtime packages are added.

- Baseline cargo test failed in harness_p12_beta_a25_gdscript_parses_cleanly_in_godot_headless: missing WorldSimNode type in scripts/ui/world_renderer.gd at lines 192 and 222 in this fresh clone. No game repair attempted because out of scope. Independent cargo clippy --workspace -- -D warnings passed.
- Extracted palette sizes: D1 31/32, D2 48/48, D3 19/20. Lower counts follow required ΔE<3 merging.

- Generation budget final: 15 successful calls, zero tool errors; D1=4, D2=5, D3=5, smoke=1. Two initial sheets rejected for source cell boundary crossings; their raw originals remain outside git. Replacements add explicit 22% empty margins. No further generation planned.
- First-pass HUD neutral selection produced insufficient light text. Revised selection keeps actual palette colors while choosing a light neutral-role color among the upper luma quartile; measured text/fill WCAG contrast exceeds 6:1 for all directions.
- Independent review found downsampling may remove a thin foot/tail and leave retained content floating. Trim post-sampling transparent bounds, bottom-align surviving pixels, validate baseline occupancy; regression test added and all 48 outputs regenerated.
- All normalized sprites pass structural tests. D1/D2 fail the 0.15 readability experiment; D3 passes. Preserve these failures as comparison evidence. No threshold relaxation or contrast-driven background cherry-picking.
- Shared shadow is a separate production-layer specification in the bible; it is not baked into binary-alpha normalized sprite outputs or their luma measurements.
- Two review rounds completed: first found baseline defect; second confirmed repair. Static Korean HUD review passed. No production decision made on behalf of owner.

## Pass 2

- The explicit pass-2 attachment authorizes this art branch and overrides the generic lead/main reset/merge workflow. Only docs/art and tools/art_pipeline change. Zero image-generation calls.
- Use the lossless selected originals: D1_sheet.png, D2_sheet_retry1.png, D3_sheet_retry1.png. Preserve pass-1 files and scores as historical evidence.
- Recover native silhouette dimensions first. Class maxima apply before the extra outside-outline padding; final canvas and anchors record that padding separately. No scale-to-fit.
- Overview is a 1024x1024 world (64x64 tiles), centered without resampling in the 1920x1080 frame. HUD remains the pass-1 Galmuri 2x layer.
- Graft graph is absent in this art clone; direct source reads provide the implementation evidence.
- Native key rule remains exactly R>150, B>150, G<110, abs(R-B)<90 (alpha>=128). Axis periods differing by more than0.3 remain independent as requested; uncertain anisotropic results are flagged regenerate rather than manually corrected.
- Fidelity is conservative: all step-3 opaque texels participate, including texels removed by required spill cleanup (compared against transparent black). Both direction mean and individual sprite mean are reported. Intentional alpha changes are diagnostic, not a separate failure.
- Skin is a native adult-derived HSV/ΔE palette proxy and may include clothing. Water has no sheet cell, so the highlight is the nearest real native color to cool #BEDCE6. Neither proves anatomical segmentation or a newly authored water sprite.
- Part1: 23 unit tests pass, including byte-exact5.9 phase recovery and each semantic check. D1/D2/D3 palette sizes67/86/57; strict content failures retained. Format checks alone no longer imply production readiness.
- H uses D1 scene ground colors plus D1 boulder/tree/reed colors, and actual D3 actor output colors. Stable ΔE<3 merging retains original input colors; 111 colors remain. Source regeneration flags are inherited. No new source art.
- Round1 review fixed H contact-cell overlap by deriving cell dimensions from actual8x sprite size. Ground ramps now select distinct source colors for sparse decorations, retaining the same base color rather than picking colors to inflate contrast.
- Full-box visibility retains the requested all-villagers-removed comparison. Contrast means use per-villager opaque contribution ID masks after y-sort/HUD occlusion; old box means remain separately reported. D1 has overlapping actor boxes, so full-box visibility can include another actor's change.
- ±1-degree clothing variation is an attempted low-saturation lower-torso proxy, not verified anatomical segmentation. Neutral colors can produce no changes; actual counts are saved. Do not claim skin exclusion.
- Part3 exact-scale renderer and 7 new compositor tests pass. Combined suite30 tests. Format audit passes64/64 sprites; content/size regeneration flags remain. All four pixel-visibility targets pass, not equivalent to human role recognition.
- Round2 independent code and visual review found no remaining delivery-blocking issues; 30 tests reproduced. Keep production holds. Board sprite atlases use the same2x scale across directions to avoid direction-dependent display magnification; game-screen thumbnails are1/4 of full files.
- Production estimate is conditional: minimum13 sixteen-cell generations for200 unique assets, about67 using current D3 individual acceptance3/16. Systematic grid/spill failures make retry convergence unproven; no production generation is authorized or started.
