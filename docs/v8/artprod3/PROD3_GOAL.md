# GOAL: WorldSim — ART-PROD3: adopt route I. PROD2 passed readability, FPS and determinism; the 10 missing looks fail on one measurement — the sleep frame's canvas fit is judged on every alpha > 0 pixel, so faint halos push a body that fits past the 112-px edge. Judge the fit on visible pixels, re-slice the preserved sources, adopt, archive, hand over to M-ART2.

Repository `hyunlord/new-world`. Base: **`v8/art-prod1` = `c63eb72b198f7cd213be5bb5dfd0fe4cc2339b73`**. Same branch (`v8-artprod3(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod`. Only `art/**`, `docs/v8/artprod3/**`. **No new generation** (budget 0; if a look still fails after §1 it stays MISSING with the fallback in INSTALL_NOTES). No game code, no merge. Target 1.5 hours, hard stop 3.

## Why
PROD2 (`c63eb72b`): R1 PASS on both reviewers at both scales (activity 12/12, item+top+hair 31–32/36, adult sex + age 20–21/22), P1 524 FPS in Godot, D1 PASS, S1 26/36. Nine of the ten missing looks fail only `S/sleep: normalization: visible extent … would clip canvas (112, 80)` although the opaque body (alpha ≥ 128) fits the band: the sleep scale is taken from the alpha ≥ 128 bbox, but the canvas-fit test counts every alpha > 0 pixel, so antialiasing halos and faint tails decide the verdict. The tenth (`adult_M_fur_cloak_long_tied`) fails the panel judge on E/work_a. The planner takes the responsibility for the inconsistent threshold and declares (D1 of this pass) that visibility is alpha ≥ 16 for fit and crop, that cropping pixels below that threshold outside the canvas is geometry (not pixel surgery), and that looks accepted this way need no new review because their sources and their reviewed frames are unchanged.

# 1. PART 0 — Visibility threshold
1. Commit this goal as `docs/v8/artprod3/PROD3_GOAL.md`; D1 as above in `PROD3_DECISIONS.md`.
2. `visible_extent` = bbox of alpha ≥ **16** for every frame (upright and sleep). Canvas fit, foot-line anchor and bands use it. Pixels with alpha < 16 that fall outside the canvas after placement are discarded (logged per frame: count, max alpha); pixels inside the canvas are kept untouched. Synthetic tests: a body that fits with a 6-px halo of alpha 3–12 passes; an opaque body that truly overflows still fails; the discarded-pixel count is reported.
3. Re-slice **only the 10 missing looks** from their preserved best sources (old/new as PROD2's table names them); run structure + panel judge; promote passes to `route_i/`. Also re-run the 26 accepted looks through the new threshold and assert they are byte-identical (no alpha < 16 pixels outside the canvas in them) or list the diffs.
4. `MANIFEST.json`, `INSTALL_NOTES.md` (fallback list updated), boards `route_i_sheet_{1x,2x}.png` regenerated.

# 2. PART 1 — Adopt
- Verdict = PROD2's R1 + P1 + D1 (unchanged, cited) + this pass's S1 (report the number; the planner adopts at ≥ 32/36 and adults ≥ 17/18, and adopts the adult set alone if only children are short).
- On adoption: `TEMPLATE_V3` is the character standard; `art/character/route_i/` + `MANIFEST.json` the asset source; move `route_p`, `route_a`, `route_b`, `candidates`, `trial5_diagnostics`, `route_i_lab/prod*` diagnostics (keep the lab tool) under `art/character/archive/` with a README naming each trial and its report; `docs/v8/artprod3/HANDOVER.md` for M-ART2: asset paths, canvas/anchor, frame plan, facings, scale per age, shadow, attribute mapping, MISSING fallbacks, the review evidence paths, and the three owner decisions still open (real N facing; elder/hair-colour tints; adding tops).
- If S1 is still short: no generation; report per look and stop; the adult-only adoption rule above applies.

`docs/v8/artprod3/PROD3_REPORT.md` (Korean, ≤ 2 pages): 판정 · 임계값 변경과 근거 · 10룩 재절단 결과 · 26룩 불변 확인 · 아카이브 목록 · 인수인계 요약 · 오너 결정. Push; `git ls-remote origin v8/art-prod1`.
