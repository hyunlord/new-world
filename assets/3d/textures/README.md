# Generated material textures — Part T

Forty 1024-square albedo PNGs and eight luminance-derived rough-surface normals.
The shared `albedo_atlas.webp` and lossless `normal_atlas.webp` are 8192-square,
row-major 8×8; each 1024 cell contains a 1008-square image and eight wrapped gutter
pixels on every side. Tile order is `../textures3d.json`. Runtime detail uses
luminance to preserve the authored material tint. The 1024-square PNGs remain
available as individual assets; gameplay loads the shared atlases.

Generated with the built-in `image_gen__imagegen` tool for this project, following
the owner's 2026-11-16 Part T brief. All forty assets were adopted on their first
generation attempt. No third-party asset, logo or in-image text is included.

Exact prompts, original generation files, processing script, 2×2 repeat checks,
seam metrics and verification logs are retained at
`/Users/rexxa/work/art-img-data/tex/` (`manifest.json`, `raw/`,
`texture_process.py`, `tiles/`, `seam_metrics.json`, `REPORT.md`).

Opposite albedo boundary texels match exactly after periodic-plus-smooth
correction. Normal gradients wrap. Import overrides enable mipmaps and VRAM
compression; ordinary headless imports do not prove windowed GPU performance.
See `docs/decisions/0068-generated-material-texture-atlas.md` for the sampling
tradeoffs, and the external report for actual screenshot and FPS status.
