# Pass 17 report and gate evidence

The canonical full gate at C `21d2b478a81d55ac138f0a60b830e06719ec0af3`
passed all 15 steps. Its immutable raw output is in [p17_gate](../p17_gate/).
The [receipt](canonical_gate.receipt.json), original sidecar and
[parsed summary](canonical_gate_summary.json) are byte-identical copies
of the external terminal evidence. Absolute paths inside them deliberately
retain the original producing locations. They certify C, not the later
report publication commit R.

[ledger_c](ledger_c/) records all 25 Pass17 commits through C and their
complete file lists. [measurement_tables.md](measurement_tables.md)
preserves the full pre-compression measurement section, including repeated
denominators. The main Korean report contains the metric, hourly-share,
activity, health, tuning and performance tables; the full appendix retains
the repeated denominator detail omitted by that compact presentation.

Assembly, coverage, provenance and root-verification JSON files are
timestamped or hash-bound snapshots from document preparation. Their
report hashes need not equal the final report: later gate/review wording
was inserted. They are not alternative terminal gate receipts.
[gate_helpers](gate_helpers/) contains the source-bound gate wrapper and
formatter checks; synthetic fixtures validate those helpers and never
substitute for an actual WorldSim gate.

R receives a separate complete gate after commit. Its immutable gate,
complete R commit/file ledger, remote-head queries, draft PR update,
authorized cleanup, user-file preservation and actual completion time are
bound by `/Users/rexxa/work/p17-data/final_publication_pass17.complete.json`.
That receipt is created only after these actions succeed; its own SHA
belongs in the later publication appendix, avoiding a self-hash cycle.

Neither technical gate changes the retained A R5, B F1 or tuning F5
failures, and the post-launch F3 correction remains disclosed.
