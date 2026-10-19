# H10 source and capture-tool preparation

Prepared read-only against Pass 21 data on 2026-10-19. No Godot, cargo, live simulation, or blind-reader answers were run/opened by this preparation. This is source discovery, not H10 acceptance.

## Reachable reconciliation source

`/Users/rexxa/work/p21-data/p2/A/chronicle_game_v26/chronicle_19603.json`

- SHA256: `d5bbd19a1454165f3adf8d917a243be0f578019059c81297412b9a4969a8e6c4`
- Recorded scene `game_v26`, seed `19603`, git SHA `8f681805`, final tick `172800`, entry count `1218`.
- Declared fields: `[tick,kind,subject,other,third,settlement,settlement2,cause,value]`; declared kind index 21 is `villages_reconciled`.
- Entry index 711 (zero-based): `[122400,21,null,null,null,1,2,0,-143]`.
- Entry index 873: `[138240,21,null,null,null,1,3,0,-145]`.
- First target: tick122400, 85 elapsed days; with the actual UI formula `floor((tick+480)/1440)+1` (`scripts/ui/chronicle_text.gd:14`), displayed day86 08:00. Second: tick138240, 96 elapsed days, displayed day97 08:00. Both are within 120 days.
- Prefer the first target for chronicle capture, so kind21 does not age out of the production panel's latest60 notable entries. The capture must still demonstrate the visible line and verify the current build's raw row. Historical source presence is not current-build reproduction.

A bounded scan of the 32 `chronicle_*.json` files in the same A/game_v26 directory found six seeds with kind21 by tick172800: 19603 (first122400, 2 entries), 19606 (133920,2), 19607 (113760,1), 19610 (119520,1), 19621 (149760,1), 19623 (165600,1). Earliest source target is seed19607/t113760, displayed day80 08:00. No broad recursive data scan was needed. All Pass21 files were read only.

Because a listed-seed source exists within 120 days, the no-source fallback is not established. Preserve real capture failures and three-strike counts; do not silently use a fake bridge as a passing capture.

## Existing Life and H10 capture tool

`tools/art_pipeline/run_life_captures_p13.py CHECKOUT NEWDIR` launches `scripts/test/art/capture_life_p13.gd` serially through `with_window_lock.sh`, at1920x1080 fixed60FPS, with an inner foreign quiet-lock check. NEWDIR must not exist. `P13_LIFE_BUILD_PROFILE` defaults to debug in the runner, accepts debug/release; release requires the debug-loaded bridge SHA to equal release SHA. `P13_BUILD_DETAILS` records profile details. `GODOT_BIN` overrides the executable.

The runner sets `P13_LIFE_OUT=NEWDIR/CASE`, `P13_LIFE_CASE=CASE`, and `P13_LIFE_BUILD_PROFILE`. Direct GDScript invocation requires a new nonexisting P13_LIFE_OUT and explicit debug/release profile. There is **no seed or tick environment override**. Fixed cases at `capture_life_p13.gd:19`:

| Case | Seed | Tick |
|---|---:|---:|
| live |20261010|85080|
| multi |19400|121080|
| estranged |19414|34680|
| crowd1 |20261010|41880|
| crowd2 |20261018|85080|
| crowd3 |20261010|42180|

The tool resets the real world, advances exactly, verifies seed/clock, freezes ticks, captures full raw history and world references, scrolls Life and chronicle pages with48px overlap, and saves image hashes and on-frame village/porter metadata. Source hashes and HEAD must remain unchanged. `reference-data.json`, registration, launch logs, result JSON, working-tree patch and private image mapping stay outside the anonymous `reader/` PNG bundle. Reader instructions prohibit opening references before answers freeze.

`_capture_multi` selects a real outbound porter, captures Life EN/KO, then road-centred Z4/Z1. The road images are **not looped over EN/KO**. `_capture_estranged` checks kinds19/20 and saves one Z1 view; it does not require two relation colours. `_chronicle_capture` loops EN/KO but does not target kind21. Thus this unmodified harness cannot satisfy13b's seed19603 and both-language road/relation requirements. Copy to a `_p13b` tool only for those necessary changes; keep `_p13` evidence intact. Retain on-frame sprite evidence and actual `SettlementLabels.capture_metadata()` for the two-colour check, then inspect actual screenshots.

`probe_porter_p13.gd` writes `porter-icon-sheet.png` and `wood-glyph-fixture.png` under mandatory `P13_PORTER_OUT`; this is a synthetic sprite fixture, not an H10 world capture. `test_relations_chronicle_p13.gd` supplies a kind21 fake-world panel test in EN/KO and asserts sentence/village roles; it has no screenshot-output override. It is fallback unit evidence only.

## Existing activity contracts

- `python3 tools/art_pipeline/run_activity_captures_p13.py CHECKOUT NEWDIR 20261018 SELECTION_EVIDENCE`: fixed worlds20261010/day30 10:00 and15:00;20261018/day60 10:00; each at Z4 and Z1. Selection evidence must be nonempty; output must be new, with existing parent. Defaults release; `P13_ACTIVITY_BUILD_PROFILE`, `GODOT_BIN`, `P13_CAPTURE_LIMIT` (1..3600, default1800), `P13_DEADLINE_EPOCH` configure execution.
- Runner sets `P13_ACTIVITY_OUT`, `P13_ACTIVITY_SEED`, `P13_ACTIVITY_DAY`, `P13_ACTIVITY_HOUR`, `P13_ACTIVITY_ZOOM` for `scripts/test/art/capture_activities_p13.gd`. GDScript also accepts optional `P13_ACTIVITY_CENTER_KIND` and `P13_ACTIVITY_SELECTED_ENTITY`. It refuses overwriting existing `capture.png`/`capture.json`; emits both, with source/build identity and raw snapshots. The output alias `P10_ACTIVITY_OUT` is set internally for inherited tooling.
- Supplements: `run_extra_activity_p13.py CHECKOUT NEWDIR SEED DAY HOUR CENTER_KIND`, with mandatory `P13_EXTRA_BASE_PLAN` pointing to frozen exclusions. Only base seed/day pairs and hours11/14, missing kind0..6 are accepted. Preregistration is written before launch; do not use this restricted stage-B wrapper as an arbitrary H10 runner.
- `prepare_blind_crops_p13.py CHECKOUT NEWDIR CAPTURE_JSON ...`: >=3 distinct base captures, expected base order; writes `exclusions-before-crops.json` before any crop, then private `key.json` and anonymous blind images/names. Body+icon union,8px padding, squared, unchanged pixels; exclude >50% coverage of either body or icon. Optional `--partial-diagnostic TERMINAL_SHORTAGE_JSON` preserves a diagnostic, not full acceptance.
- StageB must give a fresh reader the legend **first**, then14 crops and seven names. Freeze answers before any key; >=12/14 with no kind0/2. A separate third reader for without-legend is droppable. These preparations did not inspect keys or reader responses.
- `prepare_activity_villages_p13.py FROZEN_C_ANSWERS NEWDIR Z1_JSON ...` requires frozen activity answers, selects6 eligible village/capture pairs, excludes top-two ties and offscreen summaries, at most2 pairs per seed/village. Fewer than6 is NOT_RUN with count, not pass.

## Inherited fact-reading scope

Art Pass13b5.5 inherits13§7.2 and12§6.6(e)/7.5. At the final Life size, capture EN/KO: living person with dead parent, parent record, chronicle, crowded selected person, every departure cause actually present, porter Life, and quarreller if present; include reconciliation when present. Preserve absent causes. Fresh reader writes observations before private reference comparison. Every non-name statement must be correct; report name count/confusions separately and leave unverifiable claims unscored. Tool success alone establishes neither reading nor visual acceptance.

References inspected: `docs/art/production13b/GOAL.md:162`, `docs/art/production13/GOAL.md:219`, `docs/art/production12/GOAL.md:313`, and `docs/art/production12/reading/capture-tool-protocol.md`.
