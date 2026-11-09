# A3 items — art-branch handoff, 2026-11-10

**127 selected logical assets / 237 transparent PNGs.** This is an art adoption,
not a game import. All26 priority C2 objects are included. The complete122-base
catalogue was generated;120 base IDs were adopted, plus7 empty-vessel states.
Iron spear and steel edge remain withheld after poor initial candidates and an
image-service block on their replacement request. Do not describe the full
family as accepted. Four base IDs are explicitly inferred proposals, not named
canonical bible entries: bronze sword, iron pickaxe/chisel/shield.

[Manifest](MANIFEST.json), [review and attempt ledger](review.json),
[style contract](STYLE.md), [decisions](DECISIONS.md),
[import handoff](drafts/README.md). Full Korean report and raw evidence:
`/Users/rexxa/work/art-batch/A3/REPORT.md`.

| Production group | Generated | Adopted logical | Rejected | Alternate |
|---|---:|---:|---:|---:|
| Food |27|23|1|3|
| Copper/bronze chain |16|15|1|0|
| Other bronze goods |20|15|1|4|
| Iron goods |20|15|5|0|
| Tools/textiles |23|20|3|0|
| Crops/seeds |18|18|0|0|
| Other catalogue objects |15|14|1|0|
| Empty vessels |7|7|0|0|
| **Total** |**146**|**127**|**12**|**7**|

The55 pile families have count buckets1 /2–8 /9+, adding110 PNGs to the127
base/state images. The base filename is count1. All icons use64×64 RGBA,
anchor(32,32), footprint(1,1), the unchanged ENV1 fit/layout, and preserved source
alpha. Every selected item has a JSON sidecar with its prompt, null/unexposed
seed, raw/record hashes, chosen attempt, geometry and diagnostics. Raw images
remain outside Git. Era counts including empty states: paleolithic16,
neolithic41, bronze52, iron18. New brief era assignments are documented in the
draft; they do not change simulation availability.

## Compare at intended size

Each page starts with six actually imported ENV1/POLISH references.

| Page | Native64px | Half-scale32px |
|---|---|---|
|1|[sheet](sheets/selected_final_64px_01.png)|[sheet](sheets/selected_final_32px_01.png)|
|2|[sheet](sheets/selected_final_64px_02.png)|[sheet](sheets/selected_final_32px_02.png)|
|3|[sheet](sheets/selected_final_64px_03.png)|[sheet](sheets/selected_final_32px_03.png)|
|4|[sheet](sheets/selected_final_64px_04.png)|[sheet](sheets/selected_final_32px_04.png)|
|5|[sheet](sheets/selected_final_64px_05.png)|[sheet](sheets/selected_final_32px_05.png)|
|6|[sheet](sheets/selected_final_64px_06.png)|[sheet](sheets/selected_final_32px_06.png)|

Review used visible labels; it is not a blind identification study. Needles,
arrow/spear shafts and some tools remain thin at32px. Seed varieties and subtle
wine/oil fill states need labels. Boat/cart/plough images are inventory icons;
this pass does not establish larger world-object dimensions.

## Verification and limits

- All237 selected PNGs pass fresh alpha,64px canvas, exact ENV1 placement,
  count bounds and source/output hash checks. All146 attempts remain external.
- Nine pipeline tests pass; Ruff lint/format, basedpyright and the programming
  no-excuse check pass. Existing pipeline/calibration code was reused unchanged.
- Raw numeric style passes79/127 selected sources. Projection diagnostics pass
 114/127 base icons and215/237 exports. These failures remain visible in sidecars
  and `review.json`; visual adoption does not convert them into numeric passes.
- Four of six imported raw references also fail the unchanged style calibration.
  Final64px style scoring is unsuitable (all six references fail after line
  extraction). Material colours, vessel interiors and count placement can bias
  the global lighting gradient. Independent local-shading review is recorded;
  fish/goat-hide light direction remains indeterminate.
- System-table check passes. Full docs check still fails on unchanged historical
  `docs/v8/artenv1` captures and `art/proto` UID gaps present at the art base.
  Evidence: external `docs_check.log` and `docs_baseline.json`.
- No game import, headless play, full simulation gate or runtime binding change
  was performed. `rust/`, `scripts/`, runtime assets and existing importer files
  are unchanged. C2/C3 must perform their own integration and visual checks.

## Import safely by data

The draft is an additive fragment, never a replacement for the existing config.
The importer reads geometry from `meta_sources.item`, not adjacent sidecars;
[items_manifest.json](items_manifest.json) preserves22 legacy entries and adds
237 new image IDs so centered icons do not fall back to(32,63).

Only53 current simulation codes are verified binding rows. Future codes remain
null in the pending document; the seven empty states have no general runtime
selector yet. The two withheld assets have no active import rows. Preserve the
existing config, bindings and40MiB budget. Follow [drafts/README.md](drafts/README.md)
for the pinned source commit, exact paths, era conflicts and required checks.
