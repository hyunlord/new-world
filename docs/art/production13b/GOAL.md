# GOAL: WorldSim — Art Pass 13b: finish Art Pass 13 on screen — the checks that need a window, the write skip measured, the Life text back to its design size, and the biggest running cost attacked

Repository `hyunlord/new-world`. Continue the art line from Art Pass 13's WIP: `codex/art-production-2026-10-wip` = `8c1b18be8b05bfeaa062a937526fe4d9c2f53414`; `codex/art-production-2026-10` = `57ae7ac40abde7d24369606c9250d3e06e0f18d6` (not moved by Art Pass 13 because its gate was not fully green). Draft PR #511.
This run is executed by **Codex**, unattended, on the Mac, in `/Users/rexxa/work/new-world-art-prod`. No DGX.

**Windowed Godot is authorized for this run.** During Art Pass 13 the owner asked for no Godot windows (`docs/art/production13/perf/write-skip/user-interruption.txt`, 03:44:52Z), which blocked every on-screen check. That instruction applied to that run. The owner launches this one when the Mac is not in use, so windows are expected. **If the owner interrupts or asks again for no windows, stop the current step, record it, push, and go to Part 5 with what exists.** Do not continue headless as a substitute.

**What Art Pass 13 settled** (`docs/art/production13/REPORT.md`):
- The first admitted release baseline on `game_v26` (calibrated probe, R = 2.009 s):
  - W0 frozen 2.82 ms;
  - W1 running 2× p99 13.92 ms;
  - W-multi running median 16.35 ms;
  - W200 running 26.15 ms, frozen 16.26 ms;
  - W500 frozen 28.40 ms.
- The declump kernel is accepted and on by default: W500 frozen 0.796×, W200 running 0.964×, identity exact.
- New badge icons (stage A 7/7) and a tooltip.

**What it left open:**
- every check that needs a window: the final after-matrix, stage B, the real-input test, H10's captures, the framebuffer contrast, the fact reading;
- the write skip, implemented but unmeasured and off by default;
- the Life text at **48 px**.

**About the 48 px:** this came from the planner's glyph bar (≥ 12 screen pixels between any two syllables). That bar is wrong for a pixel font. Galmuri11 is drawn on a **12-pixel design grid**, and at 24 px (2×) a one-design-pixel difference such as 마/미 is 4 screen pixels and readable. At 48 px the 332-px Life column holds about seven Hangul syllables per line.

**The largest running costs are now known:** at W200 running 2×, `walker_offsets` costs 9.88 ms and `seats` 4.41 ms (W-multi: seats 5.70 ms), all inside `AgentRenderer` (`docs/art/production13/perf/running-diagnosis.md`).

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Decisions go in `docs/art/production13b/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production13b/GOAL.md` and commit it in Part 0. Re-read the current Part's section at every Part start and after any compaction.
3. **Owner feedback:** if `docs/art/production13/OWNER_FEEDBACK.md` or `docs/art/production13b/OWNER_FEEDBACK.md` exists at launch, read both. Each item is a requirement ranked above the droppable items. **A font-size instruction in them overrides 4.2.** Record how each item was handled. Never edit those files.
4. **Parts:** 0 = section 3; 1 = section 4; 2 = section 5; 3 = section 6; 4 = section 7; 5 = section 8. Push after every Part.
5. **Retries: three strikes per failing target, then stop that target and report it.** Art Pass 13 exceeded this (8 scroll-repair runs). Count every run that executes the check, debug-labelled runs included.
6. **Time rule:**
   - Part 1 ends by hour 5.
   - Part 4 starts no later than **hour 7**; the pushed build is then frozen.
   - Past **10 hours**, finish the current step, push, go to Part 5.
   - Use the time: do not end early while a never-drop item is unfinished and admission can still succeed.
7. **Drop order:** the walker kernel (4.4) → the seats kernel (4.4) → the village summary test (5.3) → the without-legend test (5.2).
   - **Never drop:** Parts 0, 4 and 5; the font revert (4.2); the write-skip measurement (4.3); the real-input test (5.1); stage B (5.2); the H10 captures (5.4); the fact reading (5.5).
8. **Rules carried over:**
   - Art Pass 13's admission rule (calibrated probe, 1.2 there): a fresh R is measured in Part 0; attempts ≥ 30 min apart, ≤ 6 per item, 3 h total wait.
   - Its identity and acceptance rules (5.1 there), its honesty rules, its allowed paths (2.1 there, with `docs/art/production13b/**` added).
   - Commit subjects `art-p13b(pN): `.
   - No hooks, no harness pipeline, never `lead/main`/`main`, never force-push.

---

# 1. WHERE AND HOW
- **Places:** evidence goes to `/Users/rexxa/work/art-production13b-evidence`. Worktrees `/Users/rexxa/work/art-p13b-base` (identity) and `/Users/rexxa/work/art-p13b-measure` (release) are removed at the end.
- **Tools:** reuse the `_p13` tools as they are. Copy a tool to a `_p13b` name only when it must change.
- **Windowed Godot** goes only through `with_window_lock.sh`. The quiet lock is used as in Art Pass 13.
- **Side branch:** prepare the Part 2 UI work on `art-p13b-ui` while Part 1 waits on admission (as Art Pass 13's 1.3); merge it in Part 3.

---

# 2. GIT
- **Branches:** WIP gets every commit. `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose **full** gate is green, the real-input steps included. Run `git ls-remote` after every push.
- **Staging:** stage explicit paths; leave the local `.gitignore`/`.ignore` edits uncommitted.
- **No simulation merge:** `codex/revitalize-2026-10` is still `354e4a13`. If it moved, do not merge; record its head.

---

# 3. PART 0 — Start, gate, calibration
1. **Start state:**
   - backup branch `art-p13b-local-backup` at `8c1b18be`;
   - commit the brief;
   - record the owner-feedback files, if any.
2. **Gate baseline:** run the full gate (`GATE_OUTPUT_DIR=docs/art/production13b/gates/p0`, Art Pass 13's opt-level settings), now with windows. Expected 120/120, including `test_real_input_p12` and `test_real_input_p13`. Every failure is a Part 1 item (three strikes).
3. **Calibration:** 15 probe runs → R (Art Pass 13's procedure).
4. **Identity reference:** at `8c1b18be` (kernel on, write skip off): six framings, running dumps (W0, W200, W-multi; 600 frames), W500 pan (frozen and 2×, 240 frames).

Push.

---

# 4. PART 1 — Font, then the frame budget

## 4.1 Owner feedback first
Apply any owner-feedback items that concern the Life text or the inspector before 4.2.

## 4.2 Life text back to its design scale (never dropped)
1. **The size.** Set the Life text's normal and bold sizes (`agent_inspector_panel.gd` ≈ L166–167) to **24** (2 × the 12-px design grid), unless owner feedback says otherwise.
2. **The glyph rule** replaces Art Pass 13's 12-screen-pixel bar.
   - **Render:** every syllable pair of `PersonNames`' table (both scripts, both weights) at **1× the design size (12 px)**.
   - **Bar:** every pair of distinct syllables differs in ≥ **1 design pixel**, i.e. no two syllables render identically.
   - **Report:** the 10 closest pairs per script and weight in design pixels, with their screen-pixel difference at the chosen size.
   - **The size itself must be an integer multiple of 12.**
   - Art Pass 13's 24/36/48 results stay on record.
3. **Layout tests:** rerun `test_life_layout_p12` / `_p13` and the scroll tests at the chosen size, windowed.

## 4.3 The write skip, measured (never dropped)
- **The candidate:** `P13_WRITE_SKIP` = 1 against the Part 0 build (kernel on).
- **Acceptance by Art Pass 13's 5.1:**
  - identity: six framings exact; running dumps on W0, W200 and W-multi, 600 frames; W500 pan, frozen and 2×;
  - speed: three valid alternating 150-frame pairs on W500 frozen Z4 and on W200 running 2×.
- **Accepted:** pooled median ≤ 0.97 on both and ≥ 2 of 3 pairs faster on each → default on. Re-capture the identity reference.
- **Otherwise:** it stays off and the evidence is kept.

## 4.4 The biggest running costs (droppable, in this order)
**1. Walker offsets in Rust.**
- **The kernel:** the same method as the declump kernel. A pure `walker_offsets` function in `art_kernels.rs` reproduces `CampSeats`' walker-offset computation and every helper it calls, **bit for bit**: f32 for `Vector2`/`Rect2` operations, f64 for `float`, the same order and ties. Art Pass 12's `verification.md` describes the semantics already pinned.
- **The wrapper and switch:** one `#[func] art_walker_offsets` wrapper; switch `P13B_WALKER_KERNEL`.
- **Fixture:** ≥ 2,000 real calls from W200 running, W-multi running and W500 frozen, replayed bit for bit (`cargo test -p sim-bridge`).
- **Acceptance:** Art Pass 13's 5.1. It must beat the current build on **W200 running 2×** (where walker offsets cost 9.88 ms) and must not be slower on W500 frozen (pooled median ≤ 1.00).

**2. Seats in Rust.** The same method for the seat assignment (`CampSeats.update` / spread), if 4.4.1 is accepted and time remains.

**Rules for both:**
- Never change the GDScript reference functions. They remain the off-switch and the oracle.
- Rust changes stay inside the art-owned files: `art_kernels.rs`, the `#[func]` wrappers in `world_node.rs`, and fixtures.

---

# 5. PART 2 — The checks that need a window
5.1 **Real input** (never dropped): Art Pass 12's 13 steps and Art Pass 13's tooltip steps, windowed, `Input.parse_input_event` only:
- hover shows name and activity;
- mouse-out hides;
- an agent with no activity shows nothing.

Each step asserts its state and saves a screenshot.

5.2 **Activity stage B** (never dropped), exactly Art Pass 13's 6.4 stage B on `game_v26` captures of the current build:
- icon-including crops;
- exclusions logged before crops;
- a fresh reader sees the legend first;
- **bar ≥ 12/14 and no kind at 0/2.**

Then the **without-legend** test (third fresh reader, reported, droppable). Also measure the badges' **framebuffer contrast** (Art Pass 13's 6.1) on grass, dirt and shelter from the captures, and record it.

5.3 **Village summary at Z1** (droppable): Art Pass 11's 6-pair protocol if 6 eligible pairs exist, else NOT_RUN with the count.

5.4 **H10 captures** (never dropped), on `game_v26`, EN and KO:
- W-multi (seed 19603) at Z4 with a porter on the road (glyph and carry pose);
- Z1 with relation lines of at least two colours;
- the chronicle panel with a "made peace" line (kind 21). Find a world and day with a `VillagesReconciled` entry in Pass 21's chronicles (`/Users/rexxa/work/p21-data/`, read-only); if none is reachable within 120 days of one listed seed, use a kind-21 fake-bridge test and record NOT_RUN for the capture.

5.5 **Fact reading** (never dropped): Art Pass 13's 7.2 on captures of the final Life text size.
- A fresh reader writes what they learn.
- **Bar:** every **non-name** statement correct.
- Names are reported (count and confusion pairs).
- Unverifiable claims are reported, not scored.

---

# 6. PART 3 — Merge the UI side branch, re-check
- Merge `art-p13b-ui` into WIP.
- Rerun the full gate windowed: it must be green, the real-input steps included.
- Rerun the identity checks only for candidates accepted in 4.3–4.4, on the merged build.

---

# 7. PART 4 — After: measurements and verdicts (start by hour 7)
Rerun Art Pass 13's 4.5 matrix and both running section timers on the frozen build, valid sessions only. The verdict table is Art Pass 13's H1–H10, with these changes:
- **H8:** stage B ≥ 12/14 with no kind at 0/2 (5.2); stage A stays Art Pass 13's 7/7; framebuffer contrast recorded.
- **H9:** 4.2's glyph rule (≥ 1 design pixel; size an integer multiple of 12), plus 5.5's fact bar, plus 5.1's real input.
- **H10:** 5.4's captures.
- **New H11 (frame budget, reported):** the running diagnosis after, next to Art Pass 13's. Each accepted kernel's ratio on its workloads.

Report every number before and after with ratios, with Art Pass 13's admitted baseline as "before".

---

# 8. PART 5 — Review, gate, publish, report
1. **Independent review** (two rounds max). Scope:
   - the font revert and the glyph rule;
   - every accepted candidate's identity and pairs;
   - the walker kernel's float semantics and fixture, if built;
   - the real-input screenshots;
   - stage B's order and freezing;
   - the H10 captures;
   - the fact/name split;
   - the retry counts against the three-strike rule;
   - allowed paths and commit subjects.
2. **Gate and publish:**
   - The full gate must be green (`docs/art/production13b/gates/final`) → fast-forward `codex/art-production-2026-10`. This makes Art Pass 13 and 13b the published art build.
   - Add a PR #511 section and remove both worktrees.
   - If the gate is not green, do not move the publish branch, and say why.
3. **Report:** `docs/art/production13b/REPORT.md` in Korean (section 10 format), and the same text in the final message.

---

# 9. 인게임 확인사항 (오너용)
- 생애 사건 글자가 24px로 돌아와 한 줄에 충분히 들어가는지, 비슷한 글자(마/미, 티/디 등)가 실제로 구별되는지.
- 사람 위에 마우스를 올리면 이름과 하는 일이 뜨고, 벗어나면 사라지는지.
- 활동 아이콘(어두운 둥근 바탕, 두 색)이 풀밭·흙·쉼터 위에서 잘 보이고 서로 헷갈리지 않는지.
- 마을 둘 이상인 세계에서 짐꾼, 여러 색의 관계선, 연대기의 "화해함"이 보이는지.
- 200명 세계를 2배속으로 돌릴 때 이전보다 부드러운지(걷는 사람 위치 계산이 Rust로 옮겨졌다면).

---

# 10. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(오너 피드백 처리, 생애 글자 크기와 새 글리프 규칙, 쓰기 생략 판정, 걷는 사람 오프셋·자리 커널, 실제 입력·툴팁, 범례 있는 블라인드 B단계와 범례 없는 결과, 화면 대비, H10 캡처, 사실 판독, 게시)
### 구현 방법
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
H1~H11 표(Art Pass 13 기준선 대비 전후·배수, release), 보정 R과 모든 probe 값, 세계·조건별 중앙값/p90/p99/최대·틱/프레임·인구, 실행 진단 전후, 후보별 판정과 유효 쌍, 블라인드 B(활동별)·범례 없음·마을 요약, 화면 대비, 글리프 최근접 쌍(설계 픽셀), 판독 사실/이름 결과, 재시도 횟수(목표별)
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
실행 중 화면은 이제 몇 명까지 버티는가? 걷는 사람 계산을 옮긴 효과는? 범례가 있으면 활동을 읽을 수 있는가? 24px 생애 글자는 읽히는가? 배포 브랜치로 옮겼는가? 다음 아트 과제는 무엇인가?
```
The final message also includes:
- every commit hash with its Part;
- both branch heads with `git ls-remote` confirmation;
- the PR #511 update;
- the time per Part, with admission waiting shown separately.