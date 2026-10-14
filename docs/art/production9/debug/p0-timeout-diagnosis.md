# Part 0 gate timeout diagnosis

Observed: cargo_test exit124 at1978 wall seconds for the1800-second bound. No assertion failure was observed before timeout. The owned child left by the runner was identified by exact executable path/start time and terminated; ps evidence is retained.

Primary supported cause: profile mismatch. Prior production8/gates/final/provenance.json records CARGO_PROFILE_TEST_OPT_LEVEL=1, CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true, CARGO_PROFILE_TEST_OVERFLOW_CHECKS=true. Its cargo log reports optimized+debuginfo; current p0 reports unoptimized+debuginfo. Long harness sources are unchanged against56059e0f. Timings: pass12_fission35.39→764.28s; pass11_fission6.93→109.62s; pass10_variety9.02→199.98s. Prior production8/DECISIONS.md records this same failure/recovery pattern.

Other hypothesis: concurrent host load contributed, but its share cannot be isolated from these observations. The merged tick(None) implementation performs option checks, but does not call Instant::now on the None path; no evidence currently establishes a simulation regression.

Next experiment: identical full gate with only the prior test-profile environment restored, fresh p0-attempt2 output, unchanged jobs6/threads6/bound1800. Dev/editor library remains debug. No simulation edits, timeout increase, threshold relaxation or test removal.
