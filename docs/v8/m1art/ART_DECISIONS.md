# M1-ART decisions

Decisions taken without asking the owner (goal §0). One line of context, the decision, the reason.

- **D1 (Part 0) — "asleep" and "facing" need no Rust change.** The agent snapshot already carries `states`
  (state tag: 3 = `Consuming` a non-agent target) and `target_kinds` (3 = `Sleep`), so "asleep" is
  `states == 3 and target_kinds == 3`; "facing west" is the sign of the last drawn step, kept by the agent layer.
  The `sleeps` column is the sleep *need* (fatigue), not a state, so it is not used for the pose. Goal 3.3's
  optional read columns are therefore not added; `rust/` stays untouched.
- **D2 (Part 0) — before shots.** `tools/v8/capture.gd --m1-set` at `a4a7eaf4` (code identical to `bba3110e`),
  seed 20261010, saved to `/Users/rexxa/work/m1art-data/before/` (not committed): work tick 481, hut finished at
  766, P4 at 826, P5 at 900.
- **D3 (Part 1) — bank tiles are a superset of goal 5.1's eight.** A water tile picks its bank from the 4-bit
  mask of land neighbours (N=1 E=2 S=4 W=8); a river one or two tiles wide has masks with two or three sides
  (opposite banks), which eight tiles cannot draw. The generator therefore draws all fifteen side masks
  (`bank_1..15`, a sand lip on each land side) plus the four inner corners (`bank_in_*`, land only diagonally).
  The four single sides and four inner corners the goal names are among them.
- **D4 (Part 1) — stockpile floor as composable parts.** Instead of three oriented tiles (corner/edge/interior),
  the sheet has one opaque floor (`stock_floor`), four 1-px border strips (`stock_edge_n/e/s/w`) and four corner
  stakes (`stock_stake_*`); the overlay draws the floor, then a strip on each side without a zone neighbour and a
  stake where two such sides meet. This draws any zone shape (L shapes, single rows) with nine small sprites and
  no rotation.
- **D5 (Part 1) — the carried bundle is split around the head.** One bundle sprite drawn over the person either
  sat on the head (M1) or covered the whole tunic (first M1-ART draft, preview). `carry_back` (two logs) is drawn
  before the person layers, so the head covers it and the logs show on both sides of it; `carry_front` (one log)
  crosses the chest after them. The tunic's lower rows stay visible.
- **D6 (Part 1) — palettes live in `tiles.json`.** Tunic, hair, elder-hair, skin and halo colours moved from
  `view.gd` constants to the sheet's `palettes` entry, so the unit test can measure the drawn tunic against the
  forest floor on the same data the game uses (goal 6.4).
- **D7 (Part 1) — tooling bookkeeping.** `tools/v8/inventory.py`'s M1 rule now names `gen_tiles.py` /
  `test_gen_tiles.py`, and the generated table in `docs/v8/V8_0_INVENTORY.md` was rewritten (it also picked up the
  V8-PLAY files it had not listed yet). Outside goal §0's path list, but needed for gate step 15 to describe the tree.
- **D8 (Part 2) — the tree layer keeps its scene node.** `scenes/main.tscn` is outside the pass's paths, so
  `ColonyView/TreeLayer` stays a `TileMapLayer` node; its cells stay empty and the canopy is drawn by child `Node2D`
  chunks (16×16 tiles, created in row order so a southern chunk draws over a northern one). A tile map cannot
  draw a 24×24 crown that overlaps its neighbours, and per-chunk `_draw` lists keep a felled tree's redraw local.
- **D9 (Part 2) — 1× zoom drops the shadows, not the crowns.** Goal 6.2 asked for 16×16 impostors at 1×. Drawn,
  they turned the forest back into rows of outlined dots (iteration 1, `m1_06_far_1x.png`) — the problem this pass
  exists to fix. At 1× the layer redraws every chunk once without the tree shadows (half the quads); the crowns
  keep their size and jitter. The impostor sprites were removed from the sheet. FPS is measured in Part 4 (8.5).
- **D10 (Part 2) — stumps are drawn by the tree layer.** The tree layer owns the "recently felled" set (version
  diffs, goal 5.2), so its `Nature` child (z 1) draws the stumps with the bushes and deadwood, under the overlay's
  log piles (same z, later in the tree) and under the canopy.
- **D11 (Part 2) — the overlay draws in two passes.** A child `Ground` node (z 1) draws stockpile floors, log
  piles and blueprint ghosts with their delivered piles (south row, 4 logs per tile: s, s, m, l), so canopies
  and people cover them; the node itself (z 4) keeps the chop marks, the blueprint label and bar, and the hovered
  stack's count above the canopy. The blueprint's 3-px bar is one bar: amber with logs delivered until all 12
  are there, then green with the work done.
- **D12 (Part 2) — sleepers sharing a tile are spread for drawing.** The simulation puts every sleeper of a
  building on its tile (P5 of iteration 3: four people on one shelter tile, three on the hut's centre), so only
  one figure showed. The agent layer lays sleepers in a hut on its four bedrolls (agent id order) and fans
  others along y; the drawn rect is still what clicks hit. The hut's bedrolls were turned east-west to match the
  goal's sleep pose (head left).
- **D13 (Part 2) — halo rules.** The halo is shown under a crown (`under_canopy`: a tree on the tile or a
  neighbour) and at night away from a fire; never on a sleeper, and not on people the fire already lights (the
  warm modulate turned the halo into a bright orange box in iteration 1).
- **D14 (Part 2) — `daylight_controller.gd`.** The hearth glow sat half a tile down-right of the hearth
  (`SPRITE_ORIGIN` is already the tile centre, then `+8` again); fixed. The night curve became static
  (`tint_at`, `night_factor_at`) so the agent layer warms people with the same curve.
- **D15 (Part 2/4) — `tools/v8/capture.gd`.** The `--m1-set` record now dumps, per shot, the screen position of
  every person (with stage, job, carry, asleep and the agent layer's frame/halo), buildings, stacks, zones, sites,
  chop marks and stumps — the data goal 8.3 compares against — and takes one more shot, `m1_10_morning_2x.png`
  (the next 10:00 at P4's framing), because the hut is finished at 20:46 and every other hut shot is at night.
- **D16 (Part 1) — a hut-floor bug caught in preview.** `_hut_walls` cleared the interior after the floor was
  painted, so the hut showed the ground through its floor (iteration 3). Fixed; `test_hut_floor_is_opaque` fails
  on the old code (checked by reintroducing the clear).
- **D17 (Part 4) — FPS measured uncapped, A/B in one session.** M1 measured 120.00 / 119.92 FPS, the display's
  refresh cap, which cannot show a 0.9× regression. `tools/v8/m1_gui_test.gd --no-vsync` turns vsync and the frame
  cap off for its FPS phase; the base (`bba3110e`, a detached worktree with the same test script) and the pass
  checkout loaded the same release bridge library from the debug path (SHA `85c47662`; the debug library
  `14fab8d5` restored after), base and after interleaved.
- **D18 (Part 4) — FPS round 2 is void.** In the second round the base run took about two hours for a six-minute
  job and measured 112 / 93 FPS, and the after run hung twice on a frame wait (a known macOS stall when the window
  gets no frames; `worldsim-godot` lesson 2026-10-11). The window was evidently not being drawn normally; round 2 is
  reported with its numbers and not used. Round 1 (clean, interleaved) is the measurement.
- **D19 (Part 4) — one revision round after blind read 1 (6/10).** Child redrawn with child proportions (8×10, big
  head), the sleep frame as a head and a blanket (no belt, no eye pixels), the blueprint ghost with hatched walls,
  a doorway and faint bedroll/hearth plan lines; the blueprint crop was re-centred (the site's dumped tile is its
  top-left) and the question asks for the main thing the crop is centred on. Round 2 uses a new subagent and the
  final capture; find-the-people is rerun on the same final capture.
