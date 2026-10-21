# GOAL: WorldSim — M1.5: the colony's own calendar and demographics, and the simulation contract (reproducible, view-independent, measured) that every later pass is held to

Repository `hyunlord/new-world`. Base: **`v8/main` = `v8/wip` = `3183e77dfbe6373f1c3d3e0d08a1121fb546b694`** (M1-ART, gate green at `2f701621`).

Executed by **Claude Code**, unattended, on the Mac in `/Users/rexxa/work/new-world-pass4`. No DGX. Headless work except the two screen checks in 10.3. Target 16 hours, hard stop 20.

**In parallel**, the art session runs the image-generation trial on branch `v8/art-trial`, touching only `art/**`, `docs/v8/arttrial/**` and `docs/images/arttrial_*`. Do not touch those paths; do not merge that branch.

## Why this pass exists

Two things the owner decided on 2026-10-21:

1. **The simulation is the product.** Visuals will change again, maybe to 3D; the logic must stay reproducible under any renderer, and must stay fast and small as it gets complex, using mathematically effective methods rather than brute force. That becomes a written contract (ADR 0016) and gate steps that enforce it. Today the contract is only half-true: the sim is Godot-free, deterministic and replayable from (seed, commands), but there is no save/load, no second renderer, no standing stress benchmark at colony scale, no memory measurement, and no spatial index (job assignment scans every candidate; 26 of 28 systems run every tick).

2. **The V7 calendar makes the colony unplayable as a long game.** `BirthRule::Provisioned` provisions a newborn every 720 ticks when the camp is fed, so 10 founders become 29 people in 10 days (M1 evaluation, every seed). Ages read as "54일을 살고 노환". ADR 0004 fixed the tick pacing (12 ticks/s, a 2-minute day) and left the calendar to this pass: **6-day seasons, a 24-day year, ages in years, and a demographic model with a fertility rate instead of provisioning.**

Both halves are done on the **colony** world only. **`baseline` stays bit-identical** (identity is a gate step), which proves that everything new is gated.

A third, smaller item: M1's job churn (about 150 jobs taken for 32 finished in 10 days) is diagnosed and fixed under a pre-registered bar.

---

# 0. RULES
- AGENTS.md and the skills apply. Read `worldsim-architecture`, `worldsim-testing`, `worldsim-pitfalls` first.
- Commit to `v8/wip`, messages `v8-m1.5(pN): …`; `v8/main` by fast-forward after a green gate only. Never `lead/main`, `main`, `codex/*`, `v8/art-trial`; no force-push; explicit paths; hook file and local ignore lines untouched.
- **Never ask the owner.** Decisions in `docs/v8/m1_5/M1_5_DECISIONS.md`, numbered with context, choice, reason, evidence.
- Parts: 0 = §4 (contract and base data) · 1 = §5 (calendar, ages, demographics) · 2 = §6 (benchmark and spatial index) · 3 = §7 (save/load) · 4 = §8 (text renderer and job churn) · 5 = §9 (evaluation) · 6 = §10 (docs, review, gate, report). Push after every Part.
- **Time rule:** past 20 hours, finish the step, push, go to Part 6.
- **Drop order:** 8.1 text renderer's `--ascii-diff` → 6.3 index use in the decision scans → 7 save/load (then it moves to M2 and the ADR says so) → 8.2's second churn remedy.
- **Never drop:** ADR 0016 and the gate steps of 6.1 (benchmark + ledger), the calendar, demographics, the evaluation, baseline identity, docs and the report.

# 1. MACHINE
Pass4 checkout plus a base worktree `../new-world-m15-base` at `3183e77d`. Data in `/Users/rexxa/work/m1_5-data/`. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.

# 2. CONTEXT (planner's survey at `3183e77d`; paths under `rust/crates/`; verify)
- **Clock:** 1 tick = 1 simulated minute; `ticks_per_day` 1,440; the time snapshot exposes `ticks_per_day`, `day_index`, `time_of_day`, `season`, `day_of_season`, `days_per_season`. Seasons: `rules.seasons: Option<SeasonRule { days_per_season, … }>`; many society rules carry per-season arrays (`[_; 4]`) and "days" values (`FirewoodRule.base_days`, `winter_prep_days`, `StockTargetRule.base_days`, `FissionRule.cooldown_days` …).
- **Ages:** `Life { born_tick, lifespan_ticks }` with `age_ticks(now)` (closed form already); `Child { born_tick }`; `AgingRule { elder_age_ticks 66,240, lifespan_mean_ticks 83,520, lifespan_spread_ticks 11,520, founder_age_min/max_ticks 8,640 / 57,600, founder_min_remaining_ticks 4,320 }`; `LifeStageRule { child_ticks 5,760 }`; `AgingSystem` (priority 146, interval 60) despawns at `age >= lifespan`; `MaturationSystem` (144, interval 60) removes `Child`.
- **Births:** `SocietyRules.birth = BirthRule::Provisioned { cooldown_ticks 720, food_cost 10, max_mean_hunger 60.0, max_mean_thirst 60.0, sustain }` — a settlement-level provision, not a per-woman process. `FamilyRule` (Pass 17) has pairing (`PairingSystem` 144), households, `last_birth_tick`.
- **Needs:** `Hunger { value: f32, growth_rate: f32 }` and the thirst/sleep twins decay every tick (`HungerDecaySystem` 130, interval 1). Floats are allowed by the forbidden-math grep (it bans transcendental functions, not `f32`); the contract (4.1) rules on this.
- **Engine:** `SimEngine::tick_profiled` records per-system wall times (`sim-engine/src/lib.rs` ≈ L850–940). `worldsim_perf` has `quick` and `stress` modes on `WorldGenConfig::stress(seed, agents, side)` (= baseline rules). hecs 0.10 (workspace); components derive serde.
- **Jobs (M1):** `runtime/jobs/{assign,execute,cancel,invariants,items,seam}.rs`; `assign.rs` filters every candidate by Chebyshev distance (`nearest(...)`, ≈ L54–110); `ColonyRule.job_scan_radius` 96. `CancelReason::Needs` released 36–178 jobs per seed in 10 days (`docs/v8/m1/M1_EVALUATION.md`).
- **Evaluation tooling:** `worldsim_probe --scene colony --commands <json> --days N --seeds a..b --out … --chronicle-out …`; `tools/v8/identity.sh`, `perf.sh`, `gate.sh` (21 steps). The M1 U-script lives in `docs/v8/m1/` (name it in DECISIONS).
- ADR numbering: 0015 is taken (colony art). This pass writes **0016** (contract) and **0017** (calendar and demographics).

---

# 3. THE CONTRACT (the text of ADR 0016; commit it in Part 0 before any code)

`docs/decisions/0016-simulation-contract.md`:

**C1 Reproducibility.** The world state at any tick is a pure function of `(rules, seed, command log)`. Any build of the same commit replays a recorded `(seed, commands)` to identical fingerprints (probe summary, raw rows, chronicle). A later commit that changes the colony's behaviour says so in its CHANGELOG entry and re-captures the colony fixtures; the `baseline` world never changes (identity gate). Saved games are a serialization of that state with a format version; `save → load → tick` equals `tick` without the save (7).

**C2 View independence.** `sim-core`, `sim-systems`, `sim-engine` know nothing of pixels, tiles sizes, sprites, cameras, frames or Godot. `sim-bridge` is the only crate that knows Godot, and it only marshals. Renderers consume the documented snapshot schema (`docs/ARCHITECTURE.md` §4 + the schema file of 8.1) and the chronicle/causal log; nothing a renderer needs is computed in GDScript from undocumented state. Proof: a second renderer exists (the text renderer, 8.1) and the GUI test and the text renderer agree on positions.

**C3 Performance budget.** Measured every pass by the standing benchmark (6.1) on the colony world, 192×192, at N = 200 / 1,000 / 2,000 agents: tick mean and p99, per-system share, bytes per agent and per tile. Hard ceilings: at N = 1,000 tick mean ≤ 5 ms; at N = 2,000 ≤ 10 ms; memory ≤ 4 KiB per agent and ≤ 96 B per tile beyond the fixed cost. Regression ceiling: any benchmark statistic worse than the ledger's previous pass by more than 10 % fails the gate unless the pass's CHANGELOG justifies it and the ledger line says so. No system may do O(agents × candidates) work per tick without a spatial index; no per-tick scan of all tiles.

**C4 Mathematical method.** Prefer closed-form state over per-tick accumulation (store `since_tick` + rate; evaluate on read, as `Life::age_ticks` does). Prefer event-driven work (dirty sets, timers, intervals) over polling. Evaluate curves (decay, probabilities, thresholds) from precomputed integer tables, not transcendental calls. New state uses integers or fixed-point (permille / per-64k); existing `f32` needs are grandfathered until a dedicated pass converts them, and no new `f32` state is added to the sim crates. Randomness only through keyed draws. Maps iterate in key order. Tie-breaks are `(y, x)` then id.

**C5 Structure.** Logic lives in the system that owns the component; a system's header lists reads/writes/events and the table in ARCHITECTURE is generated from it. Files over 1,200 lines are reported every pass; the decision tick function is not extended (M2 dismantles it as the job layer takes over).

Each of C1–C5 names the gate step that checks it (after this pass: identity, replay, colony determinism, save/load round trip, benchmark + ledger, text-renderer agreement, docs check for the schema file).

---

# 4. PART 0 — Contract and base data
1. Commit this goal as `docs/v8/m1_5/M1_5_GOAL.md`, ADR 0016 (§3 verbatim plus the Context/Consequences sections), and point AGENTS.md "Code rules" at it (one line).
2. Base data in the base worktree: identity base (`tools/v8/identity.sh` documents it), the M1 colony fixtures (U-script run on seeds 100–131, 10 days: summaries + chronicle) → `$DATA/base/`, and the first benchmark run of 6.1 **at the base commit** so the ledger has a "before" line.

---

# 5. PART 1 — Calendar, ages, demographics (colony only; ADR 0017)

## 5.1 Calendar
`WorldRules.calendar: CalendarRule { ticks_per_day: 1440, days_per_season: 6, seasons_per_year: 4 }` — present in both worlds; **baseline keeps `days_per_season: 7`** (its current value, so nothing changes), colony uses 6. Derived constants are methods, never literals: `ticks_per_year()`, `years_to_ticks(y)`, `days_to_ticks(d)`.
- Every rule that holds a duration in days or ticks is audited: list them (DECISIONS table: rule, field, current value, unit, colony value, reason). Values that mean "a season" or "winter" are expressed from the calendar; values that mean "a day" stay.
- Time snapshot adds `year_index`, `season_index`, `day_of_year`, `days_per_year`. The top bar shows `3년 차 · 봄 2일째` style (the mock-up's string); keys `UI_TIME_YEAR_FMT` (en "Year {year}", ko "{year}년 차").

## 5.2 Ages in years
- Display: age in years = `age_ticks / ticks_per_year` (integer). Inspector and chronicle use years everywhere ("54일을 살고" never appears again); deaths of old age say "{age}세".
- Colony `AgingRule` in years converted through the calendar: elder at **50**, lifespan mean **62**, spread **8** (keyed draw per person at birth, clamped 40–85), founders **16–40**, founder minimum remaining **10**.
- `LifeStageRule`: child until **15** years; add **infant until 3** years (a `Child` with `infant: bool` derived from age, no new component): infants do not forage, do not work, and are carried or stay at the hearth (use the existing child tether behaviour). Maturation at 15 removes `Child`.
- All ages computed from `born_tick` (closed form). Nothing stores an incrementing age.

## 5.3 Demographics (`BirthRule::Fertility`, colony only)
```rust
BirthRule::Fertility {
    conception_per_day_permille: 12,     // ≈ 0.25 births / woman-year at 24-day years
    fertile_from_years: 16, fertile_to_years: 45,
    gestation_days: 18,                  // 9 of 12 "months" of a 24-day year
    postpartum_days: 48,                 // ≥ 2 years between births
    requires_partner: true,
    max_mean_hunger: 60.0, max_mean_thirst: 60.0,   // the same gate as today, read from the settlement means
    food_cost: 10,
}
```
- `FertilitySystem` (priority 150, **interval 1,440**, runs at 06:00 of each day): for each fertile woman with a partner, not pregnant, past postpartum, in a fed settlement: keyed draw `("conceive", seed, agent_id, day_index)` < permille → `Pregnant { due_tick }` (new component). At `due_tick` the existing birth path spawns the child into her household (reuse Pass 17's household code; name the function). Pregnancy shows in the inspector ("임신 중 · {days}일 남음"; keys `UI_INSPECT_PREGNANT_FMT`).
- Replace nothing in baseline: `Provisioned` stays its rule.
- Expected: 10 founders (say 4 fertile paired women) → about 1 birth per year; see 9.1 for the bars.

## 5.4 Seasons at 6 days
Food regrowth per season, firewood winter prep, cold nights and the stock target all read `days_per_season` already or through the audit of 5.1. Calibrate on seeds 0–7 (as M1 §8.3: at most 6 rounds, every round recorded): the colony must survive **3 years without orders** on 8/8 seeds with no hunger, thirst or cold death. Allowed knobs: bush/deadwood density near the camp (0.5–2×), `FirewoodRule` winter days, `StockTargetRule.base_days`.

## 5.5 Tests
- `calendar_derivations` (ticks per year 34,560 in colony; baseline 40,320).
- `ages_in_years` (born at tick t, now t + 15 years → adult).
- `fertility_rate_matches_rule` (1,000 woman-years simulated through the system with full food: births within ±15 % of 250).
- `no_births_without_partner_or_food`.
- `pregnancy_to_birth_household`.
- **Baseline identity** (gate).

---

# 6. PART 2 — The standing benchmark and the spatial index

## 6.1 Benchmark (`worldsim_perf bench`, gate step `bench`)
- Scenes: colony 192×192 with **N ∈ {200, 1,000, 2,000}** founders (`WorldGenConfig::colony_stress(seed, n)`: founders scaled, bushes and deadwood scaled with N so nobody starves inside the run), seed 15100, **2,880 ticks** (2 days) after a 1,440-tick warm-up, **no orders**; plus N = 200 **with the M1 U-script scaled ×4** (60 trees, 4 huts, a 8×8 stockpile).
- Measured: tick mean, p99, max (µs, `tick_profiled`); per-system share (top 10 in the report); **memory**: RSS before `build_world_with`, after it, and after the run (`libc getrusage`/`mach_task_basic_info` on macOS; `/proc/self/statm` on Linux), reported as bytes per agent = (after build − before) / N at N = 2,000 minus the N = 200 intercept, and bytes per tile from a 64×64 vs 192×192 pair at N = 200.
- Output: `$DATA/bench/<commit>/bench.json` and the ledger line appended to **`docs/perf/PERF_LEDGER.md`** (columns: date, commit, N, tick mean/p99/max, RSS/agent, RSS/tile, note). The gate step compares the current run with the previous ledger line: > +10 % on any of tick mean, p99, RSS/agent → FAIL (unless `--justified` with the CHANGELOG line quoted). Ceilings of C3 are reported as PASS/MISS and **do not fail the gate in this pass** (they become hard in M2 if met here; otherwise M-PERF is scheduled first).
- Run it 3 times, take the median; record machine load (`uptime` load average) beside the numbers.

## 6.2 Spatial index (`sim-core/src/spatial.rs`)
- `ChunkIndex<T>`: 16×16-tile chunks over the map; `BTreeMap<ChunkId, Vec<(tile, T)>>` with sorted vectors (`(y, x)`, then id). Queries: `ring(center, radius_chunks)` iterator in ring order, `nearest_where(center, max_radius, pred)` returning the Chebyshev-nearest with `(y, x)`-then-id ties (exactly the order `assign.rs::nearest` produces today).
- Instances in `SimResources`: agents (rebuilt each tick at priority 119 from `Position`, O(N)); trees, ground items, zone cells and sites (updated on change by the code that changes them: `commands.rs`, `jobs/execute.rs`, `jobs/items.rs`).
- **`JobAssignSystem` uses the index** for every candidate search (Deliver source stacks, Build sites, Chop trees, Haul stacks and cells). Test: on the M1 fixtures (seeds 100–131, 10 days) the **colony summaries and chronicle are byte-identical** before and after the index (the index must reproduce the same nearest and the same ties).
- Report the per-system share of `JobAssignSystem` at N = 1,000 before and after.

## 6.3 Index use in the decision scans (droppable)
`nearest_gather_tile`, bush and water scans in `decision/system.rs` may switch to the index **only if baseline identity still passes** (same results, same ties). If identity breaks, revert that part and record why; M-PERF takes it.

---

# 7. PART 3 — Save and load (droppable to M2)
- `SimEngine::save(&self) -> Result<Vec<u8>, SaveError>` and `SimEngine::load(bytes) -> Result<SimEngine, SaveError>`; format = `postcard` (or `bincode` 1.x) of `{ FORMAT_VERSION: u32, rules, seed, current_tick, resources (serde), world (hecs::serialize::column with a component registry listing every component type), command_log }`. Every `HashMap` in `SimResources` is written in sorted key order (serialize through a `BTreeMap` view) so two saves of one state are byte-identical.
- Tests: `save_load_roundtrip`: for seeds 0–7, tick 2,000 → save → load → tick 1,000 == tick 3,000 continuous (probe fingerprint + chronicle); `save_is_deterministic` (two saves equal); `load_rejects_other_version`.
- Bridge: `save_world(path: String) -> bool`, `load_world(path: String) -> bool` (`#[func]`, API list updated); keys `F5`/`F9` in `view.gd` with a toast (keys `UI_SAVE_DONE`, `UI_LOAD_DONE`, `UI_SAVE_FAILED`; ko "저장함", "불러옴", "저장 실패"). Save path: `user://saves/<seed>-<tick>.wsave`.
- Gate step `save_roundtrip`.

---

# 8. PART 4 — Text renderer and job churn

## 8.1 Text renderer (C2's proof)
- `worldsim_probe --ascii --tick T [--rect x,y,w,h]` prints a glyph grid from the **same snapshot reads the bridge uses** (call the `collect_*` functions, not the systems): terrain (`.` grass, `:` dirt, `~` water, `^` hill, `#` mountain), `T` tree, `t` stump, `l`/`L` log stack (< 9 / ≥ 9), `=` stockpile cell, `H` hut, `h` shelter, `*` hearth, `b` blueprint, `x` chop mark; people as `@` (adult), `a` (child), `@` with a following `+` when carrying; `z` asleep. Legend printed with `--ascii-legend`. A `docs/v8/snapshot_schema.md` documents every snapshot read (name, columns, types, id spaces, version) and `check_docs.sh` verifies the column names against `bridge_api.py`'s extraction.
- `--ascii-diff A B` (droppable): two ticks side by side with changed cells marked.
- **Agreement test** (gate step `renderer_agreement`): at tick 900 of seed 100 with the U-script, the set of (x, y) of people from the text renderer equals the set the GUI test dumps from `agent_layer.gd` (M1's `tools/v8/m1_gui_test.gd` already dumps positions; extend if needed).

## 8.2 Job churn (pre-registered bar: finished / taken ≥ 0.5 on ≥ 28 of 32 evaluation seeds; U6 deaths still 0)
1. **Diagnose first:** on the M1 fixtures, histogram `CancelReason::Needs` by need (hunger / thirst / sleep / social / other) and by job stage; report it.
2. **Remedy 1:** a job yields to a need only at the need's **urgent** threshold (a new `ColonyRule.yield_margin_permille`: yield when the need is within this margin of the seek threshold or beyond it; start at 150 ‰ above the normal seek threshold, calibrate on seeds 0–7).
3. **Remedy 2 (droppable):** a `Haul`/`Deliver` in `ToDest` with the destination within 12 tiles finishes the leg before yielding.
4. Re-run the M1 U-script on seeds 100–131; report taken / finished / released-by-need before and after.

---

# 9. PART 5 — Evaluation (fresh seeds 200–231; calibration seeds 0–7 never feed back)
Runs: colony, **3 years (72 days)** without orders; and the M1 U-script, 10 days (for J1). All with `--check-determinism` on 4 seeds.

| # | bar | pass |
|---|---|---|
| D1 | births per woman-year (fertile, paired, fed days only) in [0.15, 0.40] | mean over seeds in range; no seed > 0.6 |
| D2 | population after 3 years in [8, 22], no extinction | ≥ 28 / 32 |
| D3 | no old-age death under 40 years; founders' old-age deaths in [45, 85] years | 32 / 32 |
| D4 | no hunger / thirst / cold death in 3 years without orders | ≥ 30 / 32 (a miss is reported with its cause) |
| D5 | a child born in year 1 is an adult at 15 years and has no `Child` marker after | 32 / 32 where a birth exists |
| J1 | jobs finished / taken ≥ 0.5 with the U-script, 10 days | ≥ 28 / 32 |
| P1 | benchmark ledger line written; regression vs base ≤ +10 % on tick mean, p99, RSS/agent at N = 1,000 | pass |
| P2 | C3 ceilings at N = 1,000 and 2,000 | reported PASS/MISS |
| S1 | save/load round trip identical, seeds 0–7 | 8 / 8 (if Part 3 kept) |
| A1 | renderer agreement | pass |
| U8 | baseline identity | pass |

---

# 10. PART 6 — Docs, review, gate, report
## 10.1 Docs (end-of-pass checklist, all of it)
CHANGELOG; ROADMAP (M1.5 done; "다음" → M2, or M-PERF if P2 misses badly — recommend); HISTORY; ADR 0016, 0017 (calendar: 6-day seasons, 24-day year, ages in years, fertility model, with the demographic reasoning: ~0.25 births/woman-year gives ~3 % growth with no migration, so growth to 100 depends on migrants as ADR 0006 intends); ARCHITECTURE (system table regenerated; §2 time scale rewritten; §4 the new reads; a new §8 "Contract and benchmark"); `docs/v8/snapshot_schema.md`; `docs/perf/PERF_LEDGER.md`; skills (`worldsim-architecture`: C1–C5 as rules, "use the index", "closed-form state", "no new f32"; `worldsim-testing`: the bench and ledger, save round trip, renderer agreement; pitfalls lessons).
## 10.2 Review (two fresh subagents)
- Code: determinism of the index (tie order), the fertility draw keys, calendar audit completeness (grep every `days`/`ticks` field against the DECISIONS table), serialization order, decision-seam line count (`git diff --numstat` on `decision/system.rs` ≤ 20 lines unless 6.3 kept and identity green).
- Numbers: recompute D1 and J1 from the raw probe output independently.
## 10.3 Screen checks (windowed, two shots → `$DATA/shots/`, paths in the report)
The top bar reading `1년 차 · 봄 1일째`; an inspector on a founder showing an age in years; if 7 kept: F5 then F9 round trip with the toasts.
## 10.4 Gate and publish
`tools/v8/gate.sh` gains `bench`, `renderer_agreement`, `save_roundtrip` (if kept), `schema_docs`. Green on a clean commit → fast-forward `v8/main` → report commit → docs step → fast-forward.

---

# 11. Not in M1.5
Needs conversion from `f32` to fixed-point (M-PERF); LOD / aggregate simulation of far agents; pathfinding; child mortality and illness (M5); migrants (M5); the 32-px view decision (art trial); any change to `baseline`.

# 12. 인게임 확인사항 (오너용)
1. 위 막대가 `1년 차 · 봄 1일째`처럼 읽히고, 계절이 6일마다 바뀐다. 1배속에서 1년이 약 48분이다.
2. 사람을 클릭하면 나이가 '세'로 보인다. 창설자는 16~40세, 아이는 15세에 어른이 된다.
3. 10일 지나도 인구가 두 배가 되지 않는다. 1년에 한두 명 태어나고, 임신한 사람은 상세 정보에 "임신 중"이 보인다.
4. 3년을 아무 지시 없이 돌려도 굶거나 얼어 죽지 않는다 (시드 대부분).
5. 나무 베기·비축·오두막 주문을 내리면 사람들이 밥 먹으러 가느라 일을 자꾸 놓지 않고, 맡은 일의 절반 이상을 끝낸다.
6. (저장이 들어갔다면) F5로 저장, F9로 불러오기가 되고, 불러온 뒤 세계가 그대로 이어진다.
7. 텍스트 렌더러: `worldsim_probe --ascii`로 Godot 없이 세계가 글자 격자로 찍힌다.
8. 벤치마크 장부 `docs/perf/PERF_LEDGER.md`에 200/1,000/2,000명 수치가 적혀 있고, 1,000명에서 틱 평균이 얼마인지 보인다.

# 13. 구현 완료 보고 (한국어; `docs/v8/m1_5/M1_5_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M1.5
### 구현 의도
### 계약 (ADR 0016 요약, 게이트 단계 대응표, 지금 지켜지는 것과 아직 아닌 것)
### 구현 내용 (달력·나이·출산, 색인, 저장/불러오기, 텍스트 렌더러, 일감 교체 — 파일별)
### 구현 방법 (달력 감사 표 요약, 키 기반 출산 판정, 색인 동률 규칙, 직렬화 순서, 교체 진단 히스토그램)
### 평가 결과 (D1–D5, J1, P1–P2, S1, A1, U8 표 + 시드별 부록 경로)
### 벤치마크 (N=200/1,000/2,000 틱 평균·p99·max, 시스템별 상위 10, 사람당·타일당 메모리, 기준 대비)
### 보정 기록 (시드 0–7, 회차별)
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항 (드롭한 것, f32 잔존, 1,200줄 초과 파일)
### 오너 결정 필요 (다음: M2 vs M-PERF 권고와 근거)
### Gate 결과
git ls-remote 결과
```
