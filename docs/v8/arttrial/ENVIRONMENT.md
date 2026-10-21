# Environment candidate selection

The environment registry currently contains **51 assets**. The real registry CLI checked all 51 and produced a 21,520-byte sheet. This is mechanical evidence for this submitted subset, not completion of the people or blind-read gates.

## Produced assets

- Buildings: hut, derived blueprint ghost, lean-to.
- Trees: oak, birch and pine in 48px and 64px boxes; derived canopy shadow.
- Work/resources: three log-pile density levels, stump, deadwood, two hearth frames, full/bare berry bushes, boulder, three stockpile-floor pieces.
- Ground: 20 variants across eight materials.
- River: four bank sides and four inner corners, derived from the selected shallow-water tile and a two-pixel sand lip.

Every generated environment subject received at least two candidates. The selected source files, rejected alternatives, prompts, timings, raw hashes and material-palette operations remain under `/Users/rexxa/work/arttrial-data/` in `buildings/`, `trees/`, `materials/`, `wood/`, `props/` and `ground/`. `banks/selection.json` identifies the deterministic bank sources. The consolidated cost ledger will include all attempts, including unsuccessful edits.

## Material processing

Global master-palette mapping alone retained too many values in some materials. A second, declared material-subpalette step reduces those pixels to at most four existing master colours. It does not paint shapes: original alpha and outline pixels are compared byte-for-byte. Material memberships and spatial regions, retained colours, changed-pixel counts and Lab distances are recorded with before/after SHA-256 hashes.

Hut wood uses four explicitly reviewed colours to preserve pale log cut ends. Frequency-only and luminance-only alternatives flattened those ends and were retained as rejected processing candidates. Small one/two-tone trunks are not given invented shading simply to reach a minimum colour count. The four-value cap is enforced; full three/four-value shading is not claimed for those tiny regions.

## Ground evidence

Sixteen texture generations were tested against the preregistered seam thresholds and inspected as 4×4 repeats. Generated grass (4), dark grass (2), deep water (2) and mountain (2) supply ten final tiles. Dirt (3), forest floor (3), shallow water (2) and hill (2) use the explicitly allowed legacy fallback after two seam failures per material. Fallback is nearest-neighbour 16→32, followed by master-palette mapping.

Mountain was not granted a style-failure fallback: material reduction was applied and its seam tests were run again. Both reduced candidates passed; the quieter first candidate was selected. The two fallback shallow-water variants become identical after palette mapping. Mixed-neighbour seams between different variants remain unverified; same-tile repeat results do not prove them.

## Preserved limitations

- Lean-to 02 has a clear south opening but four visible posts rather than the requested two poles.
- Log piles show increasing density. The middle pile was regenerated after the earlier roughly eight-log candidate missed its bracket; the new selected image visibly contains three rows of four short logs (12) at nearest-neighbour inspection. Small/large exact physical counts remain unverified; density is clearer than end-grain detail.
- The hearth flames differ, but the stone ring also changes between frames. A flame-only image edit failed to preserve the ring and was rejected. This animation remains partial.
- Stockpile stakes are faint; joined 4×4 readability must be assessed in the actual composite.
- Tree sizes are declared sprite boxes, not a guarantee of identical canopy occupancy across species.
- No runtime integration or acceptance is implied. Animals have not been generated; optional-drop decisions remain explicit in the final report.

Parent inspected the packed sheet and labelled contact sheet at native/nearest scales. Blind reading has not yet occurred.
