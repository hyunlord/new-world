# GOAL: WorldSim — ART-POLISH: close what ENV1 and STRUCT left open before M-ART2 installs the art — the wolf's poses, the inspector portraits, items that read at ½× zoom, and the wall-cap fix the owner agreed to — with instruments that can actually fail.

Repository `hyunlord/new-world`. Base: **`v8/art-struct` = `b6baa73f1f64983fee6aea08ed783a038ba2a4ce`**. New branch **`v8/art-polish`** (`v8-artpolish(pN): …`).

Executed by **Codex (GPT Astra)** in `/Users/rexxa/work/new-world-art-prod` (image tool, Python). Only `art/**`, `docs/v8/artpolish/**`. No game code, no merge. Target 4 hours, hard stop 6. **Generation budget ≤ 100 calls** (scorecards count).

## Why
- **Owner decision 2026-10-24:** the STRUCT direction is adopted — ground from above, everything standing in the same front 3/4, walls/hill faces/water/edges drawn by code — **with wall caps made dark and nearly patternless** (the STRUCT caps read as wooden decks and paved floors; in RimWorld the cap is a dark flat band and the face carries the material). M-ART2 will implement this in the game renderer; this pass fixes the parameters on the compositor so M-ART2 copies numbers, not intentions.
- **Wolf:** ENV1's wolf scored 5/5/5/5 on style but failed pose reading (stand read as sitting, front walk read as standing). Deer passed.
- **Portraits:** 33/34 numeric and 30/34 model passes, yet 0/34 accepted, because the style instruments failed their negative controls (the model gave a photo 4/4/4/4; numeric ranges admitted painterly and photo). An instrument that cannot reject a photograph cannot accept a portrait either. This pass stops using those instruments as gates for portraits and animals and replaces them with a test that can fail: **identity matching** by blind reviewers, plus structure, plus the owner's eye.
- **Items:** at 32 px reviewers named the exact kind 19/44 times: firewood vs log, ground vs stone axe, cured vs raw hide, raw vs fired pot, wraps and cloaks as garments, flint vs stone all confused. At ½× zoom the player must tell them apart without hovering.

# 0. RULES
DECISIONS in `docs/v8/artpolish/POLISH_DECISIONS.md`; data `/Users/rexxa/work/artpolish-data/`; ≤ 1.5 MB per committed file; no pixel surgery, no quantization; code-drawn changes as parameters with tests. **Declared instrument change (D1):** for portraits and animals, judge v2 numeric and the image-model style scorecard are **reported, not gating** (they failed negative controls in ENV1 — cite the files); acceptance = structure + the blind tests below + presence on the owner board. Item style keeps judge v2 as a gate (the item family calibrated). Drop order past the hard stop: portrait regeneration (keep the board of existing candidates) → item 64-px test → wolf N/A. **Never drop:** the wall-cap parameters and re-render (§1), the wolf sheet and its pose test (§2), the item redesign and its 32-px test (§4), the owner boards, the report.

# 1. PART 0 — Wall caps and the dug pocket (compositor)
1. Commit this goal as `docs/v8/artpolish/POLISH_GOAL.md`; write D1 and D2 (the owner's STRUCT decision quoted).
2. `art/compositor/walls.py` gains `cap_style = "dark"` (default) with parameters in `art/compositor/wall_params.json`: cap = the cap strip blurred (box 9 px at 1×), value × 0.45, saturation × 0.40, then a 2-px ink line (`#141210`, 80 % alpha) on every cap edge that borders a non-wall tile, and a 1-px lighter highlight (value + 0.08) on the cap's south edge where a face hangs below; **end pieces drawn only at face ends**, never on caps; inner corners need no piece (the dark cap is continuous). Stone and log caps differ only by hue (from the palette), not texture.
3. **Dug pocket:** an undug hill tile whose south neighbour is dug/passable draws the rock face; a dug tile whose **north** neighbour is undug hill draws nothing extra (the face above already covers the transition); the pocket's floor is the dug-rock floor with the 12-px blend only against other ground, never against hill faces. Tests on synthetic layouts.
4. Re-render STRUCT's owner board 1 and the closeups with dark caps → `art/boards/polish_walls_owner.png` (STRUCT caps vs dark caps, same crop, 1× and 2×) and `polish_closeups_2x.png`. Add a strip of three cap darkness settings (value × 0.35 / 0.45 / 0.60) for the owner to choose from.

# 2. PART 1 — Wolf
- New sheet(s), ≤ 3 tries, same TEMPLATE_A1 and slicing as ENV1. **Facing for animals:** S is a **front 3/4 turned 35° toward the viewer's left** (a quadruped seen head-on hides its legs); E unchanged. Pose cues in the prompt: stand — four straight legs, head level, tail low, weight even; walk — diagonal leg pairs apart, one forepaw lifted, head forward; run — extended gallop, all legs off line; attack — lunge, forelegs reaching, jaws open; dead — on its side, legs out. The deer keeps ENV1's sheet unless its S facing changes under the same rule (then regenerate it once, ≤ 2 tries, so both animals share the facing rule).
- **Blind test:** two fresh reviewers, 10 wolf + 10 deer frames each at 1× and 2× on calm meadow, no key: species and pose. Bars: species ≥ 19/20 each, pose ≥ 16/20 each at 2× and ≥ 14/20 at 1×. Miss → one regeneration round within the try limit, two new reviewers.

# 3. PART 2 — Portraits
- Keep ENV1's 34 candidates; **regenerate only** the 4 that failed the head-margin check, the 1 with an invented hairpin, and the 4 with medium/line scorecard fails (≤ 2 tries each), attaching the look's `S/idle.png` and an 8× nearest crop of its head as identity references, `anchors/portrait.png` for style.
- **Identity test (gating):** two fresh reviewers; for each of the 34 portraits, show the portrait beside **four** route I idle sprites at 2× (the right look + three from the same age/sex with different top or hair, chosen by a fixed rule), ask which sprite is this person. Bar: ≥ 30/34 correct per reviewer. Structure: head inside the canvas with ≥ 2 px margin, face inside the fixed oval (ENV1's method).
- Board `art/boards/portraits_owner.png`: all 34 at 128 and 64 px beside their sprite.

# 4. PART 3 — Items that read at ½×
- **Redesign rule** (`art/style05/ITEMS.md`): each kind gets one **silhouette-level** distinguishing feature visible at 32 px, stated before generation:

| kind | feature |
|---|---|
| Log (pile) | round cut ends facing the viewer, bark sides; large pile = pyramid of round ends (not planks) |
| Firewood | split wedges in a criss-cross bundle, pale split faces, no round ends |
| StoneAxe | grey chipped head lashed to the haft with visible binding |
| GroundAxe | smooth pale-green polished head with a bright highlight, slimmer haft, no binding bulge |
| FlintBlade | dark glossy blade with a white edge |
| Flint nodule / pile | black glossy lumps with white chalky crust |
| Stone pile | grey rounded cobbles |
| Hide | irregular raw hide with hair texture and darker edges |
| CuredHide | cream rectangle, smooth, stitched/laced holes along the edge |
| WolfHide | grey fur with the head and tail outline |
| HideWrap | a folded garment with a tie cord, one-shoulder shape |
| FurCloak | a draped cloak with a wide fur collar, dark brown |
| Clay | wet red-brown slab with finger marks |
| RawPot | grey matte unfired pot |
| ClayPot | orange fired pot with a dark rim (empty / filled with water) |
| Spear | long shaft (≥ 2× an axe's length) with a stone point |
| Meat | red cut with a bone |

- Regenerate the kinds that failed exact identification in ENV1 (firewood, ground axe, cured hide, hide wrap, fur cloak, wolf hide, raw pot, flint nodule, flint pile, log pile medium and large, flint blade, clay) — ≤ 3 tries each; judge v2 item family gates style; keep passing ENV1 items.
- **Blind test:** two fresh reviewers, all 22 item images at 32 px and at 64 px on the calm ground, no key, first name only (ENV1's scoring rules). Bars at 32 px: exact kind ≥ 17/22 per reviewer, and these pairs both correct by both reviewers: log/firewood, stone/ground axe, hide/cured hide, raw/fired pot, hide wrap/fur cloak recognised as garments. At 64 px: exact ≥ 20/22. Miss → one regeneration round on the misread kinds, two new reviewers.
- Note for M-ART2 (report): hover labels and the inspector name the item regardless; the bars are about glance-reading the colony.

# 5. PART 4 — Owner boards and report
- Boards: `polish_walls_owner.png` (+ the cap darkness strip), `polish_closeups_2x.png`, `animals_owner.png` (wolf and deer, all frames, 1× and 2×, on meadow and forest floor), `portraits_owner.png`, `items_owner.png` (22 items at 32 / 64 px with names).
- `docs/v8/artpolish/POLISH_REPORT.md` (Korean): 판정(오너 판단 대기) · 계측기 변경(D1)과 근거 · 벽 윗면 파라미터(수치)와 판 굴 규칙 · 늑대(시도, 자세 시험 라운드별) · 초상(재생성, 동일인 시험) · 아이템(설계표, 32/64px 시험, 혼동 쌍) · 보드 목록 · 변경된 파일 · 제한사항 · **M-ART2에 넘길 것**(wall_params.json 값, 동물 S 방향 규칙, 초상·아이템·동물 경로표) · 오너 결정 필요(윗면 밝기 셋 중 선택, 초상 채택). Push; `git ls-remote origin v8/art-polish`.
