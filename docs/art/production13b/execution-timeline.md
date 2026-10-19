# Execution timeline

Snapshot covers the 12 task commits from `f962f5cd` through `af30bb29`. All times below are UTC; Git author/committer timestamps with their original +09:00 offsets are retained in JSON.

**12:08 is a conservative deadline anchor, not an observed start.** GOAL filesystem creation is 12:10:39, and the first commit is 12:10:57. Part1 deadline is 17:08, Part4 freeze deadline 19:08, and ten-hour cutoff 22:08.

| Part | Evidence boundary | Status / timing limit |
|---|---|---|
| 0 | Completion documented by 12:40:03 commit 4a954fa9 | Complete; baseline 119/120 retained. Exact actual end unknown. |
| 1 | In progress by 12:42:45 commit f55cf1e0 | Still in progress; no end or elapsed completion duration. |
| 2 UI | First p2 commit 12:48:22; base capture starts 12:48:36.311838 | Current working TASKS reports complete with limitations; exact end unknown. |
| 3 | No start established | Pending in TASKS. |
| 4 | Deadline 19:08 | Pending; no freeze-completion timestamp. |
| 5 | No start established | Pending; no finish timestamp. |

Part1 and Part2 overlap. The booked 12:45:50–13:15:50 admission cooldown contains activity base capture 12:48:36–12:49:58 and supplements 12:50:46–13:04:35. Admission wait is separately **1800 seconds**, not additional elapsed time; it was used for UI work. Do not add phase durations or subtract this wait from deadlines.

Admission source: `docs/art/production13b/perf/attempts.json`, SHA256 `141a5bdaec1dcc1f465cff519e61a9b567b7a9461c8766b9d2ae2ddfaf831d15`.

The working TASKS header is newer than its committed counterpart: at af30bb29 the Part2 header still says PENDING, while current working TASKS says COMPLETE WITH LIMITATIONS. Neither wording supplies an exact end timestamp. The hashed TASKS snapshot and launch timestamps in JSON preserve this distinction.

## Task commits and file manifest

| UTC | Full SHA | Subject | Files |
|---|---|---|---:|
| 2026-10-19T12:10:57Z | `f962f5cd6e671c6b000ca6e0734b904449360650` | art-p13b(p0): preserve the execution contract before windowed validation | 1 |
| 2026-10-19T12:15:08Z | `9c023aea430c9d0122c9883f7509712f88c2b188` | art-p13b(p0): isolate new measurements from preserved pass13 evidence | 7 |
| 2026-10-19T12:17:00Z | `22d92922f513569df8f4a6e6a9b4e8d866dbb3be` | art-p13b(p0): bind new admission decisions to a fresh measured reference | 2 |
| 2026-10-19T12:20:50Z | `8900e1c69f8ba3aa8775d6f1af154e622bcb87df` | art-p13b(p0): fix screen-reading rules before collecting new answers | 3 |
| 2026-10-19T12:25:03Z | `32b33df8a53f8d6f4c2958d21106308e358e54f0` | art-p13b(p0): preserve reference-build progress and walker prerequisites | 2 |
| 2026-10-19T12:33:01Z | `c774ff518ace7abd0e01e77af3d4752149d7901e` | art-p13b(p0): establish complete deterministic reference evidence | 6 |
| 2026-10-19T12:40:03Z | `4a954fa9266ab24f7d3f9ba5c1fc3414ca33ba85` | art-p13b(p0): preserve baseline evidence before correcting hover input | 151 |
| 2026-10-19T12:42:45Z | `f55cf1e0e60a381c9c0d0199c26d07e1c57fc155` | art-p13b(p1): restore readable Life scale and honor delivered hover coordinates | 48 |
| 2026-10-19T12:48:22Z | `8ca6b93018b0ec0b795f8312c549197a97f851da` | art-p13b(p2): bind window evidence and separate switch settings from counters | 14 |
| 2026-10-19T12:59:43Z | `3bee1316acdbcb891bda0be1385ba959bbb1e7f1` | art-p13b(p2): require visible porter evidence and reachable estrangement | 7 |
| 2026-10-19T13:06:55Z | `75c2ea6a2cdfb085a4a4a90ceaf9fe257cda16a3` | art-p13b(p2): preserve screen limits and correct final event boundary | 44 |
| 2026-10-19T13:23:43Z | `af30bb29b01c88b6c3f6475e7cf179dc8232bb45` | art-p13b(p1): verify walker semantics before considering activation | 24 |

The complete per-commit changed-file manifest (Git A/M/D status and path), plus a deduplicated path list for final section 10, is in [execution-timeline.json](execution-timeline.json). Paths are literal Git changes, including evidence artifacts; no uncommitted file is silently included in those commit manifests.

Historical references `8c1b18be8b05bfeaa062a937526fe4d9c2f53414` (backup/reference base) and `57ae7ac40abde7d24369606c9250d3e06e0f18d6` (prior publication) are not task commits.

No Godot/build/check reruns or source/HEAD changes were made for this timeline. Unfinished parts have null end times; this snapshot does not establish identity, speed, final freeze, publication, or final acceptance.
