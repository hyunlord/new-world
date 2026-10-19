# Part4 after evidence — PARTIAL

This archive preserves completed attempts 1–3. **Attempt4 is pending and was not inspected, hashed or copied.** No final aggregation or final acceptance is created here. The leader confirmed attempt3 supervisor terminal exit0; its detailed audit is `control/attempt3-terminal-audit.json` and reports measurement-local full snapshots equal. Supervisor success does not imply admission success.

## Current evidence

- Matrix: W1 and W200 attempt1 admitted; W0 and W500 attempt3 admitted. W-multi attempts1–3 invalid; its raw captures remain retained. All earlier refused/invalid attempts remain separate.
- Running section timers: W200, W-multi and optional W500 attempts1–3 invalid. Preserve generated timer source/manifests, launch provenance, failed summaries, probes, logs and exits; absent raw frames are not synthesized.
- W200 ablation attempts1–3 invalid; missing raw captures remain missing.
- Every validity sidecar retains original start/end probes and process snapshots. This archive does not revalidate, normalize, or reinterpret admission. Population-loss exclusions and unavailable GPU data remain subject to the reporting lane's literal rules.

## Retention and verification

`SHA256-MANIFEST.json` follows the before-reference convention: absolute source `path`, optional `repo_path`, exact `sha256` and `bytes`. There are 193 source records (79,207,740 bytes): 183 byte-verified copies (18,418,671 bytes), and 10 complete external-only raw/duplicated aggregate JSON records (60,789,069 bytes). Both the original source and each copy were independently rehashed and size-checked after copying. `archive-verification.json` records these results. Those counts exclude the generated manifest, verification report and this README.

`attempts/` preserves relative filenames under the external `after/` root. Large matrix/section/ablation raw JSON and duplicated partial-summary JSON remain at the exact external paths in the manifest. All failures and prelaunch evidence are retained, even without raw frames. `control/` holds completed-attempt runner scripts, supervisor logs, freeze and terminal audit. `calibration/` holds the actual Pass13b reference. Keep raw files externally; summaries do not replace them.

Future completed attempts must be added only after terminal confirmation. Preserve this manifest/version and verify old source/copy hashes before extending; never overwrite historical raw files. Attempt4 scheduling is not a result.

## Source and calibration qualification

Frozen Part4 HEAD: `40aa7a463d740b776248828211251187319c6af6`. Loaded release/copied-debug bridge SHA256: `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`, the same loaded library as merged identity. Flags are walkerON, declumpON and writeSkipOFF. Matrix/section rows do not provide actual per-frame candidate getter stamps; this is frozen-source/launch-provenance evidence.

Full identity and measurement source manifests are **not equal**: the retained audit records 655 differing untracked/generated paths (183 .gd.uid, 471 .import UID-only differences, and ignored rust/Cargo.lock). It reports zero tracked runtime differences and equality of all471 referenced compiled imported-resource pairs. The measurement lockfile was not used to build the loaded bridge. Preserve `attempts/identity-source-equivalence.md`, `attempts/identity-source-manifest-differences.json`, both `built-root-Cargo.lock` and `unused-measure-Cargo.lock`, and all snapshots. Claim measurement-local snapshot stability plus the stated tracked-source/imported-byte/bridge equivalence only.

Calibration R is 2.762010041042231 seconds, SHA256 `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4`. Historical before evidence uses its own recorded calibration and is linked from `../before-reference.md` and `../before-reference.json`; do not substitute calibrations or rescale acceptance bars. Official after reports, final aggregation, H1–H11 verdicts and retry accounting belong to their separate reporting lanes.
