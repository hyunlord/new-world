# R1 people art handoff

Art branch `v8/art-batch-r1`, based on shipped Route I commit
`f182769fe06b1cb60105e4d0bf127b6524277838`. No runtime import or simulation change.

- 84 wardrobe combinations attempted,244 generated sheets:13 accepted looks,
  130 frames;71 withheld combinations reached three calls each.
- Eight supplemental tool/load sets,18 sheets:8 accepted,80 frames.
- Portraits for all13 accepted bodies:16 generated sheets plus one failed
  argument call,78 delivered portraits. Every logical asset used at most3calls.
- Total279 image calls,278 generated sheets,288 delivered asset PNGs. Contact
  sheets are additional evidence, not included in those asset counts.

Read [STYLE_CONTRACT.md](STYLE_CONTRACT.md) and [TOOLS_SCOPE.md](TOOLS_SCOPE.md).
[coverage.json](coverage.json) lists all84 body combinations and attempt outcomes.
[measurements.csv](measurements.csv) and [measurements.json](measurements.json)
contain every accepted body's raw reference/candidate values and five ratios.
[tools_manifest.json](tools_manifest.json) contains the eight activity comparisons.
All five ratios pass inclusive0.9–1.1; the outline is effective antialiased ink
coverage, not a literal integer stroke count. Other nine poses pass structural
checks and complete visual reviews; acceptance replays normalization from raw.

[Body1×](sheets/accepted_1x.png) · [Body2×](sheets/accepted_2x.png) ·
[Tools1×](sheets/tools_1x.png) · [Tools2×](sheets/tools_2x.png) ·
[Portraits1×](sheets/portraits_1x.png) · [Portraits2×](sheets/portraits_2x.png).

The [draft import](drafts/art_import.json), [bindings](drafts/bindings.json),
[identity additions](drafts/FACE_VARIANTS.additions.json) and
[appearance order note](drafts/APPEARANCE_ORDER.md) are unexecuted proposals.
**No new wardrobe has all12 combinations. Do not append these tops to the live
Cartesian look grid.** Integration needs sparse selection or complete coverage,
stable old indices, colour/variant data, source routing, importer verification
and game-scale inspection. Activity alternatives have no runtime job selector
or inspector portrait mapping; their scope is adult male short hair only.

Raw generations, all rejected comparisons and the Korean report remain outside
the repository: `/Users/rexxa/work/art-batch/R1/REPORT.md`. The full262attempt
measurement index is there, including failed looks. Adoption manifests bind
these external provenance paths; retain that evidence for future replay.

Verification:57 focused tests; Ruff lint/format; basedpyright0errors/0warnings;
all21 body/activity deliveries and all13 portrait deliveries freshly replayed;
all22 frozen region hashes verified. Full Rust/Godot/play gate not run for this
unimported art-only pass. Docs checking retains the five pre-existing artenv1
PNG and four proto UID violations; no new failing category was introduced.
