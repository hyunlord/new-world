# Code review round 1

Reviewed head: `cefc77b088d6fa3baf21b766cdc40e3fdf045d36`.

**No new actionable correctness findings in the bounded Parts1–5 review.** This is not a visual/blind verdict or final acceptance. The reviewer previously implemented Parts2/3 and the shared layer flag, so those portions are a self-review; feed, P/G capture and CPU collector were authored by other lanes. Do not present this document as a fresh independent review of every implementation.

## Evidence checked

- Feed classes, routine invisibility and exact source IDs: `scripts/ui/panels/hud_status_panel.gd`. Protected retention is240simulation ticks, expiry480ticks; injected clock covers paused behavior. Transfers/casts are excluded through `feed_food_ledger.gd` and capture advances one tick at a time before feed ingest. Gap coverage is explicitly false rather than pretending all unseen movements are known.
- `+N`: `_protected_history` deliberately persists until reset and counts expired protected rows too. A single isolated event eventually contributes+1 even without concurrent overflow. This initially looked like a competing-overflow discrepancy, but `DECISIONS.md:16`, `DESIGN.md:55` and `test_feed_priority.gd:86` explicitly agree on this interpretation for A31 across wide capture intervals. Recorded as a UX limitation below, not an undisclosed implementation bug.
- Marker/stake exclusivity, real click dispatch harness and stable parent palette; selected marker carries no distance. Meter exact thirds use overflow-safe quotient/remainder, no missing-field inference; raw pressure/theta retained separately from clamped render fill.
- Crowd lateral cap is checked in screen space and accepted shifts remain perpendicular; terrain/building exclusion runs before applying render-only offsets. Morning chill90minute window/final30fade and the selected-village layer flag are separate. No simulation source edits were introduced by these render changes.
- A30 `frame_cpu_probe.gd` has real Node priorities−1000/+1000, same Engine process-frame pairing and deferred publication. `frame_measure_session.gd` collects12alternating50-frame blocks plus36discarded transition frames,300per condition; validates frozen tick/camera/zoom/population/selection, rejects unavailable all-zero viewportCPU and records GPUzero as unavailable. Primary metric remains process+renderCPU, setupCPU separately recorded, original vsync/camera/layer states restored. The flag covers label children/meters/selected arrow, founding trail/stake/arrow, overview and explicit selected-outline early return.
- A30 seed20261013/tick126994 agrees with the exact getter confirmation in `MEASURED_DAYS.md:293`; the runtime also checks3living settlements before measurement.
- P/G plan uses distinct RESETepochs and world labels, matched d30/31/32casts, following-morning frames and separate day120final frames. Capture metadata preserves raw society getter including fission_depart_ticks, cast reply/target, P/theta, pre-cast food and scarcity-line provenance. Missing paired target is recorded without aborting. Need provenance is tied to Rust source SHA rather than an undocumented guessed scarcity constant.

## Re-run evidence

Ran the following headless scripts at the reviewed code state; all printed failures0 and completed successfully:

```bash
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path . --script scripts/test/art/test_feed_priority.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path . --script scripts/test/art/test_frame_measure.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path . --script scripts/test/art/test_restless_meter.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path . --script scripts/test/art/test_village_layers.gd
```

## Remaining limits

1. The +N count is all additional protected history, not unread news; no history-reading action exists. This interpretation is documented but its usefulness remains a screen-review question.
2. GPU/viewport timing may be delayed relative to the process stamp. Transition discards isolate on/off conditions, but the report must retain the temporal-alignment caveat rather than claim exact same-frame renderer timestamps.
3. Headless tests and this source review do not replace real A30 timing, natural/society/intervention pixel gates, paired-world causal outcomes or blind observers. No new windowed or full Rust gate was run here.
4. Exact food summaries require per-tick observations. Render-frame sampling during live fast-forward can skip opposing stock changes; coverage metadata exposes such gaps. Capture mode deliberately performs per-tick ingest.
