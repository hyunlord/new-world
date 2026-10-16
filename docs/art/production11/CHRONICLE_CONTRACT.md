# Art Pass 11 delivered chronicle contract

Inspected after merge `ecf512d886d87ba7f9ca5fa7db92a48300dc1c68`: `docs/revitalize/CHRONICLE.md`, `rust/crates/sim-bridge/src/ffi/chronicle.rs`, and registered methods in `world_node.rs:552–588`. This is source verification; runtime evidence is recorded separately. All five planned getters are delivered. Simulation-owned code is unchanged.

| Getter | Present | Arguments | Return |
|---|---|---|---|
| `get_agent_id` | yes | entity bits: int/i64 | AgentId bits: int/i64; −1 absent |
| `get_person` | yes | agent_id: int/i64 | person dictionary below |
| `get_person_chronicle` | yes | agent_id: int/i64 | packed chronicle dictionary |
| `get_settlement_chronicle` | yes | settlement_id: int/i64 | packed chronicle dictionary |
| `get_chronicle_since` | yes | start, max: int/i64 | packed dictionary plus next/total |

`get_person`: `found`, `alive` are bool; `female`, `born_tick`, `died_tick`, `death_reason`, `mother`, `father`, `ns`, `ha`, `rd`, `p` are int/i64. Female is −1 unknown / 0 male / 1 female. Missing fields are −1; missing person has found=false and alive=false. Founders may have negative actual birth ticks; a negative birth tick is therefore not a death or an invalid record. Temperament axes preserve the integer values from PersonRecord (not normalized floats). Alive additionally checks a living Agent entity and non-dead BodyHealth. Records survive despawn.

Chronicle dictionary: `found: bool`; `kind` and `cause`: PackedInt32Array; `tick`, `subject`, `other`, `third`, `settlement`, `settlement2`, `value`: PackedInt64Array. Columns have equal lengths. Absent IDs are −1. Rows are append order, indexed under every distinct person and village named. Agent IDs use the signed i64 bit representation, not entity bits; use snapshot parallel IDs or get_agent_id to translate.

`get_chronicle_since` adds `next`, `total` (int/i64). Start clamps to [0,total], max clamps to [1,512], found=true even for an empty valid page. Continue at next; do not assume one page contains all new rows. Unknown person/village gives found=false and empty arrays.

| Kind | Number | Person roles | Village roles | Cause / value |
|---|---:|---|---|---|
| Born | 0 | subject child; other mother; third father | birth village | cause 0 couple / 1 fallback parents |
| FoundingMember | 1 | subject founder | original camp | no additional semantics |
| Paired | 2 | subject woman; other man | village | value familiarity per mille |
| CameOfAge | 3 | subject person | absent | no additional semantics |
| BecameElder | 4 | subject person | absent | no additional semantics |
| Departed | 5 | subject member; other leader | parent | cause Ranked=0, Seed=1, Household=2, ElderPartner=3, TopUp=4; value fission id |
| DepartureCancelled | 6 | subject living member | parent | value fission id |
| Settled | 7 | subject living member | settlement daughter; settlement2 parent | value fission id |
| Joined | 8 | subject person | village | cause 0 no prior roster, 1 another roster, 3 present at daughter formation |
| LeftSettlement | 9 | subject person | old village | cause 0 remnant dissolved, 2 roster change |
| Widowed | 10 | subject survivor; other deceased | absent | no additional semantics |
| Died | 11 | subject deceased; other recorded partner | last roster | cause starvation=0, dehydration=1, combat=2, old age=3, exposure=4; value age ticks or −1 without Life |
| SettlementFormed | 12 | absent | village | value member count after exclusivity |
| FissionStarted | 13 | subject absent; other leader | parent | value fission id |
| DaughterFounded | 14 | absent | daughter / parent | value fission id |
| FissionCancelled | 15 | absent | parent | value fission id |
| SettlementDissolved | 16 | absent | village | cause 0 empty, 1 remnant merge |

## Differences / refinements versus GOAL planned shape

- The packed getters also return `found`, omitted from GOAL's shorthand but present in the delivered contract.
- Joined causes 0/1/3, LeftSettlement cause 2, and SettlementDissolved causes 0/1 are additional delivered detail. Keep generic joined/left/dissolved sentences where the approved key set does not differentiate them; cause 0 LeftSettlement uses the breakup sentence.
- `DepartureCancelled` and `Settled` do not name a leader in other; GOAL's leader-role omission rule remains harmless for absent other IDs.
- `get_person` has no partner field. Reconstruct partner from Paired/Widowed history as required, never invent a bridge field.
- Feed pagination is delivered, with an explicit 512-row cap and clamping; no settlement-feed fallback is required.
- No missing getter, kind-number mismatch, or death/departure cause-number mismatch was found.

## P0 measurement tooling contract

All section 1.1 p10 tools were copied to p11 paths; original pass evidence remains unchanged. `P11_BUILD_PROFILE=release` is mandatory for the p11 matrix, alternating and section collectors; the loaded debug path must hash identically to the release dylib. Runner provenance includes both hashes. `P11_BASELINE_SHA` is the committed Part 0 tooling SHA, explicitly passed to avoid an impossible self-referential source SHA. Baseline collector records it and the independent validator requires its equality to build_commit.

Running blocks reset their own world, freeze speed, advance to the specified start tick, frame the minimum-ID settlement, reset _last_tick, and then begin speed 2 or 4. Each block records start/end state and populations; end <90% start is flagged. H4 excludes flagged blocks and needs at least two unflagged after blocks. Raw flagged samples remain in tables.

`P11_WORLDS` can select world sessions. For W1, after estimating re-advance cost, `P11_SPLIT_W1=1` executes frozen plus six individual running holds with the normal cooldown. Each fragment retains its JSON/log/exit and sha-bound provenance; combined validation resets process-frame monotonicity only at documented split-process running boundaries. `P11_RUNNING_BLOCK` and `P11_FROZEN_ONLY` are collector controls used by this runner. Historical p10 debug after-verdicts must use the unchanged p10 collector and validator, not the release-only p11 validator.

The original v2 blind crop script is copied intact to `tools/art_pipeline/prepare_blind_crops_p11.py`; crop side is ceil(max(sprite width,height)+16), enlarged 4× nearest. Part 2 owns eligibility/exclusion protocol and archive helper dependencies. Identity framings remain inherited until re-derived at Part 3 per GOAL 4.3; they are not claimed to be game_v24 references yet.
