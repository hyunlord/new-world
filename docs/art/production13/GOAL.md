# GOAL: WorldSim — Art Pass 13: measure the frame budget for real, ship the declump kernel if it pays, skip unchanged agents' writes, activity icons that read with a legend, and readability judged by what can actually be measured (on `game_v26`)

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` = `codex/art-production-2026-10` = `57ae7ac40abde7d24369606c9250d3e06e0f18d6` (Art Pass 12, PARTIAL; draft PR #511).
This run is executed by **Codex**, unattended, on the Mac, in `/Users/rexxa/work/new-world-art-prod`. **The owner launches it after Pass 21 has finished** (it has: `codex/revitalize-2026-10` = `354e4a13b21a20b28c65d84fab103c40b5143fa0`). No DGX. The world-model run WM-5 may be active on the DGX; its Mac session only edits files and runs git.

**Simulation to merge:** `354e4a13`, Pass 21. It brings `game_v26`, which rebalances `game_v25`:
- estranged villages avoid each other at every food selector;
- daily caps on the conflict each village pair can lose;
- faster drift, so reconciliation happens (typically 10–15 days after an estrangement);
- more aid.

There are no new getters and no new chronicle kinds (0–21). The live game becomes `game_v26`. A dry `git merge-tree` of `354e4a13` against `57ae7ac4` was clean (2026-10-19).

**Why Art Pass 12 measured nothing.** All 21 + 21 admission holds were refused. The Mac's load was 5.1–5.2, but two foreign `python3` processes (`/Users/rexxa/github/make-meme/.venv/bin/python3`, the owner's other project) each ran at ≈ 100 % CPU. That failed Art Pass 12's "no foreign process > 50 %" rule. The three attempts per item were also spent within seconds of each other (e.g. W500 10:30:48–52). Both rules were the planner's: too strict for a 14-core machine running a single-threaded frame loop, and with no spacing. Art Pass 13 replaces them with a **measured** interference test (1.2) and spaced attempts.

**Five Parts, in this order:**
- **(0)** Merge, contract, a calibrated release baseline and the running diagnosis that has never been taken.
- **(1)** The frame budget: measure and decide the declump kernel; implement and measure the conservative unchanged-agent write skip.
- **(2)** Activity icons redesigned for legibility, an in-world tooltip, and a two-stage blind test (icon sheet, then world crops with a legend).
- **(3)** Readability judged correctly: a mechanical glyph-distinctness test, plus a fact-only reading bar; Art Pass 12's open life-text items.
- **(4)** After-measurements and verdicts.
- **(5)** Review, gate, report.

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Record decisions in `docs/art/production13/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production13/GOAL.md`. Commit it in Part 0. Re-read the current Part's section at every Part start and after any compaction.
3. **Owner feedback:** if `docs/art/production13/OWNER_FEEDBACK.md` exists in the working tree at launch (the owner may write it after playing), read it in Part 0. Treat each item as an additional Part 2 or Part 3 requirement, ranked above the droppable items. Record how each item was handled. Never edit that file.
4. **Parts:** 0 = section 4; 1 = section 5; 2 = section 6; 3 = section 7; 4 = section 8; 5 = section 9. Push after every Part.
5. **Review loops:** two rounds maximum (fresh reviewer that has not seen sections 3–7).
6. **On failure:** work around, note it, continue. Three strikes per failing target.
7. **Time rule.** Record times in `docs/art/production13/TASKS.md`.
   - Part 0 ends by **hour 3.5**. This includes up to 3 h of waiting for admission (1.2).
   - Part 1 ends at **hour 7**.
   - Part 4 starts no later than **hour 9**. The pushed build is then frozen for H1–H7. Unfinished never-drop items continue only as measurement or evidence on the frozen build.
   - Past **12 hours**: finish the current step, push, go to Part 5.
   - **Use the time.** Art Pass 12 ended after 2 h 41 min of a 12 h budget with every measurement unmade. While admission is refused, wait as 1.2 says and do Parts 2–3 work on the side branch (1.3); do not end the run early.
8. **Drop order:** the W-multi running node ablation → candidate (b) of 5.3 → the tooltip (6.3) → the life-text spot fixes of 7.3.
   - **Never drop:** Parts 0, 4 and 5; the calibrated baseline; the running diagnosis; the kernel decision (5.2); the icon redesign and both blind stages (6.2, 6.4); the glyph test (7.1).
9. **Image budget:** at most 4 generations with the image tool available to you, for icons only; failures count.
10. **Honesty rules:** never mark a failed check as passed, never lower a bar of section 8, keep rejected evidence, never re-interpret an earlier pass's verdict.
11. **This brief supersedes the repository's session-start instructions:** no hooks, no harness pipeline, never `lead/main` or `main`, never force-push.
12. **Commit subjects:** every commit subject starts `art-p13(pN): `. Art Pass 12 missed this twice.

---

# ⚠️ 1. WHERE AND HOW
## 1.1 Places and tools
- **Work and evidence:**
  - working tree: `/Users/rexxa/work/new-world-art-prod`;
  - evidence: `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production13-evidence`;
  - read-only data: `/Users/rexxa/work/p21-data/`.
- **Worktrees** (removed at the end): `/Users/rexxa/work/art-p13-base` (identity) and `/Users/rexxa/work/art-p13-measure` (release frame measurement; Art Pass 12's procedure — release dylib copied over the debug path in that worktree only, with a sha check per session).
- **Tool copies:** reuse Art Pass 12's tools (`tools/art_pipeline/*_p12*`, `scripts/test/art/*_p12*`), copied to `_p13` names writing under `docs/art/production13/**`. Keep the exact-clock identity harness, the running dump, the pan test, the alternating runner and validator, the section-timer chain, the real-input test and the declump fixture tools.
- **Environment:** `CARGO_BUILD_JOBS=6`; `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`; windowed Godot only through `with_window_lock.sh`.
- **Off limits:** other checkouts and processes you did not start. **Never stop, renice or signal a foreign process.**

## 1.2 Admission: interference measured, not guessed
**1. Calibration probe** (`tools/art_pipeline/cpu_probe_p13.py`). A single-threaded, deterministic CPU kernel of fixed work, sized for ≈ 2 s on a quiet core: pure-Python integer mixing plus a 256-KiB working-set sweep. It reports wall seconds.

**2. Reference.** In Part 0, before any Godot session, run the probe 15 times, 20 s apart. Reference R = the median of the 5 fastest runs. Record all 15 and the `ps` snapshot of each.

**3. Admission of a measured session.** Run the probe right before the session (P_before) and right after it (P_after). The session is **valid** iff all three hold:
- P_before ≤ 1.05 × R;
- P_after ≤ 1.05 × R;
- the 1-minute load is < 8.0 at both ends.

Keep the `ps` snapshots for the record only; foreign processes do not by themselves invalidate a session. The single-threaded frame loop's real competitor is a busy core, and the probe measures exactly that.

**4. Pairs.** An alternating A/B pair (5.1) is valid iff every probe inside it satisfies the rule above. Alternation cancels slow drift.

**5. Retries.**
- An invalid session is re-attempted **no sooner than 30 minutes later**, up to **6 attempts** per item.
- The total waiting for admission across the run is capped at **3 h**. Past the cap, remaining items are NOT_VERIFIED (contended), with every probe recorded.
- Never retry within the same minute.

**6. Quiet lock.** `/Users/rexxa/work/.mac_quiet.lock` works as in Art Pass 12 (one hold per measured session, ≤ 60 min). Log every probe in `docs/art/production13/perf/holds.md`.

## 1.3 Branch for side work
While Part 1 waits on admission, write Parts 2–3 on a local branch `art-p13-ui`. Do not run Godot or cargo during a hold. Merge it into WIP only after Part 1 ends, so every Part 1 candidate is judged on Part 0 plus accepted candidates alone.

---

# 2. GIT
## 2.1 Branches and scope
- **Start:** from `57ae7ac4` (must equal origin). Create backup branch `art-p13-local-backup`.
- **Commits:** WIP gets every commit. `codex/art-production-2026-10` moves only by fast-forward to a green-gated WIP commit. Run `git ls-remote` after every push.
- **Staging:** stage explicit paths only. Leave the local `.gitignore` / `.ignore` edits uncommitted.
- **May change:**
  - `scripts/**`, `scenes/**`, `shaders/**`, `assets/**` (icons only), `localization/**` (section 10's keys only), `project.godot`;
  - `docs/art/production13/**`, `docs/art/TEST_LEDGER.md`, `tools/art_pipeline/**`;
  - Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd` files;
  - the art-owned Rust: `rust/crates/sim-bridge/src/ffi/art_kernels.rs`, the `art_declump` wrapper and the art getters in `world_node.rs`, and their fixtures.
- **Simulation-owned, call only:** chronicle, relation and porter getters.
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/**`, earlier evidence, and `OWNER_FEEDBACK.md`.

## 2.2 Part 0 merge
1. **Merge:** `git merge --no-ff origin/codex/revitalize-2026-10 -m "art-p13(p0): bring in simulation pass 21"`. Simulation paths take the simulation's side; keep the art getters. Record every resolved file.
2. **Live game:** record that `init` and `reset_world` build `game_v26`.
3. **Contract check:** the relation and porter packed arrays are unchanged from `docs/art/production12/RELATIONS_CONTRACT.md`. Record any difference.

---

# 3. WHY THIS PASS (planner's review at `57ae7ac4`; verify each fact)
1. **No frame number exists for the current build.** Art Pass 12's numbers are all NOT_MEASURED. The last valid numbers are Art Pass 11's release measurements on `game_v24`/`game_v19` (`docs/art/production11/perf/after.md`):
   - W0 frozen 3.37 ms;
   - W1 running 2×: median 12.0 ms, p99 19.3 ms;
   - W200 frozen 16.3 ms, running 26.5 ms;
   - W500 frozen 28.9 ms, running 82.3 ms at 5 ticks per frame.

   The running frame has never been broken down.
2. **The kernel is proven exact but switched off.**
   - `P12_DECLUMP_KERNEL` = 1 enables `ArtDeclumpP12` (`agent_renderer.gd` L70, L724). The default is off.
   - It reproduces 4,000 real calls bit for bit, with zero pixel difference on six framings and on running and pan identity (`docs/art/production12/perf/declump-kernel/verification.md`).
   - Declump was the largest frozen section: 6.68 of 24.9 ms at W500.
3. **The unchanged-agent skip has an inventory and a recommended boundary** (`docs/art/production12/perf/memo-inventory.md`):
   - Update every timer and derive the full activity/pose/frame decision every frame.
   - Skip only the repeated property writes (frame, variant, metadata, `CarryLoad`, icon; ≈ renderer L226–266) when an owned exact key of their inputs is unchanged.
   - Keep the draw row, `_selection_index.observe`, the global geometry and the post-loop live.
   - Meta writes alone were 1.26 ms, variant 1.00, children 1.31, at W500 frozen.
4. **Activities still do not read.**
   - Art Pass 12's icon-including crops scored **3/10**, with 4 missing (`docs/art/production12/activity/blind-result.json`).
   - The icons are 16 × 16 one-colour silhouettes with a dark outline (`scripts/ui/activity_icons_p11.gd`): a flame-and-stick, a T-hammer, two heads, an arrow, a ball, a steaming cup, plus Craft's `U1_04`. They sit at `offset (−8, −16)`, scale 1, on the sprite (`agent_activity.gd` ≈ L194–200).
   - **No legend exists** and the reader had none. A player learns icons; the planner's protocol tested recognition without ever showing what the icons mean.
5. **The life reading measured the reader's OCR.**
   - Art Pass 12's attempt 2 found 12 name errors and 7 unverified build-count claims.
   - The reader is a vision model reading invented Korean names in a pixel font at 24 px, so there is no word prior to lean on.
   - The planner's bar "every statement correct" therefore mixed font legibility with model OCR.
   - The font probe found 티/디 differ by 24 px (regular) and 32 px (bold) at size 24 (`reading/font-result.json`), but only one pair was tested.
6. **What `game_v26` changes on screen** (Pass 21 evaluation, 19600–19631):
   - 13 of 19 multi-village worlds have deliveries (89 trips, 170 porters).
   - 21 estrangements, 8 reconciliations; 13 of 45 pairs estranged at day 120.
   - So relation lines are now mixed colours, and "made peace" entries (kind 21) occur.
   - Lowest seed with deliveries: **19603**.

---

# 4. PART 0 — Merge, gate, calibration, baseline, running diagnosis
4.1 **Merge and gate.**
- Backup; merge (2.2).
- Full gate: `GATE_OUTPUT_DIR=docs/art/production13/gates/p0 … gate.sh p0`, with Art Pass 12's opt-level settings.
- Fix fallout in art-owned code only.
- Commit the `_p13` tool copies plus `cpu_probe_p13.py` as the **Part 0 tooling commit**.

4.2 **Calibration reference** (1.2): 15 probes, R recorded.

4.3 **Identity reference.** Point `art-p13-base` at the tooling commit. Re-derive the six framings for `game_v26` and capture A and A′ (exact clock). Capture the running-dump reference (W0, W200, W-multi; 600 frames) and the pan reference (W500 frozen and 2×; 240 frames).

4.4 **Worlds.** Art Pass 12's W0, W1, W200, W500, plus W-multi re-selected by Art Pass 12's rule from Pass 21's `game_v26` chronicles (`p21-data`, `docs/revitalize/probe/p21-2_eval/`):
- the lowest seed of 19600–19631 with a delivered porter (cause < 10) before day 90 (expected 19603);
- start tick = 10:00 on the departure day.

Record seed, tick, population and villages in `docs/art/production13/perf/worlds.md`.

4.5 **Baseline** (release, measure worktree, tooling commit, valid sessions only), all after a fresh reset per running block:
- **Frozen and running blocks:** W0, W1, W200, W500 and W-multi, frozen Z4 3 × 300 frames and running 2× Z4 3 × 300 frames.
- **Frozen section timers:** W500 frozen Z4.
- **Running diagnosis** (never dropped):
  - section timers on W200 running 2× **and** W-multi running 2×;
  - SimReads per-getter calls and ms per frame;
  - running node ablation on W200 2× (each top-level node under `Main` off, 150 frames, with all-on and restored controls).
  - The W-multi ablation is droppable.
- **Output:** `docs/art/production13/perf/baseline.md` and `running-diagnosis.md`. The diagnosis lists every running cost ≥ 1 ms, largest first, and states inside or outside `AgentRenderer`.

Commit, push.

---

# 5. PART 1 — The frame budget
5.1 **Acceptance** (each candidate against the last accepted build):
- **Identity:** six framings exact (P5 included); running dumps on W0, W200 and W-multi, 600 frames; W500 pan, frozen and 2×, 240 frames.
- **Speed:** three alternating 150-frame pairs on **W500 frozen Z4** and on **W200 running 2×**, valid per 1.2.
  - **Accepted:** pooled median ratio ≤ 0.97 on both, and ≥ 2 of 3 pairs faster on each.
  - **REJECTED:** valid pairs that miss this.
  - **NOT_VERIFIED (contended):** no valid pair within 1.2's limits.

5.2 **(a) The declump kernel** (never dropped). The code exists. Measure it with 5.1 (identity is re-run on the merged `game_v26` build).
- **Accepted →** make it the default: the switch defaults to on, the GDScript path stays as the reference and the off-switch. Re-capture the identity reference on the accepted build.
- **Otherwise →** the default stays off.

5.3 **(b) Conservative write skip** (droppable; Art Pass 12's inventory recommendation, exactly that boundary).
- **Key:** after the agent's timers and its full activity/pose/frame decision are computed as today, build an owned exact key of the inputs to the property writes ≈ L226–266: the chosen frame and variant ids, every metadata value, the `CarryLoad` and founder-bundle inputs, the icon inputs including zoom and parent visibility.
- **Equal to last frame's key →** skip those writes only.
- **Always live:** the draw row, selection observation, global geometry, the post-loop (cold, chill, outline, shadow, cue, ring) and the activity oracle when enabled.
- **Always written in full:** the selected agent, new sprites, and the first frame after a reset, a language change or a capture-visibility change.
- **Switch:** `P13_WRITE_SKIP`. Hit and miss counters are diagnostic only.
- **Unit tests** for every forced-write case. Then 5.1.

5.4 **(c)** Anything ≥ 1 ms in the running diagnosis outside `AgentRenderer` that a small change can remove, one attempt, by 5.1. Droppable after (b).

---

# 6. PART 2 — Activities you can read
6.1 **Measure first.** Record, from a Z4 capture, each icon's on-screen size in pixels and its contrast against the three most common backgrounds behind it (grass, dirt, shelter): the WCAG relative-luminance contrast ratio of the icon's fill versus the background median.

6.2 **Redesign** (never dropped). Keep the seven meanings and the `activity_icon` path.
- **Badge plate:** each icon gets a dark rounded badge (`#211C25` at 85 % alpha, 2 source-pixel margin) behind a **two-tone** pictogram (a main colour and a highlight), so it reads against any terrain.
- **Size:** on-screen size at Z4 ≥ 20 px. Keep the source pixel-art scale integer.
- **Distinct silhouettes:** no two icons may share an outer silhouette. Check by XOR of the 7 masks after alignment: every pair differs in ≥ 25 % of the union pixels.
- **Distinct hues:** hue distance ≥ 40° between any two main colours. Record each.
- **Suggested pictograms** (keep or replace with a reason):

  | activity | pictogram |
  |---|---|
  | TendFire | a log being laid on a flame |
  | Craft | `U1_04` on the badge |
  | Repair | a hammer striking a plank |
  | HearthCircle | three heads around a small flame |
  | Explore | a footprint pair, or a path arrow |
  | Play | a ball with a motion arc |
  | SitByFire | a seated figure beside a flame, distinct from TendFire by the figure |

- **Village summary:** the same art at its 2× size.
- **Icon sheet:** an enlarged sheet with names at `docs/art/production13/activity/icons.md`.

6.3 **Tooltip** (droppable). Hovering an agent for 0.4 s shows a small tooltip with the person's name and current activity name (the existing `UI_ACTIVITY_*` keys, section 10), or nothing when the agent has no activity. Hidden on mouse-out. Test with the real-input harness.

6.4 **Two-stage blind test** (fixed here; results recorded before any change in response).
- **Stage A — the icons alone.** A fresh reader gets the 7 icons on badges at their actual Z4 on-screen size, shuffled, on a mid-grey background, and the 7 names. Answers are frozen. **Bar: 7 / 7.** If it fails, redesign the failing icons once; then a different fresh reader repeats stage A. The first result is kept.
- **Stage B — in the world, with a legend.**
  - Crops by Art Pass 12's icon-including geometry and protocol, at Z4, on `game_v26` captures (live seed, days 30 at 10:00 and 15:00; seed 20261018 day 60 at 10:00; supplements as Art Pass 12).
  - A fresh reader first sees the legend sheet (icon + name), then the crops and the 7 names.
  - **Bar: ≥ 12 / 14 and no kind at 0 / 2.**
- **Without a legend** (reported, no bar): a third fresh reader, stage B's crops and the 7 names only.
- **Village summary at Z1** (reported): Art Pass 11's 6-pair protocol if 6 eligible pairs exist, else NOT_RUN with the count.

---

# 7. PART 3 — Readability judged by what can be measured
7.1 **Glyph distinctness** (never dropped; `scripts/test/art/test_glyph_distinct_p13.gd`).
- **Render:** at the Life text's actual size and both weights used for names, render every syllable of `PersonNames`' table (46 Latin and 46 Hangul) and every syllable reachable after the blocklist carry.
- **Bar:** every pair of distinct syllables in the same script differs in **≥ 12 pixels** (binary threshold 0.5 on alpha). Report the 10 closest pairs per script and weight.
- **If it fails:** raise the size to the next integer multiple of the font's design size and repeat. Record Galmuri11's design size from the font file. Never use a non-integer pixel scale for names.

7.2 **Fact reading** (bar) **and name reading** (reported).
- **Captures:** Art Pass 12 7.5 and 6.6 (e), on `game_v26`, including one porter's Life section and, if present, a reconciliation entry (kind 21) in the chronicle panel.
- **Reader:** a fresh reader writes what they learn.
- **Scoring:**
  - non-name statements (days, causes, kinds, villages' roles, counts shown on screen) are scored against the data. **Bar: every one correct.**
  - Names read are scored separately and **reported** (count and confusion pairs). Names are judged by 7.1, not here.
  - "Unverified" claims (not visible on screen) are reported, not scored.

7.3 **Life-text spot fixes** (droppable): the items Art Pass 12's reviews left open that a reviewer can point to in a capture, each with a before/after capture.

---

# 8. PART 4 — After: measurements and verdicts (start by hour 9)
Rerun 4.5's matrix and the two running section timers on the frozen build, valid sessions only (1.2).

| # | question | rule |
|---|---|---|
| H1 | live game, W0 frozen Z4 | median ≤ **6.0 ms** |
| H2 | live game running, W1 running 2× Z4 | **p99 ≤ 16.67 ms** (median reported) |
| H3 | 200 running, W200 running 2× Z4 | median ≤ **16.67 ms** (ticks per frame reported) |
| H4 | 200 frozen, W200 frozen Z4 | median ≤ **10.0 ms** |
| H5 | 500 frozen, W500 frozen Z4 | median ≤ **16.67 ms** |
| H6 | W-multi running 2×; W500 running 2× | reported (W500 flagged blocks excluded; never a pass or a fail) |
| H7 | identity of every accepted candidate | all pass |
| H8 | activities | 6.4 stage A 7/7 (first and repeat reported); stage B ≥ 12/14 with no kind at 0/2; contrast and size of 6.1–6.2 recorded and met |
| H9 | readability | 7.1 all pairs ≥ 12 px; 7.2 every non-name statement correct; the real-input test (Art Pass 12's 13 steps, plus 6.3's tooltip if built) passes |
| H10 | `game_v26` on screen | porter glyph and pose, relation lines of more than one colour, a kind-21 sentence (unit test; live capture if present) |

Report every number before and after with ratios, the running diagnosis before and after, and each probe value.

---

# 9. PART 5 — Review, gate, report
1. **Independent review** (two rounds max). Scope:
   - the merge;
   - the calibration and admission record (every probe against R, attempt spacing);
   - the release worktree sha;
   - the kernel decision and the write-skip key (forced cases);
   - the icon metrics (XOR, hue, contrast, size);
   - both blind stages' order and freezing;
   - the glyph test;
   - the fact/name split of 7.2;
   - the H verdicts;
   - allowed paths;
   - commit subjects.
2. **Gate and publish:** full gate `GATE_OUTPUT_DIR=docs/art/production13/gates/final … gate.sh final`. Fast-forward `codex/art-production-2026-10`, add a PR #511 section, remove both worktrees.
3. **Report:** `docs/art/production13/REPORT.md` in Korean (section 12 format), and the same text in the final message.

---

# 10. Localization keys
**Reuse** the existing activity names (they already exist in all five files): `UI_ACTIVITY_TEND_FIRE`, `UI_ACTIVITY_CRAFT`, `UI_ACTIVITY_REPAIR`, `UI_ACTIVITY_HEARTH_CIRCLE`, `UI_ACTIVITY_EXPLORE`, `UI_ACTIVITY_PLAY`, `UI_ACTIVITY_SIT_BY_FIRE` (en `messages.ftl` ≈ L5266–5272, ko ≈ L5268–5274). Add one new key in all five files, as Art Pass 12 did; grep it first.

| key | en | ko |
|---|---|---|
| `UI_TOOLTIP_PERSON_ACTIVITY_FMT` | %s — %s | %s — %s |

---|---|---|
| `ACTIVITY_TEND_FIRE` | Tending the fire | 불을 돌보는 중 |
| `ACTIVITY_CRAFT` | Crafting | 만드는 중 |
| `ACTIVITY_REPAIR` | Repairing | 고치는 중 |
| `ACTIVITY_HEARTH_CIRCLE` | Sitting in the hearth circle | 화덕가에 둘러앉음 |
| `ACTIVITY_EXPLORE` | Exploring | 둘러보는 중 |
| `ACTIVITY_PLAY` | Playing | 노는 중 |
| `ACTIVITY_SIT_BY_FIRE` | Warming by the fire | 불가에서 쉬는 중 |
| `UI_TOOLTIP_PERSON_ACTIVITY_FMT` | %s — %s | %s — %s |

---

# 11. 인게임 확인사항 (오너용)
- 게임이 `game_v26`로 시작한다. 마을 둘 이상인 세계에서 관계선이 초록·회색·주황·빨강으로 섞여 보이는지, 연대기(C)에 "화해함"이 나오는지.
- 활동 아이콘이 어두운 둥근 바탕 위에 두 색으로 그려져 풀밭·흙·쉼터 위에서 모두 잘 보이는지, 일곱 개가 서로 헷갈리지 않는지. 사람 위에 마우스를 올리면 이름과 하고 있는 일이 뜨는지.
- 2배속에서 마을 하나(60~70명)가 끊기지 않는지.
- 생애 사건의 이름이 또렷한지(비슷한 글자 티/디, 키/기 등이 구별되는지).
- 콘솔 에러, 번역 키나 %가 보이는 곳이 없는지.

---

# 12. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(병합, CPU 보정 측정과 입장 규칙, release 기준선과 실행 진단, declump 커널 판정, 쓰기 생략, 아이콘 재설계와 툴팁, 두 단계 블라인드, 글리프 구별 검사, 사실·이름 분리 판독, 오너 피드백 처리)
### 구현 방법
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
H1~H10 표(전후·배수, release), 보정 기준 R과 모든 측정 전후 probe 값, 세계·조건별 중앙값/p90/p99/최대·틱/프레임·인구, 실행 진단(1ms 이상 비용 순위, AgentRenderer 안팎) 전후, 구간 타이머 전후, 후보별 판정과 유효 쌍 기록, 아이콘 지표(크기·대비·XOR·색상 거리), 블라인드 A/B/범례 없음 결과(활동별), 글리프 최근접 쌍, 판독 사실/이름 결과
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
실행 중 화면은 이제 몇 명까지 버티는가(release, 유효 세션)? 커널과 쓰기 생략은 실제로 빨라졌는가? 범례가 있으면 활동을 읽을 수 있는가, 없으면? 이름 글자는 기계적으로 구별되는가? 다음 아트 과제는 무엇인가?
```
The final message also includes:
- every commit hash with its Part;
- both branch heads with `git ls-remote` confirmation;
- the PR #511 update;
- the time per Part, with waiting time for admission shown separately.