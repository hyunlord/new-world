# Round 1 minor-finding remediation (not another review round)

Only repository write: tools/art_pipeline/validate_walker_identity_p13b.py. No capture source, Rust, HEAD, commit, Godot or build changes. Existing P3 disabled-camera/unpaused assertions are preserved. Work performed with other agents active; no unrelated edits reverted.

The running comparator now has a typed FAIL fallback and captures anticipated malformed-input exceptions. The RGBA comparator explicitly captures typer.BadParameter plus malformed nested-value exceptions. Both append error diagnostics and proceed to terminal walker-identity.json with passed=false. Input-provenance parsing captures malformed nested values as well. Evidence checksum I/O failures append errors before the final verdict. No catch-all exception handling, no relaxation of identity comparisons, source equality, polarity, frame coverage or acceptance thresholds.

Offline regression evidence: check.py creates deliberately incomplete synthetic folders outside the checkout and invokes the real validator CLI (no mocks, no Godot). before.json: missing-evidence case wrote FAIL; invalid schedule exited 2 without a report; malformed populations=null exited 1 without a report. after.json: all three exit 1 with a populated errors list and passed=false terminal JSON. Per-case stdout/stderr and report artifacts are preserved. AST parsing and git diff --check pass. This verification does not establish positive identity acceptance or replace the leader's pending complete identity run.

Output destination failures (unwritable filesystem or a reused output directory) remain operational errors; fresh-directory refusal intentionally prevents overwriting evidence. Unexpected programming/system exceptions are not swallowed.

Duplicate-ID sorting limitation remains open and explicitly bounded: the real 2,000-call fixture rejects duplicate row IDs, the duplicate spatial unit test verifies relocation/exclusion only, and the numeric fixture does not verify introsort tie order. General duplicate-ID ordering equivalence is not independently established by the current fixture; no Rust edits or new capture runs were made during the freeze.
