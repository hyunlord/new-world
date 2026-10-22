# GOAL: WorldSim — ART-PROD2: finish route I. One geometry rule (the sleep frame) sank 22 of PROD1's 23 missing looks; fix it, re-slice the 96 preserved sources, regenerate only what still fails, certify with children in the lab, and adopt.

Repository `hyunlord/new-world`. Base: **`v8/art-prod1` = `eb428ce232df77c720eb1cd756e43a891d9dd8a9`**. Continue on the same branch (`v8-artprod2(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` (image tool; Godot 4.6 for the FPS run). Only `art/**`, `docs/v8/artprod2/**`. No game code, no merge. Target 3 hours, hard stop 5.

## Why
PROD1 (`eb428ce2`) accepted 13/18 adult looks whose S/E facings agree and whose tops, hair and sex read on the boards (`route_i_sheet_2x.png`, `route_i_states_2x.png`). Of the 23 missing looks, 22 failed **only** on the sleep frame: "S/sleep canvas normalization" or "sleep width" — including all 18 children, which otherwise passed 8/10 at their first try. The cause is the planner's rule "one scale for all five poses from the standing idle height": the model draws the lying figure larger to fill its panel, so the common scale overflows the 112-px sleep canvas; and the child sleep-width band (48–60) was the adult band. The readability misses (attributes 24–28/36, sex + age 14–16/24) were measured on a diagnostic sample with no accepted children and 16/120 frames missing, so they are not yet a verdict on the set. The crowd FPS (59.27) was rAF-capped Chromium, not the Godot lab the bar names.

# 0. RULES
As PROD1 §0 (DECISIONS in `docs/v8/artprod2/PROD2_DECISIONS.md`; data `/Users/rexxa/work/artprod2-data/`; geometry only, no pixel surgery). **Generation budget: ≤ 2 new tries per look that still fails after re-slicing, ≤ 40 generations in all.** Never drop: the rule fix with tests, the re-slice of all 96 sources, the certified lab with ≥ 2 children, the review, the Godot FPS, the numbers, the report.

# 1. PART 0 — The sleep rule (before any generation)
1. Commit this goal as `docs/v8/artprod2/PROD2_GOAL.md`.
2. **Sleep scale by body length:** the sleep panel gets its own scale so that the lying figure's **length** (the long axis of its alpha bbox) equals the look's standing height (56 adult / 40 child) — the same body, lying down. The other four poses keep the common scale from S idle. Record the two scales in the sidecar. Rotation rule unchanged (auto-rotate to head-left; log).
3. **Sleep bands by age:** adult length 52–60 px, height ≤ 36; child length 37–43, height ≤ 28. Canvas 112×80 unchanged; lowest pixel y = 72, anchor x = 56.
4. Tests: a synthetic lying capsule at 1.6× the standing scale normalizes to the band; a child one too; a standing figure is unaffected.
5. **Re-slice all 96 PROD1 sources** with the fixed rules (no new generation). Report per look: best try, 10-frame structural result, panel judge. Promote every look that now passes to `route_i/`.

# 2. PART 1 — Fill the gaps
For looks still missing after Part 0: ≤ 2 new sheet generations each (PROD1 prompt + any cue noted in its per-look misreads), same 2×5 layout, slice, check, panel judge. A look with no passing try stays MISSING and is listed with its best try's failures.

# 3. PART 2 — Certified lab and review
- Lab on the 80×112 canvas from **accepted looks only**: 12 people — ≥ 2 children, ≥ 4 female adults, all three tops, all three hairs, all four activities + sleep; S and E; calm meadow and calm forest; 1× and 2×; GIFs; Playwright 8-fps capture as evidence.
- **Two fresh reviewers**, no key, PROD1 mapping rules. Bars: activity ≥ 10/12 at 2×, ≥ 8/12 at 1×; item + top + hair ≥ 30/36; **sex scored on adults only, age on all: (sex_adult_correct + age_correct) / (adults + 12) ≥ 0.83** — this replaces "sex + age ≥ 20/24" and is a declared planner change (children's sex is not a readable attribute at 40 px; RimWorld does not render it either). Record it as D1 with this reasoning.
- Miss → one targeted regeneration of the misread looks (within the §0 budget), re-slice, **two new reviewers**; report both rounds. No third.
- **Godot crowd:** 200 people at 1× in `art/proto` (PawnLab-style scene with the route_i frames), windowed, 60 s, frames rendered / elapsed, vsync off; ≥ 60 FPS. Chromium numbers are not used for P1.

# 4. PART 3 — Numbers, verdict, report
| # | measure | bar |
|---|---|---|
| S1 | accepted looks (all 10 frames) | ≥ 32/36 and adults ≥ 17/18 |
| S2 | new generations this pass | ≤ 40 |
| R1 | readability, final round (bars above) | all |
| P1 | Godot 200 at 1× | ≥ 60 FPS |
| D1 | re-slice deterministic (two runs byte-identical) | pass |

**Verdict:** S1 + R1 + P1 → **route I adopted**: `TEMPLATE_V3` is the character standard; `art/character/route_i/` + `MANIFEST.json` is the asset source; `route_p`, `route_a`, `route_b`, `candidates`, `trial5_diagnostics` move under `art/character/archive/` (history kept); write `docs/v8/artprod2/INSTALL_NOTES.md` for M-ART2: canvas and foot-line anchor relative to the 64-px tile, facings (W = flipped E, N = S for now), per-age scale, the contact shadow, the frame plan (idle/walk_a alternate for walking, work_a/idle for work, carry, sleep), look → entity attribute mapping (sex, age band, top from apparel, hair from identity), MISSING looks and their fallback (nearest look by top, then hair). If S1 fails only on children, adopt the adult set and say so; if R1 fails, stop and report the attribute.

`docs/v8/artprod2/PROD2_REPORT.md` (Korean): 판정 · 규칙 수정과 재절단 결과(룩별 전후) · 추가 생성 · 랩·리뷰 라운드별 원문 요약 · Godot FPS · 변경된 파일 · 제한사항 · 설치 노트 요약 · 오너 결정 필요(N 방향 제작 여부, 노인·머리색 변형, 상의 추가 계획). Push; `git ls-remote origin v8/art-prod1`.
