# C3A first-pass art delivery

Art-only candidates; nothing has been imported into the game. Read STYLE.md,
ADR0016 and the per-family READMEs before selecting files.

| Family | Delivery | Decision |
|---|---|---|
| Existing A2/A3/B3 | 35 immutable file references | Reuse; do not copy/redraw |
| New items | 8 families, 18 count/single PNGs | Numeric and independent visual pass |
| New stations | Mason bench, 1 PNG | Style, projection and visual pass |
| Workshops | 12 layered recipes, 37 shared layers | Proposed; smithy lacks anvil |
| Road transitions | 16 pavement + 16 reciprocal meadow tiles | Conditional; selector disabled, strict wrap fails |
| Stone walls | 16 masks + 8 cap aliases + 2 EW doors | Held visual failure; no runtime rows |

New generated originals:20 attempts, of which9 selected and11 rejected/held.
Items:16 attempts,8 selected,8 rejected; stations:4 attempts,1 selected,
3 rejected. Count variants are deterministic layouts, not extra generations.
Composite bow, iron hoe and iron plough reached the cumulative three-attempt
cap without passing. Anvil also has no full pass after three attempts. NS door
art is absent. Historical steel edge stays held and is outside this retry list.

`art_import.draft.json` is an additive fragment for the visual-v3 per-row
`from`/`meta_source` importer contract (87c67b1). It includes only35 reused
files,18 new item PNGs and1 bench. Preserve existing importer configuration.
Road32-file fragment and wall capability stay separate and disabled/held.
`bindings.draft.json` indexes symbolic proposals; unknown sim codes remain
unassigned. IronOre is iron-era. No Rust, GDScript or runtime assets changed.

Scale: items use the existing ENV1 64px canvas and(32,32)anchor,1/3/9 count
layouts and the existing item_scale defaults, without an override. World props
retain source anchors/footprints. Layer recipes retain independent origins,
anchors and order; never import a flattened room preview as a building.

Contact sheets: items/boards (64px and32px on two alpha-composite backgrounds),
reuse/contact sheets, workshops/contact sheets, roads/tiles/network_1x.png,
and walls/contact_sheet_2x.png. Raw generations, exact prompts, rejections,
checks and Korean REPORT.md are external at /Users/rexxa/work/art-batch/C3A.

Checks: targeted Python tests, Ruff, formatting, type analysis and lossless
hash/geometry checks. Run tests with --import-mode=importlib because both
family directories contain test_build.py. Full engine gate/play is not an
art-line verification. Docs check has unchanged baseline binary-PNG/UID faults;
see external checks/docs_check_final.txt. The legacy walls, subtle furnace glow,
road absolute wrap limit and held anvil remain integration risks.

## Pass2

BRIEF4 supersedes the anvil and stone-wall hold above. Cumulative anviltry4
passes unchanged style/projection and independent visual review; its28px PNG
and built smithy layers are now included. No more anvil generations are needed.
CompositeBow remains held. New wall pieces and native/log comparison rooms
are under walls_pass2; its manifest and README record the accepted native wall verdict.
Pass1 output counts above remain historical. The root importer fragment pins
the additional anvil to the immutable Pass2 asset commit. No game import is applied.
