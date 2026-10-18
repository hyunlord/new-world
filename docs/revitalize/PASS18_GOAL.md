# GOAL: WorldSim — Revitalize Pass 18: make the daily routine cheap without changing it, and let couples stay together (`game_v23`)

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `codex/revitalize-2026-10-wip` = `8cbb697c97aac800935813d2c79c34195ebef6a3` (Pass 17, gate green). Draft PR #510.
This run is executed by **Claude Code**, unattended, on the Mac, in `/Users/rexxa/work/new-world-pass4`. **At the same time an art run (Art Pass 10, Codex) works in `/Users/rexxa/work/new-world-art-prod` and measures frame times**; section 1.2 is how the two runs share the Mac. No DGX.

Two parts, in this order:
1. **A behaviour-preserving performance fix** for Pass 17's routine (`game_v21` / `game_v22` failed R5: mean tick +33 %, p99.9 +340 % against `game_v20`). Every scene — `game_v21` and `game_v22` included — must produce **bit-identical** results before and after.
2. **`game_v23`** = `game_v22` + two family changes: pairing needs less familiarity (F1 missed 0.50 at 0.495), and **an elder partner leaves with their partner when a village splits** (today elders never leave, so a splitting adult leaves their elderly partner behind — nine of nine couples split in Pass 17's tuning F5 failure were such couples). The live game moves to `game_v23`.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 8cbb697c
cp ~/Downloads/claude_goal_worldsim_pass18.md docs/revitalize/PASS18_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS18_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only; never stage outputs or gate logs of earlier passes.

**This document is `docs/revitalize/PASS18_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript, so the section 11 report, the G table, the R5/F8 re-judgement, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As Pass 16's 0.2: no `install_hooks.sh`, never `lead/main` or `main`, no `harness_pipeline.sh` (4.4 is the gate), built-in `general-purpose` subagents only (for the independent reviews: a fresh subagent given only this document, the diff and the outputs), a blocking hook other than `/goal` → record, push, report. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. The previous pass (17) was run by Codex in this same clone: its files (`docs/revitalize/PASS17_GOAL.md`, `REPORT_PASS17.md`, `tools/revitalize/*_p17*`) are the reference for the procedures this document cites.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 18`, with a RUNLOG table (Part, start, end, notes).
2. **Do not end your turn between Parts or while a measurement or probe batch runs.** Poll long jobs every 10–20 minutes with short calls.
3. **Parts:** 0 = section 5; 1 = section 6 (measure); 2 = section 7 (fix); 3 = section 8 (`game_v23`); 4 = section 9 (review, gate, report). Push after every Part.
4. **Review loops: two rounds maximum.**
5. **On failure: work around, note it, continue.** A target missed is reported as missed; a missing number is not acceptable.
6. **Time rule:** past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 4.
7. **Drop order:** section 7's structural candidates (7.3) → `game_v23`'s tuning round (8.4). Never drop Parts 0, 1, 4, the identity checks, the cheap fixes of 7.2 or `game_v23` itself.

---

# ⚠️ 1. MACHINE
## 1.1 The Mac
Only `/Users/rexxa/work/new-world-pass4` and one base worktree `../new-world-p18-base` (5.2). Never touch other checkouts or processes you did not start; no DGX. `CARGO_BUILD_JOBS=6`, tests `--test-threads=6`, probes `--workers 4`. Data in `/Users/rexxa/work/p18-data/`.
## 1.2 Sharing the Mac with the art run (both briefs carry this protocol)
Quiet lock `/Users/rexxa/work/.mac_quiet.lock` (`mkdir`; `info.txt` with owner, pid, purpose, started; removed when done). Hold it **only while measuring time** (6.2, 7.4, 8.6), ≤ 60 minutes per hold, ≥ 10 minutes between your holds. While the other run holds it: no cargo build or test, no probe, no Godot, no gate on the Mac; poll every 5 minutes; a lock older than 2 hours is stale (remove, record). Before taking it: no `cargo`/`rustc`/`worldsim_*`/`Godot` process of the other run (`pgrep -x`) and 1-minute load < 4; after 60 minutes of waiting take it anyway and mark the runs `contended`. Record every hold and wait.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green; `git ls-remote` after every push. Messages `revitalize(p18-N): …`.
- **May change:** `rust/**`, `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, `tools/worldmodel/**`, `docs/worldmodel/**`, `rust/crates/sim-test/src/worldmodel/**`.
- **No new public field in any existing public struct** except `FamilyRule`'s new field (8.2) — `FamilyRule` exists only since Pass 17; its literals are in `rules.rs` and the test at `sim-core/src/rules/routine.rs` ≈ L121 (update both; the art branch has no `FamilyRule`). **Private** fields may be added to `AgentDecisionSystem` for caches: it is `#[derive(Debug, Default)] pub struct AgentDecisionSystem;` (`agent_decision.rs` ≈ L598–599), constructed only through `::new()` (no struct-syntax or unit-value construction anywhere — grep and record), so every new field type must implement `Debug` and `Default`, and `new()` becomes `Self::default()`. `Activity` and the other Pass 17 components keep their fields. No new bridge getter keys; no change to existing getters' output for `game_v22` and older.

---

# 3. CONTEXT (planner's review of Pass 17 with a code survey at `8cbb697c`; verify each fact)

## 3.1 Where the routine's cost comes from (paths under `rust/crates/sim-systems/src/runtime/`)
- **Every tick, at any hour, `RoutineTick::new`** (`decision/routine/mod.rs` L55–131, built at `decision/agent_decision.rs` L628–632) runs stale-activity cleanup (L59–76), a rest-expiry scan building a `rests` BTreeMap (L77–84), `lifecycle::apply` (sort + `insert_one`/`remove_one`), an active list collected and sorted (L86–91), and **rebuilds from scratch** an `occupancy` BTreeMap (L92–95), a `members` BTreeMap of every member (L96–103), an `entities` BTreeMap (L113–117), a `keys` BTreeMap (L118–122), and **`places::collect`** (`routine/places.rs` L32–103): per settlement a BTreeSet of all buildings, an ownership scan, and a 29 × 29 tile scan around the formation tile with two linear `owned.iter().any` scans per tile within distance 8, plus 7 Vec allocations. Estimated 60–100 µs per tick at 500 people — most of the +111 µs per call — paid at night too.
- **The exit test** (`agent_decision.rs` L1171–1294) runs for every person in an activity every tick and computes the forage, firewood and gather predicates **eagerly** (OR'd only at L1281–1287); `choose_bush` / `choose_spot` / `nearest_gather_tile` are O(F)/O(W) scans behind cheap gates. On an exit tick the Idle cascade recomputes them (L1803 …): double cost.
- **The start arm's redraw** (`mod.rs` L173–181) calls `place()` for all 7 kinds and once more for the winner; Explore's ring has ≈ 480 tiles.
- **Replay rounds** (`routine.replay()` at `agent_decision.rs` ≈ L2587–2595 re-runs the whole loop body ≈ L1075–2585 over the re-collected rows, L1091) for social/combat invitations, more often now that activity holders count as idle-equivalent.
- **A per-tick memo the code allows:** in `places::choose` (`places.rs` ≈ L113–124) the keyed `rank` is the same for every tile of a list, so the chosen place depends only on (settlement, kind, occupancy) — not on the person.
- **Spikes (p99.9) are mid-day storms, not the 18:00 edge.** In Pass 17's own raw ticks (`docs/revitalize/perf/p17-a/raw/a/r{1,2,3}`, `*.tick_us.bin`; p99.9 = the 2nd-largest of 1,380 measured ticks, `perf.rs` ≈ L149–162) the slowest `game_v21` ticks are tick 170 (10:50, 3.0–3.2 ms), 282 (12:42, 2.5–2.7 ms), 370 and 506, the decision system taking 2.4–3.0 ms of each; the 18:00 tick (600) costs 0.93 ms (14th slowest; v20's is 0.52–0.56). 27 ticks of v21 r3 exceed the p99.9 bar (0.792 ms = 1.35 × v20's 0.587). Find what those storm ticks do (6.1's scopes; replay rounds and redraw storms are the first suspects). `RoutineObservation` / `RoutineObservationBatch` components are never removed in production (only probes read them) and multiply archetypes.
- **ECS order matters** (Pass 16, the a15 lesson): hecs iterates by archetype; anything that changes which components an entity carries, or when archetypes are created, can change the iteration order of unsorted queries (e.g. `agent/movement.rs` L238) and therefore results. Option-fetching an extra component in a query (`Option<&Lineage>`) does **not** change which archetypes match.

## 3.2 Family (Pass 17 results)
F1 (partnered share of non-elder adults at day 60) 0.495 [0.454, 0.532] vs bar 0.50 (tuning 0.542); F2 0.967, F3 0.105, F4 0.996, F5 0.077 all passed; health unchanged. Pairing: daily at 20:00, opposite-sex, same settlement, not close kin, highest familiarity ≥ 0.300 (three completed interactions) (`agent/pairing.rs`). Fission excludes elders; households = couple + Child-stage children; `select_departure` (`agent/households.rs` L69–140). In tuning, the nine split couples were all adult–elder couples where only the adult left.

---

# 4. RULES
## 4.1 Identity
Behaviour-preserving means **bit-identical**: probe summaries (flags as `tools/revitalize/identity_p17.sh` ≈ L18: `--clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism`), `--seeds 0..32`, 120 days, identical except `timing` / `wall_seconds` / `ms_per_agent_tick` between the base build (`8cbb697c`) and every candidate build of Part 2 (**I1** v19, **I3** v16, **I4** v20, **I5** v21, **I6** v22); recorder `game_v19` `--seeds 12000..12002` array sha256 (**I2**); all existing pins; and new pins (5.1). Copy `identity_p17.sh` to `identity_p18.sh` and add I6. Run I5/I6 before Part 2 is pushed and again on the final build. `rust/Cargo.lock` is git-ignored: copy the main clone's lock file into the base worktree and record `cargo tree -i hecs` in both, so both builds use the same hecs.
## 4.2 Seeds
`game_v23`: tuning **19100–19115**, evaluation **19000–19031**. Unused before; not in tests.
## 4.3 Arithmetic and draws
As Pass 17 (integers, no forbidden math, keyed draws, new streams in `STREAMS`).
## 4.4 The gate
`tools/revitalize/gate_pass18.sh` = `gate_pass17.sh`'s 15 steps unchanged, `allowed_paths` against `8cbb697c`. Under 1.2's rules.

---

# 5. PART 0 — Pins and base
5.1 `rust/crates/sim-test/tests/harness_revitalize_pass18_frozen.rs`: for `game_v21` and `game_v22`, seeds 0–4, probe fingerprint **and** `state_digest` after 1,440, 40,320 and 86,400 ticks (day 60 — past the first pairings and fissions) from the unchanged code — **one `#[test]` per (scene, seed)** so `--test-threads=6` parallelizes them (Pass 17's 40,320-tick pins took 222 s in debug; these run ≈ 2× longer) — and field-by-field asserts of `WorldRules::game_v22(s)` (no struct literal). Commit alone before any other Rust change.
5.2 Worktree `../new-world-p18-base` at `8cbb697c`; release build; I1–I6 into `docs/revitalize/probe/p18-0_base/` (summaries committed; recorder outputs in `p18-data`, sha256 list committed). Push.

---

# 6. PART 1 — Measure where the time goes
6.1 Add a Cargo feature `subprofile` to `sim-systems`, forwarded by a `sim-test` feature of the same name, **never on by default** and never a `required-features` of `worldsim_perf` (the gate's `perf_quick` uses that binary); build it in a separate `--target-dir` so identity binaries are never built with it. With the feature off the macro expands to the bare block (proof: I1–I6 and the pins identical; `cargo clippy -p sim-test --features subprofile -- -D warnings` clean): a `scope!(name)` macro writing elapsed nanoseconds into a `thread_local!` accumulator (reads `Instant` only; never touches simulation state), with scopes at: `RoutineTick::new` split into cleanup (`mod.rs` L59–85), map rebuilds (L86–122), `places::collect` (L127); the decision system's pre-loop maps (`agent_decision.rs` L633–1043); the exit test (L1171–1294); the start arm (L1917–1925); each replay round — the **whole loop iteration** while `routine_replay.is_some()`, not only `routine.replay()`; the post-pass (L2603–2938); `finish` (L2939–2941) with a count of archetype moves (inserts of a new component type + removes; an `insert_one` replacing an existing component is not a move), including the moves `lifecycle::apply` makes inside `RoutineTick::new` (`mod.rs` ≈ L85); telling a new component from a replacement may need a `world.get` check before the insert — such checks exist only inside the feature-gated code, so the feature-off path is unchanged. `worldsim_perf` built with the feature prints the scopes per tick in its JSON (`subprofile` block: per scope total, mean, max, and the 20 slowest ticks' scope breakdown).
6.2 Measure (quiet lock; copy Pass 17's runner as `tools/revitalize/perf_p18.sh` with this pass's owner string and a stage c for `game_v23` vs `game_v22`): `worldsim_perf stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules R --out …`, R ∈ {`game_v20`, `game_v21`, `game_v22`}, three interleaved repeats each, with and without the feature; record the feature's own overhead. Write `docs/revitalize/perf/p18-1/attribution.md`: which scopes explain the +111 µs per call and which ticks make the p99.9 (clock minute and scopes). Push.

---

# 7. PART 2 — The fix (results must stay bit-identical)
7.1 **Rule:** every change in this Part keeps I1–I6 and all pins identical. A change that breaks identity is reverted, recorded with the first differing value, and not retried in another form more than twice.
7.2 **Cheap fixes (do all that the attribution supports):**
a. `keys`: fetch `Option<&Lineage>` in the main decision query (rows are already sorted by AgentId) instead of rebuilding a map.
b. `members`: reuse the decision system's existing `member_of` **only when it is built** — it is filled only under the firewood or forage rule (`agent_decision.rs` ≈ L825–835, L859–868; forage also needs `stock_target`); Pass 17's routine tests run on legacy rules (`tests/pass17_routine_support/mod.rs` ≈ L12–14) where it is empty — otherwise build the map as today.
c. `entities`: build lazily, only when an invitation needs it, from its own `world.query::<&Agent>()` (not from the loop's rows, which replay rounds filter).
d. `places`: compute lazily on the first `place()` call of a tick (none at night); cache across ticks in a field of `AgentDecisionSystem`, invalidated by an equality check of the inputs (as `BedAssignmentSystem` does, `agent/beds.rs` L29–43): `building_kinds`, `building_owner`, the building registry, each settlement's `(formation_tile, member_buildings)`, the grid size, and the River/Mountain terrain of the scanned area (`places.rs` ≈ L59–73; walkability is not read). Replace the per-tile `owned.iter().any` scans by a set; build the `occupied` building set once per tick (it is not per settlement, L33–38).
e. `occupancy`: keep the per-tick rebuild from `active_rows` (deaths remove `Activity` outside this system, and `end()` leaves zero-count entries, so an incremental map is not worth its risk).
h. **Place memo:** since the chosen place depends only on (settlement, kind, occupancy) (3.1), memoize `place()` results — including `None` (the redraw's `available` array reads `is_some()`) — in `RoutineTick` (the per-tick struct, never across ticks), keyed by (settlement, kind). `occupancy` is **one world-wide map keyed by tile** (`places.rs` ≈ L10) and settlements' candidate lists can share tiles (the Explore ring reaches 14 tiles; `site_clearance` is 16), so **clear the whole memo on every occupancy change within the tick** — every `start` (`mod.rs` ≈ L196) and every `end` (`lifecycle.rs` ≈ L136–138, also reached through `invite` → `interrupt` for other people). The slot-1 redraw draw itself is per person and is not memoized.
f. **Exit test:** evaluate the cheap predicates first (need, builder assignment, social, meal wait) and the forage/firewood/gather predicates only if those are false, each short-circuiting; compute the tile lookups only when their cheap gates pass (already true) and **reuse** the results in the Idle cascade on an exit tick instead of recomputing (pass them through; same values, same claims state).
g. **Redraw:** compute each kind's place once per start (cache within the call).
7.3 **Structural candidates (only with proof):** the 18:00 edge's archetype moves and the never-removed observation components. First target the mid-day storm ticks (3.1) with whatever 6.1 shows; then, if still needed: removing `RoutineObservation*` from production entities (probes then read a resources-side log instead). (Keeping `Activity` resident would need a new field in `Activity`: not allowed.) These change archetype composition and may change iteration order: allowed **only** if I1–I6 and the pins remain identical; otherwise revert and record.
7.4 **Re-judge R5 and F8** with Pass 17's exact procedure (quiet lock; three interleaved repeats, old rules then new, `stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed`; medians; `quick` on the base and the final build; slope runs `--agents 100,500,1000 --days 2`). The bars: mean ≤ 1.15 × v20's (≈ 0.406 ms against today's 0.470), p99.9 ≤ 1.35 × v20's (≈ 0.792 ms). Planner's expectation: the 7.2 fixes plausibly meet the mean bar; p99.9 needs the storm ticks fixed. `game_v21` vs `game_v20` — mean tick ≤ **+15 %**, p99.9 ≤ **+35 %**, mean ≤ 4.0 ms; `AgentDecisionSystem` three-size slope < 1.7; `game_v22` vs `game_v21` likewise; `quick` before/after. If the p99.9 bar cannot be met without changing behaviour, report the remaining spike's cause (clock minute, scope, move count) and leave the decision to a later pass.
Push after each accepted change.

---

# 8. PART 3 — `game_v23`
8.1 `WorldRules::game_v23(seed)` = `game_v22(seed)` with the family rule changed as below; `WorldGenConfig::game_v23` = `game_v22`'s. Wire into `scenes.rs`, the probe's scene lists (flip any "unknown scene" assertion), `twin_checked`, `perf.rs --rules`. Live game (`WorldSimNode::init`, `reset_world`) builds `game_v23`; `reset_world_stress` stays `game_v19`.
8.2 `FamilyRule` gains `elder_partner_follows: bool` (false in `game_v22`, true in `game_v23`); `min_familiarity_permille` becomes **200** in `game_v23` (300 in `game_v22`).
8.3 **Elder partners follow** (`game_v23` only). `households()` already groups elders (`households.rs` ≈ L8–20, lone elders and elder–elder couples as their own groups, L58); `select_departure` drops elders only through the rank filter (≈ L80–90; `ranked` excludes elders, `fission.rs` ≈ L569, L582) — and an elder-only group would get rank `None`, which sorts **first** in the fill (≈ L114–119). So, under `elder_partner_follows`: keep only groups that contain at least one ranked (non-elder) member; within such a group keep the ranked members, their Child-stage children, and **an elder only if they are the mutual partner of a ranked member of the group**; drop everything else from the group (elder-only groups, widowed elder parents with children, elder–elder couples stay behind as today). The size checks count the kept elders, so `group_max` is never exceeded (test it). Ranking and the four-adult seed are unchanged; kept elders never count toward `min_adults` (adults are counted through the rank). Because kept elders make households bigger, the seed and the fill (bounded by `k`) will sometimes skip households they used to take — who leaves changes in `game_v23`; that is expected, not a bug (report daughter-village sizes, G4). Record per fission, **in the probe trace** (from the departing members, the couples and the `Elder` component — no field added to `FamilyEvent::Departed`), how many elders left and how many couples, and adult–elder couples, were split.
8.4 **Tuning** (19100–19115, 60 days, `--agent-trace`): familiarity grows in steps of 0.1 per completed interaction (`FAMILIARITY_BUMP`, `agent_decision.rs` ≈ L482; no decay), so 200 = two interactions and 150 would equal 200; if F1 < 0.50, lower it to **100** (one interaction) once; record. No other tuning.
8.5 **Rule table** (committed to METRICS.md under `## Pass 18` before 8.6 runs): over 19000–19031, 120 days, `game_v23` vs `game_v22` paired by seed, Pass 17's definitions and bootstrap:
| # | rule |
|---|---|
| G1 | F1 (partnered share of non-elder adults at day 60) ≥ **0.50** (point estimate; interval reported) |
| G2 | F2 (couple-parent births from day 20) ≥ 0.90; F4 (couples in the same shelter) ≥ 0.90 |
| G3 | F5 couples split by completed fissions ÷ couples at departure ≤ **0.05**, and **adult–elder couples split = 0** except when the `group_min` top-up (`households.rs` ≈ L125–132, the only path that takes an adult alone) took the adult without their household (each such case listed) |
| G4 | health: Pass 17's R2 rules for `game_v23` − `game_v22` (alive ±10 %, non-old-age deaths ≤ +1.5, shelters ≥ 0.9×, social ≥ 0.9×, stores ±15 %); plus mean daughter-village size and survival to day 120 reported (elders now leave too) |
| G5 | identity: I1–I6 and all pins identical on the final build |
| G6 | performance: `game_v23` vs `game_v22` as 7.4 |
8.6 Evaluation (`game_v23` and `game_v22` on 19000–19031, 120 days, `--agent-trace`) → `docs/revitalize/probe/p18-3_eval/eval.md` + `eval.json`. Push.

---

# 9. PART 4 — Docs, review, gate, report
1. Update `docs/revitalize/ROUTINE.md` (the performance structure), `FAMILY.md` (`game_v23`), `PERF_BUDGET.md` (new references).
2. Independent review (fresh `general-purpose` subagent, two rounds max): identity I1–I6 and pins; that `subprofile` off compiles to today's paths; each 7.2/7.3 change and its proof; the R5/F8 re-judgement procedure; `game_v23` rules and the elder-follow path; G verdicts; allowed paths; quiet-lock record. Findings in `docs/revitalize/REVIEW_PASS18.md`.
3. Gate (4.4) → `docs/revitalize/verification/p18_gate/`; fast-forward `codex/revitalize-2026-10`; PR #510 section; remove `../new-world-p18-base`.
4. `docs/revitalize/REPORT_PASS18.md` in Korean (section 11 format) and the same text in the final message.

---

# 10. 인게임 확인사항 (오너용)
- 게임이 `game_v23`으로 시작한다. 낮 활동과 가족은 Pass 17과 같고, 짝이 조금 더 잘 생긴다.
- 마을이 갈라질 때 늙은 배우자도 짝을 따라 새 마을로 가는지(혼자 남은 노인 배우자가 없는지).
- 4배속으로 돌려도 끊김이 없는지(시뮬레이션 쪽 비용이 다시 줄었는지). 화면 표시는 아트 쪽 Art Pass 10의 일이다.
- 콘솔 에러가 없는지.

---

# 11. 구현 완료 보고 (REPORT_PASS18.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
### 구현 방법
(비용 측정 방법, 각 수정과 동일성 증명, game_v23 규칙)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
범위별 비용 귀속표(전후), R5/F8 재판정(세 반복 중앙값), G1~G6 표(값과 구간), 분열 시 노인 동반·부부 분리 집계, 건강 표, 조정 기록
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
루틴 비용은 해결됐는가? 가족은 이제 함께 움직이는가? 다음은 무엇인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #510 update, and the time per Part.
