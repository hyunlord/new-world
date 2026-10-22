# Producing a style 05 asset

## Current TRIAL2b override (2026-10-22)

The earlier TRIAL2 procedure below is historical where it conflicts with this section. The map02 fallback is withdrawn pending the corrected trial; this is not production approval.

- Characters use front three-quarter, about20 degrees down, head about30% height; south/east and mirrored west. Map objects remain overhead.
- Use the active same-family crops in `anchors/manifest.json` v2. Never use a whole-scene global histogram as the acceptance instrument. Numeric v2 measures edges, local saturation, dark-ridge stroke distributions and enclosed wash variance. Twelve leave-one-crop-out folds and three person controls are the measured calibration; correlated crops and control content confounds limit generalization.
- The model judge receives two same-family crop images and the exact raw candidate. Preserve scorecards and byte identities. All four scores must be at least4; C1/C2 style acceptance is numeric AND model. Content/placement QA can separately withhold promotion and must not silently redefine these rates.
- Three attempts maximum per environment asset or part, including all prior attempts; person anchor alone permits four. Calm ground is a separately accounted replacement cohort. Charge every failed and superseded generation.
- Ground: calm large-scale wash, no scattered detail; standard deviation at most60% of person foreground and mean saturation at most0.35. Global contrast/chroma only; no quantization or contour forcing. Tile-aware generation first, explicitly disclose any mirror fallback.
- Accepted environment sprites live in `env/`; people, `ground_v2/` and `autotile_v2/` are explicitly labeled trial candidates retained to reproduce this requested comparison, not blanket production acceptance. Six original anchors remain immutable.
- Final acceptance and limits: `docs/v8/arttrial2/TRIAL2B_REPORT.md`, `ENV_ACCEPTANCE_V2.json`, `JUDGE_V2_AUDIT.md`.

## Historical TRIAL2 procedure

Read [STYLE.md](STYLE.md) for the prompt and [anchors/ANCHORS.md](anchors/ANCHORS.md) for immutable references. This is the production procedure; its existence is not evidence that calibration or a particular asset passed.

1. Name the asset with lowercase snake case under `env/<family>/`, `autotile/` or `people/`. Decide the subject, exact count, tile footprint, working size and attachment point before generation. Map tiles are 64 px; adult people approximately 56 px. Ground is 512×512; plant canopies 96–160 px; buildings 64×footprint; items 48–64 px; misc 64 px.
2. Attach the nearest locked anchor and `anchors/scene.png` to the image tool. Ground/plants → ground; buildings/walls → hut; items/props → props; body/overlays → person; portraits → portrait. Use STYLE.md's full template with exact subject constraints. Cut-outs use solid `#FF00FF`; ground uses seamless tileable natural material. On retries retain both anchors and describe the concrete defect to fix.
3. Save every unmodified output and reject outside Git in `/Users/rexxa/work/arttrial2-data/`. Record request start/end time, full prompt, attached paths/hashes, tool generation ID when supplied, original path/hash, outcome and failure reason. Environment: ≤4 attempts per asset; people: ≤3 per part; wall source pieces: ≤3 per piece. Stop a family when its running average exceeds 3.0 tries; record the denominator and failures. Count sheet generations as tool calls and per-part exposure separately, never as free successful parts.
4. Produce a working PNG with recorded operations only: chroma-key removal, Lanczos downscale, cropping, stray-pixel alpha clean-up and composition. No palette quantization, outline forcing or block reduction. Ground failing seamlessness twice may use the goal's explicit mirror-blend fallback; name that exception and its parameters. Preserve originals. Record generated/derived lineage for shadows, ghosts and autotiles instead of calling derivations new generations.
5. Run `art/pipeline2/judge.py` using its implemented CLI and the calibrated anchor configuration. Save all colour-distance, Canny edge-density and saturation results. Independently request a fresh-context judge with the two anchors and candidate: medium, line, light and palette scored 1–5 plus one sentence. All four must be ≥4 and all numeric checks must pass. Preserve exact reviewer output and mechanism; missing image-model review is NOT_VERIFIED, not a substituted pass. Calibrate before claiming acceptance: 6/6 held-out anchors pass and 3/3 wrong-style controls fail.
6. Check the subject as well as style: exact count, overhead camera, south work/opening side, alpha fringe, no clipped part, and readability at game scale. Ground needs a 3×3 repeat plus edge metric; tall objects need placement anchors; autotiles need corner/connection seam checks in the proto. A style PASS does not prove these conditions.
7. Save `<name>.png` with `<name>.json`, then add it to the family ledger/contact sheet. Each working PNG must be ≤1.5 MB; the six locked originals alone have the exception. Keep cumulative `art/` growth ≤40 MB including anchors and boards. Commit accepted assets only; rejected and pending candidates remain in the data directory. Publish trial work only on `v8/art-trial2`, with explicit paths and verified remote HEAD; never merge it into game branches.

## Sidecar contract

Each accepted asset's JSON must contain the actual full prompt, two or more anchor paths and SHA-256 hashes, try count, generation ID (or `null` when not returned), original path and SHA-256, working width/height, ordered operations with parameters, numeric results and thresholds, all four model scores, verbatim reason and reviewer identity/context provenance, final acceptance, and wall seconds. Tall assets additionally record `anchor_point` in working-image pixels and coordinate convention (origin top-left; x right, y down); rig parts record joint attachment coordinates.

For derived assets include parent paths/hashes and mapping/composition recipe. Do not invent a generation ID or pretend a parent judge score is an independent derived-image review. Record omitted or unavailable fields as `null` with a reason and acceptance as PENDING/NOT_VERIFIED until required evidence exists. Keep sidecar field names consistent with the implemented pipeline schema.

## Accounting and handoff

For each family record total tool calls, per-asset tries, accepted count, first-try and after-retry judge pass rates with explicit denominators, failures/stops, and measured wall time. Keep generation latency, processing/review time and total elapsed time distinct. Include rejected attempts in cost and sustainability projections. Retain review images at 1×/2× and the real Godot captures separately from static contact sheets. A technical test or contact sheet never substitutes for runtime, animation, readability or FPS evidence.

## Rehearsal result

ART-TRIAL2 concluded with C1/C2 failures and the preregistered map02 fallback recommendation. Do not treat this guide as production approval. Use docs/v8/arttrial2/JUDGE_PROTOCOL_V2.md for the recorded model protocol: combined calibration passes, but the model alone rejected zero of three negative controls. Preserve numeric failures and semantic misidentifications. Only packed_earth and hill_rock were accepted.
