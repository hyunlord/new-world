# Walker identity attempt 2

**Compound walker candidate identity PASS, comparator exit 0.** The root confirmed capture session 11368 terminal exit 0 after all 28 launches. The corrected collector and updated strict validator were used; no captures were restarted by the archival worker.

- All six framings P1–P6: full-frame RGBA A/A′ and A/B each have 0 differing pixels and maximum channel difference 0, including the previously failing P3 control.
- All five running scenarios: W0, W200, W-multi (600 frames each), W500_FROZEN and W500_2X (240 frames each) have exact off/on JSONL bytes and pass the inherited schedule/population/camera/render evidence checks.
- Actual walker polarity is false/false/true for all six A/A′/B framings and false/true for all five running off/on worlds. Declump kernel remains ON and write skip remains OFF. All 18 capture-time still stamps and 4,560 running-frame stamps plus 10 final running stamps pass strict validation. Still collectors do not provide per-frame stamps.
- All 28 launches have exit 0, artifacts present, no log errors and unchanged before/inside/after provenance. Source/build/HEAD are equal across the five run groups. P3 additionally passes the strengthened camera disabled/unpaused assertions.
- The full P3 A′ framebuffer was visually inspected after comparison; the pause indicator is dim. Quantitative identity uses the complete full-resolution RGBA images, with no cropped/masked/excluded pixels or tolerance.

The inherited reports retain scope `reference_integrity_only`. Candidate identity acceptance is established by the separate `comparison-attempt1/walker-identity.json` scope `compound_walker_candidate_identity`, which combines the inherited comparisons with walker polarity and provenance checks. Do not reinterpret an inherited reference-only report by itself as candidate validation.

This is identity evidence, not performance or a change to the candidate default. Performance has its separate archived result. The leader owns the final acceptance/default decision and remaining downstream gates.

## Preserved failure and correction

The original attempt remains archived under `../identity-attempt1/`: P3 control A/A′ differed by 924 pixels, max channel 149. The pause-state isolation diagnosis and first failed verdict are retained. Attempt2 is a fresh complete run following the identity-only camera isolation correction; it does not erase the first failure or weaken thresholds.

## Evidence retention

Every external file is listed in `SHA256-MANIFEST.json` with byte length and SHA256. All PNGs, JSON sidecars, provenance snapshots, launch TSVs, logs, runner metadata, comparison reports and previews are copied here and verified byte-for-byte by SHA. Complete running `.jsonl` dumps remain at their absolute external paths in the manifest (~90MB total across off/on); their exact hashes and sizes are retained here. No summary replaces the original raw dumps.

The comparator log and terminal status are `comparison-attempt1.log` and `comparison-attempt1.exit`. Machine results are in `comparison-attempt1/`; visual diff artifacts are in its `rgba-reference/` directory.

Frozen capture HEAD: `c1827b3b68bfac11155c8cd00e45ecfe69e3fdc6`. Bridge SHA256: `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`. Source-manifest SHA256: `adb480e6d471ba33f7f5cb30d3c5010453f38745a2916ccf2dfa1c6b648ffcfc`.
