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

## Refresh through f5c8951e — 2026-10-19T15:00:52.563637+00:00

The JSON now includes all 17 task commits, full SHAs and changed-file manifests through f5c8951e. Prior snapshot metadata is preserved. Part2 completion-with-limitations was reported 13:48:38.995382Z; Part1 completion 14:11:11.838176Z. These are reported checkpoints, not invented exact execution end times. Part3 gate1 failure reported 14:24:29.986878Z; gate2 exit-code pass with log errors reported 14:52:21.587959Z. Gate3 active, merged identity and Part4/final work pending. Admission waiting is now 3600 seconds; prior 1800 seconds remains historical. Earlier pending descriptions above are the original snapshot.

| UTC | Full SHA | Subject | Changed files |
|---|---|---|---:|
| 2026-10-19T13:48:17+00:00 | `d56a926e9f665f4e0c04b70c9502533b9c96aedc` | art-p13b(p1): isolate capture input after a failed control repeat | 129 |
| 2026-10-19T13:48:39+00:00 | `c1827b3b68bfac11155c8cd00e45ecfe69e3fdc6` | art-p13b(p2): preserve screen evidence without upgrading partial results | 110 |
| 2026-10-19T14:12:11+00:00 | `a5771f1f489084d6e29032cb06365ffb87ff5731` | art-p13b(p1): enable walker only after exact identity and paired gains | 315 |
| 2026-10-19T14:25:58+00:00 | `1f6ee3a0a51f41bd5b688f44738ee3a98b8ab9d0` | art-p13b(p3): make capture dispatch auditable and exercise the real bridge | 162 |
| 2026-10-19T14:52:21+00:00 | `f5c8951e0a0a5861defe632cbd2a088dac71f692` | art-p13b(p3): give isolated renderer fixtures their required art bridge | 159 |

## Part3 completion evidence — 2026-10-19T15:28:15.298609+00:00

Gate3 clean audit: 15:13:04.508215Z, 120/120 and zero log errors. Merged release build exit 0 retains SHA256 `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`. Required merged identity: 28 launches 15:13:29Z–15:26:34Z; terminal exit 0 observed 15:26:57.646379Z, comparator PASS (all 6 RGBA/all 5 running exact). Part3 completion is evidenced by this observation; exact process-exit time remains unknown. This required identity repeat is not a speed measurement rerun.

HEAD remains `f5c8951e0a0a5861defe632cbd2a088dac71f692`; the existing 17-commit census is unchanged. Part4/final work remains pending. Earlier in-progress entries are historical.

## Freeze and final-gate update — 2026-10-19T16:14:42.158968+00:00

18th task commit: `40aa7a463d740b776248828211251187319c6af6`, UTC 15:28:46, `art-p13b(p3): freeze only after clean merged gate and exact walker replay` (433changed files; complete manifest in JSON). Source/bridge remain frozen. Final gate root audit 16:11:30.179086Z confirms 120/120 and zero log errors. Part4 remains incomplete: 16 admission slots/4 collectors, only W1/W200 valid; attempt3 after 16:43:12Z planned, not run. Booked wait 5886.583217859268 seconds; future 1800seconds not yet counted. Cross-checkout provenance qualification and stable local snapshots are linked in JSON. No Part4/final completion time inferred.

## Attempt3 terminal / attempt4 waiting — 2026-10-19T16:50:43.689556+00:00

Attempt3 supervisor69016 exited 0; all measurement-local snapshots remain equal. Seven slots produced three actual collectors: W0/W500 **VALID**, W-multi **invalid end load8.11865234375 >8**. Three section timers and ablation were denied before launch. Cumulative Part4: **23 slots,7 collectors,4 valid,3 invalid-end,16 refusals**. W-multi has2 actual failed checks; W0 one failed and one valid. Section timers/ablation have0 actual collectors. No matrix target has reached three failed executions.

Attempt4 is waiting (session26762), earliest **17:18:49Z**, epoch1792430329 after last completion1792428528.382771. No terminal verdict exists. Booked wait is **7686.698958873749seconds**; next interval prospectively adds1800.617229seconds, leaving1312.683812seconds, insufficient for another30-minute cooldown. External control order is sectionsW200, W-multi, W500, ablation, then remaining matrix; sources/flags stay frozen. Earlier attempt3-planned/attempt2-total paragraphs are historical snapshots.

## Part4 terminal audit — 2026-10-19T17:21:02.485487Z

**TERMINAL NOT_VERIFIED: wait budget prevents another retry.** Attempt4 supervisor 26762 and recovery 53175 both exited 0. All local snapshot checks are exact. Final census: **29 file-slots = 7 actual collectors (4 valid, 3 invalid-end) + 21 probe/prelaunch denials + 1 wrong-item policy denial**. W-multi has two actual failed checks, not three; three-strike exhaustion is not the stop reason. W-multi valid measurement, all three section timers and ablation remain NOT_VERIFIED.

External order change leaked exported `P13_ITEM=after-ablation-W200` into matrix execution. W-multi file4 was refused under the wrong policy item **before any probe or Godot**. Fresh-shell recovery explicitly unset `P13_ITEM`/`P13_VALIDITY_OUT`; file5 was the true fourth W-multi admitted opportunity and was denied on start load 17.5810546875. Both artifacts and original ledger history remain; file numbering is not rewritten or treated as five collector runs.

Final booked wait **9487.316187858582 seconds** leaves **1312.6838121414185 seconds < 1800 seconds** mandatory cooldown. Further admission retry is barred. This is terminal execution, not completed acceptance. Source/bridge freeze is unchanged; the forthcoming Part4 documentation commit is not invented. Prior planned/waiting notes above are historical.
