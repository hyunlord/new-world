# GOAL: WorldSim — ART-PROD1: route I goes to production. Whole-figure sheets as TRIAL5 proved, with the four fixes its evidence demands: the judge measures figures not sheets, the structural rules fit whole figures, the canvas fits a raised axe, and S + E come from one generation. Same reviewers, same bars. Pass = the character standard; the assets install in M-ART2.

Repository `hyunlord/new-world`. Base: **`v8/art-trial5` = `9122b9b1c5e4c28d8dd8ff8a0413bd85963a225d`**. New branch **`v8/art-prod1`** (`v8-artprod1(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` with the image tool (generation + reference mode, transparent RGBA output). No 3D, no payment. Only `art/**`, `docs/v8/artprod1/**`. No game code, no merge. Target 6 hours, hard stop 8.

## Why
TRIAL5 (`9122b9b1`) produced the first people that read at 56 px in the 05 look: four blind reviewers scored **activity 12/12** on every board, and the planner's own look at `compare_ABPT_2x.png` and `lab_t5_1x.png` confirms it. Its "FAIL" came from three instruments the planner mis-specified and two real defects:
- judge v2 was run on the whole 1536×1024 five-figure sheet (144/144 `stroke_q90` fail) although the extracted figure panels pass 654/710 — the judge is calibrated on single-figure crops, so the sheet measurement is void;
- the "no holes > 12 px²" rule is a layered-part rule; a whole figure's gap between arm and torso is negative space, not a hole (31 of the 48 structural fails);
- the 64×96 canvas cannot hold a raised axe or a shouldered log bundle at 56 px standing height (81 normalization fails; P1 never measured);
- **real:** S and E generated separately disagree on clothing ("one facing looks more fully clothed"), and hair/cloak merge, sex reads ambiguous (attributes 26–29/36 vs 30; sex + age 16–18/24 vs 20).
This pass fixes the instruments without lowering any bar, generates both facings in one image so they cannot disagree, and strengthens the silhouette cues the reviewers missed.

# 0. RULES
No owner questions → `docs/v8/artprod1/PROD1_DECISIONS.md`. Data `/Users/rexxa/work/artprod1-data/`; ≤ 1.5 MB per committed file. No pixel surgery, no quantization; geometry (crop, uniform scale, 90°/180° rotation, flip, alpha-0 trim) and logged global tone adjustments only. **Try limit: 3 generations per sheet** (a sheet now carries both facings), ≤ 108 generations in all. Drop order past 8 h: child looks → crowd FPS → the ABPT board. Never drop: the instrument fixes (§1), the adult S+E set through checker and panel judge, the review with two fresh reviewers, the numbers, the report. Leave the worktree's untracked `.ignore`, `docs/art/`, `.omo/`, `.gitignore` change alone. Lab typecheck: 0 errors **and** warnings ≤ 20, or each remaining warning listed with a reason.

# 1. PART 0 — Instruments (before any generation)
1. Commit this goal as `docs/v8/artprod1/PROD1_GOAL.md`. Write `docs/v8/artprod1/TRIAL5_READING.md` (Korean, ≤ 1 page): why TRIAL5's S1/P1 were instrument failures and which two defects were real, with the evidence paths.
2. **Judge on panels:** `art/pipeline5/judge_panels.py` — after slicing, run judge v2 (person family, unchanged thresholds) on each **extracted figure at source resolution** (alpha-cropped, padded 8 %); a sheet's numeric verdict = all five (now ten) panels pass. Retire the whole-sheet measurement (keep its code path flagged `legacy`). Calibrate once: the five panels of the Part 0 candidate I-1 sheet-style generation must pass (if I-1's own still passes and its panels do not, the padding/crop is wrong, not the figure).
3. **Whole-figure structural rules** (`check_layer.py` whole-figure mode, TEMPLATE_V3): keep single connected component (alpha ≥ 128, after the dust rule), `contact_ok` (lowest opaque pixel on the foot line ±2), canvas fit (no opaque pixel on the canvas border), coverage (opaque height 52–58 px adult / 37–43 child standing; sleep width 48–60 px); **remove the hole rule**; add `sleep_axis_ok` with **auto-rotation**: if the sleep figure's bounding box is taller than wide, rotate 90° (toward head-left; detect the head as the end with the larger alpha mass in the outer 30 %), log the rotation; fail only if neither orientation fits. Synthetic tests for each rule.
4. **Canvas 80×112** (sleep 112×80), foot line **y = 104** (sleep: lowest pixel y = 72), figure standing height 56 px adult / 40 child, anchor x = 40; the contact shadow (lab-drawn) 28×8 centred (40, 104). Update `TEMPLATE_V3.md`, the lab, the slicer, the boards. Record how the game must anchor it (foot line to the tile's lower third; W = flipped E; **N = S until a later pass**).
5. Re-run the TRIAL5 round-1 frames through the fixed instruments and report the counts (how many of the 72 look×facings would now pass S1) — information only; nothing from TRIAL5 is promoted.

# 2. PART 1 — Generation: one sheet per look, both facings
- **Set:** sex 2 × age 2 (adult, child) × top 3 (loincloth, hide wrap, fur cloak) × hair 3 (short, long tied, bun) = **36 looks = 36 sheets**. Identity reference: `art/character/candidates/I-1.png` for every sheet; the look's own accepted sheet is the reference for its retry.
- **Sheet layout:** 1536×1024, transparent, **two rows of five**: row 1 facing S (front 3/4, ~20° above), row 2 facing E (the same person turned 90° to the viewer's right, same clothes, same hair, same build); columns in order (1) standing idle, (2) mid-stride walk, (3) two-hand stone-axe swing raised, (4) log bundle carried across the shoulders, (5) lying asleep on the side, head to the left. Equal spacing, same scale, one ground line per row. Accept only sheets with exactly 10 major components in a 2×5 grid (slicer v3 ownership rules as TRIAL5; the grid check replaces the single-row check).
- **Attribute cues (the readability fix), stated in every prompt and checked by the reviewers:**
  - adult male: full beard, broad shoulders, flat chest; adult female: visible waist, chest wrap or garment covering the chest, hips, no beard; child: 40 px, large head (≈ 32 %), round face, no beard, simple tunic or wrap;
  - hair short: cropped close to the skull, silhouette rounded; long tied: a **high ponytail standing clear of the head** outline; bun: a **high bun on top**, clear of the head outline;
  - top loincloth: bare torso (child: chest band + skirt); hide wrap: **one-shoulder asymmetric** wrap with a visible bare shoulder; fur cloak: **wide fur collar and a hood shape**, cloak hanging past the hips, colour darker than skin and distinct from hair;
  - props: axe head the size of the head, log bundle wider than the shoulders.
- Per sheet ≤ 3 tries; stop at the first try that passes grid + structure + panel judge; if none passes, keep the best (most passing panels) as `rejected` evidence and the look is missing from the set.
- Log every call (`PROD1_GENERATION_LOG.json`: prompt hash, try, verdicts, reasons).

# 3. PART 2 — Slice, normalize, check
As TRIAL5 v3/v4 with the §1 changes: ownership slicing, one scale per sheet from the S idle height, foot-line anchor, canvas 80×112, sleep auto-rotation, alpha-0 trim only. Checker on all frames; `check_set.py` on 48 sampled people. Panel judge on all panels. Outputs to `art/character/route_i/<look>/{S,E}/{idle,walk_a,work_a,carry,sleep}.png` with sidecars and `MANIFEST.json` (look fields, sheet try, scale, rotation, hashes). Boards: `route_i_sheet_{1x,2x}.png` (36 looks idle S+E), `route_i_states_{1x,2x}.png` (10 frames of four looks), `compare_ABPT2_2x.png` (TRIAL4's six cases with this pass as the last column).

# 4. PART 3 — Lab, GIFs, review
- Lab: 12 composed people drawn from the accepted set (≥ 3 female, ≥ 2 children, all three tops, all three hairs, all four props/activities), states looping, S and E, on calm meadow and calm forest floor, 1× and 2×; GIFs `p1_01..12_{1x,2x}.gif`; Playwright 8-fps capture as evidence; the conforming 80×112 canvas, so this **is** the certified lab.
- **Two fresh reviewers**, no key, the TRIAL5 answer-mapping rules unchanged: activity ≥ 10/12 at 2×, ≥ 8/12 at 1×; item + top + hair ≥ 30/36; sex + age ≥ 20/24; unknown = wrong; verbatim kept.
- Miss → **one** targeted regeneration round for the misread looks only (within each sheet's 3 tries; prompt cues sharpened for the misread attribute, logged), re-slice, re-check, **two new reviewers**; report both rounds. No third round.
- Crowd: 200 at 1× on the 80×112 canvas, 60 s, FPS.

# 5. PART 4 — Numbers, verdict, report
| # | measure | bar |
|---|---|---|
| S1 | accepted looks with all 10 frames structurally passing | ≥ 32/36 adult+child (≥ 17/18 adult) |
| S2 | generations per accepted look | ≤ 2.0 mean |
| J1 | panel judge pass on accepted frames | 100 % (by construction) |
| R1 | readability, final round | all three bars |
| T1 | model judge four scores on 6 stills | reported; ≥ 4.0 preferred |
| P1 | 200 at 1× | ≥ 60 FPS |
| K1 | generations per accepted look; +1 top = 12 sheets ≤ 36 generations | reported |
| D1 | slicing/normalization deterministic (two runs byte-identical from the same sources) | pass |

**Verdict:** S1 + R1 + P1 → **route I adopted**: `TEMPLATE_V3` is the character standard, `art/character/route_i/` the asset source for M-ART2; `route_p`, `route_a`, `route_b`, `candidates` move under `art/character/archive/` (history kept); `docs/v8/artprod1/INSTALL_NOTES.md` tells M-ART2 how to draw them (anchor, facings, missing N, scale per age, shadow). Otherwise the report names the failing attribute per look and stops; no further round.

`docs/v8/artprod1/PROD1_REPORT.md` (Korean): 판정 · 계측기 수정과 TRIAL5 재집계 · 세트와 시도 · 절단/정규화(회전·배율 분포) · 구조·패널 판정 · 리뷰어 라운드별 원문 요약 · 스타일 점수 · 군중 FPS · 비용(K1) · 변경된 파일 · 제한사항 · 오너 결정 필요(M-ART2 범위: 64-px 카메라에 이 세트 설치, N 방향, 노인/머리색 변형). Push; `git ls-remote origin v8/art-prod1`.
