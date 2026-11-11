# B2 grounds handoff

Ground masters are held by the frozen round-1 judge. Three built-in imagegen attempts per required base are the hard cap; numeric failure is never waived. See external `B2/grounds/diagnostics.json` and `review_final.json` for the final ledger. No failed master may enter the export selection.

`process.py` checks original alpha, exact raw/source hashes, reference hashes, frozen style calibration, original-art tone cap and tile seams. Global tone adjustment uses existing `cap_tone`; no local repaint or seam repair was performed. `export.py` requires a nonempty reviewed selection with all gates passing and original opaque alpha, then uses existing seasonal operations and EdgeField. Missing inputs remain explicit gaps. Spring shares summer, autumn uses DROUGHT, winter existing chroma35%; these are derived, not new generated seasonal drawings.

`fallback.py` creates evidence outside the repository under `B2/grounds/existing_recipe_demo`: original meadow/forest_floor/hill_rock/river_bank seasons and deterministic band edge demonstrations, at true 1x and nearest 2x. These are existing-art demonstrations, not tundra/steppe/scrub/marsh replacements. No biome aliases are applied. Seed5005 and blend12 reuse the existing EdgeField implementation.

Validation: ground boundary regression tests plus existing edge and ground tone suites: 18 passed; Ruff check passed. Full game import/play was not run: there are no accepted new ground exports to integrate. Import drafts stay unapplied and contain no rejected candidates.

Reproduce: `uv run --with pillow --with numpy --with pydantic --with scipy --with scikit-image python -m art.climate.grounds.process`; same environment for `art.climate.grounds.fallback` and `art.climate.grounds.export`. Empty selection intentionally raises AssertionError before export.

Final closure: 24 preserved attempts (3 per base), 0 adopted new masters, 8 capped gaps and 32 blocked seasonal frames. All final attempts are numerically rejected; see external review_final.json. Weakest tundra self-review is1/8 (12.5%), with actual final regeneration. No fourth attempts.

Exact test command: `uv run --with pytest --with pillow --with numpy --with pydantic --with scipy --with scikit-image python -m pytest art/climate/grounds/test_process.py art/compositor/test_edges.py art/pipeline2/test_ground_v2.py -q` =>18 passed0.84s. Root separately ran a different17-test suite; these counts are not interchangeable.
