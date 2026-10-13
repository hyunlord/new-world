# Part 0 baseline

Simulation merge `e1e894ef` incorporates exactly `31559232e0a24af12d29f490d440d1c5ee5f3661` into art `cd998d38` without conflicts. Tooling source commit `07e148df`. Renderer unchanged from Art Pass6.

- Default live seed20261010: d2 12:00 F2, tick1680, `baseline/live_bound/a_d2_1200_F2.png`. Actual windowed capture visually inspected: one village, existing Korean HUD, no new labels. Four calibration shots accompany this capture-tool invocation.
- Society seed20261013: d29 09:09, tick40389, exactly60ticks after founding40329. `baseline/society/soc_d29_0909_FV_all.jpg`. Original lossless `/Users/rexxa/work/art-production7-evidence/baseline_bound/shots/art_society/soc_d29_0909_FV_all.png` visually inspected: two hearth/shelter groups, no settlement labels, HUD2. Existing HUD clock floors09:09 to09:08 due floating-minute truncation, recorded for UI correction.
- Both executed windowed via launchctl asuser and with_window_lock.sh1400s; neither contains intervention. Metadata raw and entity SHA256 hashes in each manifest. Console contains0 SCRIPT ERROR.
- First live capture before tooling commit retained at `baseline/live`; source patch retained externally. Use `live_bound` for before/after comparison.
- Initial full gate45rows:43 exit0; cargo_test and clippy101 from existing test fixture missing `InterventionRecord.soaked_wood`. Strict, bridge, smoke, smoke_errors, Python and38Godot scenes pass. Original logs remain `gates/p0/`.
