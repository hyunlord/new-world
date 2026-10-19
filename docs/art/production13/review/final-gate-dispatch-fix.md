# Final gate dispatch failure and bounded correction

The first final-gate cargo run failed with three unresolved dynamic dispatch calls in perf_alternating_frames_p13.gd, lines146/149/182 at that revision. The existing all-script SimReads boundary scanner intentionally rejects unresolved call(method) syntax, even when this renderer-only switch does not access simulation snapshots. The failure is retained in gates/final/cargo_test.log; no scanner exemption or test weakening was added.

The harness now calls the two known renderer setters/getters with literal names selected by the existing write-skip condition. Candidate values and before/after readback comparisons are preserved. Product rendering and simulation sources are unchanged; historical measured sessions retain their original script revisions and are not described as newly measured.

The existing compiled harness_art_p9_sim_reads test binary was rerun against the current script tree: 4/4 passed, exit0. A full cargo workspace repeat is running separately in `/Users/rexxa/work/art-production13-evidence/headless-gate/cargo-after-dispatch-fix.log`; final acceptance requires its terminal result. The original final gate continues its other checks, with GUI invocations refused by the external headless-only wrapper.
