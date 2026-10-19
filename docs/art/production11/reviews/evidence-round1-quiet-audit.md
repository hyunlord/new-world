# P5 quiet-lock interval audit — round 1 extension

Read-only audit on 2026-10-16, extending evidence-round1.md. No runtime launched, no source edits, no children. Reviewed all discoverable launch.json and launches.tsv files under the task evidence root, all .log filesystem creation/last-write timestamps under that root and docs/art/production11, specific terminal artifacts, runner source, and the quiet hold ledger. All times below UTC on 2026-10-16.

## Result and limits

**No confirmed task-owned Godot, cargo, or gate overlap with the four completed baseline holds was found.** This is an audit of recoverable evidence, not proof of uninterrupted process surveillance. One apparent metadata overlap is a recorded quiet wait, not a Godot overlap (details below). Do not turn this result into an unqualified assertion that every process exit time was observed.

At this audit the P0 section session was still waiting, so it is not a completed hold covered by this result. Root reports no main Godot/cargo since13:35:52; the last structured running-A launch terminates at13:35:52, which corroborates the last recorded run but does not independently prove absence of unlogged processes afterward.

## Completed hold boundaries

Source: docs/art/production11/perf/holds.md HOLD_START/HOLD_END records.

| Hold | Start | End | Duration | Entry load | Prior foreign-process check |
|---|---|---|---:|---:|---|
| W0 | 10:38:34 | 10:40:23 | 109s | 3.59 | empty |
| W500 | 11:40:25 | 11:48:44 | 499s | 2.89 | empty |
| W1 | 12:23:45 | 12:28:04 | 259s | 3.77 | empty |
| W200 | 13:13:06 | 13:14:42 | 96s | 2.66 | empty |

Inter-hold gaps3602/2101/2702s satisfy the600s minimum. The measured session processes themselves are expected inside their own holds.

## P1 life evidence

| Run | Recoverable beginning | Recoverable final artifact | Relation to holds |
|---|---|---|---|
| life-attempt1 | log creation10:20:26.897 | failure.json10:25:28.262 | before W0 |
| life-attempt2 | log creation10:26:46.276, actual Godot output by10:26:48.197 | reference-data.json10:34:50.062 (complete=true) | before W0 by3m44s |
| independent raw-life replay | log creation10:40:49.791, Godot output10:40:50.413 | raw-life-replay.json10:45:37.385 | starts26.8s after W0 end; before W500 |

These are local filesystem creation/last-write timestamps, not invented explicit launch/exit records. Last log output alone is not process end: attempt2's log ends10:26:48 but its actual captures continue to10:34:50. I used the terminal reference artifact instead. capture_life_p11.gd writes its final reference then quits; the independent replay source life/raw-life-replay.gd likewise closes output then calls quit(0). Exact OS exit timestamps are not retained for these launches. The W0 admission process check at10:38:34 is empty, providing separate evidence that the earlier life Godot was no longer running when the hold started. The raw replay starts after W0, so it cannot contaminate that hold even though it is headless and outside the windowed capture wrapper.

## P2 structured launch intervals

These are launch.json started/finished records. They bound wrapper activity, and can include lock/window waits. No interval below intersects a completed hold except the explicitly marked quiet-wait case.

| Evidence path relative to evidence root | Started | Finished | Exit |
|---|---|---|---:|
| activity-attempt1/s20261010_d30_1000_z4/launch.json | 10:36:42 | 10:38:21 | 1 |
| activity-attempt2/s20261010_d30_1000_z1/launch.json | 10:42:26 | 10:44:03 | 0 |
| activity-attempt2/s20261010_d30_1000_z4/launch.json | 10:40:48 | 10:42:26 | 0 |
| activity-attempt2/s20261010_d30_1500_z1/launch.json | 10:45:41 | 10:47:22 | 0 |
| activity-attempt2/s20261010_d30_1500_z4/launch.json | 10:44:03 | 10:45:41 | 0 |
| activity-attempt2/s20261018_d60_1000_z1/launch.json | 10:53:51 | 10:54:06 | -15 |
| activity-attempt2/s20261018_d60_1000_z4/launch.json | 10:47:22 | 10:53:50 | 0 |
| activity-attempt3/s20261010_d30_1000_z1/launch.json | 10:59:35 | 11:01:14 | 0 |
| activity-attempt3/s20261010_d30_1000_z4/launch.json | 10:57:55 | 10:59:35 | 0 |
| activity-attempt3/s20261010_d30_1500_z1/launch.json | 11:02:54 | 11:04:33 | 0 |
| activity-attempt3/s20261010_d30_1500_z4/launch.json | 11:01:14 | 11:02:53 | 0 |
| activity-attempt3/s20261018_d60_1000_z1/launch.json | 11:11:00 | 11:17:27 | 0 |
| activity-attempt3/s20261018_d60_1000_z4/launch.json | 11:04:33 | 11:11:00 | 0 |
| activity-extra-d30-1100-k0/launch.json | 11:26:43 | 11:32:21 | 1 |
| activity-extra-d30-1100-k4/launch.json | 11:50:34 | 11:58:12 | 1 |
| activity-extra-d30-1400-k1/launch.json | 11:14:14 | 11:34:23 | 0 |
| activity-extra-d60-1100-k0/launch.json | 11:14:29 | 11:23:55 | 0 |
| activity-extra-d60-1400-k0/launch.json | 11:44:49 | 11:56:16 | 0 |
| activity-extra-d60-1400-k2/launch.json | 11:58:27 | 12:04:53 | 0 |
| activity-extra-d60-1400-k4/launch.json | 11:22:03 | 11:30:28 | 0 |
| activity-village-extras-round1/s20261010_d30_1100_z1/launch.json | 12:07:23 | 12:09:01 | 0 |
| activity-village-extras-round1/s20261010_d30_1400_z1/launch.json | 12:15:25 | 12:17:05 | 0 |
| activity-village-extras-round1/s20261018_d60_1100_z1/launch.json | 12:09:01 | 12:15:25 | 0 |
| activity-village-extras-round1/s20261018_d60_1400_z1/launch.json | 12:17:05 | 12:23:31 | 0 |

**Apparent W500 overlap resolved:** activity-extra-d60-1400-k0/launch.json records started11:44:49.376940 and finished11:56:16.544046. run_extra_activity_p11.py sets started BEFORE calling wait_for_quiet (lines83–85), and opens launch.log / starts the bounded process only AFTER that wait (lines95–96). Its quiet-waits.log explicitly records waiting for /Users/rexxa/work/.mac_quiet.lock at11:44:49.377024. The Godot launch log was created11:49:49.411628 and first output extends to11:49:50.465573; W500 hold ended11:48:44. Thus the evidence supports waiting during the hold and actual Godot launch at least65s afterward. There is no basis to flag this as contamination solely from the orchestration started field.

Closest P2 boundaries: rejected attempt1 finished10:38:21.579, 12.4s before W0 acquisition; attempt2 begins10:40:48.928, 25.9s after W0 release. The final village supplemental run finishes12:23:31.023, 14.0s before W1 acquisition. W1's immediately preceding CHECK has no Godot/cargo/rustc/worldsim process. P2 attempt3 and other supplements all end before their nearest later hold or begin after its prior release.

## Cargo, gate, imports and smaller tests

These entries use original external log creation/last-write timestamps unless an explicit terminal-event log exists. Copies under docs can have later creation times; I did not mistake copied-log times for process times.

| Work | Evidence timestamps | Relation |
|---|---|---|
| P0 full gate | gate logs and console07:53:00–10:08:06 | ends before first W0 hold |
| Initial measure release build |09:55:30–09:57:26 | before first hold |
| Release probe build/probe |09:57:39–09:59:47 | before first hold |
| Measure tooling build/import/confirm |10:00:20–10:01:20 | before first hold |
| W1 re-advance estimate |10:01:28–10:01:58 | before first hold |
| Baseline build/import |10:04:10–10:04:12 | before first hold |
| P0 scanner |10:08:20–10:08:31 | before first hold |
| P1 import, strict, tests, parse checks |10:17:47–10:26:30 | before W0 |
| P2 import, strict, tests, parse checks |10:35:18–10:36:30 | before W0 |
| P2 order strict check |10:54:28–10:54:39 | between W0 and W500 |
| P3 base debug build/import retries/confirm |12:36:33–12:38:26 | after W1, before W200 |
| P3 main debug build/import |12:38:03–12:38:13 | after W1, before W200 |
| P3 strict harness / clock |12:38:32–12:38:33;12:51:51–12:51:52 | after W1, before W200 |
| Five targeted Rust static tests |12:55:58–12:56:29 | after W1, before W200 |

The five targeted cargo tests have explicit completion stamps in static-repins/targeted-events.log; starts reconstructed by elapsed seconds agree with file timestamps. Their last completion is12:56:29, well before W20013:13:06. P0 gate has a documented host wall-clock discontinuity/suspension, so its reported7695s cargo duration cannot establish performance or a normal continuous timing interval. Its completed artifacts nevertheless precede the first measurement hold by roughly30minutes.

## P3 Godot captures and references

| Run | Recoverable interval | Relation |
|---|---|---|
| census-P2 | log12:38:51–12:39:57 | after W1, before W200 |
| census-P3 | log12:39:36–12:42:00 | after W1, before W200 |
| rejected six/A | launches.tsv begins12:42:13; P1 finishes12:43:47; stopped P2 log through12:44:31 | after W1, before W200; exact stopped-process exit not in TSV |
| clock diagnostic1 | log12:44:59–12:45:15 | after W1, before W200 |
| clock diagnostic2 | log starts12:52:03; artifact P5.json later | after W1, before W200 |
| completed six-attempt2/A | structured launches12:52:53–13:10:41 | ends2m25s before W200 |
| completed six-attempt2/Aprime | structured launches13:14:46–13:32:40 | starts4s after W200 release |
| running-A all four cases | structured launches13:32:40–13:35:52 | after W200 |

The full A/Aprime/running launch TSVs were individually parsed in the preceding evidence review; they do not overlap any of the four completed holds. Comparator/Python-only runs at13:06 and13:21–13:24 are not Godot/cargo/gate and must not be falsely classified as prohibited measurement workloads.

## Coverage gaps and classification discipline

- Many older task-owned launches lack explicit start/end UTC process records. File birthtime and final-artifact mtime recover evidence-production bounds, not OS process lifetimes. Silent lingering after artifact writes remains unmeasured except where the next quiet-lock admission process scan rules it out.
- Structured started can mean wrapper orchestration start, not Godot start. Conversely a final log mtime can be hours before process exit. This audit avoided both shortcuts.
- The log inventory sweep found timestamp-envelope overlap only for the expected baseline driver/session logs and the documented supplemental quiet wait. It cannot find launches that never wrote a retained log, deleted evidence, or foreign processes absent from the task's artifacts.
- The quiet lock captures process presence at admission; it does not retain a continuous in-hold process timeline. No claim about uninterrupted absence of every external process is warranted.
- Future section timer hold, optimization measurements and after matrix require the same independent interval audit once complete. Their current pending state is not a final FAIL.

Recommended final wording: “No task-owned runtime overlap was found in retained launch records and recovered artifact intervals for the four baseline holds; exact process-exit timestamps are missing for some early runs, and continuous process surveillance was not recorded.”
