# Pass 2 verification

- No image-generation calls. Three selected lossless PNG SHA256 values rechecked against manifests.
- Python unit suite:30 tests pass, including exact native RGBA recovery from5.9x scale with phase offsets, anisotropic synthetic sampling, all content checks, exact4x/1x output, y-sort/object/HUD attribution and transparent-box controls.
- `compileall` and `git diff --check` pass. Ruff/basedpyright are not installed; no lint/typecheck success is claimed.
- `audit_v2.py` verifies64 output sprites, exact palette membership, binary alpha, native dimensions plus outline, anchors, all20 1920x1080 PNGs,20 villagers per zoom, arithmetic consistency and source hashes.
- Structural pass is not semantic acceptance: D1/D2/D3 content checks fail as recorded. Regenerate/review counts D1=16,D2=16,D3=13,H=14.
- Review rounds capped at2. Round1 independent code/visual review found H contact-cell overlap, box-mean dilution and unsupported clothing-mask claims; corrected layout, attributed contrast and recorded mask limitations. The final comparison uses unchanged actor pixels, with no hand-painted semantic fixes.
- Existing pass-1 Rust baseline failed because WorldSimNode was absent in fresh-clone Godot parsing (`world_renderer.gd`:192/222); clippy had passed then. Pass2 modifies no game source and does not claim a fresh Rust/runtime/device gate.

## Reproduction

Run in `/Users/rexxa/work/new-world-art` with Python, Pillow and NumPy:

```sh
python3 tools/art_pipeline/pixel_normalize.py --v2
python3 tools/art_pipeline/hybrid_v2.py
python3 tools/art_pipeline/scene_v2.py --root docs/art
python3 tools/art_pipeline/compare_v2.py
python3 tools/art_pipeline/report_v2.py
python3 -m unittest discover -s tools/art_pipeline -v
python3 tools/art_pipeline/audit_v2.py
python3 -m compileall -q tools/art_pipeline
git diff --check
```

Pass1 legacy CLI and outputs remain available. Raw sources default to
`/Users/rexxa/work/art-direction-raw`; v2 accepts `--raw-root` and `--root`.
Each source cell's sample/native/quantized/final PNG stages are preserved beside
its manifest. `*.sample.png` is step3 before native spill removal.

## Publication evidence

Each Part was pushed only to `codex/art-direction-2026-10` and matched by
`git ls-remote origin refs/heads/codex/art-direction-2026-10`:

- Part1:7671ee29d8b133cea858ebfac66526e6dc9308fc
- Part2:64d3d31f99a7e71e8c56c78af9bac3be24f50603
- Part3:e2718f3ea0982067e0cd4d6831b56a4a3a9ce1e2
- Part4: final response records full HEAD after publication (a commit cannot contain its own hash).

Only `docs/art/**` and `tools/art_pipeline/**` are included in pass2 commits.
Any untracked `graft/` is background-generated navigation data, excluded from delivery.
