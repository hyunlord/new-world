# Art Pass 10 autonomous decisions

- 2026-10-15T15:03Z: The attached GOAL supersedes default reset/lead-main instructions, hooks and harness pipeline. All commits use core.hooksPath=/dev/null. The prescribed full art gate remains required even though it internally invokes the existing strict-check script.
- Original .gitignore and .ignore changes belong to another process/user; preserve and exclude from commits.
- Use GOAL's decision-complete Parts as execution plan; skill planning/approval defaults do not introduce an owner question or extra out-of-scope plan files.
- Copy historical measurement tools to p10 names; never alter historical evidence or reinterpret its verdicts.
- Merge 0caabf5fd1c62b9d3c99845daab1805e79799db7 was conflict-free; no manual resolution files.
- 2026-10-15T15:23Z: P9 final P6 PNG is byte-identical to P1 and visibly lacks the inspector. `agent_inspector_panel.gd` connects world_reset to close; P9 fixture opens before deferred reset delivery. P10 fixture retains world-state frame10/capture130 but opens inspector at frame12 after reset delivery, then requires visible-in-tree at capture. This is a new-baseline fixture repair, not a reinterpretation of P9's historical verdict or a production rendering change.

- Part0 measurement review: bind baseline to exact merge HEAD, canonical checkout, dylib and collector hashes; require terminal exit0 before resume. Keep W0/W500 ablation mandatory for both phases and optional W1000 frozen Z4 only. No prior evidence changed.

- After p0-attempt2 completes, third full gate will set CARGO_PROFILE_TEST_OPT_LEVEL=2 (CARGO_BUILD_JOBS remains6). Cargo.toml has no profile override, so the prior simulation suite was unoptimized. This changes test compilation optimization only, retains test debug assertions/overflow checks and all tests/assertions, leaves the debug sim-bridge rendering build and1800s limit unchanged. Preserve both earlier failed attempts.

- Part1 timer design decision: choose opt-in generated diagnostic AgentRenderer script, selected before scene entry only by the diagnostic harness. Keep ordinary production callbacks free of timing guards/calls so disabled cost is structurally zero. Generator must fail closed on ambiguous source markers and record input/output hashes; per-frame readout must align with measured process frame. Instrumented identity must be checked before trusting its attribution. FoundingGroups callback is separately attributed, not silently included in parent callback totals. Implementation waits until Part0 baseline completes.

- Raw gate logs are retained byte-for-byte with recorded SHA256. git diff --check flags trailing blank EOF lines in three original logs; these are preserved evidence, not normalized. Do not report the full raw-log diff as whitespace-clean. Source-code whitespace checks remain required.
