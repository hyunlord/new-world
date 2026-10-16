# Part 5: room to stand

Implemented section 9's expanded lattice (16 px columns, 18 px rows, 104 × 72 ellipse), 16 px separation, six-tile eligibility and 64 px reach. Existing assignment persistence, ascending IDs, nearest-seat tie rules and same-tile spread remain. `crossed` is eligible as required by section 10.2 so cold seated villagers retain their seats.

`CampSeats.declump` processes visible sprites by descending target feet y and ID. It measures canvas overlap against already-finalized front targets, tries alternating horizontal offsets up to eight pixels, queries every proposed feet tile against the whole terrain snapshot, and excludes blocked world objects. Moving and seated sprites occlude but never shift. Equal-distance nearest fronts retain the deterministic front-to-back ordering. If no shift reaches the threshold, the first allowed minimum-coverage shift wins; all blocked yields zero.

The renderer builds rectangles from manifest-derived `frame_size` and `anchor` metadata, including flips. There is no texture readback in the declump path. Targets include seats and spread before declumping and retain the 0.08 s smoothing. Sprite metadata exposes `drawn_target`, `spread_offset` and `declump_offset` for capture QA.

## Verification

All four commands exited 0 with Godot 4.6, executed through `bash tools/art_pipeline/run_bounded.sh 120` and `--path . --headless --script`:

| Script under `scripts/test/art/` | Evidence | Result |
| --- | --- | --- |
| `test_camp_seats.gd` | `test_camp_seats.log` | PASS: 41 distinct seats, separation, persistence, six-tile boundary, cold pose, reach, deterministic row ordering |
| `test_declump.gd` | `test_declump.log` | PASS: adjacent tiles below 0.30, global water/obstacles, all-blocked and minimum-coverage fallbacks, moving/seated/hidden exclusion, final-front ordering, renderer integration without textures, metadata, unchanged simulation positions |
| `test_agent_drawn_positions.gd` | `test_agent_drawn_positions.log` | 0 failures: actual renderer smoothing and seating |
| `test_agent_click_drawn.gd` | `test_agent_click_drawn.log` | 0 failures: live scene drawn-position click integration |

`git diff --check` passed for the changed spacing sources. These are headless runtime results, not visual acceptance. The click test log also records an image-loading export warning from the separately-owned cold-mask audit; it does not fail that test. Parent owns the full gate and windowed capture/visual assessment.

## Source ownership

- `scripts/ui/camp_seats.gd`
- `scripts/ui/agent_renderer.gd`: `_apply_drawn_positions` only
- `scripts/test/art/test_camp_seats.gd`
- `scripts/test/art/test_declump.gd`

No new localization text, simulation changes, old evidence edits, commits or pushes were made in this lane.
