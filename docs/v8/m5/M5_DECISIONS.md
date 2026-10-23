# M5 decisions

Numbered decisions of the M5 pass (`docs/v8/m5/M5_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m5-data/` (`$DATA`), outside the repository. The goal says "never ask the
owner": every open question is decided here.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `08f59813` (M4 report; gate green at `941410fc`); `v8/wip` at `24ea244d`
  (the goal commit on top of `08f59813`), checked with `git rev-parse` against `origin/*` at the start
  (2026-10-23 10:08 KST).
- Choice: base worktree `../new-world-m5-base` detached at `08f59813` with the main clone's `rust/Cargo.lock`;
  `$DATA` = `/Users/rexxa/work/m5-data/`; an owned `caffeinate -i` for the session; `CARGO_BUILD_JOBS=8`, tests
  `--test-threads=8`, probe `--workers 6`. Start 10:08 KST; the 22-hour target is 08:08 and the 26-hour hard stop
  12:08 on 2026-10-24. Base data (`$DATA/base/`): release binaries of `08f59813` (`bin/`), the identity capture
  (`identity/`, with the base `perf.sh` run in `identity/perf/`), the soak (`soak/`) — `$DATA/base_capture.sh`.
- The host is shared (an art session may run ART-PROD2 in parallel); timings are labelled with their load.
- Evidence: `$DATA/TIMELOG.txt`, `$DATA/base_capture.log`.

## D2 — Art record correction (goal §5.2)

- Context: M4's records (ROADMAP ART-TRIAL5 row and "다음" line, HISTORY, ADR 0018) say people art is parked after
  ART-TRIAL5. ART-PROD1 (`v8/art-prod1`, `eb428ce2`, `docs/v8/artprod1/PROD1_REPORT.md` there) has since produced
  whole-figure sheets by route I: 13 of 18 adult looks and 0 of 18 child looks accepted (130 frames); the children
  failed on the sleep frame (canvas normalisation and sleep width), not on reading; the trial's verdict was FAIL
  (S1, R1, P1) and route I was not adopted as the standard. ART-PROD2 runs on the same branch to fix the sleep rule
  and adopt.
- Choice: the records say people art is on route I (whole-figure sheets), pending ART-PROD2; "사람 그림 보류" is
  withdrawn. ART-PROD2's final state is read from `docs/v8/artprod2/` on `v8/art-prod1` in Part 7 and recorded then.
  This pass does not touch `art/**`, `docs/v8/art*/**` or merge any art branch.

## D3 — The ledger rule, second fix (goal §5.3)

- Context: M4 appended its own rows (pass "M4") at Part 5, so the Part 6 gate's `bench_ledger.py check` compared
  M4 with M4 (M4 D23 then compared M3 → M4 by hand).
- Choice: `check --pass NAME [--base COMMIT]` compares the candidate with the latest quiet rows whose pass label
  differs from NAME and whose commit is not a strict descendant of COMMIT; `append` needs `--gate-log PATH` (a
  `gate.txt` whose head matches the run's commit and whose status is 0) or `--re-measure TEXT` (an older pass that
  already passed its gate); the gate's `bench` step only checks (`BENCH_PASS`, `BENCH_BASE`), and a pass appends its
  rows in the report step, after its gate. Tests in `tools/v8/test_bench_ledger.py`.
- Built by a subagent in an isolated worktree (files listed in its report `$DATA/part0_tooling_report.md`), reviewed
  and brought in by the lead.

## D4 — The allocator byte counter (goal §4, §5.4)

- Choice: `sim_test::alloc::Counting`, a `GlobalAlloc` that forwards to `System` and counts live bytes; installed as
  the global allocator of `worldsim_perf` only (never the game, the bridge or the probe). The goal names
  `sim_core::alloc`; `sim-core` is `#![forbid(unsafe_code)]` and a `GlobalAlloc` needs `unsafe`, so the counter lives
  in the measurement crate instead of weakening the simulation crate's guarantee to `deny` (the subagent's first
  version did that). `worldsim_perf bench` records
  live bytes before and after the build and after the run; `alloc_per_agent` = (n2000 − n200 after the build) / 1,800
  and `alloc_per_tile` likewise. The ledger gains the column `alloc/agent (B)` after `RSS/agent (B)`; from M5 on the
  regression rule's memory figure is the allocator number when both rows have it (RSS stays as information).
- Reason: RSS per agent varied by up to 1 MB between runs of one binary (M4 D23: bimodal macOS allocator placement);
  live allocated bytes are a property of the program, not of the allocator's placement.

## D5 — M4 owner decisions recorded (goal §5.5)

- The teach dialog (K) auto-suggests the learner: the adult with the highest novelty seeking (`Temperament.ns`), then
  the youngest adult, among those who lack the node; the player may change it. (UI work in Part 5.)
- A re-learned knowledge keeps its original source's forgetting floor (M4 D24 F4 reversed by the owner): an entry
  whose proficiency rises through another channel keeps the source it was first held by when that source's floor is
  higher (seeded and discovered knowledge keep their floor). Implemented in Part 4 with a test.
- Forgetting speed unchanged pending the owner's playtest; pot firing unchanged.

## D6 — Where the M5 state lives

- Animals are not ECS entities: they live in `ColonyState.wildlife` (a `BTreeMap` by animal id, at most 16 at once).
  Reason: queries over `Position` without `Agent` exist in the command checks (`commands.rs`, people on a tile block a
  placement), the bridge's space reads and the probe; a beast entity would be counted as a person there. Nearest-person
  searches for wolves use the agents' spatial index; deer searches scan ≤ 12 animals.
- The storyteller's state is `ColonyState.story`; fights (person or animal against person or animal) are
  `ColonyState.fights`, resolved by `CombatSystem` beside its V7 pairs; person-pair relations are `ColonyState.bonds`;
  every one of them is saved with `ColonyState` (format v5).
- Chronicle causes are a side table of the chronicle (`Chronicle::causes`, entry index → `CauseRef`), not a field of
  `ChronicleEntry`: the probe's chronicle files carry the entry size and the approximate bytes, and identity compares
  them for the baseline; the side table is empty there.
