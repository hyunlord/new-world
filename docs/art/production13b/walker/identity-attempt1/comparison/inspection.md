# Walker identity comparison attempt 1

Compound verdict: FAIL (exit 1). Supplemental walker polarity and provenance checks have no errors. All five running worlds pass exact JSONL identity. Six-framing A/B RGBA identity is exact. A/Aprime control identity fails only for P3: 924 differing pixels, maximum channel difference 149. Full-resolution difference bounding box: (1676, 58, 1706, 90).

The inherited comparator reports retain scope `reference_integrity_only`; the supplemental report scope is `compound_walker_candidate_identity`. A failed control prevents candidate identity acceptance even when A/B is exact. No capture rerun, source change, or HEAD change was performed. Root cause is not established by this comparator.

Artifacts: `walker-identity.json`, `running-reference.json`, `rgba-reference/comparison.json`, `rgba-reference/P3-control-diff.png`, `rgba-reference/previews/P3-control-diff-x16.png`; sibling `comparison-attempt1.log` and `comparison-attempt1.exit`.
