# BOARD QA — actual final PNGs

2026-10-22. Read-only visual inspection opened all 12 PNGs in `art/concepts/boards/` individually, plus file-size and SHA-256 verification. Only this report was written. **Assembly presentation is usable; the experiment is not a fully fair/completed comparison.** Camera, content and missing-input limitations remain visible.

## Observed boards

| Opened output | Observed result |
|---|---|
| concept_01_earth_and_ash.png | S1–S8 in 2 columns × 4 rows; Korean pitch and subject ids visible; intact cells |
| concept_02_folk_woodcut.png | Same complete layout and labels; intact cells |
| concept_03_watercolor_field.png | Same complete layout and labels; intact cells |
| concept_04_diorama.png | Complete layout; S5 and S7 visibly labelled REJECTED_CAMERA |
| concept_05_ink_and_tone.png | Complete layout; S7 visibly labelled KEPT_SCALE_LIMITATION |
| concept_06_cave_ochre.png | Complete layout; S7 visibly labelled REJECTED_CAMERA |
| concept_07_storybook_gouache.png | Complete layout; S7 visibly labelled REJECTED_CAMERA |
| concept_08_cold_north.png | Complete layout and labels; intact cells |
| overview_people.png | Eight S2 subjects in one row, correct slug labels |
| overview_hut.png | Eight roofless S4 subjects in one row, correct slug labels |
| overview_scene.png | Eight S7 scenes, two per row; 04/06/07 camera rejection and 05 scale limitation labels visible |
| readability.png | Eight concept rows, forest left/meadow right; 05 and 06 both show C2:CONTENT_MISMATCH_4_LOGS; 08 has two NOT_VERIFIED unavailable-C2 placeholders and NO_MAGENTA labels |

No assembly-caused clipping, aspect-ratio stretching, label overlap or missing subject cell was observed. Square images have dark side margins because S7 determines the wider uniform cell. Some source objects approach the source-frame edge; the boards do not repair or newly crop them. Korean pitch labels are readable when opening the actual board, but small (roughly 10 px after board reduction), so inline thumbnails are less comfortable for reading. Dark strips and diagnostic strings remain visible.

## Remaining visual limitations

- Readability backgrounds show conspicuous repeated motifs and seams. The 03 meadow has a particularly obvious bright checkerboard; 05 forest repeats conspicuous roots. These are observed source/tile weaknesses, not grounds to repaint or hide seams.
- 04/06/07 rejected camp images remain diagnostic illustrations. Their presence in an overview cannot establish camera compliance or fair mood rankings.
- 05/06 four-log cutouts differ from the fixed three-log subject. Their visible flags must accompany any resulting scores.
- 08 has no valid woodcutter cutout and no two-person test image. NOT_VERIFIED is correct; no score or full-comparison claim should replace it.
- This review checks assembly visibility and integrity; it does not replace fresh blind-review responses, validate style differentiation, or make an owner choice.

## Measured integrity

All eight concept PNGs are 764×1154. People and hut overviews are 2267×320; scene overview is 618×934; readability board is 1280×3392. Every PNG is below 1,500,000 bytes: minimum 1,003,161, maximum 1,474,339 (`overview_hut.png`). Total `art/concepts/` size at measurement: **16,188,257 bytes**, below 20,000,000.

Recomputed all **79** source hashes listed in `/Users/rexxa/work/artconcept-data/checks-final/assembly.json`: **0 mismatches**. Independently recomputed the **87** generation-attempt entries exposing both `path` and `sha256` across generation manifests: **0 mismatches**. This second count describes the inspected schema subset, not a claim that every manifest uses those fields. No original, board or assembly script was modified by this reviewer.
