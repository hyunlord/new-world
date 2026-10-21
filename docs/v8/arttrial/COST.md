# ART-TRIAL generation cost ledger

FROZEN producer-complete evidence snapshot: **2026-10-21T14:39:54.982547+00:00**. Monetary cost: **NOT_REPORTED_BY_TOOL**.

Observed trial wall-clock span: **8296.983s**, from known trial start **2026-10-21T12:21:38+00:00** to this ledger-freeze checkpoint. This includes gaps and overlapping work; it is separate from call interval unions and excludes later publication minutes.

Known deduplicated call records: **834**; returned source images: **825**; known tool errors: **9**; currently in flight: **0**. Separately, **7 launched slots** in interrupted batches have unresolved individual outcomes; at least one error per batch is known. These are not silently counted as successful outputs or double-counted retries.

Baseline producer-selected source images: **272**; selected asset frames: **331** (strict ACCEPTED: **235** frames from **235** sources). Baseline integrated registry contains **367** assets; final packaged registry contains **383** assets, including 16 versioned scene overrides. These counts include generated frames, derived assets and legacy fallbacks. Selection is internal, not blind acceptance. Final selected generated sources: **285** = 272 baseline + **13** distinct versioned revision originals; the other three overrides are derived shadows and add zero generations. All 13 revision original SHA-256 values are distinct from one another and the baseline set.

| Family | Calls | Returned images | Accepted sources¹ | Baseline selected sources / frames | Revision / final selected sources | Repeated attempts² | Caller sum / overlap elapsed (s)³ | Measured / bounded / unknown calls | Unresolved batch slots |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| environment | 65 | 65 | 0 | 24 / 51 | 3 / 27 | 30 | 4112.977 / 946.494 | 63 / 2 / 0 | 0 |
| body | 11 | 11 | 0 | 4 / 36 | 3 / 7 | 4 | 367.964 / 289.278 | 11 / 0 / 0 | 0 |
| hair | 290 | 286 | 54 | 54 / 54 | 3 / 57 | 29 | 22193.551 / 3107.092 | 290 / 0 / 0 | 4 |
| top | 194 | 193 | 144 | 144 / 144 | 3 / 147 | 47 | 120695.146 / 3448.681 | 193 / 0 / 1 | 0 |
| tool | 245 | 241 | 37 | 37 / 37 | 0 / 37 | 63 | 38359.696 / 3382.491 | 244 / 0 / 1 | 0 |
| carry | 29 | 29 | 0 | 9 / 9 | 1 / 10 | 19 | 784.402 / 576.521 | 28 / 0 / 1 | 3 |

¹ Accepted sources require an explicit ACCEPTED frame record. Selected sources additionally include registered environment assets, provisional body choices and accepted carry records whose status label remains MECHANICAL_PASS. The revision column separately counts parent-reviewed scene revision selections, retaining their original status label; it does not silently relabel them ACCEPTED. These are internal selections, not blind acceptance.

² Repeated attempts means additional deduplicated calls for the same lane and normalized candidate name after removing numeric candidate/retry/board/correct suffixes. This includes mandatory alternatives; failure-driven retry counts are UNKNOWN. Renamed targets and interrupted batch slots cannot be reliably grouped and are excluded. This is an observed naming-based count, not an inferred failure total.

³ Caller sums and overlap elapsed cover measured call brackets only. Overlap elapsed is the union of active invocation intervals, excluding gaps; it is not first-to-last campaign wall time. Unknown durations contribute no numeric value and are not zero. Initial bounded calls remain separate. Family overlap unions can overlap one another and must not be summed.

Measured caller-bracket sum: **186513.736s**, overlap union: **6106.624s** (829 calls). Including 2 separately bounded initial calls, union is **≤6227.946s**. Unknown-duration records: **3**, plus unresolved batches. These are subset timings, not a bound on the complete trial.

Caller brackets include queue/orchestration and concurrent calls overlap. File timestamps are never substituted for invocation times. Materials, palette reduction, extracted frames, banks and shadows add zero generations. Same-original archives are deduplicated by generated source identity; raw/original SHA-256 equality is checked where the producer has copied the raw image. Returned originals awaiting archival are explicit audit gaps.

Audit gaps: **3**; duplicate archive aliases: **2**. Current accepted registries take precedence over mechanical-only producer selections. Unresolved selection lineage remains explicit in the machine audit.

Detailed evidence: `/Users/rexxa/work/arttrial-data/generations.json` and `generation_audit.json`. Refresh using `python3 /Users/rexxa/work/arttrial-data/reconcile_costs.py`; producers are read-only.
