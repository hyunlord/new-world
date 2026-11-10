# B3 art handoff

B3 provides art sources and reviewable integration proposals. The owner briefs
are `/Users/rexxa/work/art-batch/BRIEF.md` and `BRIEF2.md`, with the planner's
2026-11-11 continuation adding round-three animal, tech-icon and smelter fixes.
Raw generations, exact requests, tool returns and verification logs live in
`/Users/rexxa/work/art-batch/B3/`; the Korean final report is `REPORT.md` there.

- `coverage.json`: all55 A2 building families ×6 requested visual states,
  including explicit N/A, rejected and unavailable states. Do not equate
  attempted coverage with adopted sprites.
- `rooms/`:110 layered compositions for22 room/material/direction forms,
 40 A2 variant mappings, reproducible layer exporter and eight regression tests.
- `blueprints.json`: existing ghost display recipe and accepted built-source
  references. A missing source is a gap, never a fabricated ghost sprite.
- `stages_bronze/`, `stages_early/`, `stages_iron/`, `stages_mine_press/`:
  standalone construction/damage production and their lane handoffs.
- `retries_struct/`, `retries_ground/`, `items/`: round-one rejected-state
  retries with a separate B3 cap of three attempts per asset.
- `round3_animals/`, `round3_icons/`, `round3_smelter/`: explicit round-one
  replacement ledgers. Import by replacement, avoiding duplicate runtime IDs.
- `boards/` and lane board directories: native1x and nearest2x review sheets.

Import drafts are fragments, not replacement game configurations. Use each
lane's metadata manifest; the importer does not consume adjacent sidecars by
itself. C2/C3 must resolve real content codes, state selection and independent
layer sorting. No simulation enum, numerical binding code or progress threshold
is assigned here. The repository's runtime source/config trees remain unchanged.
