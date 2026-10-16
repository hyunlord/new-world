# Seat input cache acceptance

Accepted after exact six-framing RGBA identity, W0/W200 600-frame running equality and focused tests. Measured production hashes match focused-test source hashes.

Three alternating 150-frame pairs (30 warmup each), W500 frozen Z4, noncontended: control median 51.3775 ms; candidate 35.7685 ms; speedup 1.43639. Full distributions/raw frames and before/after source/build manifests are in attempt2-accepted/. This is candidate acceptance, not the final H1–H5 verdict.

Attempt1 remains a failed reset-race measurement. Its raw data is excluded. Corrected harness settled deferred resets before controls; attempt2 valid and source-stable.
