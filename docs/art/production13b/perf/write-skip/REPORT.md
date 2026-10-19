# Write-skip measurement record

**Decision: DEFAULT OFF.** W200 has one valid session with three alternating pairs, but its pooled candidate/control median ratio **0.9986985164993292** exceeds the required **0.97**. W500 attempt3 is **VALID** with three faster pairs and pooled ratio **0.947285393220096**, passing its threshold. The mandatory never-drop write-skip measurement is **COMPLETE**. Joint acceptance still fails because W200 exceeds 0.97. W500 attempt1 collector failure and attempt2 pre-Godot admission denial remain preserved.

## Identity

[Part0 reference](../identity/README.md) uses detached `8c1b18be8b05bfeaa062a937526fe4d9c2f53414`, kernel on/write skip off, with an optimized-dev bridge. Twelve A/A′ framing launches and five running-reference launches completed: **17 launches**. Six framings have zero differing RGBA pixels. Running schedules are W0/W200/W-multi **600 frames each**, and W500 frozen/2× pans **240 frames each**. Reference self-comparison establishes integrity only, not candidate acceptance.

Independent write-skip candidate identity reports [six.json](../identity/write-skip/six.json) and [running.json](../identity/write-skip/running.json) both pass: all six A/B framings exact and all five running/pan comparisons pass. The candidate identifier is `write-skip`, kernel state `on`. This does not override the speed rule. The W200 speed-summary field `pixel_identity: NOT_VERIFIED` is preserved verbatim; separate identity reports supply the candidate result.

## Speed evidence

All four attempts record release bridge SHA-256 `7c4144a3ab6633568c9a17e565c8634c7b074afa7d43b80034fd0e45e5e2c73b` and measurement checkout HEAD `8c1b18be8b05bfeaa062a937526fe4d9c2f53414`. Their launch manifests retain exact before/after source hashes and commands. This release provenance is distinct from the Part0 optimized-dev reference provenance.

| Attempt | Outcome | Blocks / samples | Valid pairs | Exit |
|---|---|---:|---:|---:|
| W200 attempt1 | VALID; speed threshold fails | 6 / 900 | 3 | 0 |
| W500 attempt1 | Collector FAIL | 2 / 300 | 0 | 1 |
| W500 attempt2 | Admission denied before Godot | 0 / 0 | 0 | 75 |
| W500 attempt3 | VALID; speed threshold passes | 6 / 900 | 3 | 0 |

The speed-attempt inventory is **4 attempts, 3 Godot launches, 2 valid sessions, 1 collector failure, 1 pre-launch admission denial**. Each collected block has 150 measured and 30 discarded frames. W200 is running 2× Z4, with six fresh blocks, population200 maintained and simulation ticks advancing. Primary CPU is process_ms + render_cpu_ms; setup is separately reported. GPU timings are unavailable, not measured zero cost.

| W200 pair | Control median ms | Candidate median ms | Candidate/control |
|---|---:|---:|---:|
| 1 | 24.524499997 | 25.085499997 | 1.022875084102465 |
| 2 | 25.368000005 | 24.920000001 | 0.982339955691544 |
| 3 | 24.994999999 | 24.862500001 | 0.994698939883130 |

Pooled medians: control **24.9714999958873 ms**, candidate **24.93900000065565 ms**, 450 measured frames each. Two of three pairs are faster, but the pooled ≤0.97 requirement fails. The unchanged [W200 summary](W200-attempt1/summary.json) includes p90/p99/maximum and all block metrics. W200 is not rerun to seek a better ratio.

W500 attempt1 ran the inherited `_p13` collector at simulation_speed0, tick1440 unchanged. Its terminal error is **“Renderer switch changed inside a measured block”**. Two collected blocks do not form an eligible pair: collector valid=false and wrapper exit1. No ratio is promoted from this failed session. The record is a collector failure, not a valid speed failure or population-loss exclusion.

W500 attempt2 ran the `_p13b` wrapper at 2026-10-19T13:16:01.404881Z. Start load1m **23.0029296875**, probe **2.7825928339734674s**; admission exit75 preceded Godot. Session log is empty and frames.json absent. End probe **2.8250994589179754s**, load1m **21.72119140625**. The retained driver traceback reports “Missing terminal session evidence”; that downstream error does not convert the denied admission into a launched measurement.

Calibration R is **2.762010041042231s**, SHA-256 `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4`. All attempts’ start/end probes, loads, checksums and verdicts are retained in compact validity JSON; complete process snapshots remain in the hashed originals.

W500 attempt3 completed **VALID, exit0**, 2026-10-19T13:46:24.254803Z–13:47:16.311677Z. Full before/after launch provenance is identical. It uses the `_p13b` collector with the recorded **3600-second admission wait**. All six blocks retain population500 and tick1440 at simulation_speed0 (frozen), each with150 measured and30 discarded frames. No collector errors or contention flags are present.

| W500 pair | Control median ms | Candidate median ms | Candidate/control |
|---|---:|---:|---:|
| 1 | 32.047499989 | 30.344999997 | 0.946875731567341 |
| 2 | 32.273500006 | 30.383999999 | 0.941453514304018 |
| 3 | 31.743999999 | 30.185999992 | 0.950919858662764 |

Pooled W500 medians: control **32.01200000703335ms**, candidate **30.324500014424302ms**;450 samples each. Ratio **0.947285393220096**, all3 pairs faster: W500 speed PASS. The pooled ratio was independently recomputed from raw frames. [W500 summary](W500-attempt3/summary.json) retains full metrics. Start/end probes **2.8551294580101967s / 2.802626542048529s**, load1m **6.6953125 / 6.53125**; wrapper VALID. Earlier failures remain historical evidence, not eligible speed pairs.

## Preservation and limits

[manifest.json](manifest.json) records exact source paths, bytes and SHA-256 for every attempt artifact and existing identity report. Launch manifests, both valid world summaries, session logs/exits and the admission driver traceback are byte-identical copies. `frames-compact.json` omits per-frame arrays but preserves terminal metadata, block state, sample/discard counts and the original frame-file hash. `validity-compact.json` explicitly omits only process-list text. Giant raw frames and original process snapshots remain external and are not duplicated.

Historical failures remain visible. This documentation task launched no runtime, changed no source, and made no commit. The valid third W500 session completes the mandatory measurement. No fishing rerun is needed or authorized by this report: the existing valid W200 failure remains decisive, and default remains off.
