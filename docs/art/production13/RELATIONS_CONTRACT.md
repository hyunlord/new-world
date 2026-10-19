# Art Pass 13 relation and porter contract

Status: SOURCE_VERIFIED at merged art HEAD `c9c0c35577dc8f0871c0bd47b8f95f0eebbb330b`. This read-only audit does not establish runtime, performance or visual acceptance. No product source was edited by this audit.

## Live scene

`rust/crates/sim-bridge/src/ffi/world_node.rs:204` builds both `WorldGenConfig::game_v26(seed)` and `WorldRules::game_v26(seed)`. `init` calls that helper at line236 with default seed20261010; `reset_world` calls it at line328 with the requested seed. `reset_world_stress` still uses game_v19 via `build_stress_engine`. Source regression `live_game_is_game_v26` at line3313 exists; execution belongs to the Part0 gate, not this audit.

## Unchanged packed contract

Compared entire files, byte-for-byte, against Art Pass12 `57ae7ac40abde7d24369606c9250d3e06e0f18d6`. The following also produce an empty `git diff 57ae7ac40abde7d24369606c9250d3e06e0f18d6 HEAD -- <paths>`:

| File | Working-tree SHA256 | Against Pass12 |
|---|---|---|
| `rust/crates/sim-bridge/src/ffi/relations.rs` | `411edb8c277a376fcb608cc1100949b6aa5dd5ce6da8fae5a642913a1af7258a` | identical |
| `rust/crates/sim-bridge/src/ffi/chronicle.rs` | `81c8ed75798b23abc409f53a24527a9a7209fb794c466c643adda2f0dc7c2105` | identical |
| `rust/crates/sim-core/src/chronicle.rs` | `1869f9d676516750a6ffb420a799bf14115915412a3adb4055a971c9d7a94928` | identical |
| `rust/crates/sim-core/src/components/porter.rs` | `5e0402b85f36ffdd7187dadbd58c6a47224c096f5b988d590a19266722980ffb` | identical |

`world_node.rs` diff contains the live-version change and added tally test fields; the relation/porter getter wrappers are unchanged. `get_settlement_relations` and `get_porter_snapshot` are both present. The exact contract in `docs/art/production12/RELATIONS_CONTRACT.md` remains applicable:

| Getter | Columns | Packed type |
|---|---|---|
| get_settlement_relations | a, b | PackedInt64Array |
| get_settlement_relations | value, base | PackedInt32Array |
| get_settlement_relations | estranged | PackedByteArray |
| get_porter_snapshot | ids, from, to | PackedInt64Array |
| get_porter_snapshot | kind, stage | PackedByteArray |
| get_porter_snapshot | amount | PackedInt32Array |

Relation rows follow ordered village-pair BTreeMap order. Porter rows sort by `(AgentId, entity_bits)`; `ids` contain bit-preserving signed entity bits, not person AgentId. Join against agent snapshot ids, never by array position. kind0=food/1=wood, stage0=outbound/1=return. Amount is departure quantity, saturating to i32; pose must use actual carried_foods/carried_woods rather than infer a load from departure amount or stage. Arrays are parallel, same-length, empty without applicable rows.

Chronicle enum `ALL` still has22 variants, discriminants0–21. Packed tick/subject/other/third/settlement/settlement2/value are int64, kind/cause int32. Missing optional ids are-1 in bridge columns, null in evaluation JSON. Person identifiers in chronicle are AgentId, distinct from porter entity bits.

| Kind | Roles | Cause / value |
|---|---|---|
| 17 PorterDeparted | subject porter; settlement sender, settlement2 receiver | 0 aidfood, 1 aidwood, 2 exchangefood, 3 exchangewood; departure amount |
| 18 PorterDelivered | subject porter; settlement receiver, settlement2 sender | success<10; turnback10+departure cause; delivered amount (failure0) |
| 19 Quarrel | subject picker, other rival; their villages respectively | relation after quarrel |
| 20 VillagesEstranged | no persons; lower/higher village ids | relation |
| 21 VillagesReconciled | no persons; lower/higher village ids | relation |

Pass21 changes simulation behavior, not this interface: estrangement avoidance, capped daily conflict, faster drift and more aid. Earlier Pass12 observations such as zero reconciliation are historical and not current claims. Current cohort evaluation reports8 reconciliations; actual displayed sentence and relation-line colors remain pending live capture.

## Evidence commands

```sh
git rev-parse HEAD
git diff 57ae7ac4 HEAD -- rust/crates/sim-bridge/src/ffi/relations.rs rust/crates/sim-bridge/src/ffi/chronicle.rs rust/crates/sim-core/src/chronicle.rs rust/crates/sim-core/src/components/porter.rs
git diff 57ae7ac4 HEAD -- rust/crates/sim-bridge/src/ffi/world_node.rs
rg -n 'build_game_engine|fn init|fn reset_world' rust/crates/sim-bridge/src/ffi/world_node.rs
```

Whole-file equality also checked with Python `subprocess.check_output(['git','show', baseline + ':' + path]) == Path(path).read_bytes()`, all four true. Discovery used one `graft ask ... --source` call, followed by exact source/diff inspection. W-multi's new source recipe and provenance are recorded in `perf/world-selection.md`; runtime counts stay pending until census.
