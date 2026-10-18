# Pass 9 independent review, round 1

Scope: `git diff 4c35e295..HEAD -- rust/ tools/revitalize/ tools/worldmodel/` (HEAD `05bb3ecd`), PASS9_GOAL.md sections 3.2, 4, 6, 7, 8 and 11, RNG_INVENTORY.md, and DECISIONS.md `## Pass 9`. Everything below was checked read-only. Beyond reading the code I ran two read-only Python checks against the recorded fork data in `/Users/rexxa/work/worldsim-wm-data/p9/forks`; they are described in the "Checked and clean" section.

**Verdict: no High findings.** The code, the fork semantics and the VRF pairing all match the specification. One Medium finding: a test does not check what its name claims. The rest are Low.

## Findings

### M1 (Medium): `harness_v15_newborns_carry_their_first_parents_key` does not test what its name claims

- **Where:** `rust/crates/sim-test/tests/harness_revitalize_pass9_rng.rs`, the test after `harness_lineage_keys_do_not_collide`. Its comment says "the founders' birth counts add up to the births".
- **What it asserts:** three things only:
  - the keys are distinct;
  - every living agent has a `Lineage`;
  - `births > 0`.
- **What it never checks:**
  - that a newborn's key equals `first_parent.child(seed)`;
  - that any `births` counter adds up.
- **Failure scenario:** suppose the parent lookup in `settlement_system.rs:784-789` silently fails, for example because the query changes or `parent_ids` picks a dead id. Every newborn would then get the fallback key `keyed_u64(seed, STREAM_NEWBORN, sid, tick, u32::MAX, 0)` (`settlement_system.rs:790-795`). That key contains the settlement id and the birth tick, both of which can differ between the arms of a fork. A birth shifted by one tick would then change the newborn's key, its lifespan and its movement, which is exactly what lineage keys are meant to prevent. The test would still pass, because fallback keys are also unique and also attached.
- **Production is fine today.** My data check confirms the parent path works in practice (0 of 1,845 newborns use the fallback). The defect is the missing regression guard.
- **Suggested fix:** assert one of these:
  - every non-founder key lies in the child closure of the founder keys (`Lineage::founder(seed, i)`, then `child`), which is what my check does;
  - or a counter of fallback births stays at 0.

### L1 (Low): divergence "moved share after 1 h / 6 h / 24 h / 72 h" is off by one row

- **Where:** `tools/revitalize/fork_metrics.py:275-277`. It uses `row = hours - 1`.
- **Why it is wrong:**
  - Row 0 is the intervention tick itself (`divergence` keeps `t >= start_tick`, line 152).
  - Row 0 is taken before that tick's interventions, so it is pre-intervention.
  - The column labelled "1 h" is therefore the 0 h row, and the others are 5 h, 23 h and 71 h.
- **Evidence in the committed output:** `fork_metrics.json` gives windfall `median_moved_share_by_row` = [0.0, 0.405, 0.446, 0.429, …]. `fork_metrics.md` reports "after 1 h" = 0 for windfall, while the real 1 h value is 0.405. The DECISIONS line saying "differs 60 ticks after (windfall)" is correct, because it comes from `first_moved`.
- **Second issue:** the function assumes 60-tick rows without reading `every`.
- **Fix:** look up the row whose tick is `start + 60·hours`, and relabel or regenerate the table.

### L2 (Low): AgentId order is still a residual cross-arm dependence (by design, not documented as one)

- **Where AgentId still decides things in v15:**
  - decision order (`agent_decision.rs:919-922`, `:2073-2076`);
  - death order (`survival/mod.rs:198-200`);
  - the first parent (`settlement_system.rs:768-773`);
  - the "lowest-AgentId peer" picks (`agent_decision.rs:1008-1015`, `:1186-1190`, `:1228-1232`).
- **Why it matters:** AgentIds come from the global counter. After the fork, an extra birth in one arm renumbers every later newborn in that arm.
- **Why it is Low:** relative order between agents matched across arms changes only if their birth order differs. No key contains an AgentId.
- **Suggested fix:** state this in DECISIONS as a known residual, not just "AgentId order".

### L3 (Low): `--fork-tick 0` is accepted but does not resample founders' lifespans

- **Where:** `export.rs:624-626` only rejects `tick >= days*1440`, so tick 0 is accepted.
- **The mismatch:**
  - Founders' age and lifespan are drawn at worldgen with `noise_replicate_for(0)` (`camp.rs` Pass 9 block), before the recorder sets the fork.
  - The documented rule says `r_b = noise_replicate_for(0)`, which would be `k` for a fork at tick 0.
- **Impact:** none of the committed experiments forks at tick 0.
- **Fix:** reject tick 0, or document the exception.

### L4 (Low): the exact-zero variance test is only safe for integer targets and power-of-two R

- **Where:** `fork_metrics.py:53` (`if v_p == 0.0`).
- **Why it works here:** the targets are integer-valued and R = 8 is a power of two, so numpy's pairwise mean of a constant column is exact and the variance is exactly 0.
- **Where it breaks:** with fractional targets or non-dyadic R, a constant column gives a tiny non-zero variance instead of 0. In my synthetic check:
  - `np.var(np.full(6, 0.1), ddof=1)` = 2.3e-34;
  - `vrf_of(y_f + 2.0, y_f)` with fractional `y_f` returns 6.8e31 instead of `inf`;
  - two near-constant arms would produce a ratio of rounding errors instead of "no agent noise".
- **Fix:** compare against a tolerance scaled to the data, e.g. `v <= 1e-12 * max(1, mean(y²))`.

### L5 (Low): an all-quiet bootstrap resample is counted as +∞ instead of being excluded

- **Where:** `fork_metrics.py:72-73`. A resample with V_p = V_u = 0 becomes +∞.
- **Effect:** it slightly inflates both bounds. On this data the effect is negligible: the probability of an all-quiet resample is ≤ 1.3e-3 (windfall dark nights, 26 of 32 seeds quiet) and 2.6e-5 for rot_wood dark nights.
- **Fix:** drop such resamples, or document the choice.

### L6 (Low): the replicate-0 identity skips missing unforked runs silently

- **Where:** `fork_metrics.py:231` (`if unforked.exists()`).
- **Effect:** a partial `--unforked` directory would shrink the denominator, and G0a would still pass.
- **Mitigation:** the committed counts (224 = 7 schedules × 32 seeds) show nothing was missing.
- **Fix:** fail when a run is missing, or report the expected count.

### L7 (Low): two latent position ties in the construction completion sort

- **Where:** `construction_system.rs:201`, `sort_by_key(position)`.
- **The issue:** if two completed sites shared a position, the stable sort would keep HashMap order. This is the inventory's O8 condition, which shelter planning never produces. Recorded for completeness.

## Checked and clean

1. **Sequential draws in v15.**
   - The only sequential draws left after tick 0 are the movement fallback for a mover without a `Lineage` (`movement.rs:316-326`) and the AgentId-keyed `newborn_life`/`founder_life` (taken only when `lineage` is `None`).
   - Neither is reachable in a v15 game world:
     - founders get a `Lineage` at worldgen (`camp.rs`);
     - newborns always get one (`settlement_system.rs:836-838`);
     - no other in-game spawn path exists (`SimEngine::spawn_agent` is called only by worldgen, `bootstrap_spawn_agents` and tests; the bootstrap runs only in `init_production_engine`, i.e. legacy).
   - Newborn `MovementRng` seeds still use the AgentId, but the stream is never advanced in v15.
   - Worldgen draws (`camp.rs:122-126`, `:242-258`, noise, terrain) happen at tick 0. Weather hashes are keyed on (seed, year/day).
2. **Keys.**
   - Movement: (seed, stream, lineage key, tick, 0, r).
   - Lifespan: (seed, LIFESPAN, key, 0, slot, r_b), with no tick and `r_b` taken at the birth tick.
   - Founder: (seed, FOUNDER, 0, 0, i, 0).
   - Newborn: (seed, NEWBORN, parent key, 0, births, 0).
   - None of them contains an AgentId, an Entity, an event id or a counter. The fallback contains the settlement id and tick, but it is unreachable.
   - **Data check 1:** I reimplemented `keyed_u64` in Python and confirmed that every newborn key in 61 sampled fork episodes (1,845 newborns) lies in the founders' child closure. No fallback key and no zero key appeared.
3. **Draw counts.**
   - The pause draw depends only on a rule constant.
   - Skipped dx/dy draws shift nothing, because every draw has its own key.
   - The founder top-up (slot 4) is always computed.
4. **Order dependence.**
   - O1–O4, O6 and O7 are implemented, and every one is gated on `keyed_draws`.
   - O5 inherits its order from its emitters, as documented.
   - Rechecked independently:
     - the prelude peer maps are consumed via `min()`/`any()` on AgentId;
     - aging, fire, stockpiles, combat and social are already sorted;
     - resource regen works per tile;
     - StaleSeekTarget works per entity;
     - `haulers` is a BTreeSet;
     - the migration target uses a total tie-break (`agent_decision.rs:1533-1538`);
     - `sync_membership`/`despawn_agent` are commutative over the `settlements` HashMap;
     - there are no id- or Entity-based staggers.
   - No loop was missed.
5. **Fork semantics.**
   - `replicate_at` is correct.
   - `apply_noise_fork(0)` returns early.
   - Systems see `current_tick` = completed ticks (`lib.rs:630`).
   - The recorder takes row T, then forks, then applies interventions, then ticks (`export.rs:309-321`), so rows ≤ T are pre-fork.
   - The replicate-0 test sets the fork directly. The prefix test checks rows ≤ T are equal and that rows differ later. Both test what they claim.
6. **VRF.**
   - `np.roll(y_f, -1, axis=1)` equals f(s, (k+1) mod R); within-seed `ddof=1`; mean over seeds; ∞ and "no agent noise" handled.
   - The cluster bootstrap uses a fresh `default_rng(9)` per variant and target, with `inverted_cdf` quantiles.
   - An independent loop implementation matches `vrf_of` exactly on synthetic data (51.055; constant factual gives 1.0; all-zero gives None).
   - **Data check 2:** my recomputation on real data matches `fork_metrics.json` for rot_wood and windfall, dark nights and cold member-hours (e.g. rot_wood cold hours: V_p 677.10, V_u 1226.85, VRF 1.812).
   - The 8.1 identity compares every `.npy` sha256 and treats amount-0 records symmetrically.
7. **v8–v14 invariance.**
   - Every new behaviour is gated, and `keyed_draws` is false in v8–v14 and in the default (tested in `rules.rs`).
   - `Option<&Lineage>` in the movement query does not change archetype matching.
   - The `Vec` collects keep hecs order.
   - `fingerprint.rs` and `long_run.rs` are unchanged.
   - Legacy test edits only add `keyed_draws: false`, or are schema moves listed in TEST_LEDGER.
8. **Other.**
   - Lineage key collisions are negligible: `mix` is a bijection, different streams separate founder and newborn chains, and a 20,000-key test exists.
   - Resume compares `noise_fork`.
   - `load_family` and `families_manifest` validate replicate and fork tick.
   - `shift_pass9.py` pairs seeds, uses the same resample for both versions, and uses a fresh `rng(9)` per statistic.
