# Allocation bundle rejected

The six-component candidate passed all six frozen pixel identities, both600-frame running checks and six focused fixtures. Its valid noncontended W500 alternating pair regressed from34.8350ms control to41.6375ms candidate (0.83663x), so it is NOT accepted.

A separate non-frame-time100,000-call helper diagnostic identifies call overhead: native scalar metadata writes about2.8ms versus37.5ms cached-helper calls; native child lookup about5.8ms versus52ms cached-helper calls. These microbenchmarks are diagnostic only, not quiet-lock frame acceptance. Disabled wrappers also add overhead.

The reduced retry restores native metadata and child lookups, removing those wrapper implementations; retains frame choices, variant-path cache, pose output reuse and selection index. This records unsuccessful optimization of metadata/child references rather than claiming those requested subitems improved. Earlier class-declaration abort remains aborted143, not a pixel failure. Raw sources and all rejected/aborted evidence retained in ZIP.
