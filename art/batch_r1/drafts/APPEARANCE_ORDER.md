# R1 appearance/import handoff — draft only

Do not copy these JSON files over the live importer or bindings. They are
explicit file rows and sparse availability proposals, not a runtime-compatible
replacement configuration. `../coverage.json` is authoritative for every one
of the84 planned combinations; withheld rows have no invented alias.

Preserve the shipped age order child/adult, sex order M/F, top order
loincloth/hide_wrap/fur_cloak, and hair order short/long_tied/bun. Existing
look indices must not move. Proposed new tops in production order are
neo_plant, neo_first_wool, bronze_linen, bronze_wool_cloak,
bronze_earth_linen, iron_work, iron_winter. Because coverage is sparse, simply
appending these tops to a Cartesian appearance grid would reference missing
frames and is unsafe. C2/C3 must implement sparse availability or defer the
whole top until all twelve combinations qualify; this pass assigns no new
numeric top codes and changes no appearance.ron.

The original adult male hide_wrap/fur_cloak long_tied slots use shipped bun
fallbacks. New long_tied candidates must actually have tied hair; those source
fallbacks are recorded for comparison and do not authorize new fake variants.

Keep FACE_VARIANTS v1…v6 exactly, including its actual older face slots, scars,
freckles and beards. The portraits are six independent complete128px parchment
cells; no grey-hair or identity transform is applied here. The exact inherited FACE_VARIANTS additions are supplied in
`FACE_VARIANTS.additions.json`. Extend the existing FACE_VARIANTS,
variant_transforms and top_colours data only after import-side review. Each body supplies ten complete S/E frames; N reuses S and W mirrors E.
Sleep alone uses112×80 anchor(56,72); upright80×112 anchor(40,104).

Before integration: route each new look to its batch_r1 source (the current
people importer uses one global frame/portrait source template), preserve old
look indices, validate six portraits and ten frame hashes, run importer verify
and runtime appearance tests, then inspect the game at1×/2×. The explicit
file rows include source hashes but do not by themselves implement runtime
selection. Era and job selectors remain unassigned. Generic axe-work/log-carry
poses do not certify the B1 supplemental spindle/quern/tongs/plough/load sets;
those remain blocked and are not recycled from the rejected B1 art.
