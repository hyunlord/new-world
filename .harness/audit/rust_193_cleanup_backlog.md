# Rust 1.93 Clippy Baseline Cleanup Backlog

**Status**: Tracked under Rule 7.1 v3.2.1 (clippy baseline tolerance).
**Created**: 2026-05-03
**Owner**: lead/main
**Trigger**: Rust 1.93.0 toolchain bump introduced 31 pre-existing clippy
issues at `origin/lead/main` HEAD across files unrelated to any in-flight
change. Recognised as **environmental** (toolchain drift) per Rule 7.1
v3.2.1; baseline registered to unblock shipping while cleanup is
sequenced.

> Reference registry: `.harness/baseline/clippy_baseline_raw.txt`
> (31 fingerprints) and `.harness/baseline/known_clippy_issues.md`
> (human-readable companion).

---

## Why this exists

Without this backlog the baseline registry is a one-way ratchet:
toolchain drift gets absorbed and forgotten. Each cleanup PR shrinks the
registry; once the registry hits 0 the baseline file should be deleted
and Rule 7.1 v3.2.1 reverted to "clippy clean, no tolerance".

## Cleanup principles

1. **Per-PR scope**: one crate (or one lint family) per PR. Never bundle
   cleanup with feature work.
2. **Registry must shrink**: every cleanup PR regenerates
   `clippy_baseline_raw.txt` via `tools/harness/extract_clippy_fingerprints.py`
   and commits the smaller file in the same PR.
3. **No new violations**: PRs that clean N lints but introduce M new ones
   (where M > 0) are rejected. The registry tracks NEW vs baseline.
4. **Test coverage preserved**: do not silence rustc warnings with
   `#[allow]` — fix them.
5. **Toolchain-aware**: if a future Rust release introduces additional
   lints on currently-clean code, register them here as a separate batch
   with the rustc version that surfaced them.

## Distribution by file (31 total)

| Count | File |
|------:|------|
| 14 | `crates/sim-test/src/main.rs` |
|  3 | `crates/sim-systems/src/runtime/mod.rs` |
|  2 | `crates/sim-bridge/src/lib.rs` |
|  1 | `crates/sim-systems/src/runtime/steering_derive.rs` |
|  1 | `crates/sim-systems/src/runtime/steering.rs` |
|  1 | `crates/sim-systems/src/runtime/pairwise.rs` |
|  1 | `crates/sim-systems/src/runtime/knowledge_learning.rs` |
|  1 | `crates/sim-systems/src/runtime/health.rs` |
|  1 | `crates/sim-systems/src/runtime/band.rs` |
|  1 | `crates/sim-engine/src/frame_snapshot.rs` |
|  1 | `crates/sim-data/tests/ron_registry_test.rs` |
|  1 | `crates/sim-data/src/lib.rs` |
|  1 | `crates/sim-core/src/config.rs` |
|  1 | `crates/sim-core/src/components/social.rs` |
|  1 | `crates/sim-core/src/components/knowledge.rs` |

## Distribution by lint (31 total)

| Count | Lint |
|------:|------|
| 10 | `clippy::field-reassign-with-default` |
|  2 | `clippy::len-zero` |
|  2 | `clippy::int-plus-one` |
|  1 each | `rustc::unused-variables`, `rustc::unused-imports`, `clippy::useless-vec`, `clippy::unnecessary-map-or`, `clippy::unnecessary-get-then-check`, `clippy::unnecessary-cast`, `clippy::redundant-closure`, `clippy::needless-range-loop`, `clippy::needless-borrow`, `clippy::manual-range-contains`, `clippy::manual-is-multiple-of`, `clippy::manual-contains`, `clippy::implicit-saturating-sub`, `clippy::if-same-then-else`, `clippy::identity-op`, `clippy::collapsible-if`, `clippy::assertions-on-constants` |

## Suggested cleanup waves

Wave order is by **safest-first** (test code, then non-hot-path
production, then hot-path production), not by raw count.

### Wave 1 — Test/dev code (16 lints, lowest risk)

- [ ] `crates/sim-test/src/main.rs` — 14 lints (mostly
      `field-reassign-with-default`, `len-zero`, dead vars).
      Single PR, no behaviour change possible (test binary).
- [ ] `crates/sim-data/tests/ron_registry_test.rs` — 1 lint.
- [ ] `crates/sim-data/src/lib.rs` — 1 lint (loader test fixture).

### Wave 2 — sim-core / sim-engine non-hot-path (4 lints)

- [ ] `crates/sim-core/src/components/social.rs` — 1 lint.
- [ ] `crates/sim-core/src/components/knowledge.rs` — 1 lint.
- [ ] `crates/sim-core/src/config.rs` — 1 lint.
- [ ] `crates/sim-engine/src/frame_snapshot.rs` — 1 lint.

### Wave 3 — sim-systems runtime (8 lints, requires care)

- [ ] `crates/sim-systems/src/runtime/mod.rs` — 3 lints.
- [ ] `crates/sim-systems/src/runtime/band.rs` — 1 lint.
- [ ] `crates/sim-systems/src/runtime/health.rs` — 1 lint.
- [ ] `crates/sim-systems/src/runtime/knowledge_learning.rs` — 1 lint.
- [ ] `crates/sim-systems/src/runtime/pairwise.rs` — 1 lint.
- [ ] `crates/sim-systems/src/runtime/steering.rs` — 1 lint.
- [ ] `crates/sim-systems/src/runtime/steering_derive.rs` — 1 lint.

Hot-path code — every PR MUST run a full harness pipeline (no ENV-BYPASS
permitted for cleanup work; the whole point is to stop tolerating these).

### Wave 4 — sim-bridge FFI (2 lints)

- [ ] `crates/sim-bridge/src/lib.rs` — 2 lints.
      Run Godot smoke test after each PR to confirm FFI snapshot still
      decodes correctly.

### Wave 5 — Registry retirement

- [ ] When `clippy_baseline_raw.txt` is empty, delete it.
- [ ] Revert CLAUDE.md Rule 7.1 v3.2.1 toolchain-drift wording (or
      replace with "no current tolerance — clippy clean is required").
- [ ] Remove `--all-targets` per-crate iteration from
      `authorize_env_bypass.sh` once the workspace short-circuit is no
      longer needed (re-evaluate after cleanup).

## Re-running cleanup verification

```bash
# After each cleanup PR, regenerate the registry from a fresh build:
cd rust
for crate in sim-core sim-data sim-systems sim-engine sim-bridge sim-test; do
    cargo clippy -p "$crate" --all-targets -- -D warnings \
        2>&1 | tee "/tmp/clippy_${crate}.log" || true
done
python3 tools/harness/extract_clippy_fingerprints.py /tmp/clippy_*.log \
    > .harness/baseline/clippy_baseline_raw.txt
# Commit the shrunken registry alongside the cleanup.
```

---

# In-flight ENV-BYPASS follow-ups

Active bypass commits awaiting formal pipeline re-run within 7 days:

| Feature | Bypass commit | Authorised | Re-run deadline | Status |
|---------|---------------|-----------:|----------------:|--------|
| `shelter-ring-center-fix-v1` | `3b1799e0` | 2026-05-03T10:58:24Z | **2026-05-10** | pending API recovery (resets 2026-05-05 10:00 KST) |

Procedure when bypass deadline approaches:

1. Verify Claude API rate limit cleared.
2. Run `bash tools/harness/harness_pipeline.sh shelter-ring-center-fix-v1 tools/harness/prompts/shelter-ring-center-fix-v1.md --quick`.
3. On APPROVE: append `verified-post-bypass-3b1799e0 score=<N>` line to `.harness/audit/env_bypass.log`.
4. On REJECT: revert `3b1799e0` or fix immediately and re-pipeline.
5. Tick the row above to "verified" once logged.

If a deadline is missed, the bypass is escalated — open an audit issue
referencing this file before any further commits land on `lead/main`.
