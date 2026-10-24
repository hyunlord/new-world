# D29 — cached house-ranking keys

The only simulation change replaces stable `sort_by_key` with stable
`sort_by_cached_key` for the existing pure house-location tuple. Candidate
generation, complete tie order, ordinary commands and the frozen D27 policy
remain unchanged. The plan was registered as D29 before the edit.

## Semantic evidence

External evidence root: `/Users/rexxa/work/mux-data/`.

- `optimization1/on-exact-comparison.json`: all eight calibration-seed ON
  summaries (48 days) are literal-byte equal before/after and to frozen round5;
  all eight command logs match; seed5 tick69120 full saves match.
- `candidate-opt1-compare.log`: baseline identity has 228748 shared values,
  zero differences; 64 detail files match except the established git_sha
  metadata exclusion; all nine colony OFF summaries are literal-byte equal.
- `a-sim-test-full-1.log`: completed with exit0, 1403 passed, zero failed,
  one ignored across102 result summaries (including empty doc-test output).
  The ignored historical migration case has separate execution evidence in
  `a-v6-test-3.log`; it is not counted as passed in this run.
- `d33-final-runtime-source.json`:673 runtime/data hashes verified unchanged
  after the actual-window run. Later UI-only commits did not change this Rust
  candidate relative to the frozen optimization binaries.

## Actual debug-window diagnostics

`active-perf-2/mux-active-perf.json` completed ON/OFF1441 ticks each, errors0,
after `a-build-9.log` rebuilt the ordinary debug bridge. ON advance-call mean
2422.547us, median1512us, p992686us, max500592us; OFF mean1634.375us,
median1573us, p992526us, max10872us.

ON planning boundaries at0/360/720/1080/1440 cost
500592/238384/320699/182202/16250us. Earlier uncached diagnostics recorded
1048501/515069/890300/458772/17526us. Both raw rounds are retained in
`d29-active-comparison.json`. These are sequential development diagnostics,
not an admitted quiet, interleaved release speedup measurement. Host load and
foreign activity are recorded in `active-perf-2-host-load.log`.

Large visible stalls remain. This instrument advances one tick per rendered
frame, so neither its average frame interval nor reduced observed spike
establishes normal-speed gameplay FPS acceptance. The new
`mux_gameplay_perf.gd` uses the real automatic accumulator at1x/4x; its source
review is `gameplay-perf-harness-review.md`, actual execution is pending.

The active harness now writes complete world saves only after all samples.
Both saves retain tick1441 and have verified SHA256:

- ON: `2709f96a471d5acb80cab73dc77215c4d886100beb5e594a6f24cd0536090af3`
- OFF: `bae367468dc4b19e9cf108a8927a32968476f6a4f00b9f7d2f8d80a9b6607912`

These are frozen inputs for a separate compiler-profile experiment, not
evidence that a future profile already preserves behavior. A8 held-out
acceptance remains FAILED and policy remains frozen. Full gate/publication
are still pending.

## Subsequent evidence (after the p9 record)

The automatic gameplay run subsequently completed in `gameplay-perf-1/`;
D34's separate package-profile commit `78af5e245` completed a second run in
`gameplay-perf-2/`. See [D34 verification](D34_VERIFICATION.md) for exact
build scope, byte-identical debug saves, fresh OFF safety and remaining
latency. The pending statements above describe the p9 evidence boundary;
no original timing or acceptance failure is superseded.
