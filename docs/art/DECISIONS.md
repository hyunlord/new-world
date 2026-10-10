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
