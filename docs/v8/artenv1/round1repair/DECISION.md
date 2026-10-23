# One permitted readability repair

Status: registered before any Round 1 rendering. This is the only repair after the retained Round 0 failure. No further repair follows Round 1, including if its scores fail.

Round 0 failed six of 54 summed cells: day 1× activities and A, day ½× carriers and A, night ½× A, winter ½× A. Half-zoom counting passed six of six answers and all 24 timed observations were within the limit. Original scores, 144 task observations and 72 activity subpoints remain unchanged. The retained comparison also records native/declared mismatches; this repair does not make those scenes controlled equivalents.

## Preserved evidence

`/Users/rexxa/work/artenv1-data/compositor/round0_archive/` contains 161 byte-verified files totaling 100,307,247 bytes: all 48 matrix PNGs and raw render keys, both boards and sidecars, the source/render/selected manifests, compositor code and scene inputs, original documentation and original review packets/scores. Its `archive_manifest.json` SHA-256 is `9fdee5b20747df94cfa41b57b01ed5122928b420379c800f592bd4576b54bc81`. Each entry records the original path, archived path, hash and size. Existing committed Round 0 review image paths remain untouched.

Before editing, Graft traced `activity_glyph` callers through `person` → `compose` → `main`, with `test_half_marker_is_exactly_ten_pixels_including_ring` also calling `person`. No game renderer depends on this Python function. The independent Godot crowd glyph implementation is owned by the leader and must be updated/retested separately if it adopts the same shape.

## Diagnosis and fixed parameters

Direct native-pixel inspection shows the old half-zoom axe occupies approximately 8×6 pixels: its tiny horizontal head merges with a short diagonal handle, resembling a cargo mark. The old 9×5 log is similarly small. The 1× adopted S/work_a frame has a raised axe but no visible wood target beside the actor, allowing a generic tool-swing interpretation.

One repair addresses that activity representation:

| Element | Round 0 | Round 1 fixed choice |
|---|---|---|
| ½× chop glyph | Approximately 8×6; short handle/head | 14×14 bounding box, diagonal ink-edged wood handle, broad light-stone axe head |
| ½× carry glyph | 9×5 rounded bar | 14×8 cylindrical log with a distinct circular cut end and grain line |
| Glyph anchor | Person center + (6, −7) pixels | Unchanged |
| Person circle | 10 px including 1 px ring | Unchanged |
| 1× chopping frame | Adopted S/work_a | Same exact adopted frame |
| Chopping context in figure views | None | Existing accepted `stump.png`, uniform 0.5×zoom scale, center at actor foot + (22, −4)×zoom pixels, composited before the figure |
| ¼× marker and sleep glyph | Existing | Unchanged |

The stump is explicitly declared display context attached to an existing declared chopping actor, not a new simulated object, item count, target, activity or actor position. It is omitted from LOD views, where the activity glyph supplies the context. Its bytes remain unchanged. No asset generation, paint edit, source/overlay position change, identity/population/action change, ground/lighting adjustment, answer text, scoring change or threshold change is included. Person visibility counts exclude stump pixels, glyphs and letters.

The implementation author will inspect native previews and send them to the leader for a pixel verdict before the final Round 1 freeze. The complete 48-image matrix and both boards will then be verified and paired with fresh raw keys. Two fresh reviewers and the independently rerun crowd measurements remain leader-owned gates.

## Leader pixel verdict before authoritative rendering

The leader directly inspected the following native-pixel previews and reported implementation **PASS**: broad pale axe head/diagonal handle distinguishable from the horizontal log; stump beside A reads as chopping context. Readability remains **NOT_VERIFIED** until two fresh Round 1 reviewers complete the unchanged tasks. The registered repair parameters remain unchanged.

- `/Users/rexxa/work/artenv1-data/compositor/round1_preview/scene_day6_noon_day_1x_figures.png` — SHA-256 `abe826694acfc6a54aa1e5581cc849ffaa61a7063e49d91dac9de94128a04129`.
- `/Users/rexxa/work/artenv1-data/compositor/round1_preview/scene_day6_noon_day_half_lod.png` — SHA-256 `1e0df4b75a8a1905b4997fd2eecf5fb0f529e5eb350e9b46ba162b74bbb07b05`.

Original Round 0 test, lint and typecheck logs are additionally preserved in `/Users/rexxa/work/artenv1-data/compositor/round0_checks_archive/` with their own hash manifest.

## Round 1 artifact freeze

All 48 authoritative images were rendered and paired with their raw evidence before the fresh review packets. Finalizer verification at 2026-10-23 07:45:54 UTC confirmed 48/48 image/raw hashes, 48/48 original HUD crops, 93,888/93,888 ground crop checks and all 161 immutable Round 0 archive files. Source hashes, ground measurements, tick, visible population and carriers equal Round 0 in all 48 outputs. Selected six truth entries retain every actor, activity, storage target and wolf bounding box; only output/raw identity and round metadata change. Winter `wolf_hide` items are not animal targets.

The two regenerated 3200×520 owner boards were directly inspected with all six native windows. Board sizes are 1,061,593 and 948,370 bytes; largest full render is 1,246,505 bytes. Fresh checks: 14 pytest PASS, Ruff 0 diagnostics, BasedPyright 13 implementation modules with 0 errors/0 warnings. The initial direct pytest entrypoint failed collection due to a missing project import path; the preserved diagnostic is separate from the successful unchanged-code `python -m pytest` run.

- Final render manifest SHA-256: `a4e98dd3af6c2d44286a69083358bcaf2e1cae39c91df76f770c55e4e96b3b98`.
- Final selected truth SHA-256: `8d8986820816a777b68103bab774610d78e9aa0e85e11f2cab5e7d106d9360f5`.
- Final verification: `../renders/verification.json`; bounded raw/hash audit: `final_hash_audit.json`.

This freezes the implementation; Round 1 blind readability and the separate Godot crowd measurements retain their own gates. No second repair or asset generation followed this freeze.
