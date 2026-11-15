# ADR 0066 — Iron chain on the C2 metal machinery

## Date
2026-11-15

## Source
C3 goal §3 (`goal_worldsim_c3.md`), content bible §4.4, decisions in `docs/v8/c3/C3_DECISIONS.md`. Builds on ADR 0063 (surface ore), 0064 (scripted chain preparation) and 0065 (mining, provenance by kind).

## Context
The bible's iron era needs ore, a bloomery, a forge, iron tools, a stone house and the rest of the §4.4 column. C2 already has outcrops, a mining job, workshop recipes with several inputs, provenance by item tags, axe upgrade by speed, the fuel diagnosis and steward `Bills` with `stocked`/`replace_tag`. The pass is content-first: new code only where data cannot express a step.

## Decision
- Iron is data on the C2 machinery. `iron_outcrop` (hill/mountain) becomes Connected and a second row `bog_iron_outcrop` (plain/forest below river level) is appended; both use the per-kind keyed ore draw, so copper and tin placement and baseline worlds do not change.
- Items, recipes, stations and templates are appended rows. A bloom and a wrought bar are `ingot`-tagged `metal_iron` items, so the first bloom and the first iron axe reuse FirstIngot, ToolUpgraded and FirstToolUsed. Station tags keep metals apart (`bloomery` ≠ `smelting`); forge recipes are tagged `forging` and run at a forge or an anvil.
- The iron-era stone house is a new template (`cut_stone_house`, tier 5, upgrade from `mud_brick_house`). The neolithic `stone_house` row is unchanged, because saved sites are re-expanded from the current template and upgrade history checks `upgrade_from`. The template tier range becomes 1..=5.
- Content that has no runtime hook stays Inert with its pass named: paved-road speed, warehouse capacity, steel, plough, horses, writing, markets, law, temples and the rest.
- The chain (C3-b) adds no new mechanism: steward rules 14–17 are `Bills` data (a shared `pre_iron_axe` tag lets one `replace_tag` rule replace stone, copper and bronze axes), two mind observation facts name bloomery and forge work, and first-metal chronicle lines read their causes (gating knowledge, place, fuel) from the recorded metal witness through the existing evidence column.
- Chapters 8/9 and the `no_iron_nearby` card are bridge facts beside the C2 ones (tag-parameterised helpers). Published C2-b format-23 saves load through an exact-revision adapter (`legacy_bronze_v23`).

## Alternatives considered
- Swamp terrain or a river-adjacency origin predicate for bog iron: worldgen code and identity changes for one row. Rejected for this pass; the elevation band is data.
- Mutating `stone_house` into the iron house: smaller catalogue, but breaks loading of saves that hold a stone house. Rejected.
- An OR condition for "iron ore or bog iron" in knowledge discovery: not needed; iron smelting is discovered by practice with charcoal and copper smelting, so ore-less worlds can still learn it (and then see the card).

## Consequences
- Every colony world now generates iron deposits, so colony fixtures change (declared). Old saves get no deposits retroactively; they can reach iron by trade (`wrought_iron` goods row).
- A bog deposit on open ground can later be covered by trees or buildings; nothing protects the tile yet (debt).
