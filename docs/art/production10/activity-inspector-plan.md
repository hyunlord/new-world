# Parts 2–3 execution plan (after Part 1 accepted)

Read-only code/atlas survey, 2026-10-16. No production changes or launches made for this plan. Preserve accepted Part 1 implementation and its evidence before these intentional pixel changes.

## Integration and execution order

1. Add `get_activity_snapshot` to `scripts/ui/sim_reads.gd` GETTERS, the equivalence tool GETTERS in `scripts/test/art/test_sim_reads_equivalence_p9.gd`, and PROTECTED in `rust/crates/sim-test/tests/harness_art_p9_sim_reads.rs`. Use the established `SimReads.read` route. Join activity `ids` to agent snapshot `ids` (entity bits), not `agent_ids`. Preserve the nine-key detail schema.
2. Extend `scripts/ui/agent_activity.gd`: resolve state 4 before existing branches and explicitly handle target 8; accept activity kind and arrival/facing data. Keep old callers compatible via default arguments. Add explicit per-body pose/view chains below; do not let generic fallback accidentally select a different activity. `explore` remains a separate logical activity using idle frames so `camp_pose` cannot turn it into sit/hop. Add a facing override to `render_facing`; for arrived Explore it wins over the old idle-to-S timeout.
3. In `scripts/ui/agent_renderer.gd`, obtain activity rows via memoized reads, compute place direction from the appropriate settlement hearth nearest the place, nearest shelter for Repair, and stockpile for Craft. Carry `activity_place_known = until_tick >= 0` into draw rows and metadata. Before arrival use the normal walking path (Play uses run only E/W). Add Craft tool and Explore direction icons using existing drawable atlas entries or procedural drawing; include `activity_icon` in the same zoom >= 2 icon visibility rule.
4. Update `scripts/ui/camp_seats.gd` only after Part 1 spatial optimization is accepted: arrived holders are ineligible for seating, receive no declump shift, and receive no walker offset. They remain in tile spread and in declump fronts so other agents avoid them. Specifically override spread's current `_walking(row)` exclusion for an arrived playing child drawn as run; do not globally redefine `_walking`.
5. Extend the actual inspector file `scripts/ui/panels/agent_inspector_panel.gd`. Its `_ready` currently adds `_vbox` directly to a 360 x 620 PanelContainer. Insert a ScrollContainer under that panel and put the existing VBox inside it, retaining the width. Append temperament and family after `_cold_label`. Existing `_make_bar` indexes fixed three-element icon/color arrays; introduce a separate temperament helper with max_value 1000, never a fourth call to `_make_bar`.
6. Replace clamped state/target lookup in `display_agent` with explicit bounds. State 4 uses its kind label from the entity-bit activity join; target 8 formats `INSPECTOR_TARGET_ACTIVITY_PLACE_FMT` with activity place coordinates. Extend label mappings explicitly; unknown tags must not silently alias the last valid label. Family and temperament getters remain direct selected-agent calls only while the panel is open.
7. Add household rings after panel/selection works: reuse `agent_id -> sprite` lookup and pool Line2D rings with the existing selection-ring geometry at lower alpha. Show living partner/children only while inspector visible; clear/recycle on close, reset, missing relation or despawn.

## Actual atlas availability and required chains

Survey used `assets/art/atlas/world_atlas.json` sprites with drawable=true and QA status accepted/derived/procedural/fallback, matching `build_frame_groups`. W uses E with flip_h; there are no W groups. All three bodies have idle/walk E,N,S. Elders use adult_a/adult_b plus elder hair, never an elder atlas body.

| kind | adult_a | adult_b | child | arrived facing / approach |
|---|---|---|---|---|
| 0 TendFire | crouch S -> sit E/S -> idle | crouch S -> sit E/N/S -> idle | sit E/S -> idle | force S for crouch; otherwise hearth; approach walk |
| 1 Craft | sit E/S -> idle | sit E/N/S -> idle | sit E/S -> idle | stockpile direction if sit supports it, else S; approach walk |
| 2 Repair | build E/N/S -> idle | build E/N/S -> idle | build E/S -> idle | nearest shelter; unsupported child N falls to explicit build S; approach walk |
| 3 HearthCircle | sit E/S -> idle | sit E/N/S -> idle | sit E/S -> idle | hearth direction if supported, else S; approach walk |
| 4 Explore | idle E/N/S | idle E/N/S | idle E/N/S | away from hearth, no camp-pose rewrite; approach walk |
| 5 Play | idle E/N/S fallback | idle E/N/S fallback | hop S alternating run E/W every 0.5s | unsupported run direction uses hop S; approach run E/W else walk |
| 6 SitByFire | sit E/S -> idle | sit E/N/S -> idle | sit E/S -> idle | hearth direction if supported, else S; approach walk |

Unexpected kind/body combinations above still have defined fallbacks and require unit coverage. Current crossed exists adult_a/adult_b S only; children have no crossed. Crouch exists adults S only. Hop exists child S only; run child E only. Build child has no N. Test chosen sprite/view/flip, not only logical pose text, across every kind/body/view.

## Preserve override precedence exactly

Current renderer sequence is `resolve -> pickup override -> render_facing -> camp_pose -> meal override -> Cold.pose -> choose_frame`. Pickup chooses gather_wood; meal then chooses eat. Cold/morning chill only replace stationary idle/sit/hop with crossed, so eat and gather_wood remain unchanged. Preserve sheltered-sleep hiding. Activity selection goes before these existing overrides, and capture sidecars must record base activity, chosen pose, final atlas frame and each applied override. Explore's logical string must survive camp_pose, while its idle rendered base must still receive the applicable existing cold treatment. Cold's crossed fallback is body/view-sensitive, especially for children: validate expected final frame through the current safe fallback logic.

## Family selection identity contract

`display_agent` resolves `detail.agent_id` with existing `_resolve_entity_bits`, which reads parallel snapshot `agent_ids` and `ids`. `get_agent_family(entity_bits)` returns parent/partner/children **AgentIds**, with -1 absent links. Resolve each link against the current live snapshot before showing a button. For -1, unresolved/dead links show existing `PANEL_FAMILY_NONE`; never pass a family AgentId directly to `get_agent_detail` or `get_agent_family`.

Click callback resolves the AgentId afresh, calls `get_agent_detail(entity_bits)`, verifies found=true, then calls `display_agent(detail)`. Current inspector has no independent public selection signal: selection is observed through `get_selected_entity_bits()` and `visible`; retain that contract. Agent names are not exposed by the existing nine-key detail, so use localized `INSPECTOR_AGENT_FMT` with the numeric AgentId. Show all living-child count and at most the first five returned sorted IDs as buttons. Hide temperament when found=false; family missing fields must not leave stale buttons from previous selection.

## Localization audit and exact additions

`rg` over all five required files found **none** of the 14 new keys at survey time. Recheck before implementation in case parallel work changes this. Registry currently has active_key_count=5221, key_count=5226, removed_key_count=5, max ID=5225; `keys` is an ordered ID list and `key_to_id` maps key to ID. Compiled en/ko have `strings` dictionaries of 5221 entries and `meta.active_key_count/key_count`. Append IDs without reindexing existing/removed entries.

| proposed ID | key | English | Korean |
|---|---|---|---|
| 5226 | UI_ACTIVITY_TEND_FIRE | Tending the fire | 불을 지키는 중 |
| 5227 | UI_ACTIVITY_CRAFT | Making things | 무언가 만드는 중 |
| 5228 | UI_ACTIVITY_REPAIR | Mending a shelter | 쉼터를 손보는 중 |
| 5229 | UI_ACTIVITY_HEARTH_CIRCLE | Sitting at the hearth | 화덕가에 둘러앉음 |
| 5230 | UI_ACTIVITY_EXPLORE | Looking around afar | 멀리 둘러보는 중 |
| 5231 | UI_ACTIVITY_PLAY | Playing | 노는 중 |
| 5232 | UI_ACTIVITY_SIT_BY_FIRE | Resting by the fire | 불가에서 쉬는 중 |
| 5233 | INSPECTOR_TARGET_ACTIVITY_PLACE_FMT | Place (%d, %d) | 장소 (%d, %d) |
| 5234 | UI_INSPECTOR_TEMPERAMENT | Temperament | 기질 |
| 5235 | UI_TEMPERAMENT_NS | Curiosity | 호기심 |
| 5236 | UI_TEMPERAMENT_HA | Caution | 신중함 |
| 5237 | UI_TEMPERAMENT_RD | Warmth | 정 |
| 5238 | UI_TEMPERAMENT_P | Persistence | 끈기 |
| 5239 | UI_FAMILY_CHILDREN_FMT | Children: %d | 자녀: %d명 |

Part 2 adds first eight: active=5229, total=5234. Part 3 adds remaining six: active=5235, total=5240. Update both Fluent files, both compiled JSON files including counts, and key_registry in each Part. Reuse UI_FAMILY, PANEL_FAMILY_SPOUSE, PANEL_FAMILY_NONE, UI_MOTHER, UI_FATHER, UI_FEMALE, UI_MALE. Do not reuse classical TEMPERAMENT_* keys.

## Verification deliverables

- Resolve and final-frame matrix covering seven kinds, three bodies, four views, walking/arrived, Explore timeout, and cold/meal/pickup precedence. Spatial tests prove arrived holders remain spread/front obstacles while not seated/shifted/walker-offset.
- Fake-world inspector tests: four exact permille values, hidden missing component, all family links, absent/dead partner, selection click changes entity bits, first-five child links, close/reset clears highlights. Exercise overflow by opening a populated family and scrolling within the 620px panel.
- Live day30 10:00/15:00 and chosen populous day60 10:00 captures at Z4/Z1 with every activity-holder pose/override sidecar and zero mismatches. Population selection uses the specified release probe with four workers and 30-minute cap; stress v19 worlds cannot substitute.
- Partnered-parent inspector capture with household rings. English/Korean UI inspection without raw localization keys.
- Freeze 14 labelled crop IDs and hidden answer key, then send only crops plus seven names to a fresh blind reviewer; save answers before revealing key. Report per-kind and aggregate accuracy, no invented pass bar.
