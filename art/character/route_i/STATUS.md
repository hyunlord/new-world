# ART-PROD3 route I: ADOPTED (2026-10-23 owner decision)

The adopted inventory is 34/36 looks: adults 16/18, children 18/18.
The planner/owner explicitly waived the original adult >=17/18 bar for
adult_M_fur_cloak_long_tied and adult_M_hide_wrap_long_tied. Their failures
remain recorded; fallback to adult_M_fur_cloak_bun and adult_M_hide_wrap_bun,
respectively. This decision does not turn the original numerical S1 into PASS.

Eight missing looks passed re-slicing of preserved sources. No new generation
was performed. The two remaining looks stay MISSING with explicit fallbacks in
`docs/v8/artprod3/INSTALL_NOTES.md`. TEMPLATE_V3 is the character standard;
this directory and MANIFEST.json are the asset source. Historical routes and
production diagnostics are preserved under `art/character/archive/`.

The 26 PROD2 accepted look directories are byte-preserved. Their independent
alpha16 reruns passed, but 202/260 PNGs differ because registration now uses
the visible bbox. These reruns were not substituted into the accepted set.
Every original PNG also passes the fresh alpha16 structural checker.

See `docs/v8/artprod3/PROD3_REPORT.md` and `RESLICE_RESULTS.json`. PROD2 R1/P1/D1
remain cited evidence, not measurements repeated in PROD3. This adoption pass
performs no game-code installation or merge.
