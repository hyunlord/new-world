# Short-budget Part4 launch preparation (no runtime launched)

Original cutoff03:03UTC =1792119780. Intended final readiness02:30–02:40 leaves33–23minutes, not the earlier70-minute planning estimate.

Ready parent command, only after final feature/capture source is frozen and active capture processes terminate:

```sh
python3 /Users/rexxa/work/art-production10-evidence/reporting-draft/final-measurement-driver/supervisor/priority/supervise_matrix.py \
  /Users/rexxa/work/new-world-art-prod \
  /Users/rexxa/work/art-production10-evidence/final-matrix-priority-attempt1 \
  1792119780
```

Output must be new. Existing after.json/after.md aggregate protection remains. The supervisor records source/build/tool hashes including this external supervisor and priority runner. It uses the original atomic deadline gate: before cutoff it may start a world; after cutoff the already-launched Godot world may finish, but no new measurement can start. A waiting owned quiet wrapper is interrupted with its ordinary terminal status retained. Missing worlds remain missing; final aggregate may be incomplete/nonzero and must not be described as complete. Partial actual worlds can still support their individual H rules via the existing comparator; missing rules remain NOT_VERIFIED.

The existing runner has NO subset/order switch. priority_matrix.sh is an exact copy with ONLY two substitutions: repository-root binding (because copy is external), and world order W0,W500,W200,W1,W1000. W1000 stays disabled by supervisor. All per-world conditions, sample lengths, ablation blocks, retries, validation and cooldowns are unchanged. Reverse-transform equality and bash syntax pass. Existing three supervisor fake-only tests pass, including allowing active measurement to finish and leaving a foreign identical-wrapper process untouched. priority-provenance.json pins source and generated copies.

Prioritize matrix FIRST. W0 addressesH1, then W500 addressesH3/H4/H6; W200 addressesH2 ahead of W1 if time permits. Do not put the31-second section capture first: its mandatory600-second cooldown can consume the opportunity to launch W500. Sections can be scheduled only if all intended matrix work completed and a new section measurement can start before cutoff; do not launch parallel or reuse an old renderer's section capture as final.

Four worlds require at least three600-second cooldowns, already30minutes without hold durations/initial cooldown. All four are therefore very unlikely within23–33minutes. Two worlds may be feasible if quiet conditions allow, particularly since a W500 launch before03:03 can finish afterward. This is an estimate, not a promise; current final build may differ substantially from baseline hold durations.

Critical limitation: a NEW quiet wrapper waits up to3600seconds before permitting a contended launch when load>=4 or anotherGodot is present. A02:30–02:40 start under persistent contention may produce ZERO worlds before cutoff. No change here bypasses quiet thresholds, cooldown or the waiting requirement. Preserve zero/partial evidence honestly rather than relaxing those conditions. Final gate/report/publication remain separate leader-owned work.
