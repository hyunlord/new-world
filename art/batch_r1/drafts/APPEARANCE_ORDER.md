# R1 appearance/import handoff — draft only

Do not copy these JSON files over the live importer or bindings. BRIEF4 requires
all twelve child/adult × M/F × short/long_tied/bun bodies before a wardrobe can
enter the proposed top axis. `../importability.json` lists body completeness and
portrait readiness separately; `../coverage.json` retains all 84 combinations.
Partial wardrobes are evidence only, even when an individual look passes.

Preserve the shipped age order child/adult, sex order M/F, top order
loincloth/hide_wrap/fur_cloak, and hair order short/long_tied/bun. Existing
indices must not move. Only complete wardrobes listed in art_import.json may be
proposed after those tops. This pass assigns no numeric top codes and changes
no appearance.ron. Missing combinations have no aliases or renamed fallbacks.

The shipped adult male hide_wrap/fur_cloak long_tied slots use bun measurement
fallbacks. New long_tied candidates must actually have tied hair. Those source
fallbacks do not authorize fake hair variants.

Keep FACE_VARIANTS v1…v6 exactly, including older faces, scars, freckles and
beards. Pass 2 generates portraits only after a wardrobe reaches twelve bodies.
Historical partial-wardrobe portraits remain outside the importable additions.
FACE_VARIANTS.additions.json contains only complete-wardrobe identities with
accepted portraits. It never changes the live face plan.

Integration still requires source routing, stable old appearance indices,
variant_transforms/top_colours and era selectors, importer verification,
runtime appearance tests and actual game inspection at 1×/2×. Every body has
ten S/E frames; N reuses S and W mirrors E. Sleep uses112×80 anchor(56,72),
upright80×112 anchor(40,104). These draft rows do not implement runtime selection.

The eight supplemental activity sets retain their adult-male-short scope.
They have no runtime job selector or inspector portrait mapping and do not
complete any wardrobe. Rejected B1 assets are not substituted for missing art.

## Pass 3 complete-wardrobe proposals

Complete body wardrobes: neo_plant, bronze_wool_cloak.
Portrait-ready wardrobes (all 12 looks, all v1-v6): neo_plant.
Historical valid portraits are reused; missing sets are generated only after twelve bodies pass. No live importer or appearance indices changed.
