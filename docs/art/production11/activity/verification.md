# Activity verification — final P2 evidence

H7: **NOT_VERIFIED**. C has only12/14 eligible crops and diagnostic recognition fails in both rounds. D improves from2/6 FAIL to5/6 PASS in round2; this does not complete C or pass H7. No further rounds.

## Captures and pose preservation

Six prescribed base captures completed in attempt3; five valid Z4 supplements and four fixed Z1 extensions give15 valid captures,503 holder checks,0 pose mismatches. All15 sidecars have complete schema2 world-sprite draw ordering. Screenshot runtime used the explicit debug bridge, while populous seed20261018 was selected by the separate release game_v24 cohort. Screenshot elapsed times are not performance measurements.

| Capture | Seed/day/hour/zoom | Holders checked | Pose mismatches |
|---|---|---:|---:|
| s20261018_d60_1000_z1 | 20261018/60/10/1 | 35 | 0 |
| s20261010_d30_1500_z4 | 20261010/30/15/4 | 18 | 0 |
| s20261010_d30_1000_z1 | 20261010/30/10/1 | 6 | 0 |
| s20261018_d60_1000_z4 | 20261018/60/10/4 | 35 | 0 |
| s20261010_d30_1500_z1 | 20261010/30/15/1 | 18 | 0 |
| s20261010_d30_1000_z4 | 20261010/30/10/4 | 6 | 0 |
| activity-extra-d30-1400-k1 | 20261010/30/14/4 | 12 | 0 |
| activity-extra-d60-1400-k2 | 20261018/60/14/4 | 69 | 0 |
| activity-extra-d60-1400-k4 | 20261018/60/14/4 | 69 | 0 |
| activity-extra-d60-1100-k0 | 20261018/60/11/4 | 37 | 0 |
| activity-extra-d60-1400-k0 | 20261018/60/14/4 | 69 | 0 |
| s20261018_d60_1100_z1 | 20261018/60/11/1 | 37 | 0 |
| s20261010_d30_1100_z1 | 20261010/30/11/1 | 11 | 0 |
| s20261018_d60_1400_z1 | 20261018/60/14/1 | 69 | 0 |
| s20261010_d30_1400_z1 | 20261010/30/14/1 | 12 | 0 |

The exact final oracle output is activity-all-valid-final-pose.json inside raw-evidence.zip. Source paths, SHA256, rejected inputs, launch identity and logs remain in the archive manifest.

## C: full protocol incomplete; diagnostic failure retained

Fixed base order and authorized supplements yield counts `[1,2,2,2,1,2,2]`:12 unique eligible holders, missing one TendFire and one Explore. All129 final exclusions were recorded before crop extraction. The original requirement14, >=10/14 and no kind0/2 remains unchanged. Full C verdict **NOT_VERIFIED**; diagnostic results are round1 **0/12**, round2 **1/12**, never scaled to14. Multiple fully sampled kinds remain0/2. No excluded or duplicate padding, crop enlargement, pose modification or threshold relaxation.

A leader-authorized terminal shortage record and source hashes preceded extraction. Actual no-arrived attempts are retained separately from skipped centers whose arrived holders were all >50% rectangle-covered. Original fixed base cameras were not reframed. The original Art Pass10 body-centered crop geometry remains; distinguishing icons mostly lie outside that geometry, a limitation recorded before crop creation.

## D: completed fixed evidence extension

The original three Z1 frames provide only3 eligible village/capture pairs; d30_15 top-two tie remains excluded. After round1 C answers froze, all four supplemental instants were preregistered and actually captured in fixed order d30_11,d60_11,d30_14,d60_14, using the standard overview camera. This documented extension selects no favorable instant. Six pairs were then selected deterministically, max2 per(seed,village), no top-two ties, with the >=5/6 bar intact. Round1 **2/6 FAIL**; round2 **5/6 PASS**. Both outcomes remain in the record.

Each round's answers were frozen before that round's score/key reveal; exact hashes and per-item scoring are retained. See blind-score.md for both rounds and per-kind denominators. Neither D's second-round pass nor the pose oracle implies C visual recognition passes.

## Rejected evidence and limits

Attempt1 lacked required renderer oracle inputs and is rejected. Attempt2 omitted world/descendant blockers; its five frames and stopped launch remain rejected. Neither supplied blind crops. Failed supplemental launches remain raw failures/no-arrived evidence, never valid captures. All actual attempts, selection histories, terminal/protocol records, answers and scores are archived.

Coverage uses the union of higher-rank world Sprite2D rectangles, not alpha-pixel masks. UI overlap outside WorldObjects is not established by that rule. The archive preserves the source/build and capture-specific limitations rather than converting static or pose checks into visual acceptance.
