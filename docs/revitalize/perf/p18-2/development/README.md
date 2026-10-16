# Part 2 development measurements (not the official 7.4 re-judgement)

Three quiet-lock runs of Pass 17's R5/F8 procedure (`perf_p18.sh` stages a and b) plus the attribution runs (`perf_attr_p18.sh`), each on a successive Part 2 build, made while choosing the fixes. They are reported in full and never substituted for the single official run on the final build (`../official/`), which was declared in DECISIONS.md before it ran.

| folder | build (pre-rebase → published hash) | hold (KST) | stage a (R5) | stage b (F8) |
|---|---|---|---|---|
| `1_f11301ee` | f11301ee → 59c92f70 (C1–C4 + exit sub-scopes) | 01:42:44–01:43:36 (shared with Part 1's attribution) | FAIL: mean +16.6 %, p99.9 +49.0 % | PASS |
| `2_a53803af` | a53803af → 7e6b0c31 (+ C5a exchange, C5b trip memo) | 02:08:37–02:09:14 | FAIL: mean +15.03 %, p99.9 +38.0 % | PASS |
| `3_2b6b60bc` | 2b6b60bc → 2b5bdfd3 (+ C6a occupancy, C6b gather memo) | 02:34:14–02:34:52 | FAIL: mean +16.7 %, p99.9 +52.4 % | PASS |

The raw run directories (JSON, tick sidecars, `commands.jsonl`, `provenance.json`) stay in `/Users/rexxa/work/p18-data/perf/`; `stage_*.json` embed the provenance and the per-run file hashes. `quiet.log` is the complete lock log of all Pass 18 holds and waits, including the waits for Art Pass 10's holds.
