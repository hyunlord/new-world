## Static audit (7.4)

Systems audited: every system with ≥ 5 % of tick time at n = 500 (`AgentDecisionSystem` 60.0 %, `SettlementSystem` 21.9 %, `MemorySystem` 7.0 %) and every system classed superlinear or worse by any slope (`AgentMovementSystem`, stress slope 1.21 — superlinear by 0.01; natural slope 0.82). No system is near-quadratic by any slope. Read-only reading of the code at `8f052142` (a fresh `general-purpose` subagent read the three large systems; line references were re-checked by hand: `settlement_system.rs` L480–485, L341–350; `agent_decision.rs` L763–766, L1106–1114; `memory_system.rs` L88–92, L109–114). Nothing below was changed in this pass.

Symbols: A agents, M members of a settlement, S settlements, F food tiles, T tiles with a causal log (≤ W·H), k agents on the same tile, E memory entries per agent (≤ 32).

**Structural root cause shared by all three.** When the first settlement forms, everyone within `SETTLEMENT_PROXIMITY_RADIUS` is admitted (`settlement/settlement_system.rs` L480–485) with no `SETTLEMENT_MAX_POP` (50) cap — the cap only limits later joins (L298–299). In the stress world M = A = 500, so every per-settlement-members term below becomes O(A) or O(A log A). Three systems rebuild an `AgentId → position/entity` map of all agents every tick (`agent_decision.rs` L949, L2154; `settlement_system.rs` L233; `memory_system.rs` L100), and two scan every causal-log tile every tick for this tick's events (`settlement_system.rs` L588; `memory_system.rs` L88, L124; `CausalLogStorage::lookup` in `sim-core/src/causal/storage.rs` L99 is a linear scan).

### AgentDecisionSystem (`runtime/decision/agent_decision.rs`, tick L599) — 60 % at n = 500, natural slope 0.98, stress slope 1.13

About 17 full passes over all agents per tick, about 8 BTree collections of size A, about 4 per-tile peer maps and 2 sorts of all agents; then 10–15 BTree lookups per agent.

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L1020–1025, L1076, L1499–1500, L1521, L1561, L1567 (and the map builders L850, L907, L929) | 10–15 `BTreeMap`/`BTreeSet` lookups per agent keyed by `AgentId` (`local_food`, `children`, `elders`, `social_retry_after`, `founding`, `meal_camp`, `member_of`, `local_home`, `assigned_builders`) | O(A log A) | no | read `Option<&Child>`/`Option<&Elder>`/`Option<&Founding>` in the query; one per-agent context struct with a single lookup |
| 2 | L612–637, L641–652, L763–765 + L794–796 (`member_of` built twice), L817–829 | ~7 per-tick BTree maps/sets of every member/child/elder rebuilt from scratch (allocation per tick) | O(A log A) + A node allocations per map | no | build one member table per tick (or cache it behind a membership generation counter); reuse buffers |
| 3 | L766 `wood_in_hand`, L799–800 (`firewood.rs` L291, `food_economy.rs` L66, L178) | per settlement: query all agents, filter by membership, collect, sort, sum — 3 times | O(S·A log A) | no | one pass accumulating per-settlement sums |
| 4 | L838–870, L879–896, L903–918, L925–938 | 4 extra world queries, each a `HashMap<(x,y), Vec<…>>` with a heap `Vec` per occupied tile | O(A) + per-tile allocations | partly | one pass filling all four; a reused dense per-tile index |
| 5 | L1106–1114, L1135–1143, L1284–1288, L1326–1330, L1866–1872 | same-tile peer scans (`filter(id != self).min()`, `any`) | O(A·k) | **yes** — k is the crowd on the hearth/bed tiles | keep the two smallest ids per tile |
| 6 | L1507 `choose_bush` (L91), L1533 `choose_spot` (L139), L1567/L2210/L2281 `nearest_gather_tile` (L553), L2287, L2267/2272/2290 | per seeking agent: scan all food / wood / water / sleep tiles | O(A_seek·F) | no (transitions) | per-settlement candidate lists rebuilt on tile change; reuse the arm's tile in the post-pass |
| 7 | L1129, L1354–1366 (`memory_weight_delta` L331), L1303, L1321 | per idle agent: several passes over its memory entries | O(A·E) | no | one pass computing all arm deltas |
| 8 | L1179, L1416 | `causal_log.lookup` per combat / bias flip: linear scan of every tile log | O(flips·T·8) | indirect | `EventId → tile` index |
| 9 | L998–1001, L2179–2183 | collect + `sort_by_key(AgentId)` of every agent, twice per tick | O(A log A) ×2 | no | persistent id-ordered entity list |

The measured stress slope 1.13 [1.10, 1.16] and the rise of µs per agent-tick from 0.61 (n = 500) to 0.81 (n = 2,000) are consistent with rows 1–2 and 9 (log A factors) plus row 5 (crowding); the natural slope (0.98) shows no superlinear term at game densities.

### SettlementSystem (`runtime/settlement/settlement_system.rs`, tick L1023) — 22 % at n = 500, natural slope 1.08, stress slope 0.98

With S = 1 and M = A almost all of the cost is per-tick O(A)/O(A log A) bookkeeping; there is no A² term under the hearth anchor.

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L341–361 `resolve_exclusivity` | per settlement: collect + sort members; `HashMap<AgentId, Vec<SettlementId>>` with one heap `Vec` per member | O(Σ M log M) + A allocations | no | skip when S = 1 or no anchors overlap |
| 2 | L288–302 `sync_membership` | filter all agents in radius, collect, sort — then `break` at once because `len() ≥ cap` | O(S·A log A), wasted | yes | check the cap before building the list |
| 3 | L271–275, L233–239 | `settled` set and `snapshot_agents` map of all agents rebuilt each tick | O(A) + allocation | no | one shared per-tick agent index |
| 4 | L415–417 `run_formation_scan` | collect + sort + dedup all agent positions, unused under the hearth anchor | O(A log A), wasted | no | build only in the legacy branch |
| 5 | L436–458 (legacy anchor only, not `game_v19`) | per candidate tile: count all agents and buildings in radius | ≈ O(A²) | yes | prefix sums — not on the `game_v19` path |
| 6 | L588–617 `ingest_community_history` | scan every causal-log event for `tick == now` | O(T·8) | no (area) | per-tick event list |

### MemorySystem (`runtime/memory/memory_system.rs`, tick L80) — 7 % at n = 500, natural slope 0.94, stress slope 1.03

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L109–120 | every agent × every memory entry decayed each tick, each with `recalled.iter().any(..)` | O(A·E·(1 + recalls)) | recalls grow with crowd flips | sort `recalled` and binary-search; skip empty memories |
| 2 | L88–98, L124–140 | two full scans of every tile's causal log to find this tick's events | O(T·8) ×2 | no | per-tick event list in `CausalLogStorage` |
| 3 | L203, L215, L219 | `causal_log.lookup` per construction event | O(events·T·8) | no | `EventId → tile` index |
| 4 | L100–103 | `HashMap<AgentId, Entity>` of all agents per tick | O(A) + allocation | no | shared per-tick index |

### AgentMovementSystem (`runtime/agent/movement.rs`, tick L171) — 1.4 % at n = 500, natural slope 0.82, stress slope 1.21

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L189, L196–205 | `BTreeMap<AgentId, home>` of every member rebuilt each tick, then one `home.get` per moving agent | O(A log A) | no | the shared per-tick member table |

The stress slope sits 0.01 above the superlinear threshold and 0.39 above the natural slope (not crowding-driven by the 0.5 rule); its share stays ≤ 1.5 % at every size.

### What this means

No loop on the `game_v19` path is quadratic in the population. The superlinear part that exists is log-factor bookkeeping (BTree maps and sorts of every agent rebuilt each tick, in four systems) plus same-tile peer scans that grow with crowding. The largest single lever is a shared per-tick agent/member index (removes ~10 rebuilt maps across `AgentDecisionSystem`, `SettlementSystem`, `MemorySystem`, `AgentMovementSystem`); the second is a per-tick event list in `CausalLogStorage` (removes the O(T) log scans, which grow with explored area rather than people). Neither is needed for the 500-person target on this machine.
