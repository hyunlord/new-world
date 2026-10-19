# Alternating collector diagnosis

W500 attempt1 at12:45:32Z was admitted but ended unsuccessfully. Never performance evidence. Full raw files external/write-skip/W500-attempt1 remain immutable. Error: Renderer switch changed inside a measured block. Control block and candidate block each completed180 frames; candidate counters necessarily changed.

Confirmed source cause: get_perf_candidate_p13 returns configuration plus hits/misses/forced. Original collector compares the entire after dictionary with before, interpreting legitimate counter increments as switch changes. Per-frame configuration stamps all passed; no switch mutation is reported there. Alternatives (deferred reset, kernel drift) are not supported by those frame stamps.

Copy collector to perf_alternating_frames_p13b.gd. Exclude only the three known counters from final configuration equality, retain all other fields and all per-frame strict readbacks. Preserve full before and after dictionaries, all30 discarded+150 measured frames, unmodified validator and acceptance thresholds. Copy new collector into detached measure checkout as an explicitly hashed instrumentation-only change; do not modify baseline render code or bridge. Runner p13b points to this copy.

W500 attempt2 cannot start before13:15:50Z (30 minutes after prior terminal completion). W200 has not been attempted and can run after this repair. Failing-target count W5001/3. Strict GDScript check passed. Runtime validation W200 attempt1 completed exit0 and unchanged validator accepted all6 blocks/1080frames. Performance threshold failed honestly (ratio0.9986985). Candidate block180/180 raw stamps report valid; final observed hits73732/misses16268/forced500 while all configuration fields remain expected.
