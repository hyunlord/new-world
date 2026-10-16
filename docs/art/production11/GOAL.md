# GOAL: WorldSim — Art Pass 11: every life told on screen (names, the life story, the "why?"), activities you can read at a glance, and the frame budget measured honestly

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` = `codex/art-production-2026-10` = `cdf69c0e85a5876c821a8bf22b18089bf740c581` (Art Pass 10, TIMEBOXED_PARTIAL; draft PR #511).
This run is executed by **Codex**, unattended, on the Mac, in `/Users/rexxa/work/new-world-art-prod`. **The owner launches it only after the simulation's Pass 19 has finished** (section 1.2). No DGX.

**Simulation to merge:** the head of `codex/revitalize-2026-10` at launch (expected: Pass 19's green fast-forward; it brings Passes 18–19: `game_v23` elder partners follow, `game_v24` staggered day ends 17:00–18:59, and **the chronicle** — an append-only life record plus a person table that outlives death, read through new bridge getters). The live game becomes `game_v24`.

Four Parts of work, in this order: (0) merge and a release-build baseline; (1) **names, the life story and the "why?"** in the inspector and a chronicle panel (roadmap step 4); (2) **activities you can read** — icons for all seven, a village summary when zoomed out, and a blind test with a bar this time; (3) **the frame budget at 500**, starting from measured section timers. Art Pass 10's unmeasured after-verdicts are filled in when a hold is free (4.6).

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Decisions in `docs/art/production11/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production11/GOAL.md`, commit it in Part 0, re-read the current Part's section at every Part start and after any compaction.
3. **Parts:** 0 = section 4; 1 = section 5; 2 = section 6; 3 = section 7; 4 = section 8; 5 = section 9. Push after every Part.
4. **Review loops: two rounds maximum** (fresh reviewer that has not seen sections 3–7).
5. **On failure: work around, note it, continue.** Three strikes per failing target.
6. **Time rule:** record times in `docs/art/production11/TASKS.md`. **Start Part 4 no later than hour 9** (Art Pass 10 lost its final matrix to the cutoff while waiting for a quiet Mac). At that point the current pushed build is frozen for H1–H5; unfinished never-drop items may continue only as measurement or evidence on the frozen build (captures, reviewer sessions) or as docs — anything needing a code change is reported unfinished. Past **12 hours**, finish the current step, push, go to Part 5.
7. **Drop order:** Part 3 changes after the first accepted one → Art Pass 10's after-verdicts (4.6) → the chronicle panel's world feed (5.5) → the village activity summary (6.3; then 6.5 (d) is NOT_RUN and H7 says so). Never drop Parts 0, 4, 5, the names (5.1), the Life section (5.3), the seven icons (6.2) or the blind test (6.5 (c)).
8. **Image budget: at most 4 generations** with the image tool available to you (icons only, if procedural drawing or existing sprites cannot make a distinct silhouette); failures count.
9. **Honesty rules:** never mark a failed check as passed, never lower a bar of section 8 or 6.5, keep rejected evidence, never re-interpret an earlier pass's verdict.
10. **This brief supersedes the repository's session-start instructions:** no hooks, no harness pipeline, never `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW
## 1.1 Places and tools
- Mac only, `/Users/rexxa/work/new-world-art-prod`; evidence to `CAPTURE_EVIDENCE=/Users/rexxa/work/art-production11-evidence`; worktrees (removed at the end): `/Users/rexxa/work/art-p11-base` (identity A/A′, 4.3) and `/Users/rexxa/work/art-p11-measure` (frame measurement, 4.4). Never touch other checkouts or processes you did not start; no ComfyUI, no dev servers, no DGX.
- **Every worktree, after every checkout:** build its bridge (base: `cargo build -p sim-bridge`; measure: 4.4), then `"$GODOT_BIN" --headless --path <worktree> --import`, before any Godot launch (a fresh worktree has no `.godot/extension_list.cfg` and no imported textures; new icons need re-import).
- `CARGO_BUILD_JOBS=6`; `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`; windowed Godot only through `tools/art_pipeline/with_window_lock.sh`; limits via `run_bounded.sh`. Frame-time sessions inside a quiet-lock hold; identity captures outside holds (`run_identity_p10.sh` L34–53 waits for the lock to be absent).
- Reuse Art Pass 10's tools, copied to `_p11` names writing under `docs/art/production11/**`: `perf_frames_p10.gd`, `run_perf_matrix_p10.sh` (update its hard-coded baseline SHA, L17), `frame_measure_session_p10.gd`, `frame_cpu_probe_p10.gd`, `summarize_perf_p10.py`, `compare_perf_p10.py`, `summarize_ablation_p10.py`, `run_alternating_p10.py`, `validate_alternating_p10.py`, `perf_alternating_frames_p10.gd`, the section-timer chain (`generate_agent_sections_p10.py` — refresh its markers, L23–41, after Part 3 restructures the loop — `section_profile_loader_p10.gd`, `profile_agent_sections_p10.gd`, `agent_section_clock_p10.gd`, `run_agent_sections_p10.py`), `run_identity_p10.sh`, `capture_identity_p10.gd`, `capture_framings_p10.gd`, `pixel_identity_p10.py`, `capture_running_identity_p10.gd`, `compare_running_identity_p10.py`, `with_quiet_lock_p10.sh`; and Art Pass 10's blind crop script (`oracle-code-v2/prepare_blind_crops.py` inside `docs/art/production10/activity/verified-captures/raw-evidence.zip`) with its crop size.
## 1.2 Sharing the Mac
The owner starts this run after Pass 19 has ended; another simulation run may still start later. Quiet lock `/Users/rexxa/work/.mac_quiet.lock` as Art Pass 10 (`with_quiet_lock_p10.sh`: one hold per measurement session, ≤ 60 minutes, ≥ 10 minutes between holds, load < 4 and no foreign `cargo`/`rustc`/`Godot`/`worldsim_*`, contended after 60 minutes of waiting, stale after 2 hours, every hold and wait in `docs/art/production11/perf/holds.md`). While the other run holds it: no cargo, no Godot, no gate. **During your own holds you may edit and commit** (no Godot, cargo or gate outside the measured session).

---

# 2. GIT
## 2.1 Branches and scope
- Start from `cdf69c0e` (must equal origin); backup branch `art-p11-local-backup`. WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate is green; `git ls-remote` after every push. Messages `art-p11(pN): …`; stage explicit paths only; leave the local `.gitignore` / `.ignore` edits uncommitted.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**` (new icons only), `localization/**` (the keys of section 10 only), `project.godot`, `docs/art/production11/**`, `docs/art/TEST_LEDGER.md`, `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn` files (re-pinned with the same meaning; one ledger row each under `## Art Pass 11`).
- **Rust exceptions:** in `rust/crates/sim-bridge/src/ffi/world_node.rs` only the art-owned getters (`get_intervention_state`, `get_society_snapshot`, `get_tile_causal_events`), their collectors and tests. **The chronicle getters are simulation-owned: call them, never edit them.** Nothing else under `rust/`; no change to `sim_bridge.gdextension` in any committed tree.
- **Must not change:** everything else under `rust/`, `docs/revitalize/**`, `docs/worldmodel/**`, `tools/revitalize/**`, `tools/worldmodel/**`, `tools/harness/**`, `.claude/**`, `.harness/**`, evidence of earlier passes.
## 2.2 Part 0 merge and the chronicle contract
`git fetch origin`; record `git log -1 origin/codex/revitalize-2026-10` and `git merge-tree` of the two heads; `git merge --no-ff origin/codex/revitalize-2026-10 -m "art-p11(p0): bring in simulation passes 18 and 19"`; record the merged SHA. Conflicts: simulation-owned paths take the simulation's side; keep the art getters; record every resolved file.
**Contract:** read Pass 19's delivered getters (`docs/revitalize/CHRONICLE.md`, `rust/crates/sim-bridge/src/ffi/chronicle.rs`). Planned: `get_agent_id(entity_bits)`; `get_person(agent_id)` → `{found, alive, female (−1/0/1), born_tick, died_tick, death_reason, mother, father, ns, ha, rd, p}`; `get_person_chronicle(agent_id)` and `get_settlement_chronicle(settlement_id)` → packed arrays `{tick, kind, subject, other, third, settlement, settlement2, cause, value}` (−1 = none); `get_chronicle_since(start, max)` (+ `next`, `total`; first in Pass 19's drop order). Entries are indexed under **every** person they name (subject, other, third). Kinds 0 Born (subject child, other mother, third father, cause 1 = fallback parents), 1 FoundingMember, 2 Paired (subject woman, other man, value familiarity ‰), 3 CameOfAge, 4 BecameElder, 5 Departed (settlement parent, other leader, cause `DepartCause` 0 Ranked / 1 Seed / 2 Household / 3 ElderPartner / 4 TopUp), 6 DepartureCancelled, 7 Settled (settlement daughter, settlement2 parent), 8 Joined, 9 LeftSettlement (cause 0 remnant broke up), 10 Widowed (subject survivor, other deceased), 11 Died (cause `DeathReason::as_u8` 0 starvation / 1 dehydration / 2 combat / 3 old age / 4 exposure; value age in ticks, −1 without `Life`; other partner), 12 SettlementFormed (value members), 13 FissionStarted (other leader), 14 DaughterFounded, 15 FissionCancelled, 16 SettlementDissolved. Write `docs/art/production11/CHRONICLE_CONTRACT.md` (each getter present or not, keys, types, numbering); sections 5 and 10 are written for the planned contract — **follow the delivered one** and record every difference. If the merged head has no chronicle (Pass 19 not fast-forwarded), do 5.1–5.2 only, H8 = NOT_RUN, and record.

---

# 3. WHY THIS PASS (planner's review with a code survey at `cdf69c0e`; verify each fact)
1. **Art Pass 10's verdicts are open.** H1–H4 and H6 were never measured (0 sessions: the Mac stayed at load 34–125 from Pass 19's identity probes until the 12-hour cutoff). The last accepted alternating pair put W500 frozen Z4 at 29.04 ms (from 51.39; seat-input cache 1.436×, spatial hash 1.163×, allocation reductions 1.066×); the section timers exist only from before (`docs/art/production10/perf/agent_sections.md`: total 47.21 ms — seats 16.03, declump 12.15, frame choice 3.54, pose 2.87, post-loop 2.15, children 1.35, variant 1.23, walker offsets 1.07).
2. **The measurements used a debug bridge.** `perf_frames_p10.gd` L69 hard-codes `build_profile: "debug"`; `sim_bridge.gdextension` L6–13 maps Godot's debug feature (editor and `--path` runs) to `rust/target/debug/libsim_bridge.dylib`. Frozen frames are mostly GDScript, but running frames carry debug-built simulation ticks (W500 running 2× p90 162.7 ms).
3. **The running W500 world is not stationary:** the unfed `game_v19` stress world (500 founders on one camp); Art Pass 10's blocks fell 500 → 376 within one 1,495-tick block, and to 109 by the last. `perf_frames_p10.gd` never resets between blocks.
4. **Activities do not read.** Blind identification 6/14 (TendFire 0/2 — taken for Play twice; Play 0/2 — taken for Repair twice; SitByFire 0/2; Craft 1/2; HearthCircle 1/2; Repair 2/2; Explore 2/2; `docs/art/production10/activity/verified-captures/blind-score.json`). Ten of the fourteen crops came from Z1, where icons are hidden (zoom < 2, `agent_activity.gd` L182–183); only Craft (`U1_04`, 18 × 18) and Explore (a procedural 9 × 9 arrow, L261–269) have icons, shown while the activity is active (walking or arrived; `agent_renderer.gd` L224–225). The visual review rated Z1 FAIL, Z4 LIMITED, the crowded day-60 centre FAIL.
5. **People have no names:** the inspector heading is `INSPECTOR_AGENT_FMT % agent_id` (`agent_inspector_panel.gd` L181); family links show ids.
6. **The inspector calls `get_agent_temperament` and `get_agent_family` every frame** while open (L391–401), and `_resolve_entity_bits` scans the snapshot for every family link and child (L228–243, L412, L425).
7. **Where a chronicle fits:** `Main/UI` (`scenes/main.tscn` L91–105) holds `CausalPanel` (12,124, 620 × 480, toggled with Q, L217–220), `HudTopbar`, `AgentInspectorPanel` (1548,124, 360 × 620, one ScrollContainer, no tabs), `HudStatusPanel` (the news feed, 12,836, 620 × 232) and `GodHandPanel`. **C is unbound**; `UI_CHRONICLE_TITLE` exists (en `messages.ftl` L4170, ko L4165), as do `UI_DECEASED` (en L4215 "Deceased", ko L4210 "사망자"), `UI_CAUSE_OF_DEATH` (en L4159) and `UI_LIFE_EVENTS` ("Life Events" / "생애 사건").
8. **The tick cache's first P5 failure was attributed to water animation frame 2 vs 0;** the other two (816 and 1,680 px) were not analysed (`docs/art/production10/perf/tick-rejected/README.md`); the pair measured 1.05×.
9. **Zoom:** steps `[1, 2, 3, 4, 6, 8]`, default 4 (`camera_controller.gd` L4–7); the FAR LOD tier is unreachable (`zoom_lod_controller.gd` L34, L61–63). At Z4 a 1920 × 1080 view shows ≈ 480 × 270 world pixels — part of a 500-person camp.
10. **Village names:** the labels draw `Selection.name_for(id, locale)` (`settlement_labels.gd` L99) from a display number (`settlement_selection.gd` L120–132; an id never observed gives 0); the daughter form appends "· from %d".

---

# 4. PART 0 — Merge, gate, release baseline
4.1 Backup; merge and contract (2.2); full gate (`GATE_OUTPUT_DIR=docs/art/production11/gates/p0 bash tools/art_pipeline/gate.sh p0`; reuse Art Pass 10's `CARGO_PROFILE_TEST_OPT_LEVEL=2` for the 1,800 s step limit and record); fix fallout in art-owned code and tests; three strikes. Record that the live game builds `game_v24`. Commit the `_p11` tool copies (the **Part 0 tooling commit**: merge + tools, no UI change).
4.2 The `_p11` collector records `build_profile` from `P11_BUILD_PROFILE` and refuses to run unless that value matches the dylib it loads (4.4's sha check); fresh-reset rule (4.5).
4.3 **Identity reference:** Parts 1–2 change pixels on purpose and need no pixel identity. At the start of Part 3, point `art-p11-base` at Part 2's final commit, build and import (1.1), re-derive the framings for `game_v24` as Art Pass 10 did for `game_v22` (capture scripts as untracked copies), and capture A and A′ there; P5 (`reset_world_stress`) must reproduce exactly.
4.4 **Release measurement worktree** `/Users/rexxa/work/art-p11-measure`: check out the commit under test (first the Part 0 tooling commit), `cargo build --release -p sim-bridge`, copy `rust/target/release/libsim_bridge.dylib` over `rust/target/debug/libsim_bridge.dylib` **in that worktree only**, import (1.1). The collector refuses to run unless the two files' sha256 are equal; record it per session. No debug cargo command and no gate ever run there (they would overwrite the copy). Godot checks only the extension API version, not a hash.
4.5 **Baseline** (release, measure worktree, Part 0 tooling commit): Art Pass 10's matrix for W0 and W1 (now `game_v24`), W200 and W500 (stress `game_v19`) — frozen Z4/Z1 three 300-frame blocks, ablation on W0 and W500 — and running 2× and 4×, **each running block after re-running the world's own setup** (`perf_frames_p10.gd` L99–101: reset — `reset_world` for W0/W1, `reset_world_stress` for W200/W500 — then `advance_ticks` to the start tick at speed 0), re-framing the camera and resetting `_last_tick` to the start tick (else the first sample has negative `ticks_run`, L176). Estimate W1's re-advance of 126,994 ticks first; if six re-runs do not fit a 60-minute hold, split W1's running blocks across holds. Record each block's population at start and end; a block whose end is below 0.9 × its start is **flagged**. Section timers at W500 frozen Z4. `docs/art/production11/perf/baseline.md`. Commit, push.
4.6 **Art Pass 10's after-verdicts** (droppable, 0.3.7; whenever a hold is free before Part 4): in the measure worktree with `cdf69c0e` checked out and **its debug bridge** (`cargo build -p sim-bridge` there, before any release copy; record the sha), the `_p11` runner and lock but **`cdf69c0e`'s own collector** (`res://scripts/test/art/perf_frames_p10.gd`, unchanged, `--phase after`): W0 and W500 frozen Z4/Z1 and the W500 ablation (its W0 ablation and running blocks run too: recorded and ignored), and the W500 section timers → Art Pass 10's H1, H3 and H6 before → after. `docs/art/production11/perf/art10_after.md`, reported as Art Pass 10's numbers.

---

# 5. PART 1 — Names, the life story and the "why?"
5.1 **Names** (`scripts/ui/person_names.gd`, `class_name PersonNames`): `display_name(agent_id: int, lang: String) -> String`, deterministic from the agent id alone (works for the dead), built on the existing overflow-safe hash `scripts/ui/art_hash.gd` (L5–9): h = `ArtHash.mix(ArtHash.mix(agent_id) ^ 0x2545F491)` (non-negative; the double mix decorrelates names from looks). Syllable count: 2 if h % 4 == 0, else 3. Syllable i = TABLE[(h >> (5 × i + 2)) % 46] (use integer ops only; no `/` between ints — the gate's strict check treats INTEGER_DIVISION as an error). TABLE (en / ko), 46 entries: a/아 i/이 o/오 e/에 ka/카 ki/키 ku/쿠 ko/코 ta/타 ti/티 tu/투 to/토 na/나 ni/니 nu/누 no/노 ma/마 mi/미 mu/무 mo/모 ra/라 ri/리 ru/루 ro/로 sa/사 si/시 su/수 so/소 ha/하 hi/히 hu/후 ho/호 ya/야 yu/유 yo/요 wa/와 ba/바 bi/비 bo/보 bu/부 da/다 do/도 ga/가 go/고 — 44 listed, plus de/데 and ge/게. **Blocklist** (fixed in the file before Part 1's first capture; the agent may extend it only before that capture, recorded): en contains `nig`, `kak`, `baka`; ko equals or contains `시바`, `바보`, `미치`, `시비`; on a hit, the last syllable index +1 mod 46 until clean. The language is `Locale`'s `_current_lang` (read as `hud_topbar.gd` L95 does; `scripts/core/locale.gd` L15 has no public getter); first letter capitalized in English. Names are proper nouns, not localization keys (record). The inspector heading becomes `INSPECTOR_PERSON_FMT` (`%s · #%d`). Unit tests: determinism, both scripts, the blocklist, and the share of colliding names among 500 and 10,000 consecutive ids (reported).
5.2 **Read discipline:** temperament, family and every chronicle getter are called directly on the bridge (they take arguments; `SimReads` routes only no-argument getters, `sim_reads.gd` L4–10), **only when the selection changes or the simulation tick changes**; cache getter results by (agent id, tick), never formatted text — re-format when `_current_lang` changes. Map agent id → entity bits with one dictionary built once per tick from the agent snapshot's ids (replaces the per-link scans, L228–243). A test over 120 frames at a fixed tick: ≤ 1 call per getter, excluding `get_agent_detail` (called every frame by design, L249).
5.3 **The Life section** in the inspector, after Family (heading `UI_LIFE_EVENTS`): the person's entries (`get_person_chronicle`) in time order, one line each: `CHRONICLE_DAY_FMT` + " · " + the sentence. **Day:** one static helper, day = `floori(float(tick + clock_offset_ticks) / ticks_per_day) + 1` with both values from `get_world_info` (matches `hud_topbar.gd` L161). **Sentence by the viewed person's role:**
   - as **subject**: the section 10 sentence of the kind and cause (`CHRON_BORN_FALLBACK_NOTE` appended when Born.cause = 1; Paired's `%s` = the other party; Departed's last `%s` = the `CHRON_CAUSE_*` of its cause);
   - as **other or third**: Born → `CHRON_CHILD_BORN_FMT` (the child); Paired → `CHRON_PAIRED_FMT` with the subject; Died → omitted (their own `Widowed` line covers it); Departed / DepartureCancelled / Settled with other = the leader → omitted (FissionStarted covers it in the village's record).
   Every person named in a line is a link: alive → select them (as the family links do); dead or no live entity → **record mode**. Villages in sentences: `Selection.name_for(id, locale, false)` (the plain form); an id whose `number_for` is 0 is registered in Selection's numbering on first sight; abandoned villages keep their plain name. More than 12 lines: show the last 12 and a `UI_CHRONICLE_MORE_FMT` line that expands the rest.
5.4 **Record mode** (a dead person — reached from a family link or a Life line): the name with `UI_DECEASED`; `INSPECTOR_RECORD_LIFE_FMT` (born day, died day), or `INSPECTOR_RECORD_FOUNDER_LIFE_FMT` when `born_tick < 0` (founders); `UI_CAUSE_OF_DEATH` with the `CHRON_DEATH_*` noun; parents and partner (from `get_person` and the last `Paired` / `Widowed` entries) as links; temperament bars from the person table if present; the Life section. No live bars, portrait or position. A living person's family rows show a dead mother, father or partner by name with `UI_DECEASED`, linking to record mode (instead of Art Pass 10's disabled `PANEL_FAMILY_NONE`). Esc or selecting a living agent leaves record mode.
5.5 **The chronicle panel** (C toggles; title `UI_CHRONICLE_TITLE`; the `CausalPanel` slot 12,124, 620 × 480 — opening one closes the other): the world's notable entries, newest first, up to 60 — Born, Paired, Died, FissionStarted, DaughterFounded, FissionCancelled, SettlementDissolved — from `get_chronicle_since` (polled once per tick from the last `next`) or, if it was not delivered, the merged `get_settlement_chronicle` of every settlement id seen. A line with a subject reads `CHRONICLE_DAY_FMT · <subject name link> — <sentence as subject>`; village lines use the village sentences. Names → select or record mode; a village → center the camera on its hearth if it exists. The news feed stays as it is.
5.6 **Verification:** GDScript tests with a fake bridge for every kind, role and cause (en and ko), record mode (incl. a founder), the links and the read discipline; real captures on the live seed (`game_v24`, `DEFAULT_GAME_SEED`) at day 60 — or the first day in 60–90 where each exists — of (a) a living person with a dead parent, (b) that parent in record mode, (c) people who left in a split, one per `DepartCause` present, (d) the chronicle panel; EN and KO each. A fresh reviewer reads each capture and writes what they learn about the person (recorded before any key, marking guesses as "inference"); the non-inference statements are compared with the chronicle data: every one correct; no raw key or `%` visible.

---

# 6. PART 2 — Activities you can read
6.1 **Rule:** each of the seven kinds gets an icon, shown on today's gate (the activity is active — walking or arrived — and zoom ≥ 2; `agent_renderer.gd` L224–225, `agent_activity.gd` L182–183); zoomed out (< 2) a **village summary** shows what each village is doing. Poses, seats, declump and overrides stay as Art Pass 10 drew them; its tests stay green.
6.2 **Seven icons**, each a distinct silhouette and hue, 12–18 source pixels at scale 1 (as `U1_04`'s 18 × 18), procedural (GDScript-drawn or a small generated texture) or existing sprites; generations only per 0.3.8. Suggestion — keep or replace with a reason: TendFire a stick over a flame; Craft keep `U1_04`; Repair a hammer; HearthCircle two heads facing a small fire; Explore keep the arrow; Play a ball; SitByFire a cup or blanket by a flame (TendFire and SitByFire must not share a silhouette). On the existing `activity_icon` path (`agent_frame_choices_p10.gd` L93–97, `agent_activity.gd` L253–259). `docs/art/production11/activity/icons.md`: an enlarged sheet of the seven with their meaning.
6.3 **Village summary at zoom < 2** (`settlement_labels.gd`, which knows the zoom, L78): under each village label up to three icons of its most common current activities among **arrived** holders, each with a count (`%d`, no new key), ordered by count then kind number. Data, recomputed once per tick (`_update_labels` runs every frame, L73): `get_activity_snapshot` rows with `until_tick ≥ 0` → entity bits → agent ids via the agent snapshot's ids (`world_node.rs` L1762–1787) → village via `SettlementSelection.home_for_agent` (`settlement_selection.gd` L101–104).
6.4 Unit tests: every kind has an icon; the summary's data path, ordering and hiding.
6.5 **Verification with bars** (fixed here; results recorded before any change in response):
   - (a) 6.4's tests pass; (e) Art Pass 10's sidecar check (every holder's pose) still 0 mismatches.
   - (b) **Captures** (`game_v24`, release-selected as Art Pass 10 did with `worldsim_probe --scene game_v24`): the live seed at day 30 10:00 and 15:00 and a populous world at day 60 10:00, at Z4 and Z1, with legend sidecars.
   - (c) **Blind identification at Z4.** Crops: for each kind, the first two **arrived** holders (`until_tick ≥ 0`) in AgentId order across the Z4 captures in the fixed order d30 10:00, d30 15:00, d60 10:00; a holder is excluded if more than 50 % of its sprite rect is covered by sprites or buildings drawn above it (computed from the sidecar), the exclusions logged before any crop is made; if a kind has fewer than two eligible holders, add Z4 captures of the same seed and day at 11:00 and 14:00 centred on the lowest-AgentId holder of that kind, recorded before any crop. Crops with Art Pass 10's script and size. A fresh reviewer (not shown this section, the icon sheet or the key) gets the 14 crops and the seven names; answers frozen. **Bar: ≥ 10 / 14 and no kind at 0 / 2.**
   - (d) **Village summary at Z1**, answered after (c)'s answers are frozen and before any key: six (village, capture) pairs from the Z1 captures, at most two per village, excluding summaries whose top two counts tie; the reviewer names each village's most common activity. **Bar: ≥ 5 / 6.** The keys of (c) and (d) are revealed only after (d).

---

# 7. PART 3 — The frame budget at 500 (release measurement; reference = Part 2's final build, 4.3)
7.1 Targets in the order of 4.5's section timers, largest first. Each change: three alternating 150-frame pairs (`run_alternating_p11.py`), kept only if faster **and** identical (7.3).
7.2 **Candidates** (check each against the timers first):
   a. **Appearance-only work for off-screen agents.** For non-selected agents whose previous-frame drawn rect lies entirely outside the viewport grown by 64 screen px, skip **only**: `SpriteLibrary.apply_variant`, `_update_activity_icon`, `Cold.apply`, `_morning_chill.apply`, `_apply_settlement_outline`, shadow position and visibility, `_update_cue`, and `activity_oracle_inputs`; do them in full on the first frame the rect re-enters the margin. Everything else runs for every agent — including `sprite_id` / `SpriteLibrary.apply`, `flip_h`, `facing`, `visible`, `CarryLoad.apply` / `apply_founder`, the pose (incl. `Cold.pose`) and the `pose` / `activity` / `sheltered_sleep` / `tile_x` / `tile_y` / `sprite_id` metadata — because footprint rects (`agent_renderer.gd` L656–670), seats and declump (`camp_seats.gd` L48–51, L119–123, L165, L241), facing hysteresis (L201–203), `world_effects.gd` L86–103, `shelter_cold.gd` L71, `god_hand_effects.gd` L176, `morning_chill.gd` L46 and the inspector portrait (L258–269) read them. Skipping more requires first listing, by grep, every reader of what is skipped.
   b. **The tick cache, diagnosed** before any new attempt: find why it changed the water animation frame in P5 (the water animation's time source and what the cache changes about it) and analyse the two other failures. If the cause is in the identity harness (e.g. a wall-clock or accumulated-delta read), fix the harness so A and A′ are reproducible and document it; never mask water. One attempt after the diagnosis.
   c. Anything else above 1 ms in the timers.
7.3 **Identity:** the six framings (B vs A/A′ by Art Pass 10's rule); the running dump (W0 and W200, 600 frames) restricted to sprites inside the viewport and the 64 px margin band; and a **pan test** on W500 at Z4: start at the W500 hearth − (1,200, 0) world px, move + 10 world px per frame (40 screen px, below the margin) for 240 frames, frozen and at 2× (`--fixed-fps 60`, camera process disabled as `perf_frames_p10.gd` L93–96), dumping Art Pass 10 5.2b's property list for sprites inside the viewport and margin band every frame — equal to the base build's.

---

# 8. PART 4 — After: measurements and verdicts (start by hour 9)
Rerun 4.5's release matrix on the frozen build (0.3.6), with the fresh-setup rule.
| # | question | rule |
|---|---|---|
| H1 | the live game, frozen Z4 (W0, `game_v24`) | median ≤ **6.0 ms** |
| H2 | 200 people, frozen Z4 (W200) | median ≤ **12.0 ms** and p99 ≤ 16.67 ms |
| H3 | 500 people, frozen Z4 (W500) | median ≤ **16.67 ms** |
| H4 | 500 people, running 2×, tick-carrying frames, unflagged blocks only | median ≤ 16.67 ms, with ticks per frame and populations; flagged blocks reported and excluded; fewer than 2 unflagged blocks → NOT_VERIFIED (non-stationary world), never PASS or FAIL |
| H5 | Part 3 identity (7.3) | all pass |
| H6 | AgentRenderer ablation at W500 | before → after (no bar) |
| H7 | activities | 6.5 (c) ≥ 10/14 with no kind at 0/2; (d) ≥ 5/6 (or NOT_RUN if 6.3 was dropped); (a), (e) pass |
| H8 | life story | 5.6 tests pass; every non-inference reviewer statement correct; no raw key (NOT_RUN without the chronicle, 2.2) |
Report every number before and after with speed-ups, section timers before and after, and Art Pass 10's after-verdicts (4.6) in their own table.

---

# 9. PART 5 — Review, gate, report
1. Independent review (two rounds max): the merge and the chronicle contract; the release worktree (sha per session, nothing committed to `sim_bridge.gdextension`); Art Pass 10's after-verdict procedure; names and the blocklist; the read discipline; sentences against the chronicle data, by role; the blind protocol (exclusions and added captures recorded before any crop; answers frozen before the key; (d) after (c)); Part 3 identity incl. the pan test; localization in all five files; the H verdicts; allowed paths; quiet-lock record.
2. Full gate (`GATE_OUTPUT_DIR=docs/art/production11/gates/final bash tools/art_pipeline/gate.sh final`); fast-forward `codex/art-production-2026-10`; PR #511 section; remove both worktrees.
3. `docs/art/production11/REPORT.md` in Korean (section 12 format) and the same text in the final message.

---

# 10. Localization keys
Add in the Part that first uses them, in all five files (`localization/fluent/{en,ko}/messages.ftl`, `localization/compiled/{en,ko}.json` via `python3 tools/localization_compile.py --project-root .`, `localization/key_registry.json` with the next ids). **Reuse** (same meaning): `UI_CHRONICLE_TITLE`, `UI_LIFE_EVENTS`, `UI_DECEASED`, `UI_CAUSE_OF_DEATH`, `UI_FAMILY`, `PANEL_FAMILY_SPOUSE`, `UI_MOTHER`, `UI_FATHER`. Grep each new key first; reuse where the meaning is the same and record. Korean sentences avoid particles that depend on the name (을/를, 이/가, 와/과), hence the dash forms. Every row has the same `%` specifiers in the same order in en and ko.

| key | en | ko |
|---|---|---|
| `INSPECTOR_PERSON_FMT` | %s · #%d | %s · #%d |
| `INSPECTOR_RECORD_LIFE_FMT` | Born day %d · died day %d | %d일째 태어남 · %d일째 세상을 떠남 |
| `INSPECTOR_RECORD_FOUNDER_LIFE_FMT` | A founder · died day %d | 첫 주민 · %d일째 세상을 떠남 |
| `UI_CHRONICLE_MORE_FMT` | %d earlier entries | 이전 기록 %d개 |
| `CHRONICLE_DAY_FMT` | Day %d | %d일째 |
| `CHRON_BORN_FMT` | Born in %s. Parents: %s and %s. | %s에서 태어남. 부모: %s, %s |
| `CHRON_BORN_FALLBACK_NOTE` | (parents by the village's reckoning) | (마을이 정한 부모) |
| `CHRON_CHILD_BORN_FMT` | A child, %s, was born. | 아이가 태어남 — %s |
| `CHRON_FOUNDER_FMT` | One of the founders of %s. | %s의 첫 주민. |
| `CHRON_PAIRED_FMT` | Became partners with %s. | 짝이 됨 — %s |
| `CHRON_CAME_OF_AGE` | Came of age. | 어른이 됨. |
| `CHRON_BECAME_ELDER` | Became an elder. | 노인이 됨. |
| `CHRON_DEPARTED_FMT` | Left %s in %s's group: %s | 떠남 — %s, 이끈 이: %s. %s |
| `CHRON_CAUSE_RANKED` | chosen to go. | 떠날 사람으로 뽑힘. |
| `CHRON_CAUSE_SEED` | among the first to go. | 먼저 떠나기로 함. |
| `CHRON_CAUSE_HOUSEHOLD` | went with the household. | 가족을 따라감. |
| `CHRON_CAUSE_ELDER_PARTNER` | followed their partner. | 짝을 따라감. |
| `CHRON_CAUSE_TOP_UP` | joined to make the group large enough. | 무리를 채우려 합류함. |
| `CHRON_DEPARTURE_CANCELLED_FMT` | The move from %s was called off. | 이주 취소 — %s |
| `CHRON_SETTLED_FMT` | Settled the new village %s, from %s. | 새 마을에 정착함 — %s (%s에서 옴) |
| `CHRON_JOINED_FMT` | Joined %s. | 합류함 — %s |
| `CHRON_LEFT_FMT` | Left %s. | 떠남 — %s |
| `CHRON_LEFT_BROKE_UP_FMT` | Left %s when it broke up. | 흩어진 마을을 떠남 — %s |
| `CHRON_WIDOWED_FMT` | Lost their partner %s. | 짝을 잃음 — %s |
| `CHRON_DIED_FMT` | Died aged %d days, of %s. | %d일을 살고 세상을 떠남 — %s |
| `CHRON_DIED_NO_AGE_FMT` | Died of %s. | 세상을 떠남 — %s |
| `CHRON_DEATH_STARVATION` | hunger | 굶주림 |
| `CHRON_DEATH_DEHYDRATION` | thirst | 목마름 |
| `CHRON_DEATH_COMBAT` | a fight | 싸움 |
| `CHRON_DEATH_OLD_AGE` | old age | 노환 |
| `CHRON_DEATH_EXPOSURE` | the cold | 추위 |
| `CHRON_SETTLEMENT_FORMED_FMT` | %s was founded by %d people. | %s 세워짐 — %d명 |
| `CHRON_FISSION_STARTED_FMT` | %s began to split; %s leads a group away. | %s 갈라지기 시작 — 이끈 이: %s |
| `CHRON_DAUGHTER_FOUNDED_FMT` | %s was founded by people from %s. | %s 세워짐 — %s에서 갈라져 나옴 |
| `CHRON_FISSION_CANCELLED_FMT` | The split of %s was called off. | %s 분열 취소 |
| `CHRON_SETTLEMENT_DISSOLVED_FMT` | %s was abandoned. | %s 사라짐 |

`CHRON_DIED_FMT`'s `%d` = floori(value / ticks_per_day) (integer division without `/` between ints — use `floori(float(value) / ticks_per_day)`); value < 0 → `CHRON_DIED_NO_AGE_FMT`. The news feed's `HUD_FEED_DEATH_*` are full sentences and `DEATH_OLD_AGE` / `DEATH_STARVATION` cover two of five causes, hence the new nouns.

---

# 11. 인게임 확인사항 (오너용)
- 게임이 `game_v24`로 시작한다. 저녁 17:00~19:00에 사람들이 흩어져 일을 마치는지.
- 주민을 클릭하면 이름(예: 카루미 · #123)이 보이고, 아래 "생애 사건" 칸에 태어난 날·부모·짝·어른/노인이 된 날·떠난 마을과 이유(가족을 따라감, 짝을 따라감 등)가 문장으로 나오는지. 부모의 기록에는 "아이가 태어남 — (아이 이름)"으로 나오는지.
- 죽은 부모나 배우자 이름을 누르면 그 사람의 기록(태어난 날 또는 "첫 주민", 죽은 날, 사인, 생애 사건)이 열리는지.
- C 키로 연대기 창이 열리고, 출생·짝짓기·죽음·마을 분열이 최근 순으로 나오는지. 이름과 마을을 누르면 그 사람/마을로 가는지.
- 가까이서 낮에 일하는 사람 머리 위에 일곱 가지 서로 다른 아이콘이 보이는지. 멀리 줌아웃하면 마을 이름 아래 "지금 많이 하는 일" 아이콘과 숫자가 보이는지.
- 영어·한국어 모두 번역 키나 %가 그대로 보이는 곳이 없는지, 콘솔 에러가 없는지.

---

# 12. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
(병합과 연대기 계약, release 기준선, 이름, 생애 사건·고인 기록·연대기 창, 일곱 아이콘과 마을 요약, 프레임 예산 수정, Art Pass 10 사후 판정)
### 구현 방법
(release 측정 작업트리, 블록마다 새로 만든 실행 측정, 읽기 규율, 역할별 문장 생성, 블라인드 절차와 막대, 화면 밖 생략과 팬 테스트, 틱 캐시 진단)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
H1~H8 표(전후·배수, release), Art Pass 10 사후 판정 표(H1·H3·H6, debug), 세계·화면·조건별 중앙값/p90/p99/최대와 블록별 인구(표시된 블록 포함), 구간 타이머 전후, 노드별 비용 전후, 블라인드 결과(활동별·제외 기록)와 마을 요약 결과, 생애 화면 리뷰어 기록 대조
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
사람의 일생이 화면에서 읽히는가? 낮의 활동이 이제 읽히는가? 화면은 몇 명까지 버티는가(release 기준)? 다음 아트 과제는 무엇인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #511 update, and the time per Part.
