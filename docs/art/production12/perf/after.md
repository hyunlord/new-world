# Frozen Part4 release measurements

Frozen pushed build: c08076dd67ee0a54470f16fc6b993c3172752320. Actual release build/import completed after checkout; loaded debug-path library equals release SHA1e6f76187e9e0d87e87426fe2b0aa510c3c797efd61b9cb14fed383531d2bf91. See after-build-provenance.json. Product source equals corrected capture c46e66bd; no subsequent implementation changes.

**NOT_VERIFIED (contended)**: 21/21 separate admission holds rejected before Godot launch (exit75). Five matrix worlds x3 plus W500 frozen/W200 running sections x3. All snapshots contain foreign make-meme Python PID39003 above50% CPU; start/end loads range 4.992–5.712. Foreign processes untouched; no own build/identity/gate during holds. Raw start/end snapshots and hashes: after-admission-index.json, after-admissions/. Holds: holds.md.

| World/condition | Before median/p90/p99/max | After median/p90/p99/max | Ticks/frame and block populations | After attempts/accepted | Ratio |
|---|---|---|---|---|---|
| W0 frozen/running2x Z4 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |
| W1 frozen/running2x Z4 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |
| W200 frozen/running2x Z4 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |
| W500 frozen/running2x Z4 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |
| W-multi frozen/running2x Z4 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |
| W500 frozen exclusive section timers | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |
| W200 running2x exclusive section timers | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | 3/0 | unavailable |

No raw frame blocks exist, so no population-loss blocks can be flagged or pooled. Census populations in worlds.md are not measured block populations. No ranking of costs>=1ms, inside/outside renderer attribution, FFI getter counts/ms or improvement ratio is established. W500 running remains reporting-only, never PASS/FAIL. H1–6 are NOT_VERIFIED(contended); H7 no accepted optimization, candidate identity PASS with speed unverified/defaultOFF. H8 remains FAIL3/10+4missing. H9 final independent reading is separate pending evidence; H10 unit/live captures are separate from timing.

Initial section invocations failed before output directory/admission because P12_BUILD_PROFILE was omitted. All six command-error logs E/p4-sections-*.log retained; corrected invocations explicitly set release and used new paths. These configuration errors supplied no measurement and consumed no hold retry. Corrected six holds exhausted the original three-per-target ceiling. Matrix exit1 means no accepted sessions, not a product performance failure.
