# Part 4 frozen-producer runtime evidence

Producer: `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`; release hashes and build receipts are under `build/`. WIP was remotely verified at this producer before execution. Raw outputs remain in `/Users/rexxa/work/p17-data/part4-smoke-r1/`.

- v21 and v22, seeds 42–43, three days, trace on/off: deterministic twins pass and each paired summary has zero differences except timing. v21 writes no family traces.
- v22 seed 42, 60 days: 76 lifetime people (31 female, 45 male), 23 pair events, 57 births (50 real couples, seven fallbacks), one completed fission and 1,440 hourly samples. Exact day 60 is tick 86399 / elapsed 86400. All observation misses are zero; 89 original raw file hashes verified.
- Independent routine reconstruction: short worlds 53 people / 43 ever-adults / 105 completions; day60 76 people / 72 ever-adults / 5,651 completions. Zero discrepancies.
- Independent family reconstruction: 13,672 and 16,662 checks for short seeds; 597,844 for day60. Zero discrepancies. Human and JSON outputs retain input hashes, daily births, nightly denominators and fission joins.
- Actual evaluator integration: all eight checks pass, including exact eligibility, first snapshots and eight both-orientation Pearson correlations (maximum difference 1.39e-17), finite JSON and Markdown serialization. These small-seed diagnostics assign no Stage B acceptance verdict or tuning decision; any serializer placeholders were discarded.

Hand family: female 29 and male 30 pair at tick 23760; child 50 is born at 26821 with those real parents. At the first post-birth 20:00 sample (28080), all three share bed `[29,26]` and household 29. By fission departure 84840, child 50 is an adult paired with 49; both depart together and parents 29/30 remain. Fission 1 completes at 85009, with one split among 13 parent-settlement couples. This is not a claim that adult multi-generation descendants remain one household.

Available observations cannot establish exact death/unpair timing between hourly samples or reconstruct each shelter's capacity from aggregate capacity alone. Focused runtime tests cover those mechanics separately. No art-rendering acceptance is inferred.
