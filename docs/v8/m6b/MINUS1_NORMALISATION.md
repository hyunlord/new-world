# Stage −1 identity policy

The owner requires BASE `fbc03bddb8f5b68f43fba1b2dad283982e5cf436` behavior with M-LOOP presentation. No fixture is declared changed. Raw evidence is retained outside the repository; normalized comparisons never rewrite it.

`tools/v8/read_summary_normalise.py` is the shared pairwise summary policy. It rejects duplicate JSON keys, nonfinite numbers, missing fields, different scalar types, array-length differences and incomplete colony seed/day coverage. Canonical JSON bytes are compared after this policy; ordinary unknown fields remain significant.

Only these qualified paths are excluded:

- Baseline summary schema 13: `/runs/*/wall_seconds`, `/runs/*/ms_per_agent_tick`, and `/timing/{wall_seconds,ms_per_agent_tick}/{median,p10,p90,min,max}`. All twelve templates must contain finite nonnegative numbers.
- Historical BASE colony schema asymmetry only: `/seeds/*/logs_ground_by_day` and `/seeds/*/cap_target_by_day`. This applies only when the caller binds BASE to the full SHA above, BASE has neither field, candidate has both, seed/day coverage matches, the log vector has one nonnegative integer per day, and every daily cap is null. Shared observer fields are compared, not removed. Same-schema comparisons in Stage 0 retain both vectors.

No recursive name exclusions exist. `why_not`, `diagnosis_cache`, `requests_cache`, arbitrary nested timing names, fingerprints, command results and provenance remain significant. A caller must separately authenticate binary/source provenance; passing a BASE string is not itself such proof.

The standing baseline identity protocol is supplemented by strict structural comparison of all 32 worlds over 120 days: summaries, raw per-seed samples, chronicles and relations. Root source labels are authenticated against each arm's source/build receipt before normalization; no historical zero-only removal exemption is admitted for Stage −1. The legacy comparator's result alone is insufficient. Determinism remains a separate required check.

Required matrix: baseline identity; colony determinism, soak, story, neighbours, knowledge F10, steward and m6a_steward; M-CORE baseline/U/soak; M-READ preservation. F10 deliberately removes the only knowledge holder and is a fault-injection fixture, so equal expected loss is required there. It is not a waiver of ordinary-world knowledge bars.

Tests cover observer asymmetry and shared observer changes, nested names, types, missing/extra fields, duplicate/nonfinite JSON, invalid day/seed coverage, non-null caps, bench ceiling MISS/missing scenes and incomplete stall evidence. Full runtime identity remains pending until exact-commit receipts are recorded in M6B_REPORT.
