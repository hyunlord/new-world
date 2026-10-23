# GOAL: WorldSim — M5: pressure and story — a storyteller that applies seasons' pressure (cold snaps, drought, beasts, simple illness), drafting people to meet a threat, hunting and hides that connect knowledge to warmth and to the art's apparel, mood and quarrels that reconnect temperament, family and relations to the new loop, and a chronicle in which "who quarrelled with whom and why" can be followed. Plus the M4 debts: the ledger rule (second fix), tool wear that actually consumes blades, the art record corrected.

Repository `hyunlord/new-world`. Base: **`v8/main` = `08f598133552cebd27b16994b662e50811bd32e7`** (M4 report; gate green at `941410fc`). `v8/wip` at the same commit.

Executed by **Claude Code**, unattended, in `/Users/rexxa/work/new-world-pass4`. No DGX. Headless except 12.3. Target 22 hours, hard stop 26. The art session may run ART-PROD2 on `v8/art-prod1` in parallel; do not touch `art/**`, `docs/v8/art*/**`, do not merge art branches.

## Why this pass exists

ROADMAP M5: "한파가 오면 장작이 모자라고, 누가 왜 다퉜는지 연대기로 따라갈 수 있다." VISION §7 items 5 and 6: the storyteller applies pressure at times the system chooses (the god's hand is gone), beasts are met by RimWorld-style drafting, illness stays simple; and V7's third signature — the person detail, the chronicle and "why?" — must carry the new loop's causes. Today the colony has needs, rooms, jobs, knowledge; it has no weather beyond the calendar's winter severity, no animals, no apparel, no mood, and people never quarrel. The V7 relation machinery (`RelationState` −1,000..1,000 with kin bases, `RelationshipState` familiarity/hostility, `CombatSystem` 137, `Temperament` NS/HA/RD/P, `Family`, `Memory`) exists but nothing in the colony drives it.

M5 also closes the economy's loose end from M2–M4: skill growth (F8) was never measurable because nothing consumes blades. Hunting and hide work give blades a real sink (knives wear), so the measurement finally has a subject. And the art line now has people (route I); hides → apparel is the first place the simulation decides what a person wears, which the art's three tops already depict.

---

# 0. RULES
- AGENTS.md and the skills (`worldsim-architecture` first; procedures for commands, jobs, items, blueprints, rooms, knowledge). Commit to `v8/wip` (`v8-m5(pN): …`), `v8/main` by fast-forward after a green gate. Never `lead/main`, `main`, `codex/*`, `v8/art-*`; no force-push; explicit paths; hook/ignore files and the untracked `.ignore`, `docs/revitalize/`, `.gitignore` change untouched.
- **Never ask the owner** → `docs/v8/m5/M5_DECISIONS.md`.
- C3 ceilings and the regression rule (with the Part 0 second fix) are hard. `decision/system.rs`: ≤ +20 lines.
- Parts: 0 = §5 · 1 = §6 (storyteller + weather pressure) · 2 = §7 (animals, hunting, drafting) · 3 = §8 (hides, apparel, knife wear) · 4 = §9 (mood, quarrels, chronicle) · 5 = §10 (UI) · 6 = §11 (evaluation) · 7 = §12 (docs, review, gate, report). Push after every Part.
- **Time rule:** past 26 h, finish the step, push, go to Part 7.
- **Drop order:** 6.4 illness → 8.5 fur cloak from wolves (keep hide wrap) → 9.5 year summary in the chronicle → 7.6 deer flee behaviour (deer stand) → 10.6 chronicle filters (keep the list).
- **Never drop:** Part 0, the storyteller with cold snap and drought, wolves + drafting + spears, deer hunting → meat + hide → hide wrap, knife wear, mood with the two thresholds, quarrels with a cause chain in "why?" and the chronicle, evaluation, identity, docs, report.

# 1. MACHINE
Pass4 checkout + base worktree `../new-world-m5-base` at `08f59813`. Data `/Users/rexxa/work/m5-data/`. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.

# 2. CONTEXT (planner's survey at `08f59813`; verify)
- **Rules:** `sim-core/src/rules/{baseline,calendar,colony,knowledge,routine}.rs`; `winter_severity_permille` draws mild/normal/harsh with `burn_per_hour [2,3,4]` and regen per day — the only weather today. `FireSystem` 147 / `HearthFeedSystem` 146 / `ExposureRule`: night cold outside roofed rooms; M3 roofed rooms protect.
- **People:** `Temperament { ns, ha, rd, p }` u16 (NS idle wander, HA bedtime, RD loneliness, P load); `Social.loneliness` f64 (baseline, `SocialDecay`/`SocialInteractionSystem`); `RelationState { value, estranged, base }` per pair (kin bases), `RelationshipState { familiarity, hostility }`; `CombatSystem` 137 (pairs in `SimResources.combat_pairs`, `BodyHealth`, deaths); `Memory` (32 entries, salience); `Family`, `Lineage`, births (M1.5), ages.
- **Colony:** jobs, items (M2–M4: Log, Firewood, Stone, Flint, FlintBlade, StoneAxe, GroundAxe, Hide, Clay, RawPot, ClayPot), workshops (KnappingSpot, WoodBench, DryingRack (no recipe yet), PotteryBench, GrindingStone, hearth firing), bills, priorities (7 skills incl. Pottery), tool wear (StoneAxe 40 chops, GroundAxe 120), knowledge 24 nodes incl. `tracking` (lore only) and `hide_scraping` (gates DryingRack), rooms/roles, nav + regions + fords, chronicle kinds 0–28, the causal "why?" panel, `debug_kill` (tests/probe only).
- **No animals exist** in the colony crates (verify `grep -rn Animal rust/crates`); V7 "wildlife" is not present at M4.
- **Bench ledger (M4 lesson):** `bench_ledger.py check` compares against the latest quiet pass rows; M4 appended its own rows at p5, so the gate at p6 compared M4 with M4. The M3→M4 comparison was done by hand in F12: n200 ×1.20–1.25 (fords; justified), n1000 p99 ×1.106 (unexplained), RSS/agent ×1.14 (2,030 → 2,321 B; plan was +160 B).
- **UI keys in use:** F1–F4, F6–F10, R, P, T, K, Space, Home/H, F5/F9. Free: **D** (draft/undraft), **M** (mood overlay), **C** (chronicle, if not already bound — verify; else keep the existing key).
- **Art state (§5.1):** ART-PROD1 (`eb428ce2`): route I (whole-figure sheets) accepted 13/18 adult looks, children failed only on the sleep-frame rule; ART-PROD2 running to fix and adopt. M4's "ART-TRIAL5 최종 실패 · 사람 그림 보류" is **out of date**: people art is route I, pending PROD2.

# 3. DESIGN

## 3.1 Storyteller (ADR 0027)
`StorytellerSystem` priority 170, interval 1,440 (once a day at tick 0 of the day). State in `SimResources.story: Storyteller { next_check_day, last_major_day, history: Vec<(day, PressureKind)> }`. Keyed RNG `STREAM_STORY`.
- **Pressure kinds and effects:**

| kind | season | duration | effect | mercy |
|---|---|---|---|---|
| ColdSnap | winter | 2–4 days | severity harsh: burn_per_hour ×2 (fire), outdoor exposure ×2, work outdoors ×1,300 ‰ ticks; people in a roofed room with a hearth-fed hall are unaffected at night | not in the first winter's first 2 days |
| Drought | summer | 4–6 days | forage regen ×400 ‰, clay banks and river tiles unchanged, thirst decay ×1,200 ‰ | not before day 12 |
| Beasts | any but winter's last days | until resolved | a wolf pack (2–4) spawns at the map edge nearest the forest and moves toward the camp; see 3.2 | not before day 12; not within 6 days of the last Beasts |
| Illness (6.4, droppable) | autumn/winter | 3 days per person | one random adult: `Sick { until }`: work ticks ×1,500 ‰, sleep need ×1,500 ‰; 20 %/night to a roommate in the same bedroom; no death by itself | not before day 18 |

- **Scheduling:** each season gets a budget: 1 major (ColdSnap/Drought/Beasts) + 1 minor (Illness); the day of the major is drawn uniformly inside the season's days 2–5; the kind by season table above with weights ColdSnap 60/Beasts 40 in winter, Drought 60/Beasts 40 in summer, Beasts 70/Illness 30 in spring/autumn; **intensity scales with wealth**: wolves = 2 + (items in stockpiles / 150) capped 4; drought/cold snap duration +1 day when adults ≥ 20. Minimum gap between majors 6 days. Everything deterministic by seed. Events: chronicle `PressureBegan { kind }`, `PressureEnded { kind, cost }` (cost: deaths, items lost, days of work lost); HUD banner during a pressure.
- **"Why?"** on a frozen person, an empty forage, a wounded person names the pressure as the cause (causal event `PressureCause`).

## 3.2 Animals, hunting, drafting (ADR 0028)
- **Animals:** new entity archetype with `Animal { kind: Wolf | Deer, hp: u16, state: Wander | Flee | Hunt | Attack { target } | Dead }`, `Position`, in the spatial index. `AnimalSystem` 171, interval 3: deer wander within forest/plain, **flee** (7.6, droppable) from people within 6 tiles at speed 2 tiles per 3 ticks; wolves hunt: target the nearest person outside a roofed room within 24 tiles, attack adjacent (uses `CombatSystem` pairs with the wolf as attacker, `DAMAGE` 6 per resolved combat on a 100-hp body; a wolf has 40 hp), retreat when the pack loses half its number; despawn when all dead or after 2 days. Deer population: 6–12 per map, respawn 1 per 4 days up to the cap, only in forest regions ≥ 20 tiles from the hearth.
- **Hunting:** designation `Hunt` on a deer (click) or a **Hunt zone** (rect) — gated by `tracking` knowledge; `JobKind::Hunt { animal }` (`WorkType::Hunt`, new 8th skill `Hunting`): approach within 2 tiles, attack ticks 60 modulated by skill, deer hp 30, hit chance table by level (40 % at 0 → 80 % at 20); a kill drops `Meat 3` (food item: eaten like foraged food, 2 hunger units each, spoils after 6 days → `consumed` in the ledger) and `Hide 1` at the tile. A spear (3.3) doubles hit chance class. Chronicle `Hunted`.
- **Drafting:** command `Draft { agent }` / `Undraft { agent }` (recorded): a drafted person cancels their job and takes only direct orders `MoveTo { tile }` and `Attack { animal | agent }`; `AgentDecisionSystem` skips drafted people (seam ≤ +20 lines); needs still tick, and a drafted person at a critical need undrafts themselves with a toast. Weapons: a held StoneAxe/GroundAxe = damage 8/10, a Spear 12, bare hands 4; the drafted person's attack uses `CombatSystem` with the same pair machinery (person vs animal). UI in 3.5.
- **Spear:** recipe `haft_spear` at the WoodBench: FlintBlade 1 + Log 1 → Spear 1, 60 ticks, skill Woodwork, gated by `hafting`; durability 30 attacks. Auto-arm: a drafted person without a weapon picks up the nearest Spear/axe within 12 tiles first.

## 3.3 Hides, apparel, knife wear (ADR 0029)
- **Hide work:** DryingRack recipe `scrape_hide`: Hide 1 + **knife use** → CuredHide 1, 60 ticks, skill `Knap` (reuse), gated by `hide_scraping`. **Knife = a FlintBlade held as a tool** with durability **8 uses** (scrape, butcher); it breaks like an axe (chronicle `ToolBroke`). Butchering is implicit in the kill (no extra job) but **consumes 1 knife use** from the hunter's blade if they carry one (else meat 2 instead of 3). This is the blade sink F8 needs.
- **Apparel:** component `Apparel { top: None | HideWrap | FurCloak, since: u32 }` on people; items `HideWrap` (recipe at the DryingRack: CuredHide 2, 90 ticks, gated `hide_scraping`) and `FurCloak` (8.5, droppable: WolfHide 2 → from a wolf kill; 120 ticks). Effects: HideWrap exposure cold ×700 ‰; FurCloak ×500 ‰; outdoors work in a ColdSnap unaffected by ×1,300 for FurCloak wearers. **Dressing** is a job: `JobKind::Equip { item }` (Haul type) created by a bill-like standing order "모두 가죽 옷" (default on) that assigns the warmest available top to the person with the lowest cold tolerance first (deterministic order: hardiness, then AgentId). Apparel is a snapshot field (the art's top: loincloth = None).
- **Ledger:** Meat, Hide, CuredHide, HideWrap, FurCloak, Spear, WolfHide join the per-kind invariant; spoilage and breakage are `consumed`.

## 3.4 Mood, quarrels, chronicle (ADR 0030)
- **Mood** (`MoodSystem` 172, interval 60, Warm): `Mood { value: i16 /* 0..=1000 */ }` on adults and children ≥ 10. Each interval: value moves toward a target by 1/8 of the gap; target = 500 + Σ modifiers: slept in a roofed bedroom +80, slept outside −120, ate today +40 / hungry (below seek threshold) −150, thirsty −100, cold last night −120, quarrelled today −100, taught or learned today +60, discovered something +150 (3 days), a kin died −250 (6 days), sick −80, worked at a gated craft they are a practitioner of +40, drafted ≥ 1 day −60, pressure active −40. `Temperament.ha` scales negative modifiers (×(800 + ha/5) ‰), `rd` scales social ones.
- **Thresholds:** `value < 250` → **tantrum risk**: quarrel chance ×3 and work ticks ×1,200 ‰ (state word 우울); `value < 100` → **breakdown**: refuses jobs for 1 day, wanders (chronicle `Breakdown`, "why?" lists the three largest modifiers); `value ≥ 750` → 흡족: work ticks ×900 ‰.
- **Quarrels:** in `SocialInteractionSystem` (verify its colony use; if baseline-only, add a `QuarrelSystem` 173, interval 30): for two people within 2 tiles who are not kin-close (relation base ≥ 300) and whose `RelationState.value < 0` **or** either has mood < 250, chance per check `QUARREL_PERMILLE = 2 + (250 − min(mood))/25 + (−value)/100` (cap 40 ‰); on a quarrel: relation −60 (×NS/500 of the initiator), hostility +0.1, mood −100 both, chronicle `Quarrel { a, b, cause }` where **cause** is the largest negative mood modifier of the initiator (hunger / cold / sleep / grief / pressure / relation); if hostility ≥ 0.6 and both mood < 250 → a fight via `CombatSystem` (damage 4, stops at first resolved combat; chronicle `Fight`). Kin bases and births keep feeding `RelationState` as in V7 (verify they run in the colony).
- **Chronicle upgrade:** entries carry `cause: Option<CauseRef>` (a chronicle entry id or a pressure/modifier tag) so the panel can show "→ 왜?" chains; new kinds `PressureBegan, PressureEnded, Hunted, Quarrel, Fight, Breakdown, Drafted, WolfKilled, Sick, Recovered`; the panel gets **filters** (10.6, droppable: by person, by kind group) and a **year summary** (9.5, droppable) at each new year: births, deaths by cause, pressures, discoveries, quarrels count, era progress. Person detail gets a "이야기" tab: that person's chronicle lines.

## 3.5 UI
- **D — draft:** select person → D toggles draft (red outline); right-click tile = move, right-click animal/person = attack; a drafted list in the HUD with undraft-all; auto-undraft toast.
- **Animals drawn** (temporary sprites in `gen_tiles.py`: wolf, deer, carcass); **Hunt** by F7 submenu `3` (deer click) or F7-4 Hunt zone drag; wolves' approach banner "늑대 무리가 다가온다 (3)" with a pulse on the edge.
- **Pressure banner** in the HUD: "한파 2일째 · 장작 12 (하루 18 필요)" — the firewood line computes need from burn rate × hearths; "가뭄 3일째"; illness badge on the person.
- **M — mood overlay:** people tinted by mood band (우울 파랑, 보통 없음, 흡족 노랑); inspector "기분: 우울 (210) — 춥게 잠, 배고픔, 다툼" (top 3 modifiers).
- **Apparel:** inspector line "옷: 가죽 랩 (추위 −30 %)"; the standing order toggle in the priorities panel header.
- **Chronicle:** causes shown as "→" lines; filters; year summary; the person's 이야기 tab.
- Localization keys in `docs/v8/m5/M5_LOCALE.md` (≥ 90 expected).

---

# 4. PERFORMANCE PLAN
Animals ≤ 16 entities at once (deer ≤ 12 + wolves ≤ 4): `AnimalSystem` ≤ 20 µs/tick at N = 1,000. Mood every 60 ticks over N: ≤ 30 µs/tick amortized. Quarrel checks ride the existing pair iteration or the chunk index (only pairs within 2 tiles, only people not at work-in-room): ≤ 40 µs/tick. Storyteller daily: negligible. Budget for all M5 systems ≤ 120 µs/tick at N = 1,000; memory +24 B/person (Mood 2 + Apparel 6 + Sick 4 + padding). **Measure allocation precisely:** Part 0 adds a bytes-per-agent line from the allocator counter (`sim_core::alloc::Counting` wrapper used only by `worldsim_perf`), reported beside RSS/agent; the regression rule applies to the allocator number from M5 on (RSS stays as information).

---

# 5. PART 0 — Records and debts
1. Commit this goal as `docs/v8/m5/M5_GOAL.md`.
2. **Art record correction:** ADR 0018 status, ROADMAP (ART-PROD1 row: route I, 13/18 adults, sleep-rule failure; ART-PROD2 running), HISTORY; "사람 그림 보류" withdrawn. Final state from `docs/v8/artprod2/` at Part 7.
3. **Ledger rule, second fix:** `bench_ledger.py check` compares the candidate with the latest quiet rows **whose `pass` label differs from the candidate's** (and whose commit is not a descendant of the base); `append` is refused unless the gate log for that commit exists and passed (`--gate-log`); the gate step orders: check first, append only in the report step. Re-check M4 (`941410fc`) against the quiet M3 rows and record the true ratios in the ledger note; the n1000 p99 ×1.106 gets one profiling look (perf summary per system at N = 1,000, M3 vs M4; name the system or say "not found within 30 min").
4. Allocator byte counter (§4) in `worldsim_perf` and the ledger (new column `alloc/agent (B)`; RSS kept).
5. M4 owner decisions recorded: teach dialog auto-suggests the learner (highest NS, then youngest adult; editable); a re-learned knowledge keeps its original source's floor (D); forgetting speed unchanged pending playtest; pot firing unchanged.
6. ADR 0027–0030 drafted. Base data → `$DATA/base/`.
Push.

# 6. PART 1 — Storyteller and weather pressure
§3.1. Tests: the schedule is deterministic and respects mercy/gaps on seeds 0–7 × 2 years; ColdSnap doubles burn and a roofed hall-with-hearth colony takes no night cold; Drought halves forage regen for its days; Illness spreads only within a bedroom (if kept); chronicle begin/end with cost; identity.

# 7. PART 2 — Animals, hunting, drafting
§3.2. Tests: deer spawn/respawn in forest regions only; a Hunt job kills a deer and drops Meat 3 + Hide 1 (ledger); a wolf pack approaches and attacks a person outside, never one in a roofed room; `Draft` cancels the job, `Attack` resolves through `CombatSystem`, a 3-spear party kills a 3-wolf pack with ≤ 1 person below 50 hp (seeds 0–7, ≥ 6/8); auto-undraft at critical need; the decision seam diff ≤ +20 lines; identity.

# 8. PART 3 — Hides, apparel, knife wear
§3.3. Tests: scrape → CuredHide; HideWrap made and worn by the person with the lowest hardiness first; exposure ×700 ‰ applied; knife uses decrement and break (ledger); meat spoils at 6 days; FurCloak from a wolf hide (if kept).

# 9. PART 4 — Mood, quarrels, chronicle
§3.4. Tests: mood converges toward the modifier target (unit); breakdown at < 100 refuses jobs for a day and is chronicled with three modifiers; a quarrel between two low-relation hungry neighbours has `cause = Hunger` and the "why?" chain resolves to the chronicle entry; a fight only above hostility 0.6; kin never quarrel below base 300; chronicle causes serialize (save v5) and replay byte-identical.

# 10. PART 5 — UI
§3.5. `m5_ui_test.gd` (≥ 30 checks). Screen set `capture.gd --m5-set`: wolves approaching with the banner, a drafted party with spears, the mood overlay, a quarrel's "why?" chain, a cold snap banner with the firewood line, the person's 이야기 tab, a hunted deer, hide wraps in the inspector.

# 11. PART 6 — Evaluation (held-out seeds 800–831; calibration seeds 0–7, ≤ 8 rounds, logged)
**M5-U script** (resolved per seed): M4-U's economy + house + storehouse + teach orders; day 2 `Make 4 Spear`; day 3 a Hunt zone over the nearest deer forest; day 4 `Make 6 HideWrap` standing; `Maintain 2 StoneAxe`, `Maintain 6 FlintBlade`; nothing else. Run **48 days (2 years)**; storyteller on. Also a **no-pressure control** (storyteller off) for F6/F9.

| # | bar | pass |
|---|---|---|
| F1 | every world sees ≥ 1 ColdSnap and ≥ 1 Beasts in 2 years; mercy and gaps hold | 32/32 |
| F2 | ColdSnap cost: firewood falls below one day's need during the snap in ≥ 24/32 (the pressure bites) **and** cold deaths over 2 years ≤ 1 in ≥ 28/32 (a prepared colony survives) | both |
| F3 | Beasts: the player's scripted draft (4 adults with spears at wolf spawn) kills the pack with 0 deaths in ≥ 24/32; without the draft (control script) ≥ 1 injury in ≥ 24/32 (drafting matters) | both |
| F4 | hunting: ≥ 6 deer killed by day 48, meat eaten ≥ 12 units, hides ≥ 6 | ≥ 28/32 |
| F5 | apparel: ≥ 4 people wear a HideWrap by day 30; their winter night cold events ≤ half of unclothed people's | ≥ 24/32 |
| F6 | mood: in the control, breakdowns ≤ 1 per world over 2 years in ≥ 28/32; with pressure, ≥ 1 world in 4 has a breakdown (pressure is felt) | both |
| F7 | quarrels: ≥ 2 quarrels per world-year, each with a resolvable cause chain in the chronicle; fights ≤ quarrels/4; 0 quarrel deaths | ≥ 28/32 |
| F8 | skill growth, finally: knives consume ≥ 10 blades per world by day 48; top knapper level ≥ 6; blade quality mean days 37–48 − days 1–12 ≥ 8 | ≥ 24/32 |
| F9 | demographics unchanged by M5 in the control: births/woman-year 0.20–0.30, 3-year no-orders run still ≥ 30/32 no cold death | all |
| F10 | ledger per kind exact (meat spoilage, knives, spears, hides); reservations; soak green (now with knowledge + wolves off) | all |
| F11 | M5 systems ≤ 120 µs/tick at N = 1,000; alloc/agent ≤ M4 + 40 B; C3; regression vs M4 quiet rows ≤ +10 % on the allocator number and tick means (bench with the fixed rule); identity; replay | all |

# 12. PART 7 — Docs, review, gate, report
- Docs: CHANGELOG; ROADMAP (M5 done; 다음 → M-ART2 (route I install, 64-px camera, environment tiles) recommended if PROD2 adopted, else M6 prep); HISTORY; ADR 0027–0030 final; ARCHITECTURE (system table 170–173, §12 story/animals/mood data flow; save v5); snapshot schema; PERF_LEDGER with the allocator column; skills (`worldsim-architecture`: storyteller cadence rules, "add a pressure kind", "add an animal", apparel/equipment procedure, mood modifiers table; `worldsim-testing`: control vs pressure runs; pitfalls: the self-comparison ledger lesson).
- Review: two fresh subagents — code (storyteller determinism; combat pair reuse for animals; draft/undraft cancellation paths; knife/meat/spear ledger; quarrel cause resolution; seam ≤ +20); numbers (F2, F3, F7, F8 recomputed from raw).
- 12.3 Screen checks (windowed): the eight shots of §10 to `$DATA/shots/`, `docs/images/m5_contact_sheet.png` replaces M4's.
- Gate (29 steps: + `story` fixture: 2 seeds × 48 days with the storyteller, 0 panics, ledger exact, F1 schedule check) green on a clean commit → fast-forward → report commit → ledger append (with the gate log) → docs → fast-forward.

# 13. Not in M5
Neighbours (migrants, traders), raids, disease beyond the simple illness, livestock/taming, weather rendering beyond banners, temperature per tile, mood memory beyond modifiers, LLM text, V7 inter-village relations.

# 14. 인게임 확인사항 (오너용)
1. 겨울에 "한파 2일째 · 장작 n (하루 m 필요)" 배너가 뜨고, 장작이 모자라면 밤에 사람들이 떨며 기분이 파랗게 내려간다. 창고에 장작을 쌓아 두면 버틴다.
2. "늑대 무리가 다가온다 (3)" 배너 → 사람을 고르고 D로 소집 → 우클릭으로 늑대를 치게 한다. 창 든 넷이면 이긴다. 소집 해제하면 하던 일로 돌아간다.
3. F7-3으로 사슴을 표시하면 추적을 아는 사람이 사냥하고, 고기와 가죽이 떨어진다. 고기는 6일 지나면 썩는다.
4. 건조대에서 가죽을 손질(날이 닳는다)하고 가죽 랩을 만들면 사람들이 입는다 — 인스펙터에 "옷: 가죽 랩", 화면의 사람 상의가 바뀐다(아트 설치 후).
5. M을 누르면 기분 색이 보인다. 배고프고 춥고 밖에서 잔 사람이 파랗고, 그런 둘이 붙어 있으면 다툰다. 연대기에서 "A가 B와 다퉜다 → 왜?"를 누르면 "배고픔"이 나온다.
6. 사람 상세의 "이야기" 탭에 그 사람의 일생이 줄줄이 보인다.
7. 날이 계속 닳아서 돌 다듬는 사람이 계속 만들고, 2년째엔 품질이 눈에 띄게 오른다.
8. 48일을 돌려도 FPS·틱 시간이 M4와 다르지 않다.

# 15. 구현 완료 보고 (한국어; `docs/v8/m5/M5_REPORT.md`와 최종 메시지)
```
## 구현 완료 보고 — M5
### 구현 의도
### Part 0 기록 (아트 정정, 장부 규칙 2차와 M3→M4 실제 비율·p99 조사, 할당 바이트 열, M4 결정 기록)
### 구현 내용 (이야기꾼·압력 4종, 동물·사냥·소집·창, 가죽·옷·칼 마모, 기분·다툼·연대기 원인, UI — 파일별)
### 구현 방법 (일정 규칙과 자비 규칙, 전투 쌍 재사용, 기분 수렴식과 수정자 표, 다툼 확률과 원인 선택, 결정 틱 접점 줄 수)
### 성능 계획과 실제 (시스템별 µs, 할당/사람, 장부)
### 평가 결과 (F1–F11 표 + 시드별 부록, 대조군 vs 압력)
### 보정 기록
### 변경된 파일 목록 (커밋별)
### 리뷰 결과
### 확인된 제한사항
### 오너 결정 필요 (다음: M-ART2 범위; 압력 강도 체감; 기분 수정자 균형; 소집 UI 흐름)
### Gate 결과
git ls-remote 결과
```
