# Wood conversion contract

`python3 tools/art_pipeline/wood_candidates_p4.py --sheet V_a4 --raw /Users/rexxa/work/art-production4-raw/V_a4/attempt_1.png`

The same command supports V_b4, V_c3 and W1; `--attempt 2` writes a distinct attempt without overwriting review evidence. The configuration binds the existing approved reference selection by SHA256. Four sheets contain exactly 64 physical cells. The generation checkpoint denominator remains 64 regardless of conversion or visual failures.

Processing reuses v4 recovery, palette projection, fit, frame/group guards and the bounded despeckler. Wood sheets opt out of the old procedural completion path, since unsupported wood poses must not be invented. Existing sheets retain their prior completion behavior. Adult height is 28 and child height 21. Carry wood has the existing standing and four-frame cycle guards; gather wood has the existing standing guards. W1 caps are 24×16 for cells 1–8, 24×20 for piles and exact 16×16 for icon cells 12/16. Hair masks use the frozen Part1 body palettes and classifier on both native and outlined sprites.

The raw full sheets and raw crops remain outside git. Native, outlined, unfitted and hair images remain in processed evidence. Each frozen candidate ledger carries the source raw hash, candidate fingerprints, technical reports and board hashes. The independent reviewer sees boards and a blank template only. Every attempted cell is retained, including technical failures. No candidate is installed or considered accepted by conversion alone.

Deadwood primary stages 3/2/1/0 are W1_01/03/05/07; alternate variants are 02/04/06/08. Pile primary stages 1/2/3 are 09/10/11, alternates 13/14/15. Icon primary is 12, alternate 16.

`production_sheets.SHEETS` knows all four sheet definitions, so the existing installer resolves their exact semantics. Final installation must preserve the currently installed Part2 result, overlay only independently approved candidates passing all current technical and assembled-selection guards, bind reviewed hair maps, and run the existing installer. This preparation script writes no production assets.
