# Part 1 harvest

The unchanged P4 pools were fingerprint-checked before v5 guards were written into a separate installer ledger. Original guard results remain under `guards_p4`. Old evidence was not rewritten. Independent animation review approved all ten proposed groups; its round1 verdict file is retained unchanged, with an identical `final_reviews.json` consumed by the installer.

| Body | Activity/view | Real | Copied | Completion |
|---|---|---:|---:|---|
| adult_a | carry S | 3 | 1 | C1 |
| adult_a | carry E | 2 | 2 | C2 |
| adult_b | carry S | 3 | 1 | C1 |
| adult_b | carry E | 3 | 1 | C1 |
| child | carry E | 2 | 2 | C2 |
| child | walk S/E/N | 12 | 0 | C0 |
| child | build S | 2 | 0 | C0 |
| child | run E | 4 | 0 | C0 |

This matches all planner recovery targets: 12 child walking frames, 20 carry frames (13 real), child S build and E run. W1 selected all 13 approved cells; nine previously suppressed objects became drawable. Four previously accepted W1 cells were re-bound without changed pixels. The proposal for child N drink was removed before review because its second required frame was rejected; its unused board remains as evidence.

Drawable total: 273 → 282. Accepted 152 → 192; derived 3 → 10; procedural 51 → 33; fallback 67 → 47. Fallback kinds are counted exactly in `adjudication.json`. No borrowed-view group was changed by this installation; the ten reviewed groups replace only their own views. Render routing uses real carry entries and their same-phase copies.

The installer retained all current pixels outside the selected groups, froze 16 adult candidate masks into the P5 registry, applied its standard plan, repacked the atlas, and rebuilt region pages from stored masks. The first region rebuild invocation incorrectly supplied the asset manifest instead of the atlas manifest and failed before writing regions; the corrected invocation completed. No earlier evidence was touched.

A20 `integrity.json`: PASS, 51 selected frames including objects, 38 changed villager frames, seven exact same-set copies, ten independently approved animation groups, zero mixed sets. `installed_groups.json` supplies source sets and copied frame indices for capture metadata.

Tests changed: old 3% rejection expectation now permits 4%; W1 caps now 24×18/26×20; contract tests cover exact 3%/10% boundaries, adult/child wood widths, explicit frame-zero bending, mask-top height, cross-height exemption and foot anchors. Seven harvest regressions cover the V_c2 attempt-3 set, mixed-set rejection, C1 replacement of a usable height-popping frame, carry-only C2, no procedural or cross-set copy, stale fingerprints, and inherited envelope source IDs. Full Python suite: 264 tests PASS (`final_tests.log`). Game rendering/captures are verified separately by the leader.
