# Revitalize baseline — Part 0

Recorded on DGX Spark, 2026-10-10, 12:07–12:18 KST.
Base: `d409e9b941ccc868747e4eb63953eafa6831bff7` (`origin/lead/main`).
Dedicated branch: `codex/revitalize-2026-10`.
No simulation behavior changed in this Part.

| Environment | Observed |
|---|---|
| Host | aitopatom-d6bb, Linux 6.14.0-1015-nvidia, aarch64 |
| CPUs | 20 |
| Memory | 119 GiB total, 102 GiB available at start |
| Disk `/home` | 848 GiB available at start |
| Rust | rustc 1.93.1 (01f6ddf75 2026-02-11), pinned 1.93 toolchain |
| Cargo | 1.93.1 |
| Build/test parallelism | CARGO_BUILD_JOBS=10, test-threads=8 |
| Godot | 4.6.stable.official.89cea1439 Linux arm64; official ZIP SHA512 verified |
| Pillow | 12.2.0 |
| Xvfb | unavailable; passwordless sudo denied |
| Git hooks | `.git/hooks/pre-commit` absent; harness hooks not installed |

## Executed gate

| Check | Result | Wall time |
|---|---|---|
| cargo build -p sim-bridge | PASS | 30 s |
| Godot headless import | PASS | 5 s |
| cargo test --workspace -- --test-threads=8 | 1,729 passed, 0 failed | 315 s |
| cargo clippy --workspace -- -D warnings | 0 warnings | 13 s |
| cargo clippy --workspace --all-targets -- -D warnings | 0 warnings | 4 s |
| cargo test --workspace --release -- --test-threads=8 | 1,729 passed, 0 failed | 121 s (includes release build) |
| Forbidden simulation transcendental scan | 0 matches | included |
| gdscript_strict_check.sh | 14 files clean; 10 FFI calls match; PASS | 3 s |
| Godot headless 900-frame smoke | PASS; no non-audio ERROR/SCRIPT ERROR | 6 s |
| git diff --check | PASS | included |

Every Godot command ran through `timeout 600`, one process at a time.
Memory remained above the 24 GiB safety threshold. Raw command outputs are
retained locally under `.harness/evidence/revitalize/logs/p0/`.

## Comparison with the supplied x86 reference

The supplied reference says 1,730 tests; this pristine base executes 1,729
in both debug and release, across 107 test-result groups including empty
and documentation groups. No Rust source or test file was changed for this
measurement. No architecture-dependent test guard was found in sim-test.
The historical one-test difference cannot be attributed without that
planner's exact worktree/log; it is not silently added to the measured
count. All subsequent count accounting starts from **1,729**.
Clippy and strict-check counts match the supplied reference.

## Capture exception

`xvfb-run` is missing, and `sudo -n apt-get install -y xvfb ripgrep` failed
because a password is required. Following Part 0 item 5, every screenshot
capture step is skipped. No fabricated or substituted screenshots are
provided. Headless smoke proves loading/execution, not visual acceptance.

## Part 1 legacy probe

Pending instrumentation; measurements will be appended after the probe runs.
