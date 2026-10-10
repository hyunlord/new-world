# A17 runtime correction

Initial evidence remains at `/Users/rexxa/work/art-production4-evidence/shots`; an additional untouched copy was made at `/Users/rexxa/work/art-production4-evidence/initial-a17-shots`. Parent recaptures use a separate `round2` root.

The initial audit reported 69 absent woodpile observations, 15 stale stock labels, and four stale deadwood stages. In `a_strip_13`, authoritative wood stock was 34 while the label still showed 30. Hearth animation read current state every render frame, but resource sprites, piles and labels refreshed only every 30 frames. The renderer now refreshes that presentation on simulation-tick changes while retaining the periodic retry for legacy snapshots without a tick.

The fixed west-adjacent pile position was unoccupied, but its sprite footprint extended back into the hearth tile. The existing footprint guard correctly refused it, and the renderer did not try another position. Pass 5 piles now search eight deterministic directions at radius 1, then 2, accepting only placements that pass the unchanged occupancy and footprint guards. The no-wood-column legacy path retains its original west-only placement.

Focused structure regression verifies a nonzero pile despite the west footprint obstruction, tick 1 to 2 label 30 to 40, pile stage 1 to 2, deadwood stage 3 to 0 before the old 30-frame boundary, legal final footprint, and immediate pile removal at zero stock. The structure and nature-clearance tests pass. Forest density remains 302/1190 (25.378%). Windowed round-2 capture validation is still required; no simulation parameters or metric thresholds changed.
