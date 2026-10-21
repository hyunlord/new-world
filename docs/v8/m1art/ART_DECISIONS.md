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
