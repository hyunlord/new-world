# W1 independent visual review

Reviewed 2026-11-11. Read-only review of `roofs_and_doors_1x.png`, `walls_diagnostic_1x.png`, existing `art/style05/struct/knapping_spot.png`, `measurements.json`, export metadata, and BRIEF_W1.md. Decisions below select individual art candidates; they do not certify a complete W1 set or runtime adoption.

## Individual selections

| Candidate | Decision | Visual evidence and limits |
| --- | --- | --- |
| `hide_built` | SELECT as a roof art candidate | Radial overlapping skins, seams and restrained hatching read as a hide covering. Warm earth wash, irregular dark contours and the overhead surface vocabulary are compatible with the existing field-guide art. The silhouette is a roof, without an invented full hut beneath it. Numeric checks pass. The 225×219 export has opaque bounds (4,4)–(221,215), and is labelled 3×3 with anchor (48,103). Those dimensions and labels do not independently prove coverage of the actual wall ring: elevation, eave overhang and anchor placement still need a runtime composite. |
| `thatch_built` | SELECT as a roof art candidate | Four slopes and the central ridge read immediately; bundles and straw marks give material texture without a modern roof material. It is more saturated than the knapping reference, but golden straw plausibly accounts for that difference and contours remain compatible. Numeric checks pass. The 289×286 export has opaque bounds (4,4)–(285,282), labelled 4×4 with anchor (48,103). Its nearly square roof plane is acceptable as surface art, but cannot establish the intended projected footprint or concealment of the 4×4 walls without a composite. |
| `door_ew_closed` | SELECT as an EW door art candidate | Split timber, cord lashings and an uncomplicated lintel fit the requested period. The closed slab and supports remain legible at native size, with an apparent ground contact line and no metal hardware. Numeric and recorded projection checks pass. This does not prove a seamless join to the rejected wall modules. |
| `door_ew_open` | SELECT as an EW door art candidate | The swung slab and transparent aperture communicate the open state clearly at native size. Material, ink and proportions agree sufficiently with the closed candidate. Numeric and recorded projection checks pass. Common frame registration must be checked before using the two states as an animated pair: independent normalization produces canvases 63×68 and 69×68, anchors (31,63) and (34,63), and slightly different scales. A centred alpha bounding box is not proof of a stationary jamb. |

The selections above are suitable to retain for integration and further review. They are not evidence that the complete generation/evaluation gate passed: all recorded `judge_v2` entries have missing model status, null model scores and `passed: false`, including the numeric-pass candidates.

## Frozen rejections

- **Hide construction frame: REJECT.** The visible ring, radial poles and lashings are understandable, but the third attempt fails the frozen wash-flatness check (0.02901058). Visual readability does not waive that failure.
- **Thatch construction frame: REJECT.** The hips, rafters and ridge communicate a construction stage, but the third attempt fails wash flatness (0.03325998). Do not adopt it as a completed frame deliverable.
- **Log-wall set: REJECT.** The diagnostic assembly visibly changes projection between raised EW courses and long flat-looking NS strips. Corners leave gaps or uneven overlaps, and T junction arms do not share a coherent height/edge treatment. The atlas demonstrates disconnected modules rather than a continuous interlocking wall. Existing renderer imagery beside it does not cure those candidate defects. The capped failure remains a failure; do not relabel the set game-ready.

## Remaining risks and required proof

1. Roof coverage is declared in metadata, not demonstrated against the actual 3×3 and 4×4 hut wall footprints. Show each selected roof on its correct ring at 1×, including eaves, ground anchor and occlusion/fade behavior. A roof-only contact sheet cannot prove placement.
2. Check the EW open/closed states using one fixed frame coordinate in a joined wall. The board does not demonstrate stable jamb registration or clean wall joins.
3. The required coherent wall family, usable construction frames, NS doors and damaged straights are not established by these four selections. W1 cannot be reported as the complete requested set on this evidence.
4. The reference knapping spot supports a general ink-and-earth-wash style comparison; it does not establish all wall topology, roof elevation or runtime scale requirements.

No additional generation attempt or frozen-check exception is recommended by this review.
