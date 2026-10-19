# Walker performance evidence

**Speed gate PASS; candidate remains OFF pending identity attempt 2.** This archive establishes performance only. It does not approve installation, visual acceptance, or identity. The first identity attempt failed P3 A/A′ control reproducibility; its failure is not waived by these timings.

Each world has three alternating control/candidate pairs: six blocks, each with 30 discarded frames and 150 counted frames. Thus each world contributes 450 counted control frames, 450 counted candidate frames, and 1,080 independently checked walker/kernel/write-skip stamps including discarded frames. Declump kernel stays ON; write skip stays OFF. The measured primary metric is process_ms + render_cpu_ms. Before/after and cross-world source/build provenance matched.

| World | Control median ms | Candidate median ms | Pooled candidate/control | Faster pairs | Walker speed criterion | Result |
|---|---:|---:|---:|---:|---|---|
| W200 running 2×, Z4 | 35.3630 | 22.9585 | 0.6492237648 | 3/3 | ≤0.97 and ≥2/3 pairs faster | PASS |
| W500 frozen, Z4 | 31.9980 | 30.2260 | 0.9446215390 | 3/3 | ≤1.00 and ≥2/3 pairs faster | PASS |

W200 pair ratios: 0.6472445182, 0.6466932538, 0.6541020268. W500 pair ratios: 0.9499485291, 0.9436013447, 0.9386296695.

The unchanged inherited validator applies ≤0.97 plus ≥2/3 faster to both worlds. Its original `speed_pass` values are preserved and happen to be true for both. The walker-specific W500 ≤1.00 pooled ceiling is reported separately, retaining the ≥2/3 faster-pairs condition. Attempt3 is the current corrected supplemental verdict; inherited reports remain unchanged.

## Preserved evidence

- `W200/` and `W500/`: actual collector summary, launch provenance, calibration/session validity, terminal exit, and session log.
- `comparison-attempt1/`: failed validation invocation, exit 1. The inherited validator defaulted to the production13 calibration because `P13_REFERENCE` was absent; both production13b ledgers were rejected for calibration provenance mismatch. This is a comparator invocation failure, not another timing measurement.
- `comparison-attempt2/`: successful validation invocation, exit 0, unchanged inherited reports, supplemental report, and invocation correction. `P13_REFERENCE` explicitly selects production13b/perf/calibration.json, SHA256 `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4`, matching both recorded validity ledgers. No recapture or source edit occurred between comparator attempts.
- `comparison-attempt3/`: current corrected validation, exit 0. Review round1 found that attempt2’s supplemental W500 expression omitted ≥2/3 faster pairs. The expression and rule text are corrected for both worlds; six regression/boundary cases passed. Both measured workloads already had 3/3 faster pairs, so the measured result is unchanged. Attempts1 and2 remain untouched.
- `SHA256-MANIFEST.json`: source paths, archive paths, byte lengths, and SHA256 for every copied file. All copies were byte-hash checked. The original complete `frames.json` files remain externally retained at the absolute paths in this manifest, with hashes and lengths recorded here; they are not replaced by summaries.

The raw sessions remain at `/Users/rexxa/work/art-production13b-evidence/walker-performance-W200-attempt1` and `/Users/rexxa/work/art-production13b-evidence/walker-performance-W500-attempt1`.

Final candidate acceptance requires the separately recorded full identity result. Keep candidate OFF until the leader verifies identity attempt 2 and authorizes acceptance. This document records the pending status at archive creation; it is not a later identity verdict.
