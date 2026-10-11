# Part 8 measurement protocol

Run from repository root after final captures:

```
python3 tools/art_pipeline/make_blind_grid_p6.py create --raw RAW_ART --output docs/art/production6/blind --key CAPTURE_EVIDENCE/blind_key.json
python3 tools/art_pipeline/make_blind_grid_p6.py grade --labels OBSERVER_1 --key CAPTURE_EVIDENCE/blind_key.json --output docs/art/production6/blind/observer1_result.json
python3 tools/art_pipeline/make_blind_grid_p6.py grade --labels OBSERVER_2 --key CAPTURE_EVIDENCE/blind_key.json --output docs/art/production6/blind/observer2_result.json
python3 tools/art_pipeline/blind_p6.py docs/art/production6/blind/observer1_result.json docs/art/production6/blind/observer2_result.json docs/art/production6/blind/result.json
python3 tools/art_pipeline/review_boards_p6.py RAW_ART RAW_LONG RAW_GOD
python3 tools/art_pipeline/harvest_integrity_p6.py
python3 tools/art_pipeline/measure_p6.py RAW_ART RAW_LONG GATE_DIR RAW_GOD
python3 tools/art_pipeline/presentation_p6.py RAW_ART RAW_LONG RAW_GOD
```

A11 observers see only numbered blind images and allowed labels. Freeze answer key before dispatch. Each observer must independently score 4x >= .85, 2x >= .70, carry >=2/3. No averaged PASS. Natural capture-set expected 21 long frames; ART 94 unchanged.

## Independent visual observer instructions

Do not provide pipeline source or implementation goals. Provide original captures plus identity-only boards in `review/boards`, `review/board_sources.json`, and cold cycle images where available. Boards annotate object identities only, not expected cold/wood state. Original lossless screenshot remains authoritative; annotation does not replace it.

Write `visual_observations.json` with `frames` mapping original stems to records:

- `sha256`: original screenshot hash from board_sources (not annotated JPEG hash).
- `hud_text`: literal visible HUD text, `severity_visible`: whether a winter severity line is visible.
- `frosted_shelter_ids`: building IDs with visible roof frost, `ash_hearth_ids`: hearth IDs visibly showing cold ash. Empty lists mean none observed.
- `doorway_cycle_visible`: only true after seeing the captured doorway breath cycle; a single still cannot establish recurrence.
- `chill_agent_ids`: agent IDs visibly showing morning chill cues.
- `visible_cues`: any visibly present `frost`, `doorway_breath`, `cold_ash`, `tint`, `hud_count`.
- `intervention_effects`: visible effects, e.g. `rain`, `cold_snap`, `windfall`, `gift_wood`, `spoil_food`, `panel`. Empty for natural frames.

No inferred state from filenames or metadata. If indistinguishable, omit cue and explain separately. Measurement independently derives cold sleepers, fire state, HUD count/severity from bridge snapshots. Missing independent visual proof fails rather than treating renderer self-reports as visibility.

A18 checks every natural frame at strict <=1 over-occluded villager; empty frames have 0. A17 scans every natural frame and requires >=3 distinct visible real wood carriers across >=2 bodies; fallback rows retain reasons. A19 genuine cold keys remain forbidden on unmarked villagers; separate chill keys are allowed and measured by A22. A20 requires the new final audit `production6/harvest/integrity_final.json`: `harvest_integrity_p6.py` verifies current installed sprites/native pixels/hair atlas masks against immutable Pass 5 installation reports and original approvals. All earlier evidence is read-only; old verdicts are never reinterpreted. Carry-layer integrity is separate and cannot substitute for A20.

Presentation only copies actual framebuffer pixels. `cold_night.png` requires natural F2Z with cold flags. No controlled-scene fallback. Missing source artifacts appear under `missing` in presentation_artifacts.json. Same framing names do not imply camera equality; before/after provenance records actual camera/zoom equality.

## Part 8 hardened observation rules

A22 uses `cold_truth_trace` sampled from FFI independently of renderer memory. For a cleared flag, the true clear tick lies in `(last_cold_tick, first_clear_tick]`; the trace gap must be at most eight ticks. Definitely eligible awake visible agents must show chill, possibly eligible boundary agents may show it, and agents outside the possible interval must not. `chill_since_tick` is checked against the independent interval rather than used as eligibility evidence. Missing trace is a failed check.

A24 accepts a panel observation only for an explicit `_panel_log` screenshot. A visible panel cannot replace a rain, flash, cold-snap, or windfall observation. Windfall at +0.2 seconds must visibly fall; at +1 second the observer must explicitly record `windfall_ended`, with no `windfall` in the same record. This records the expected end of the 0.7-second fall rather than requiring an expired effect to remain.

`review_boards_p6.py` also writes lossless `review/boards/hud_batch_NNN.png`, four HUD crops per board at their original pixel size. Every row includes the original capture stem and full SHA256. `review/hud_batches.json` indexes the batches. Review all natural HUD rows, not only representative winter frames; use identity boards and untouched originals for scene cues. Five-second `.cold_cycle.gif` sequences provide doorway recurrence evidence alongside their full-resolution source frames.

Exhaustive `carry_2x.png` is explicitly labelled **STAGED NATIVE TEXTURES**, with every installed carry frame's retained source, installed approved layer composition (or unchanged original), and an eat reference at exact 2x. It is not natural gameplay evidence. `carry_2x.json` lists every frame and group. Actual framebuffer lineup comparisons remain separately in `carry_lineup_2x.png`; `before_after.png` also includes the required real 2x carry crop alongside the two matched scene rows.

To preserve the 30 MB documentation budget, full-resolution identity boards are written outside git under `$CAPTURE_EVIDENCE/review/identity` (default `/Users/rexxa/work/art-production6-evidence/review/identity`). The small source/hash index and readable lossless HUD batches remain in `production6/review`. No original capture or annotated source is silently substituted.

A11 observer labels include `observer_id` (or `session`) with the actual independent reviewer identity. Grading preserves it alongside the frozen label hash. Two independent observers may give byte-identical answers; agreement does not invalidate independence. The combiner requires two distinct nonempty identities, not two different hashes. Grid generation freezes exactly three carry crops, and the carry confusion row must total exactly three with at least two correct.

A16 excludes only the screen-space tint entity from world-occluder rectangles; its actual pixel color still participates in the unchanged B:R ratio. A17 animation frames lacking explicit motion records use the immediately preceding same animation frame, only with 1–8 ticks elapsed and matching agent/generation ID sets. Each inferred displacement stores both original sidecar paths and SHA256; no raw metadata changes.
