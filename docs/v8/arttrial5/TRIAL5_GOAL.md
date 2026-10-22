# GOAL: WorldSim — ART-TRIAL5: whole figures, never parts. The owner picks the look first (Part 0 stops for that), then the chosen route produces the full set: route I = image-generated 5-pose character sheets sliced and normalized; route B = a human-modelled free rig (Quaternius UBC) rendered through the TRIAL3b ink pipeline with its mask defect fixed. Same checker, same GIF review, same bars.

Repository `hyunlord/new-world`. Base: **`v8/art-trial4` = `9f9c6d6bacf641825a594f13467e1a09c458242d`**. New branch **`v8/art-trial5`** (`v8-arttrial5(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` with the image tool (generation + reference/edit mode, transparent background output) and, for route B only, Blender 4.5 as in TRIAL3b. No Higgsfield, no Meshy, no payment, no login beyond what the image tool already has. Only `art/**`, `docs/v8/arttrial5/**`. No game code, no merge. **Part 0 target 2 hours, then STOP and report.** After the owner's pick, Parts 1–4 target 6 hours, hard stop 8.

## Why
Every failed people trial decomposed the person: part sheets assembled on a rig (TRIAL2b: hollow torsos), layers generated inside template guides (TRIAL3-A: grey crouched figures), a 7-bone toy rig (TRIAL3b-B: peg dolls), code-drawn capsules (TRIAL4: toy pawns, owner: "다 별로"). The one time the model drew a whole person in the 05 look — the TRIAL2 person anchor — it was good enough to be the style reference, and the environment succeeded (35/38) because each asset was generated whole. So this trial never splits a figure. Variation that used to come from layers comes from more sheets; the set is sized to what the game needs now (36 looks), not to a wardrobe. Items that must vary (axe, log bundle) are baked into the poses that use them. And the owner's eye moves from the end of the trial to the start: nothing is mass-produced until the owner has approved one figure at 1×.

Also answered here: "Blender로 캐릭터를 만드는 건?" — an agent cannot sculpt an organic human by script (TRIAL3b's authored outfits show the ceiling). Blender is only useful with a human-modelled rig; that is route B, with a proper rig this time, judged on the same Part 0 board.

# 0. RULES
No owner questions except the Part 0 stop → `docs/v8/arttrial5/TRIAL5_DECISIONS.md`. Data outside the repo: `/Users/rexxa/work/arttrial5-data/`; ≤ 1.5 MB per committed file. No quantization, no hand-editing of pixels; global tone adjustments (hue shift inside a detected mask, contrast cap) are allowed and must be logged. **Try limits:** Part 0 ≤ 3 generations per candidate; Part 1-I ≤ 2 generations per sheet; Part 1-B ≤ 2 render setups. Drop order past the hard stop: N facing (already excluded) → child looks → elder tint → crowd FPS. Never drop: Part 0 board + stop, the adult S+E set through the checker, the GIF review with two fresh reviewers, the numbers, the report. Leave the worktree's untracked `.ignore`, `docs/art/`, `.omo/` and the `.gitignore` change alone.

# 1. PART 0 — Look candidates, then STOP
1. Commit this goal as `docs/v8/arttrial5/TRIAL5_GOAL.md`.
2. **Route I candidates (4):** one standing idle figure each, front 3/4 from ~20° above, feet on a flat ground line with a soft contact shadow, **transparent background**, 1024×1536, generated with `art/style05/STYLE.md` rules and the person anchor + hut anchor as references. Looks: I-1 adult male, hide wrap, short hair; I-2 adult female, fur cloak, long tied hair; I-3 adult male, loincloth only, bun; I-4 adult female, hide wrap, braid. Stylized proportions as TRIAL2b §3.1 (head ≈ 30 %, broad shoulders, simplified limbs, no realistic anatomy) — in the prompt, not as guides. ≤ 3 tries each; pick by judge v2 (person family) + your reading; log every try.
3. **Route B candidate (1):** download Quaternius Universal Base Characters (Standard, CC0; record URL, SHA-256, licence) — choose the base with the most stylized proportions; if the pack includes clothing/hair meshes, dress one figure (hide-like top, short hair). Render the idle S still through TRIAL3b's `render_b.py` with the frozen look 3 (warm 3-band + paper grain). A still only; no clips, no passes, no mask work in Part 0.
4. **Board `art/character/boards/look_candidates.png`:** five columns I-1..I-4, B; three rows: full resolution crop (head to feet, 384 px tall), 2× (112 px tall) on calm meadow, 1× (56 px tall) on calm meadow next to the current pawn and the existing TRIAL3-A figure for scale. Plus `look_candidates_1x_crowd.png`: each candidate ×12 scattered on calm meadow at 1×.
5. Report `docs/v8/arttrial5/TRIAL5_PART0_REPORT.md` (Korean): the board, tries per candidate, judge v2 numbers, your ranking with one line each, and the question for the owner: **"I-n / B / none"**. Push, `git ls-remote`, **stop.**

# 2. PART 1-I — Sheets (if the owner picks an I candidate)
- **Set:** sex 2 × age 2 (adult, child) × top 3 (loincloth, hide wrap, fur cloak) × hair 3 (short, long tied, bun) = 36 looks; facings S and E (W = flip; N excluded) = **72 sheets**. The chosen candidate is the identity reference for every sheet (edit/reference mode); the S sheet is the reference for the E sheet of the same look.
- **One sheet** = 1536×1024, transparent background, "the same person five times in a row, equal spacing, full body, same scale, front 3/4 from slightly above, feet on one ground line": (1) standing idle, (2) mid-stride walk, (3) two-hand stone-axe swing raised (axe in hands), (4) carrying a log bundle across the shoulders, (5) lying asleep on the side, head to the left. ≤ 2 tries; accept by judge v2 + a panel count check (exactly five figures found).
- **Slice and normalize (`art/pipeline5/sheet_slice.py`):** connected components of alpha → five figures left to right (a component touching another is split at the narrowest alpha column; log it); each figure scaled so standing height = 56 px (child 40 px) at 1× from the idle panel's height, the **same scale applied to all five panels of that sheet**; lowest opaque pixel on the foot line y = 88 of the 64×96 canvas (sleep: 96×64 canvas, lowest pixel on y = 56); Lanczos. Walk from panel 2 only gives one frame: derive walk_b as the horizontal flip of walk_a's leg region? **No** — no pixel surgery. Walk animation = walk_a (panel 2) alternating with idle (panel 1) plus a 1-px bob; work = panel 3 alternating with idle; carry = panel 4 with a 1-px bob; sleep = panel 5. Record this frame plan in TEMPLATE.md v3 (whole-figure).
- **Tints (logged, droppable):** elder = grey hue shift inside a hair mask found by colour clustering at the head top (report mask quality); hair colours 2 extra by hue shift of the same mask. If the mask is unreliable (< 90 % of sheets pass a visual spot check of 20), drop tints and say so.
- **Checker:** `check_layer.py` in whole-figure mode: contact_ok, single connected component, no holes > 12 px², coverage of the 64×96 canvas within the TEMPLATE bounds, alpha edge sanity; `limb_ok` reported as information. Accept = 100 % of sliced figures; a failing figure → regenerate its sheet (counts toward the sheet's 2 tries).
- **Output:** `art/character/route_i/<look>/<facing>/{idle,walk_a,work_a,carry,sleep}.png` + sidecars (prompt hash, try, judge numbers).

# 3. PART 1-B — Rig (if the owner picks B)
- Fix the TRIAL3b mask-propagation defect first (`b_render_masks.py`): masks derive from material/object id passes only, with a bounded edge dilation of ≤ 1 px at 1×; synthetic test (a known two-material capsule → masks complete, no islands). Then the UBC rig: outfits/hair from the pack where present, else TRIAL3b's authored meshes retargeted; clips: the pack's if present, else Blender keyframes ≤ 6 as before; frames and canvases as TRIAL3b §3; passes by id into the layer contract (v1 regions). ≤ 2 render setups starting from look 3. Checker as TRIAL3b with `limb_ok` and `contact_ok`.

# 4. PART 2 — Lab, GIFs, review (either route)
- Lab page: 12 composed people (varied sex/age/top/hair) × states × S/E on calm meadow and calm forest floor, 1× and 2×; GIFs `t5_01..12_{1x,2x}.gif`; Playwright 8-fps capture of real playback as evidence (TRIAL3b method). Boards `lab_t5_{1x,2x}.png`, `compare_ABPT_2x.png` (TRIAL4's six cases + this route as a fourth column).
- **Two fresh reviewers**, no key: activity, item, top, hair, sex, age. Bars: activity ≥ 10/12 at 2×, ≥ 8/12 at 1×; item + top + hair ≥ 30/36; sex + age ≥ 20/24; unknown = wrong; verbatim kept.
- Miss a bar → **one** revision round (route I: re-prompt and regenerate only the sheets behind the misread attribute, within tries; route B: shader/camera only), rerun checker, **two new reviewers**; report both rounds.
- Crowd: 200 at 1×, 60 s, FPS (droppable). Style: judge v2 on 6 stills + model judge.

# 5. PART 3 — Numbers, verdict, report
| # | measure | bar |
|---|---|---|
| O1 | owner picked a candidate in Part 0 | required to continue |
| S1 | checker pass of the delivered set | 100 % |
| S2 | tries per accepted sheet (I) / render setups (B) | ≤ 2.0 / ≤ 2 |
| R1 | readability, final round | all three bars |
| T1 | judge v2 + model score on 6 stills | reported; ≥ 4.0 preferred |
| P1 | 200 at 1× | ≥ 60 FPS |
| K1 | generations (or render minutes) per accepted figure; projection for +1 top (I: 24 sheets; B: 1 mesh) | reported |

**Verdict:** S1 + R1 + P1 → the route is the character standard (TEMPLATE v3 for I, v1 for B); route_p pawns and A/B archived under `art/character/archive/`. Otherwise the report names the failed attribute and the owner decides; no automatic further trial.

`docs/v8/arttrial5/TRIAL5_REPORT.md` (Korean): 판정 · Part 0 결과와 오너 선택 · 세트 구성(룩·시트·시도) · 절단/정규화 기록(분할된 컴포넌트, 스케일) · 틴트 결과 · 구조 검사 · 리뷰어 라운드별 원문 요약 · 스타일 점수 · 군중 FPS · 비용 투영(K1) · 변경된 파일 목록 · 제한사항 · 오너 결정 필요. Push; `git ls-remote origin v8/art-trial5`.
