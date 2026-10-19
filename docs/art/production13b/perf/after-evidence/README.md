# Part4 after evidence — terminal archive, NOT_VERIFIED

Collection has ended with `TERMINAL_PARTIAL_WAIT_BUDGET`; overall Part4 measurement coverage is **NOT_VERIFIED**, not PASS. Attempt4 and recovery supervisors (26762/53175) terminated exit0. Terminal4 and recovery full snapshots equal the frozen measurement snapshot. Supervisor exit0 records orchestration completion, not admission success.

## Terminal classification

The exact audit is `control/part4-terminal-audit.json`: 29 file slots comprise 4 valid captures, 3 collected-invalid captures, 21 probe/prelaunch refusals and 1 policy refusal. W1/W200 attempt1 and W0/W500 attempt3 are admitted. W-multi has no admitted result; required W200/W-multi running section timers remain NOT_VERIFIED, as do optional W500 sections and W200 ablation. Failed raw captures remain external and all prelaunch evidence is copied.

Attempt4 reordered commands left `P13_ITEM=after-ablation-W200` exported for the matrix. W-multi file slot4 was denied under that wrong item before probes/Godot. The fresh recovery shell explicitly unset `P13_ITEM` and `P13_VALIDITY_OUT`; true W-multi file slot5 was denied at start load17.5810546875. Preserve these separate policy and probe outcomes; do not call slot4 a completed W-multi measurement or erase it from retry history.

Union admission waiting consumed9487.316187858582seconds; remaining1312.6838121414185seconds is shorter than the mandatory1800second cooldown. The terminal audit therefore ends collection with missing targets unverified. This archive authorizes no further measurement.

## Retention and verification

`SHA256-MANIFEST.json` lists absolute source paths, optional repository copies, SHA256 and byte lengths. Raw matrix captures and duplicated large partial aggregates remain external. Every copy and retained external raw has been rehashed and size-checked. `archive-verification.json` supplies current exact counts; owner-confirmed terminal aggregation and report preparation outputs are now preserved under `aggregation/` and `report-prep/`, including their helper source. Earlier report drafts remain under `historical/report-prep-snapshots/`.

`attempts/` retains every completed external after artifact under its original relative name, except manifest-referenced external-only large data. `control/` includes original scripts, supervisor logs, terminal audits and the bounded prepublication invariant audit (whose timestamp/scope is preserved; it is not a final publication gate). Calibration, complete source snapshots, generated timer instrumentation, failed summaries, logs, probes and both lockfiles remain copied.

`historical/partial-attempts1-3/` preserves the earlier partial README, SHA manifest, worker verification and root independent audit verbatim. The terminal extension first reverified all193 historical source records and copies, and did not overwrite historical raw files. The top-level terminal manifest supersedes scope, not historical evidence. No collector, Godot process, build, test or commit was run by the archival worker.

## Source and calibration qualification

Frozen Part4 HEAD: `40aa7a463d740b776248828211251187319c6af6`. Loaded release/copied-debug bridge SHA256: `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`, the same loaded library as merged identity. Flags are walkerON, declumpON and writeSkipOFF. Matrix/section rows do not provide actual per-frame candidate getter stamps; this is frozen-source/launch-provenance evidence.

Full identity and measurement source manifests are **not equal**: the retained audit records 655 differing untracked/generated paths (183 .gd.uid, 471 .import UID-only differences, and ignored rust/Cargo.lock). It reports zero tracked runtime differences and equality of all471 referenced compiled imported-resource pairs. The measurement lockfile was not used to build the loaded bridge. Preserve `attempts/identity-source-equivalence.md`, `attempts/identity-source-manifest-differences.json`, both `built-root-Cargo.lock` and `unused-measure-Cargo.lock`, and all snapshots. Claim measurement-local snapshot stability plus the stated tracked-source/imported-byte/bridge equivalence only.

Calibration R is 2.762010041042231 seconds, SHA256 `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4`. Historical before evidence uses its own recorded calibration and is linked from `../before-reference.md` and `../before-reference.json`; do not substitute calibrations or rescale acceptance bars. Official after reports, final aggregation, H1–H11 verdicts and retry accounting belong to their separate reporting lanes.

The final inventory contains 280 source records: 269 copies and 11 external-only raw references. Terminal report preparation distinguishes 28 genuine probe sidecars/56 endpoints from the separate policy denial. Its raw inventory includes seven captures, three invalid; raw presence is not acceptance.
