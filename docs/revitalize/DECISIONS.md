# Revitalize decisions

Run started 2026-10-10 12:07 KST (03:07 UTC); nine-hour boundary 21:07 KST.
Authoritative specification: owner attachment, copied outside the clone to
`/home/hyunlord/work/revitalize-goal.txt`.

- Dedicated clone `/home/hyunlord/work/new-world-revitalize`, branch
  `codex/revitalize-2026-10`, base `d409e9b941ccc868747e4eb63953eafa6831bff7`.
- The owner explicitly overrides stale preflight/reset, main-branch push,
  interactive harness, and planning approval instructions. Never install git
  hooks or write harness audit/baseline/reviews or OMX/OMO state.
- Existing macOS checkout is not a delivery workspace. All implementation,
  builds, probes, and captures run on DGX.
- Host has no Rust or Xvfb in its default environment. Install the requested
  toolchain and Godot using official distributions; retain PATH explicitly.

## Execution and evidence ledger

| Part | Deliverable | State | Evidence |
|---|---|---|---|
| 0 | arm64 extension, baseline gates | complete | 1729 debug + release tests; clippy and Godot gates pass |
| 1 | tallies, shared scenes, probe, captures, frozen legacy pin | pending | |
| 2 | default-legacy rules, world generation, terrain/building FFI | pending | |
| 3 | gated survival/growth loop, 64-seed tuning | pending | |
| 4 | deterministic art, truthful renderers/HUD, visual QA | pending | |
| 5 | full verification, Korean report, pushed draft PR | pending | |

Each Part runs the owner's gates before its commit/push; remote landing is
checked against local HEAD. No legacy behavioral assertion may be weakened.

## Environment exception: captures

`xvfb-run` is absent and `sudo -n true` reports a password requirement. Per
Part 0 item 5, all screenshot capture steps are skipped on this host; no
password request and no substituted screenshots. Capture tooling will still
be delivered. Headless smoke and strict checks remain mandatory. Visual
acceptance cannot be claimed from static checks.

## Native legacy precision and measurement

V7 `Hunger.value` is f32, whereas thirst, sleep, social needs and health are
f64. Preserve the existing component and its exact legacy trajectory. The
fingerprint hashes native `Hunger.value.to_bits()` (4 little-endian bytes),
and native 8-byte bits for the f64 values. Metric aggregation widens hunger
to f64 and sorts agents before summation. No FFI f32 snapshot downcasts are
used for fingerprints.

The remote clone has no Graft CLI/index; the local macOS graph describes an
older checkout. Use bounded source reads on the remote clone rather than
using that stale graph as evidence.
