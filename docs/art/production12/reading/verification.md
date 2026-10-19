# Reading verification (in progress)

- Ten focused production/test suites passed; see first.tsv and external p2/tests logs. This is not the complete visual gate.
- Font at actual Life size24px: 카루미 with/without joiners has0 differing pixels in regular and bold; 티/디 differs by24/32 pixels. Native glyph captures inspected; no font-size escalation needed.
- Stockpile root cause: locale reload had no notification while caption refresh waited for the next reconciliation tick/frame interval. The locale-change signal synchronously rebuilds captions even while frozen.
- Real-input attempt1 failed initial agentclick: handler read current OS mouse rather than eventposition. Transforming eventposition by inversecanvas fixed real queued-event dispatch without altering tilefallback. Attempt2 passed clicks,parentrecord,wheel,Escape,Copen but test locator could not derive GUI hover from injected motions. Attempt3 uses actual click probes and meta_clicked observations, not direct actions; all13 steps passed, including chroniclepersonselection and KO→EN. The firsttwo failures remain.
- Fullgate now runs this acceptance test windowed through the sharedlock, with a new outputdirectory; ordinary scripts retain headless execution.
- Chronicle clipping: RED4 clipped lines, then whole-line viewport fitting plus bottompadding; windowed64 checks0 failures and EN/KO top/bottom screenshots visually inspected. All60 feed entries remain.
- Inspector literal clipping test first found11 failures. Adaptive bottom layout reservation now fits complete visible text bands at both extremes without masking/removing content. Strengthened actual glyph/line bounds test passes EN/KO headless and windowed; allfour images reviewed. Initial REVISE and failed log retained.
- Activity and crowd blind tests pending. Live Life/porter/quarrel evidence follows Part3; synthetic layout fixtures do not establish live readability.

Raw evidence root: /Users/rexxa/work/art-production12-evidence/.

- Optional link hover now reuses the settlement-outline shader in ring color and existing offscreen marker geometry. Hover enter/leave, hidden-owner cleanup, missing-person handling, marker placement and selected-village restoration pass; windowed on/offscreen captures inspected. No sprite is invented for a dead/missing person. Source-only hover changes followed the frozen activity captures, whose raw hashes remain unchanged.
- Activity retest terminal result is3/10 diagnostic with4missing, H8FAIL. Village taskNOT_RUN due2of6eligiblepairs. Three natural crowd geometry proofs pass; their human-readable activity score and live Life histories remain dependent evidence forPart3.

## Part3 source checks before live capture

- SimReads: actual bridge tick-cache64 assertions and existing497-equivalence checks pass. Fake-world reads remain live/uncounted; world reset, replacement, mutation, peek and next-tick behavior covered.
- Chronicle17–21: RED40 failures, GREEN92checks; roles/causes/receiver-sender ordering, raw-adjacent grouping across512-entry pages, hidden separators, locale and unknown-kind counters covered. All15 authorized keys appended at IDs5276–5290 in allfive localization files.
- Porter: actual carried food/wood chooses pose, rowamount only supplies inspector text. Meal and existing Cold.pose precedence retained. Wood uses accepted baked carry art; no duplicate wood prop. Helper/actual-renderer fixture/CarryLoad regressions pass; these are synthetic boundary tests, not live acceptance. Procedural16px glyph sheet is porter-icon-sheet.png.
- Inspector status immediately follows the name, uses entity-bit join, correct outbound receiver/return sender and localized noun.16checksPASS; existing72 Life checks and literal layout checks pass. Initial missing-status RED and an initial test parse error retained.
- Relations: current formation anchors, strict zoom<2,2screenpx width, threshold colors and estranged red dashed localized midpoint.36checksPASS and existing settlement-label testPASS. Real-world pixel acceptance pending.
- Strict initially rejected implicit integerdivision in procedural strap. Explicit equivalent shift repaired it; final53files parse/warnings and6FFI calls pass. All17 P12 Python tests pass. Section instrumentation regenerated successfully without replacing ordinary statements.
- Full raw logs: /Users/rexxa/work/art-production12-evidence/p3/. Every pre-capture pass is implementation evidence only; live reading/porter/crowd acceptance follows the source freeze.
