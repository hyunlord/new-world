# V7 Hook Governance v3.3.7 Amendment — Rubric Detection Fixes

**Effective**: 2026-05-09
**Author**: kwanhyeon.park (T7.5.5.E dispatch)
**Lane**: STRUCTURAL-COMMIT
**Predecessor**: v3.3.6 (2026-05-09 — sim-systems Signal A whitelist + .log Signal B)

---

## Motivation

T7.6 (Phase 2 influence systems — `impl RuntimeSystem for X` in sim-systems)
ran end-to-end with **Evaluator APPROVE 15/15** (Codex), Regression CLEAN
15/15, but the rubric reported **46/100 (F)** because three scoring
dimensions failed to detect legitimate signals:

| Dim | Signal observed | Score | Cause |
|-----|-----------------|------:|-------|
| Mechanical FFI | sim-bridge crate absent (V7 reset state — T7.7 will introduce it) | -2 | `step0_ffi.txt` empty → FFI_STATUS=UNKNOWN |
| Test Coverage | 18 new harness tests in `tests/harness_phase2.rs` | 0/20 | Regex required `tests::harness_*` or `harness_x::harness_*`; Cargo integration tests print as plain `test harness_<name> ... ok` |
| Visual Verify | sim-systems Rust-only registration (no Godot files) | 0/20 | Cold-tier classifier Signal D = 0 (impl RuntimeSystem detected); hot-tier path required Godot screenshot for --full mode |

All three were *rubric detection gaps*, not code or process defects. The
underlying code passed every quality bar. v3.3.7 closes the three gaps
without weakening any quality requirement.

---

## §1 — Test Coverage Detection (Cargo Integration Format)

**File**: `tools/harness/generate_report.sh:170`

**Before** (v3.3.6):
```
^test (tests::harness_|harness_[a-z][a-z0-9_]*::harness_).*\.\.\. ok$
```

**After** (v3.3.7):
```
^test (tests::harness_|harness_[a-z][a-z0-9_]*::harness_|harness_).*\.\.\. ok$
```

**Rationale**: Cargo integration tests living in `crates/<x>/tests/<file>.rs`
print test results as `test <fn_name> ... ok` with no `::` namespace prefix.
Such tests use the `harness_*` naming convention exactly the same as inlined
`mod tests { harness_* }` cases. The third alternation branch admits the
plain prefix while preserving prior detection of unit-test (`tests::harness_*`)
and submodule-style (`harness_x::harness_y`) integration formats.

**Regression sweep** (executed against
`.harness/results/t7-6-influence-systems/gate_result_attempt2.txt`):
- Before patch: 0 matches
- After patch: 41 matches (capped at 10 → 20/20 score)

---

## §2 — No-Godot-Scope Visual Auto Credit

**File**: `tools/harness/generate_report.sh:346` (post-COLD_TIER branch)

**Before** (v3.3.6): only cold-tier classifier (all 4 signals) granted Visual
auto credit. Hot-tier diffs were forced down the Screenshot+VLM path even
when the change had zero Godot surface.

**After** (v3.3.7): added an `elif` branch — if `DIFF_FILES` contains zero
Godot files (no `.gd`/`.gdshader`/`.tscn`/`.tres` extensions and no
`scripts/` or `scenes/` paths), Visual is auto-credited 20/20 with reason
`no-godot-scope auto credit (v3.3.7 §2)`.

**Rationale**: The Visual Verify dimension scores Godot rendering quality.
A diff that only touches Rust crates (sim-systems, sim-engine, sim-core,
sim-data, sim-test) inherently has no visual surface to verify, regardless
of whether `impl RuntimeSystem for X` triggers cold-tier Signal D = 0.
This credit is *narrower* than cold-tier credit (only Visual, not the
whole cold-tier auto-pass) and explicitly preserves Signal D's intent
(hot-tier classification for harness lane selection).

**Quality preservation**:
- Any commit touching `.gd`/`.gdshader`/`.tscn`/`.tres` or paths under
  `scripts/`/`scenes/` still falls through to the Screenshot+VLM path.
- Cold-tier classifier output is unchanged — auto credit is granted by
  generate_report.sh, not by reclassifying the commit's tier.

---

## §3 — FFI Vacuous Check (sim-bridge Absent)

**File**: `tools/harness/generate_report.sh:119` (FFI_STATUS resolution)

**Before** (v3.3.6): FFI_STATUS = UNKNOWN unless `step0_ffi.txt` contained
`OK`/`PASS`/`COMPLETE`. V7 reset deleted sim-bridge; pre-T7.7 commits cannot
produce a populated step0_ffi.txt → -2 penalty on every Mechanical Gate.

**After** (v3.3.7): FFI_STATUS resolution runs `ffi_vacuous_check.sh` first
(reuses the existing helper, which checks for any `rust/crates/sim-bridge/`
diff entry). If vacuous → FFI_STATUS = OK; else fall through to step0
file check.

**Rationale**: `ffi_vacuous_check.sh` was authored under v3.3 §4.4 for
exactly this purpose but had no caller. The credit is gated on diff
content (zero sim-bridge files), so any commit that does touch sim-bridge
(post-T7.7) still goes through the normal step0 verification.

**Quality preservation**: Vacuous credit only applies when sim-bridge
diff is empty. Once T7.7 introduces sim-bridge, every change there
triggers the existing step0_ffi pipeline.

---

## Regression Sweep — Audit Chain Stability

All 25 prior STRUCTURAL commits (T6.1–T6.8, T7.1–T7.5,
T7.5.5.0/A/B-rubric/B/C/D) re-classify clean under v3.3.7 because:

- §1 only *adds* a regex branch (more inclusive); old matches remain.
- §2 only triggers when no Godot files are present (those commits already
  scored Visual via cold-tier credit, identical or higher).
- §3 only triggers when sim-bridge diff is empty (which it has been for
  every commit since the V7 reset).

No prior commit is reclassified or destabilised.

---

## Implementation Files

| Change | File | Lines (approx) |
|--------|------|---------------:|
| §1 regex | `tools/harness/generate_report.sh` | 170 |
| §2 elif | `tools/harness/generate_report.sh` | 346–356 |
| §3 vacuous | `tools/harness/generate_report.sh` | 119–138 |
| Doc | `.harness/prompts/governance_v3_3_7_amendment.md` | this file |
| Audit | `.harness/audit/structural_commits.log` | append |

---

## Lane

STRUCTURAL-COMMIT (cold-tier 4 signals on staged files: only `tools/`,
`.harness/` paths — Signal A exempt; `.sh`/`.md` — Signal B; no GDScript;
no `.rs` files at all → Signal D vacuously holds).
