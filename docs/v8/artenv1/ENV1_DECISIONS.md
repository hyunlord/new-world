# ART-ENV1 Decisions

These decisions implement `ENV1_GOAL.md` without changing its bars. Initial record: 2026-10-23. They are execution choices, not owner approval. Later evidence may add a superseding decision while retaining the original entry.

## D01 — Scope, base and time remain fixed

The task explicitly authorizes `v8/art-env1` from `159ef0f9a1cfb7ee9e6005e85d5388d677e07fa8`, with the goal committed as `3fb5813c37470488c3a64ebbdffba108aba71ee2`. This overrides the general development branch convention for this art pass. Only `art/**` and `docs/v8/artenv1/**` are changed; external raw data goes to `/Users/rexxa/work/artenv1-data/`. Root docs, skills, game code and existing unrelated dirt are outside scope. No merge or installation is authorized.

The goal service creation timestamp `1792736230` is the time authority: start **2026-10-23 15:17:10 KST**, eight-hour target **23:17:10 KST**, ten-hour hard stop **2026-10-24 01:17:10 KST**. A preliminary 14:17 estimate was incorrect and is not used. Continuation does not reset the deadline.

## D02 — Current adoption supersedes historical Route I failures

Use the current `artprod3/HANDOVER.md`, TEMPLATE_V3 and route_i manifest. They record owner/planner adoption of 34 looks with two explicit adult fallbacks. Older trials' non-adoption remains historical evidence, not a reason to switch character routes or regenerate accepted people. The existing 34 looks and their pixel bytes stay unchanged; this pass does not claim a new owner decision.

## D03 — Simulation identity is pinned independently of the art base

Use `v8/wip` commit `3a3b236c125c0c25e9b74261be88536473fc01e9` acquired at the start in a detached, source-unmodified worktree. Build outputs and scene runs are external. Record the actual CLI, calendar and available metadata before drawing. `--seeds 0..1` is half-open and requests seed 0. If `people-json` does not expose sex/life stage, use an existing richer read-only source if available; otherwise explicitly declare appearance defaults and report the data gap. Never invent real metadata from a glyph or from a generated appearance.

## D04 — Time pressure reduces only the authorized optional layers

Preserve mandatory inventory, walls/workshops, wolf/deer, real-scene day/night/winter at 1× and ½×, comparison, crowd test, boards and report. Optional work drops in this order: portraits, drought, ¼×, item variants beyond one per kind. The shared monotonic ledger counts every launched generation attempt, including errors/rejections. Global limit is 260; normal asset and animal sheet limits are three; portrait limit is two. Repackaging or renaming a failed candidate cannot reset its budget.

## D05 — Stronger ENV1 ground cap and no mirror-blend

ENV1's ground saturation mean ≤0.30 supersedes STYLE.md's historical 0.35. Ground luminance mean ≤0.40 and standard deviation ≤60% of the foreground-masked person anchor are measured with documented colour-space/mask conventions. Apply only logged global ground transforms and geometry. Wall strips/corners are generated and geometrically derived; no mirror-blend, quantization, pixel surgery or repaired painted contours.

## D06 — Separate visible truth, declared overlay and missing comparison facts

Real text snapshots and the M3–M5 overlay remain separate sources with stable IDs. Every overlay object is `declared=true` and identified in the render legend. Whether apparel/action overlays modify existing agents or add new agents is explicit. Current-renderer contact-sheet crops are baseline evidence with exact crop/source hashes; seed/day/condition/zoom/content mismatches are registered before scoring. Unknown baseline truth stays unknown, and an unanswerable or incomparable cell cannot establish non-inferiority.

## D07 — Reviewers see pixels without the key; timing is observed and qualified

Scene comparison covers all 3 conditions × 2 zooms × 2 renderers × 2 reviewers × 6 tasks. Use fresh context and independent reviewers per condition, randomized opaque packet IDs, actual image opening, immutable raw answers and a separately held answer key. The fixed per-image response budget is 60 seconds; retain actual observed timing, late answers and latency qualification. Model/tool elapsed time is not human perceptual reaction time. If timing cannot be observed, record `NOT_MEASURED`.

Score count within ±10%, exact carriers, location/presence answers, time class and three individual activities under the preregistered rubric in `ENV1_PLAN.md`. Compare the two-reviewer sum for every task/zoom/condition, and require both ½× LOD counts within ±10%; overall averages cannot erase failed cells. A scene-readability repair is allowed once, followed by a new pair of reviewers per condition and a complete matrix rerun. Keep every round.

## D08 — Animal calibration is distinct from human geometry

Wolf and deer use whole figures, S/E rows and five specified poses with common S-stand scale; carcasses reuse dead frames. Freeze two animal anchors and their family calibration before candidate acceptance; do not retune thresholds after failures. Preserve TEMPLATE_V3's ownership/connectivity/no-clipping principles while defining animal shoulder-height measurements explicitly. A 96×64 dead canvas cannot contain a y=72 foot anchor: A1 must specify an in-canvas dead contact anchor and record this geometry interpretation instead of clipping or stretching the figure.

Animal review retains each reviewer/species/zoom denominator of ten: species ≥9 and pose ≥8. A favorable aggregate cannot compensate for one failed reviewer or zoom. The item 32 px naming review has no invented numerical acceptance threshold; record exact confusions.

## D09 — FPS means actual completed renders in the required real window

Measure the standalone art prototype, not historical PROD2 numbers or simulation speed: 200 Route I people, 400 environment sprites and ground; 60 seconds each at 1× and ½× with LOD. Count completed rendered frames, separately from `_process`; preserve capture, engine/renderer/vsync/viewport/settings and asset hashes. A headless/dummy run does not satisfy the bar. Both measured values must be ≥60 without upward rounding or favorable-run selection.

## D10 — Publication evidence does not imply art or owner acceptance

Use a conservative decimal file limit of 1,500,000 bytes. Optimize PNG losslessly or resize boards geometrically; mandatory full-resolution renders remain 1600×900 and are never quantized. Verify each final artifact visually and by decode/hash. Publish the Korean report with all failed/deferred/unverified criteria and both owner boards. Explicit staging and a matching `git ls-remote origin refs/heads/v8/art-env1` prove delivery only. The report must not claim the owner's approval or bridge installation.

## D11 — Native bridge layers supplement the text view without replacing it

The pinned probe text/people output does not encode every item kind, structure or activity. A read-only extractor joins the same saved world through the existing bridge collectors, after verifying the native ASCII and people coordinates exactly. `scenes/native_layers.json` preserves actual mixed stockpiles, completed structures, blueprint sites, zones and animals. Declared M3–M5 objects remain a separate overlay; their coordinates must never become gameplay truth. The genuine unfinished walled stockpile and the declared finished storehouse are both valid storage targets in the scene review.

The first winter noon is tick 26160 (day 19); day 30 is spring under the source calendar. Native routines that the current renderer draws as a standing frame are scored as visually awake/idle, with the invisible routine retained separately. The after overlay repositions and changes actions of seven existing people; it does not add population.

## D12 — Freeze calmer ground before the first scene review

The first review uses a stricter implementation cap: mean luminance ≤0.32, luminance standard deviation ≤25% of the person anchor, saturation mean ≤0.22, with a 90% target margin for quantized channel rounding. These are inside the original maxima 0.40 / 60% / 0.30, not replacement acceptance thresholds. Per-tile evidence measures the final transformed pixels. Night retains the prescribed global value factor 0.55 and five-tile hearth restoration radius; winter changes ground only and uses the one generated snow texture at alpha ≤0.60. A snow texture failure remains a candidate failure even when compositing obeys the alpha limit.

## D13 — Preserve failed calibration and bounded semantic results

Wolf try3 exhausts the species budget. Fresh round1 reviewer D scores wolf poses 7/10 at both sizes, below the unchanged 8/10 bar; deer passes all round1 groups. Animal numeric negative controls reject only 1/3 historical negatives. A later model AND screen rejecting all three does not erase that numeric calibration failure. Portrait numeric and model negative controls each reject only 1/3; all 34 portraits remain candidates. Item silhouette confusions at 32px remain explicit, without inventing an aggregate acceptance threshold. No fourth animal attempt, third portrait attempt, second snow generation or painted repair is authorized by these findings.

## Resolved acquisition decisions

| Item | Evidence/action needed | Current state |
|---|---|---|
| Exact source ticks/calendar | `SCENE_SOURCE.md`, source manifest | VERIFIED: 7440 / 6840 / 26160 |
| Real people metadata availability | Native text/people joined to read-only engine and bridge extraction | VERIFIED; same save, positions and IDs |
| Baseline comparability | `BASELINE.md`, `READABILITY_PROTOCOL.md` | Real same-save captures; declared content/action mismatch retained |
| Animal anchor provenance and shoulder measurement | `TEMPLATE_A1.md`, animal calibration and reviews | Frozen; numeric calibration FAIL retained |
| Exact source/scene/render output manifest | `render_manifest.json`, per-render source/asset evidence | Final freeze required before packets |
| Optional work retention | Portraits, drought, quarter zoom | Retained within the original time and call budgets |

## D14 — Inventory UI-state deferrals and bounded final repair

The goal permits explicit per-row inventory deferrals. Deferred designations M00–05/M07–08 and room-role labels R00–04 need a coherent UI symbol/label vocabulary; this pass chooses to leave that convention to M-ART2 integration instead of declaring a new one through static art. B15/E17 roof/indoors depend on live visibility/roof clipping policy. E06–08 illness/draft/mood need state-to-cue binding and owner-facing UI treatment; those states are not inferred from portraits or poses. These are chosen scope boundaries, not lack-of-time or technical-impossibility claims. Required built/workshop asset categories and code site wash are delivered separately. Deferred/partial counts and acceptance failures remain unchanged.

Round0 readability regressions triggered the single permitted repair documented in `round1repair/DECISION.md`: larger distinct axe/log marks and declared stump context, with unchanged actors/actions/positions/thresholds and preserved prior evidence. Round1 uses six new reviewers, the same neutral tasks and new opaque IDs. No second repair follows its outcome.

The crowd harness uses a frozen manifest whitelist after an early diagnostic-image preflight failure. Round1 glyph PNGs are exported from the compositor function, whole geometry only. An uncertain window-start overlap is retained as such; the final pair uses `process.wait()` before launching the next visible window. No result is selected for being fastest. See `FPS.md` for every attempt.
