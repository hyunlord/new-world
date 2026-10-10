# Decisions and provenance

- Started 2026-10-10 (Asia/Seoul). Fresh clone: `/Users/rexxa/work/new-world-art`; base `origin/lead/main`; branch `codex/art-direction-2026-10`. The explicit art brief overrides the repository's reset/push-to-lead workflow. No harness hooks installed.
- Fonts: npm `galmuri@2.40.3`, https://registry.npmjs.org/galmuri/-/galmuri-2.40.3.tgz. Galmuri11, Galmuri11-Bold, Galmuri9 preserved with upstream OFL.md, verified to name SIL OPEN FONT LICENSE Version 1.1.
- Image smoke test succeeded with built-in image_gen tool. Requested model per brief: gpt-image-2; tool does not expose or return a model identifier, so model identity is not independently verified. Requested 1024 square; actual size recorded in raw JSON sidecar. Opaque magenta background; no transparency requested.
- Raw images remain outside git under `/Users/rexxa/work/art-direction-raw/`; originals in the session CODEX_HOME generated_images directory are retained. Prompt, reference path, size and timestamp are recorded per output.
- All three directions retain 1-texel outside outlines. Top and side edges keep transparent margins; bottom outline may touch the baseline, resolving the foot-on-lower-edge requirement without clipping outlines.
- PNG normalized sprites are exact palette members. JPEG target frames are concept references, not native-grid production assets; compression and generated pixel density are explicitly outside sprite palette validation.
- User-approved Pillow, NumPy and Python standard library suffice; no extra runtime packages are added.
