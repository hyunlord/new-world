# M5.5 decisions

Numbered decisions of the M5.5 pass (`docs/v8/m5_5/M55_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m55-data/` (`$DATA`), outside the repository. The goal says "never ask the
owner": every open question is decided here.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `cb58a801` (M5 report; gate green at `ef1bf3b7`); `v8/wip` at the same commit,
  checked with `git rev-parse` against `origin/*` at the start (2026-10-23 22:44 KST; the local branch ref
  `v8/main` is stale at `4874c9dc` and is not used — `v8/main` moves on the remote by explicit refspec).
- Choice: base worktree `../new-world-m55-base` detached at `cb58a801` with the main clone's `rust/Cargo.lock`;
  `$DATA` = `/Users/rexxa/work/m55-data/`; an owned `caffeinate -i` for the session; `CARGO_BUILD_JOBS=8`, tests
  `--test-threads=8`, probe `--workers 6`. Start 22:45 KST; the 22-hour target is 2026-10-24 20:45 and the 26-hour
  hard stop 2026-10-25 00:45. Base data (`$DATA/base/`): release binaries of `cb58a801` (`bin/`), the identity
  capture (`identity/`, with the base `perf.sh` run in `identity/perf/`), the soak (`soak/`) — `$DATA/base_capture.sh`.
- The host is shared (the art session runs ART-STRUCT in parallel); timings are labelled with their load.
- Evidence: `$DATA/TIMELOG.txt`, `$DATA/base_capture.log`.

## D2 — Art record (goal §5.2)

- Read from `origin/v8/art-env1` (`429a1800`, `docs/v8/artenv1/ENV1_REPORT.md`): ART-ENV1 ended **PARTIAL** — item
  style 12/13, deer readable / wolves not (neither adopted), built pieces 13/23 accepted with seams 246/256 failing,
  scene readability failed, FPS passed. The owner's verdict (goal §2): the structures look unnatural (mixed
  projection). ART-STRUCT runs on `v8/art-struct`, which is not on the remote yet (its data folder
  `/Users/rexxa/work/artstruct-data` exists). Route I is the adopted people standard (ART-PROD3, 34/36).
- Choice: ROADMAP gets ART-ENV1 (끝·PARTIAL) and ART-STRUCT (진행 중) rows; HISTORY and ADR 0018 get the record. No
  art branch is merged and nothing under `art/**` or `docs/v8/art*/**` is touched. ART-STRUCT's state is read again
  in Part 6 for ROADMAP's "다음" line (goal §11: M-ART2 when the owner approves ART-STRUCT, else M6 prep).

## D3 — ADR 0016 amendment: the memory instrument and the RSS outer bound (goal §3.4, §5.3)

- Context: M5 D19 moved the C3 per-agent and per-tile memory ceilings from RSS to the allocator counter, and did it by
  stopping a running gate. The change was sound (RSS on macOS read 83.0–117.5 B per tile for one binary across
  sessions; the allocator read 63.1 B every time) but the contract did not say it, and fragmentation is real memory
  that the allocator counter does not see.
- Choice: ADR 0016 gains an amendment: the ceilings judge the allocator counter when the run has it; **RSS keeps an
  outer bound** — `worldsim_perf bench` runs, after its timed repeats, 5 build-only processes each of N = 200,
  N = 2,000 and the 64² scene (`bench-one --build-only REPEAT`: build, read RSS and the allocator, exit; about 0.1 s
  each), pairs them by repeat, and judges the median of the 5 per-repeat slopes against 5,120 B per agent and 128 B
  per tile (`rss_outer` in `bench.json`; ceilings `rss_per_agent_le_5120`, `rss_per_tile_le_128`). Build-only runs
  never enter the timed medians (`summarise` splits them by the `build_only` flag, so `bench-summarise` of an
  interleaved session handles them too). Without build-only runs the bound is `n/a`, which `bench_ledger.py check`
  fails like any ceiling that is not PASS (absence is a failure). `bench_ledger.py check` prints the bound every run
  and the gate summary carries the line (`rss outer`).
- The rule "never change an instrument while a gate runs" is written into ADR 0016's amendment and the pitfalls skill
  (Part 6): a gate that finds an instrument wrong is stopped, the finding recorded, the instrument changed in its own
  commit, and the gate restarted from step 1.
- Evidence: `bench.rs` test `the_rss_outer_bound_is_the_median_of_the_paired_build_only_repeats` (five repeats with an
  outlier: the median is 5,120 B per agent exactly — PASS at the bound — and 129 B per tile — MISS; without
  build-only runs: `n/a`). Earlier sessions' RSS per tile (M5 gate `ef1bf3b7`: (9,296 − 5,504) KiB × 1,024 / 32,768 =
  118.5 B) sit under the 128 B bound.

## D4 — M5 owner decisions recorded (goal §5.4)

The owner's decisions on the M5 report (goal §2 and §5.4), each implemented in Part 1 and recorded in an ADR
amendment:
- **No pressure overlap** (ADR 0027): illness never starts during a major or within 2 days after one; a major waits
  while 2 or more are sick.
- **The undrafted flee** (ADR 0028): drafting must matter; an undrafted person near a wolf runs to a roofed room or the
  hearth and defends at half damage.
- **The hunter carries the meat** (ADR 0028): the kill chains a haul of the meat to a stockpile.
- **Knife = 4 uses** (ADR 0029), **hide wrap = 1 cured hide** (ADR 0029).
- **Mood and quarrels may be tuned** within the goal's limits (§3.1.6; ADR 0030): pressure modifiers up to ×2,
  breakdown threshold to 150, tantrum threshold to 300, the quarrel formula's constants.
- **Deer art yes, wolf art pending**: the art line may install ART-ENV1's deer; the wolf stays on the code-drawn
  placeholder until a wolf passes review (no art work in this pass).

## D5 — V7 migration code is not used for migrants (goal §2)

- Context: goal §2 asks to inspect `components/settlement_migrant.rs` and `runtime/settlement/` (V7 migration and
  fission) before reviving them.
- Finding: `SettlementMigrant` is a marker for a non-member walking to a simulated settlement through the decision
  system's `Seeking { Agent(member) }` arm and joining by proximity; fission splits a simulated settlement. Both
  assume both ends are simulated villages, and the colony runs with fission off (ADR 0014).
- Choice: migrants are new colony code (`runtime/neighbours/`): they are spawned as colony people at a map edge and
  walk to the hearth with the job layer's walking state (as drafted people do, the decision seam skips them while
  they wait); accepting a wave is a command. No colony code depends on the V7 migration path.

## D6 — The ledger rule for this pass

- The gate's bench compares with the latest quiet rows of another pass (`check --pass M5.5`): M5's rows appended at
  its report (`55c638c0`, pass "M5"). M5.5 appends its rows in its report step with `--gate-log`.

## D7 — Calibration and held-out seeds (goal §0, §10)

- Calibration on seeds 0–7 only, at most 8 rounds, each logged here (D-numbered rounds table) with what changed.
- Held-out seeds 900–931, grepped before use: no earlier pass used them (M3 400–431 and 600–631, M4 700–731, M5
  800–831). They are measured once at the end (Part 5) and once after the review fixes; never used to tune.
