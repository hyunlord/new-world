# Art Pass 12 execution

Start: 2026-10-18T09:32:00Z (initial brief read; conservative clock).
Pauses: none recorded.
Deadlines: Part 0 12:02Z; Part 1 15:32Z; Part 4 starts by 18:32Z; 12-hour rule 21:32Z.

| Part | State | Start UTC | End UTC | Evidence |
|---|---|---|---|---|
| 0 merge, contract, release baseline and running diagnosis | in progress | 2026-10-18T09:32:00Z | | GOAL byte copy; merge evidence |
| 1 structural frame budget candidates | pending | | | |
| 2 names, life, crowds and input/blind tests | pending | | | |
| 3 porters, relations and chronicle | pending | | | |
| 4 frozen after measurements H1–H10 | pending | | | |
| 5 independent review, full gate, publication/report | pending | | | |

## Part 0 checkpoints

- Merge: bf85dac60579dd4a9135dd9f6f92a3139c8d1f6e, no conflicts.
- Brief preservation: 51ac34dff554e851e65f95a7b63b11e650859516.
- Tooling reference: 9934f78513cd1657a5df8fa2f7854b9df9e02e95, WIP pushed/ls-remote verified. Production branch remains b83dc5ae.
- First full gate: 87/88, sole scanner path failure repaired; full workspace rerun exit0. Clean full gate running under gates/p0-clean.
- Base/measure worktrees created at tooling reference. APFS clone of existing build cache, followed by actual cargo builds per worktree; release dylib copied to debug path only in measure worktree. Build/import logs under external evidence root.
- Preliminary main-checkout census ongoing; P1=29 people/one village, P2 lit-night, P3=37 people/two occupied villages; Godot double=false. These are partial raw results, not complete identity acceptance.

- Build/import chain status at this checkpoint: running; successful completion is not yet verified.

- 2026-10-18T10:08Z: optimized-dev reference and release measure builds/imports verified; release/debug dylib hashes equal only in measure. Complete game_v25 census: all 11 rows, no failures; W-multi reproduces two outbound porters. Six-framing A launched.

- Clean full Part0 gate terminal exit 0, 88/88 checks passed (observed process completion). Tools provenance follow-up separately passed 5 unittest methods plus shell syntax/diff checks.
