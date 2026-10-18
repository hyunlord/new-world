# Part 0 release baseline

Status: **NOT_VERIFIED (contended)**. No performance session was admitted and no frame-time number is accepted.

Corrected tooling commit: `a4901cb12e29afb74783039ed66aef4a74014148`. Measure worktree release bridge SHA256: `3b38f12812cea65c3c2c9e5d5d126dfb359c5dc0bc3860a976a554a304ca03c2`; copied debug path byte-identical. Both worktrees rebuilt/imported after checkout. Identity uses separate optimized-dev bridge, never performance evidence.

| Target | Attempts | Accepted | Median/p90/p99/max/ticks per frame | Verdict |
|---|---:|---:|---|---|
| W0 frozen/running2x Z4 | 3 (one interrupted before admission) | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W1 frozen/running2x Z4 | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W200 frozen/running2x Z4 | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W500 frozen/running2x Z4 | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W-multi frozen/running2x Z4 | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W500 frozen sections | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W200 running2x sections | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |
| W200 running2x node ablation | 3 | 0 | NOT_MEASURED | NOT_VERIFIED (contended) |

Admission snapshots are retained under `admissions/`, with source paths/hashes in `admission-index.json`; lock holds/waits in `holds.md`. Raw runner evidence: `/Users/rexxa/work/art-production12-evidence/p0-measurements`. Every completed admission returned75 before launching Godot. The first W0 attempt also exceeded load6 (18.205); corrected attempts had loads4.774–5.272 but foreign make-meme Python processes used93.7–100% CPU. Some snapshots also contained Chrome/node over50%. Foreign processes were untouched. No own cargo/Godot ran during these holds.

The later attempts used separate holds but occurred seconds apart when load was already below6; their snapshots still independently failed the foreign-CPU rule. They are not independent performance samples. No additional retries were granted. W0's interrupted pre-admission attempt is conservatively included in its three-attempt ceiling.

There are no collected blocks to pool, flag for population loss, or compare. Starting populations and settlements are independently established in worlds.md; they must not be represented as measured running-block populations. Art Pass11 values remain historical and do not fill this missing baseline.
