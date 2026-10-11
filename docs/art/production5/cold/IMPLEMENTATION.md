# Part 6 cold presentation and section 5.8 carry playback

- `scripts/ui/cold_effects.gd` feature-detects the typed optional cold column, resolves eligible poses, adds deterministic one-pixel shiver, and creates runtime 3×2 white breath sprites. Agents expose `cold`, requested `pose`, actual `cold_pose`, `shiver_offset`, and `puff_alpha`.
- `scripts/ui/cold_audit.json` freezes RGBA hashes from the existing reviewed p4 registry only for natural-audit `wrong_pose`/`wrong_view`. Frame grouping hashes the lossless imported atlas crop once; changed RGBA is eligible again. Imported `CompressedTexture2D` source images support export; no render-target or ImageTexture GPU readback is used.
- Cold fallback order is crossed/current view, crossed/S, sit/current view, idle/current view. Only cold choices exclude unchanged rejected art. Meal/asleep/talk keep their pose. Spacing lane enables crossed seats.
- Shiver is applied after smoothing and never fed into the smoothing accumulator; drawn-position access and selection use actual sprite position. A sixty-frame regression checks drift and picking.
- `building_renderer.gd` renders separate food and wood labels. The wood child uses `stockpile_wood_label`, exposes `wood_stock` and `wood_target`, and alone receives the shortage tint. Targets select the closest settlement row and are never summed.
- HUD and inspector read typed cold markers. Fluent keys were added once per language; `tools/localization_compile.py --strict-duplicates` updated compiled outputs and key registry. 5,184 active keys; zero duplicates or missing keys in both languages.
- `_real_carry` accepts same-cycle copies; a body still needs an accepted carry frame to activate real carry borrowing. C1 and C2 four-slot looping tests cover all three bodies.
- `ingame_qa_wood_p4.py` now parses split wood labels and validates target/tint when present while retaining old capture support. Historical evidence is unchanged.

## Verification

Seven headless Godot scene tests passed with no script errors: cold presentation, HUD wood, inspector cold, inspector wood, agent activity, drawn positions, drawn clicking. Python wood QA: 12 tests passed. GDScript strict parse/warnings and FFI checks passed. `git diff --check` passed. Logs are in this directory.

No Rust static-source assertions were changed in this lane; no mutation audit is required for this lane. Full final gate, windowed captures, A19 observed breath cycles and visual acceptance are owned by the parent execution lane and are not claimed by these headless tests.
