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
