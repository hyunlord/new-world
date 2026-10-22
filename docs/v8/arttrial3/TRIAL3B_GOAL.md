# GOAL: WorldSim — ART-TRIAL3b: route B only — a rigged 3D stone-age figure rendered through an ink shader into the TRIAL3 template, checked by the same structural checker, judged on GIFs; plus the two contact fixes the owner saw in route A (floating feet, stub limbs)

Repository `hyunlord/new-world`. Base: **`v8/art-trial3` = `352f4101351d3d920d958aa9ba019b3e8a5623da`**. Continue on the same branch (`v8-arttrial3b(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod`. The owner has (re)done `higgsfield auth login` and selected a workspace; **Part 0 verifies this first** and stops with a precise report if any Higgsfield command still fails (which command, exit code, stderr). Blender 4.x and Godot 4.6 as TRIAL3. Only `art/**`, `docs/v8/arttrial3/**`. No game code, no merge. Target 6 hours, hard stop 9.

## Why
TRIAL3 proved the template + layer contract + checker (87/87 layers, 603 combinations, no holes or floating parts). Route A then showed the limit of 2D generation under guides: grey wash, blurred lines, stiff stubby figures, and feet that seem to hover above their shadow (style 2.5/5). Route B — one 3D model rendered from the game camera — never ran because the CLI had no workspace. B is the route whose consistency comes from geometry, not from a model's goodwill, so it gets its own run. If B fails too, the pre-registered recommendation (a 2D artist against the template) stands.

# 0. RULES
As TRIAL3 §0 (no owner questions; data in `/Users/rexxa/work/arttrial3-data/b/`; no quantization; try limits: **3 generation setups** for the model, **3 render setups** for the look; stop and report when exhausted). Parts: 0 = §1 · 1 = §2 · 2 = §3 · 3 = §4 · 4 = §5. Drop order past 9 h: N direction → second outfit → crowd FPS. Never drop: Part 0, one body S+E through the checker, the GIF test, the numbers.

# 1. PART 0 — Tool check and two template amendments
1. `higgsfield --version`; `higgsfield auth status` (or the equivalent); list models/skills; one trivial generation (a 64×64 test texture) to prove the account works and record the credit cost. Any failure → report and stop.
2. **Contact amendment** to `art/character/TEMPLATE.md` + checker: the contact shadow's **centre is on the foot line** (y = 88), not below it; feet must **overlap** the shadow ellipse by ≥ 3 px; the lowest opaque body pixel must be within +0/−2 px of the foot line (no hovering). Add the check `contact_ok` and a synthetic test (a figure 4 px above the line fails).
3. **Proportion amendment:** hands and feet must be distinct shapes (hand width ≥ 6 px at 1×, foot length ≥ 8 px); limbs' minimum width 4 px; the head's alpha box must fill ≥ 70 % of the head box. Add `limb_ok` and a test. Re-run the checker on route A's 87 layers and report how many now fail (expected: many; this is information, A is not reworked).

# 2. PART 1 — The model (Higgsfield game-generation skill; Meshy only if the skill cannot output a rigged humanoid)
- Prompt: stylized stone-age adult male, proportions of TEMPLATE (head ≈ 30 % of height, broad shoulders, clear hands and feet), flat neutral grey skin material, plain loincloth; **two outfits** as separate meshes or material slots (hide wrap, fur cloak); **two hair meshes** (short, long tied); props: stone axe (with a grip empty), log bundle. Export rigged GLB(s). ≤ 3 setups.
- Clips from the preset library: idle, walk, two-hand chop/swing, carry walk, lie down/sleep. If a clip is missing, author it in Blender with ≤ 6 keyframes and say so.
- Record every call, credit, and what came back (`B_MODEL_LOG.md`).

# 3. PART 2 — Render (`art/pipeline3/render_b.py`, Blender headless)
- Import the GLB; orthographic camera for **S** (front, 20° down), **E** (model rotated 90°), **N** (180°); framing so that feet sit on the foot line and height = 56 px at 1× (render at 4× = 256×384, then Lanczos to 64×96); sleep rendered on the 96×64 canvas.
- **Look, in three steps, judged before animating:** (1) Freestyle line, 2 px at 4×, colour `#141210`, crease angle so inner lines appear on cloth folds and the face; (2) 3-band toon shading with the 05 wash: paper-warm base tones from `palette_05.json`, shadows as a darker flat band, no gradients; (3) compositor paper grain at 8 %. Render the idle S still with each setup, run judge v2 (person family) and your own one-line reading; pick the best of ≤ 3 setups and freeze it.
- **Passes by object id → the layer contract:** body (skin + loincloth), top, hair, item, each as its own RGBA + mask (region ids from material ids: skin 1, hair 2, cloth primary 3, cloth secondary 4, item 5). The contact shadow is **not** rendered (the lab draws it on the foot line).
- Frames: `idle`, `walk_a/b` (the stride's two extremes), `work_a/b` (swing up / impact), `sleep`; for S, E, N. The fixed-upper-body rule applies to the **overlays**, not to the 3D body: render overlays from the idle pose only, and verify with the checker that they fit every frame's torso (if a cloak clips the swinging arm, widen the cloak's alpha margin, not the body).
- Run `check_layer.py` / `check_set.py` (with the §1 amendments) on everything. Fix camera/framing/shader, never the pixels.

# 4. PART 3 — Lab, GIFs, tests (as TRIAL3 §5, route B only)
- Character lab page for B: all combinations (2 tops × 2 hair × 2 items × 3 dirs), six frames looping, on calm meadow and forest floor, 1× and 2×; GIFs `b_01..12_{1x,2x}.gif`; boards `lab_b_{1x,2x}.png`, and a **side-by-side with A** `compare_AB_{1x,2x}.png` (A's existing stills in the A column).
- **Readability on real playback:** two fresh reviewers watch the GIFs in a browser tab you open for them (Playwright screenshot sequence at 8 fps of the rendered page counts as "watched in real time"; store the sequence as evidence); same questions and bars as TRIAL3 §5.2 (activity ≥ 10/12 at 2×, ≥ 8/12 at 1×; attributes ≥ 30/36). One revision round (shader/camera only), then rerun with new reviewers.
- Style: judge v2 on 6 stills + the model judge's four scores; report.
- Crowd: 200 at 1× in the lab, 60 s, FPS (droppable).

# 5. PART 4 — Numbers and report
| # | measure | bar |
|---|---|---|
| S1 | structural pass of accepted layers (incl. contact_ok, limb_ok) | 100 % |
| S2 | generation setups used / render setups used | ≤ 3 / ≤ 3 |
| R1 | GIF readability | §4 bars |
| T1 | style score | reported; ≥ 4.0 preferred |
| P1 | 200 at 1× | ≥ 60 FPS |
| K1 | credits + minutes per accepted layer; projection to 4 bodies × full wardrobe | reported |

Verdict rule as TRIAL3 §6 applied to B alone: S1 + R1 + P1 → **B viable**; else the artist recommendation stands, with B's evidence attached. `docs/v8/arttrial3/TRIAL3B_REPORT.md` (Korean, TRIAL3's §8 format plus "A 재검사 결과" for the §1 amendments). Push; `git ls-remote origin v8/art-trial3`.
