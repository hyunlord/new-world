# Hair-only region integration

Runtime region pages now contain only R=3/G=0..2 for hair and R=G=B=0 elsewhere, preserving source alpha. `regions.classify` remains a historical full-body diagnostic API; `build_atlases` never calls it or `regions_v2.classify`. The historical `tables_path` argument remains accepted for caller compatibility but cannot influence runtime output.

Frozen P4 masks are matched by body, activity and exact source RGBA SHA256. Activity distinguishes identical source pixels reused as asleep fallbacks. The loader checks file SHA256, declared dimensions, hair-only encoding, duplicate consistency and source alpha/dimensions. Rejected masks cannot be reused; pending masks are available for the explicitly required audit stage. Unmatched source pixels use `hair_maps.classify`; no mask is reused merely because the frame ID is unchanged. Non-asleep/non-lying adults require at least one hair texel. Children, asleep and wider-than-tall lying frames have empty masks.

`install_v4.apply_plan` performs the same hair validation before its first asset write. All existing frozen selection/evidence, lineage, physical group, UV/geometry, alpha, palette, color-fidelity and animation safeguards remain. New algorithm masks are deterministic; Part 2 must record accepted candidate mask provenance in the P4 registry when freezing its selected frames.

## Retired assertions

- Runtime v2 table classification at arbitrary lower rows was replaced by frozen P4 mask preservation and an exact-RGBA change test. V2 classification remains tested separately as a historical diagnostic.
- Runtime full-body class 5 output in `test_regions_v4.py` was replaced by the stronger hair-only allowed-code assertion plus unchanged source alpha. Its exact UV and explicit-only diagnostic-output checks remain.
- Runtime metadata fields `v2_rgba_sha256` and full-body comparison diagnostics were replaced by exact frozen mask path/SHA evidence. Tampered file, incompatible alpha, conflicting duplicate and declared-size guards remain; invalid full-body codes and rejected masks now fail explicitly.
- No installer guard was retired: inspection found no full-body-region requirement in `install_v4.py`. The hair requirement is additive.

## Verification

Red before implementation: 8 updated runtime-map tests failed against the old full-body registry reader. The installer missing-adult-mask test also failed before its pre-write validation was added.

After implementation: 23 `test_regions*.py`, 15 `test_hair*.py` and 16 `test_install*.py` tests passed. Python compilation succeeded. The authorized production region build processed 169 frames on one page; `world_0_regions.png` contained R values only 0/3 and G values only 0/1/2, with zero non-hair G values.

No color atlas repack, Git action, Rust change, window launch or simulation edit was performed by this lane. The existing 354-line installer received only an import and a narrow pre-write check; no unrelated refactor was introduced.
