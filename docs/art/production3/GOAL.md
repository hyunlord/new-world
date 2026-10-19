# GOAL: WorldSim — Art Pass 3: calm the camp, finish the villagers, show old age and seasons

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` and `codex/art-production-2026-10`, both at `b2f0ce15d069aee6dd3e09cb1248a2608cbb7600` (draft PR #511, stacked on the simulation PR #510).
The simulation's Pass 4 is finished: `codex/revitalize-2026-10` and `-wip` are both at `5cbcbc4f9bd02bf3f00695bae810368b93fe6300` (elders, old-age deaths, seasons, purposeful foraging; the live game builds `game_v10`).
This run is executed by **Codex**, unattended, on the Mac.

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production3/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production3/GOAL.md`, commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.** Fix once, re-review once; then accept, record, move on.
5. **On failure: work around, note it, continue.** Partial delivery with an honest report beats a stalled run.
6. **Time rule:** record start time and each Part's start and end. Past **8 hours**, finish the current step, push, go to Part 7.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** fill generation (7.4) → inspector age line (section 8, item 3) → the d40 capture (9.1). Never drop sections 4, 5, 6, 7.1–7.3, 10 or 11.
9. **Image budget: at most 12 new generations** (the 18 left after Production 1, minus the 6 Art Pass 2 used), failures included. On a quota or rate-limit message, stop generating and continue.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a QA threshold of section 10, keep rejected evidence. A scope correction (10.1) is recorded as such, with the old and new scope side by side.

---

# ⚠️ 1. WHERE AND HOW

- Mac only, the existing clone `/Users/rexxa/work/new-world-art-prod`. Raw generations: `/Users/rexxa/work/art-production-raw/` (Production 1) and `/Users/rexxa/work/art-production2-raw/` (Art Pass 2); new raws go to `/Users/rexxa/work/art-production3-raw/<fill_id>/attempt_<n>.png`. Raws never enter git.
- Never touch `/Users/rexxa/work/new-world-pass4`, `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Windowed Godot only through `tools/art_pipeline/with_window_lock.sh <seconds> <command...>` in one command. If the session cannot open windows (a background tmux session), run the same single command in the GUI session with `launchctl asuser $(id -u) …`, as the simulation Pass 4 did, still inside the lock. Headless Godot needs no lock; other limits use `tools/art_pipeline/run_bounded.sh`.
- Image generation: the built-in `$imagegen` skill (gpt-image-2), magenta background, returns 1254 × 1254. Prompts never name existing games, studios, artists or characters.

---

# 2. GIT

- `git fetch origin && git checkout codex/art-production-2026-10-wip`; it must equal `origin/codex/art-production-2026-10-wip` (`b2f0ce15`). Local-only commits go to a backup branch `art-p3-local-backup` first.
- **Part 0 merges the simulation Pass 4 head** `5cbcbc4f` with `git merge --no-ff 5cbcbc4f -m "art-p3(p0): bring in simulation pass 4"`. Art Pass 2's local compatibility merge with `7a8ae053` was clean. If there are conflicts: under `rust/crates/sim-*/src/**`, `docs/revitalize/**` and `tools/revitalize/**` take the simulation's side; in Rust static-source tests that pin GDScript, keep the art assertions and the simulation's new assertions both; record every resolved file.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 7) is green. Never force-push. After every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `art-p3(pN): <what changed>`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (ledger row each, `docs/art/TEST_LEDGER.md` under `## Art Pass 3`, mutation audit for the changed assertions only).
- **Must not change:** anything else under `rust/`, `docs/revitalize/**`, `tools/revitalize/**`, `tools/harness/**`, `.claude/**`, `.harness/audit|baseline|reviews/**`.

---

# 3. WHY THIS PASS (planner's review of b2f0ce15)

Art Pass 2 turned the screen into a game for the first time: real trees, shelters, hearth, bushes and villagers; forest coverage 27 %; blind activity identification rose from 4/24 to 16/24 at 4×. What remains is mostly placement and two pipeline bugs, plus two gaps in the planner's own instructions:

1. **The camp drowns in rocks.** The 8 villagers below contrast 0.05 in `a_d2_1200_F2` all have grey or white hair and most stand in front of grey boulders; A5 fell from median 0.238 to 0.106. `nature_renderer.gd` places rocks (`code == 3 and h % 100 < 25`, every `code == 2` tile) and reeds **outside** the clearing check — the planner's clearing rule named only trees, shrubs and logs.
2. **Grey and white hair are not reserved for elders.** `AgentActivity.elder_look` limits non-elders to hair 0–2 only while an elder is present (the planner's rule), so 15 of 25 villagers are grey or white before the first elder appears, and every adult's hair changes the moment one does.
3. **The ground is busy again.** `terrain_renderer.gd:_refresh_decals` puts a decal on 30 % of grass and 20 % of forest tiles (585 on the map), excluding only the object tiles themselves; leaves, flowers and mushrooms up to 12 px are as large as a 28-px villager's head.
4. **Idle villagers show their backs.** Idle keeps the last movement facing; at noon most idle villagers face N or W and read as hair blobs.
5. **Region maps break on non-standing poses.** 34 of 49 new-fill rejections and 11 of 41 raw rejections are `region_map_wrong`: `regions.classify` splits hair and clothing by fixed height bands, which fail for bending, kneeling and carrying poses. 28 raw rejections are `artefact` (isolated stray texels such as a blue texel in a crown) that a despeckle step would remove.
6. **Carry is a placeholder.** 26 carry frames are idle sprites with a floating basket icon; build was misread as walk or sit in all three line-up crops.
7. **A1 fails on the calibration line-up, not on the game.** The two calibration frames have 61 colours (< 64) because the line-up is deliberately plain.

The simulation now really produces elders (`life_stages == 2`), old-age deaths (`deaths_old_age`, reason 3) and seasons (`season`, `day_of_season`, `days_per_season`); Art Pass 2's feature-detected code has only been seen against a temporary merge. Note for staging: in `game_v10` about 80 % of villagers are idle at noon (simulation W8), so a calm, readable camp at midday matters more than ever.

---

# 4. PART 0 — Baseline

1. Record start time, `ART3_BASE_SHA`, `uname -a`. Commit `GOAL.md`.
2. Merge `5cbcbc4f` (section 2). Full gate: `GATE_OUTPUT_DIR=docs/art/production3/gates/p0 bash tools/art_pipeline/gate.sh p0`. Fix merge fallout in art-owned paths only and record it.
3. Commit, push WIP.

---

# 5. PART 1 — Calm the camp and the ground

## 5.1 Clearings for everything (`nature_renderer.gd`, `nature_layout.gd`)
Every nature object (trees, shrubs, logs, saplings, **rocks, rock clusters, outcrops, reeds, cattails**) respects the clearings: nothing within Chebyshev 4 of a building, construction site, hearth, settlement formation tile or berry bush. Move the rock and reed placement inside the clearing check.

## 5.2 Rocks in outcrops, not gravel
Hill tiles (`code == 3`): rock chance from 25 % to a clustered 4–8 % (use the grove value noise of `nature_layout.gd` on its own salt so rocks gather in outcrops); mountain tiles (`code == 2`): from every tile to a clustered 40–50 %. At most one rock object per tile.

## 5.3 Decals: few, small, in patches (`terrain_renderer.gd`)
- Density: at most 4 % of grass tiles and 3 % of forest tiles carry a decal, placed in patches (value noise, own salt) instead of uniformly; river lily pads unchanged.
- No decal within Chebyshev 3 of a building, construction site, hearth, formation tile or bush.
- Size: re-fit every N3 decal with v4 to at most **8 × 8** native texels (leaves at most 6 × 6); drop a decal whose fitted art fails review rather than shrinking it into noise.

## 5.4 Tests
Extend `test_nature_density.gd` and `test_nature_clearance.gd`: zero rocks or reeds inside the clearings, hill rock share 4–8 %, mountain share 40–50 %, decal density limits, no decal inside the camp radius, decal native size ≤ 8 × 8, placement identical after a reload. Commit, push WIP.

---

# 6. PART 2 — Readable villagers

## 6.1 Hair means age
`AgentActivity.elder_look`: elders (`life_stages == 2`) always get hair variant 3 (grey) or 4 (white) by `ArtHash.mix(agent_id) % 2`; every other adult and child always gets 0–2. Remove the `elders_present` condition; nobody's hair changes when the first elder appears.

## 6.2 Face the viewer when idle
After a villager has been idle for 1.0 s of render time, its facing turns to S; `talk` keeps facing its partner; any movement restores movement facing at once.

## 6.3 Idle at camp sits
Between 06:00 and 23:00, an idle adult or elder within Chebyshev 3 of the hearth or a shelter uses its sit frames (elders always sit when idle at camp; other adults sit when `ArtHash.mix(agent_id + day) % 3 != 0`, so a few stand); children idle at camp alternate idle and hop by the same hash. This is a pose choice for an idle agent, not new behaviour: the simulation state is unchanged and the inspector still says idle.

## 6.4 Tests
`test_elder_variant.gd` (no grey or white on non-elders with or without elders present; elders always grey or white), `test_idle_facing.gd`, `test_idle_camp_pose.gd`. Commit, push WIP.

---

# 7. PART 3 — Fix the two pipeline bugs, then re-review before generating

## 7.1 Region maps by colour, per body
For each body (adult_a, adult_b, child) build a colour → region table from its reference S idle, whose region map was approved in Art Pass 2 (majority region per palette colour; ambiguous colours recorded). Classify every frame of that body by exact colour lookup; a colour missing from the table takes the region of the nearest table colour if ΔE76 ≤ 8, else `other`. Keep the old height-band rules only as a fallback for colours that remain `other` in hair-height rows of standing frames. Write `tools/art_pipeline/regions_v2.py` with tests (a bending and a kneeling synthetic frame classify like the reference; a colour unique to the frame becomes `other`).

## 7.2 Despeckle
Before review, a texel whose colour occurs at most twice in the sprite and has no 8-neighbour within ΔE76 12 is replaced by the most common 4-neighbour colour (transparent if most 4-neighbours are transparent). Record the count per cell; a cell needing more than 3 % changes is not speckled but broken, and stays rejected. Tests for the stray blue texel case and for an intentional 1-texel eye.

## 7.3 Re-review
Run v4 + region maps v2 + despeckle on every slot Art Pass 2 left unaccepted (41 raw rejections, 49 fill rejections, the 54 slots without a coherent guard-passing candidate, the 14 pending cycles) and review them with the Art Pass 2 rubric (`docs/art/production2/REVIEW_RUBRIC.md`) in a fresh reviewer session, two rounds maximum. Add one rubric line: **the action must read at 2× without the icon** (carry shows the load, build shows the tool, drink shows hands at the mouth). Install accepted cells exactly as Art Pass 2 installs (manifest, atlas, region atlases).

## 7.4 Fill generation (≤ 12 calls)
Only for what is still missing after 7.3, in this order: carry (adult_a, adult_b, child; S, E, N; 4 frames each), build and drink (every body and view), asleep, then anything else. One body per sheet; references: D3 style sheet and the body's reference idle at 8× nearest. No texel-count or palette wording. **Checkpoint after 4 calls:** fewer than half of the attempted cells accepted → stop generating and record why.

## 7.5 Fallbacks
Unchanged from Art Pass 2, except: the idle + floating basket icon is no longer allowed for carry when any real carry frame of that body exists in any view (mirror or borrow the body's other carry view instead). Commit, push WIP.

---

# 8. PART 4 — Old age and seasons, now with real data

1. With Pass 4 merged, confirm on the live game that the Art Pass 2 consumers work: elder hair (6.1), HUD death count including `deaths_old_age`, death news per cause including old age, `death_viz_renderer` grey for reason 3, the season label (`HUD_SEASON_DAY_FMT`, day 1-based), and the season tint on ground and nature only.
2. Fix whatever does not; record what was wrong.
3. **Inspector age line:** `agent_inspector_panel.gd` shows the life stage and age from `life_stages` and `ages_days` (skip the line when `ages_days` is −1 or absent). New keys:

| key | en | ko |
|---|---|---|
| `INSPECTOR_AGE_FMT` | Age %d days | 나이 %d일 |
| `INSPECTOR_STAGE_CHILD` | Child | 아이 |
| `INSPECTOR_STAGE_ADULT` | Adult | 어른 |
| `INSPECTOR_STAGE_ELDER` | Elder | 노인 |

Add them to `localization/fluent/{en,ko}/messages.ftl` and recompile with the same compiler and registry step Art Pass 2 used (check duplicates first). Godot test for the line with and without the keys. Commit, push WIP.

---

# 9. PART 5 — Long-run captures

## 9.1 Plan
Extend `scripts/test/art/capture_plans.gd` with a plan `art_long` (and accept it in `capture_framings.gd` and `capture_art.sh`): display days 10 (summer), 18 (autumn) and 24 (winter) at 12:00 with framing F2, day 24 at 23:00 with F2N, day 24 at 12:00 with F1, and day 40 at 12:00 with F2 (second spring: elders and their replacements). Seasons are 7 display days each starting on day 1, so these land in summer, autumn, winter and spring. Record `tallies`, `time_snapshot` and `life_stages` counts in each frame's metadata (the existing schema already carries the first two).

## 9.2 Run
`bash tools/art_pipeline/with_window_lock.sh 2400 bash tools/art_pipeline/capture_art.sh art docs/art/production3/shots`, then the same with `art_long`, then `calibration` into `docs/art/production3/calibration`.

---

# 10. PART 6 — Measure

## 10.1 A1 scope correction
A1 (frame not degenerate: standard deviation and colour count) applies to gameplay frames only; the calibration line-up frames are judged by A2, A3, RAMP and A11. Record the change in `ingame_qa.md` with the Art Pass 2 scope beside it. Thresholds unchanged.

## 10.2 Checks (thresholds unchanged unless new)
- A1–A14 as in Art Pass 2, A5 measured on **both** `a_d2_1200_F2` and the day-24 winter noon frame (median ≥ 0.15, minimum ≥ 0.10, no unmeasurable villager in each).
- A11: the line-up covers walk, carry, gather, build, eat, drink, sit, talk and child; 4× ≥ 0.8 and 2× ≥ 0.6. Carry crops must use real carry frames wherever 7.5 allows.
- **A15 camp and ground:** zero rocks, reeds or decals inside the clearings; decal density within 5.3; every installed decal ≤ 8 × 8; hill rock share 4–8 %; mountain 40–50 %.
- **A16 age and seasons on screen:** the HUD season label is present in the d10, d18, d24 and d40 frames with the right season; the d24 ground is cooler than d10 (mean blue/red ratio of terrain pixels higher by ≥ 3 %) while villagers' pixel colours match their ramps (RAMP check on that frame); in the d40 frame every visible villager with `life_stages == 2` has hair variant 3 or 4 and no other villager does; the HUD death number equals the sum of the four death tallies in the frame metadata.

## 10.3 Before/after
`docs/art/production3/before_after.png` against Art Pass 2 at the same framings, plus a season strip (d10, d18, d24, d40 side by side); noon and evening GIFs (40 frames, ≤ 8 MB each). Commit, push WIP.

---

# 11. PART 7 — Final gate, compatibility, report, PR

1. Full gate on the final WIP head: `GATE_OUTPUT_DIR=docs/art/production3/gates/final bash tools/art_pipeline/gate.sh final`. Fast-forward `codex/art-production-2026-10` if green; otherwise say exactly what is red.
2. If `origin/codex/revitalize-2026-10` moved past `5cbcbc4f`, repeat Art Pass 2's local, unpushed compatibility merge and record it.
3. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production3/**` ≤ 40 MB.
4. `docs/art/production3/REPORT.md` in Korean in the section 13 format, answering every item of section 12 with its evidence.
5. Update PR #511's description with an Art Pass 3 section (`gh pr edit 511 --body-file …`; say so if `gh` is not authenticated). Keep it a draft.

---

# 12. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 화덕과 천막 주변이 비어 있고, 바위는 언덕과 산에 무리 지어 있다. 캠프 안에 바위 더미가 없다.
2. 땅의 꽃·버섯·낙엽이 드문드문 무리로만 보이고, 주민 머리보다 작다.
3. 회색·흰 머리는 노인만 있다. 첫 노인이 생겨도 다른 주민의 머리색이 바뀌지 않는다.
4. 가만히 있는 주민은 1초 뒤 정면을 본다. 낮에 캠프에서 쉬는 어른과 노인은 대부분 앉아 있다.
5. 바구니는 식량을 들고 걸을 때 몸에 들려 보이고, 머리 위에 뜬 바구니 아이콘이 없다. 건설은 돌망치, 마시기는 입에 댄 손으로 알아볼 수 있다.
6. 여름, 가을, 겨울, 봄에 땅과 나무 색이 달라지고, 상단에 계절과 날짜가 나온다. 주민 색은 계절에 따라 바뀌지 않는다.
7. 노인이 늙어 죽으면 소식창에 노환 소식이 나오고, 상단 사망 수가 늘어난다.
8. 주민을 클릭하면 정보창에 아이·어른·노인 구분과 나이가 나온다.
9. 콘솔에 SCRIPT ERROR가 없고 프레임 시간이 Art Pass 2와 비슷하다.

---

# 13. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
빈터와 바위·장식 배치, 머리색 규칙, 대기 방향과 앉기, 색 기반 영역 지도, 잡티 제거, 재검토와 생성, 계절·노인·노환 확인

### 되살린 결과
재검토로 새로 승인된 칸 수(원본·신규별), 새 생성 횟수(최대 12)와 승인 수, 남은 대체(종류별), 운반 프레임 상태

### 실제 게임 화면 검증
A1~A16 표(수치), A1 범위 수정 전후, 블라인드 식별 혼동 행렬, 계절 스트립·before/after·GIF 경로

### 시뮬레이션 작업과의 호환
Pass 4 병합 결과(충돌과 해결), 노인·노환·계절이 실제 데이터로 보이는지

### 변경된 파일 목록
(커밋별)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test 수치, clippy, strict, bridge, smoke, Python, Godot 아트 장면 테스트)

### 정직한 판단
처음 보는 사람이 이 화면을 완성도 있는 픽셀 아트 게임으로 볼까? 아직 아닌 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, whether PR #511's description was updated, image calls used, and the paths of `before_after.png`, the season strip, the two GIFs and the review sheets.