# ROUND1 — Pass A functional / design integrity

**Verdict: REVISE. Confidence: high for the located defect; medium-high for the reviewed surface overall.**

Read-only review of `b83dc5ae..33b8f7c7`, native Godot 1920×1080. One blocking presentation inconsistency below. No repository files changed, no Godot/cargo invocation, no prior reviewer outputs, GOAL sections 3–7, or life-reader answers consulted. This review does not replace the pending full gate or frozen performance measurements.

## Blocking item A1 — Porter inspector exposes the internal self-target as “Another villager”

`p3/live-attempt1/multi/porter_life_en_page_000.png` visibly says **“Carrying food ×10 to Village 1”** beneath Hugade #69, but also **“Target: Another villager”**. These are contradictory destination explanations for the same current trip.

- `scripts/ui/panels/agent_inspector_panel.gd:312` formats the generic target unconditionally, even after `_refresh_porter_status()`; lines 339–343 correctly derive the trip destination separately.
- `scripts/ui/panels/agent_inspector_panel.gd:53` maps target kind 5 to the ordinary social-agent label.
- `rust/crates/sim-systems/src/runtime/settlement/relations.rs:238` explicitly identifies this as the porter's **self-targeted walk**, not a trip to another person.
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs:3056` routes the porter to its stage's hearth.
- `p3/live-attempt1/multi/reference-data.json`: agent 69 / entity 4294967342 has target_kind 5, current tile (21,48), destination tile (21,41), carried_foods 10; its authoritative trip is from settlement 2 to 1, outbound, amount 10. This corroborates the PNG and rules out a real secondary social target.

**Required bounded correction:** while a porter trip is active, suppress or replace the misleading generic target with the current stage destination; retain the authoritative porter line and ordinary target behavior for non-porters. Verify outbound and return stages, both locales, and transition back to a non-porter. Re-capture the affected live inspector view before accepting this pass.

## Evidence-backed checks

| Area | Assessment / evidence |
|---|---|
| Real live UI / existing art | PASS. `ArtUI.panel()` applies shared `worldsim_theme.tres`; actual Label, RichTextLabel, ScrollContainer, Button and Sprite2D nodes remain. Inspector refresh reads live detail/snapshots; world sprites use the existing sprite library. No screenshot substituted for live widgets. Viewed live world, portrait, panels, terrain, buildings, carriers and crowds. |
| Names / weight / wrapping | PASS for representative rendered samples. `person_names.gd:42–58` keeps canonical spelling separate, adds word joiners for display, and installs Galmuri11-Bold for rich names. English and Korean chronicle/life samples show whole bold names and village links. `p2/font-attempt1/font-result.json` records size 24 and zero joiner pixel differences for regular/bold, while ti/di differ by 24/32 pixels. This supports glyph identity without treating metadata as visual inspection. |
| Copy cleanliness | Source-supported, actual OS clipboard interaction NOT_VERIFIED in this review. `person_names.gd:63–73` strips U+2060 for keyboard copy and deferred menu copy. Existing name test checks conversion and selection availability, not a real clipboard round trip. |
| Scroll ends | PASS for inspected endpoints. `real-input-final/04_wheel_end_last_line.png` shows the entire final death sentence. JSON gives final line top 664, height 28, viewport bottom 728, scroll at max 663. `p2/layout-attempt2/ko-bottom.png` shows whole final name/line; `multi/chronicle_en_page_014.png` shows complete final lines. Inspector measured line-band padding and chronicle bottom inset are real layout code. |
| Frozen locale update | PASS for inspected evidence and update paths. `real-input-final/10_language_english.png` has English HUD, inspector, village label, news and hand panel; its step records no visible Hangul. Inspector refreshes every visible frame and life cache includes language; chronicle refresh tests language independently of tick; stockpile locale invalidation exists. |
| Overlap input coordinates / cycling | PASS. `world_renderer.gd:79` transforms `event.position`, rather than global mouse state. Real-input JSON uses (960,540) with the actual OS mouse elsewhere and obtains 25→56→55→25. This is genuine Input.parse_input_event routing, with arranged real sprites as setup. |
| Selection draw priority | Source and live-crowd evidence support PASS. `agent_renderer.gd:800` raises selected sprite/icon and restores the prior pair. `crowd3/crowd3_en.png` and sidecar identify selected Agent_30 at rect [904,456,84,112], body/icon ranks 533/534, visible within viewport. Caution: real-input screenshots alone do NOT establish visual raising, because that input fixture disables AgentRenderer processing; live crowd and renderer/test evidence are the separate support. |
| Porter pose / glyph / line | PASS apart from A1. `multi/porter_road_z4.png` visibly shows carried food baskets, bag glyphs and selected ring; sidecar identifies existing accepted V_b2_03 carry frame, food load layer and p12_porter_food icon. Raw snapshot confirms actual food 10. `Porter.activity()` uses actual food/wood, and destination selects `to` outbound / `from` return from current settlement snapshot. z1 view shows current destination route; route drawing is deliberately disabled at zoom ≥2. |
| Current relation lines | PASS. Current anchors are rebuilt from current settlement snapshots, not historical registered names (`settlement_relations_p12.gd:40–63`). Estranged view shows red dashed relation; sidecar has current pair 1–2, value -441, red c0392b, width 2px and current anchors. Normal multi-world view shows the green connection. |
| Chronicle 17–21 roles / grouping | Source and sampled visual evidence support PASS. Departure maps source→destination; delivery reverses settlement fields appropriately; quarrel other-view resolves counterpart's village; 20/21 omit person prefix. `_ingest_row()` breaks pending group on EVERY non-18 raw row and keys merges on tick, settlement, settlement2 and cause, including across fetch boundaries. This avoids merging across filtered hidden rows. Reviewed dedicated tests for cross-page merging, hidden separators and failed-trip causes, without rerunning them. Quarrel life and world-feed screenshots render whole corresponding sentences. |

## Representative PNGs opened directly

All paths below are relative to `/Users/rexxa/work/art-production12-evidence/`:

- `p3/real-input-final/04_wheel_end_last_line.png`, `02_click_1.png`, `10_language_english.png`
- `p3/live-attempt1/multi/porter_road_z1.png`, `porter_road_z4.png`, `porter_life_en_page_000.png`
- `p3/live-attempt1/multi/chronicle_en_page_007.png`, `chronicle_en_page_014.png`, `chronicle_ko_page_000.png`
- `p3/live-attempt1/estranged/chronicle_en_page_000.png`, `quarreller_life_en_page_003.png`
- `p3/live-attempt1/crowd3/crowd3_en.png`
- `p2/layout-attempt2/ko-bottom.png`, `en-top.png`

## Optional / remaining limits

- Add a real keyboard-copy and context-menu-copy round trip to the input evidence when convenient; check pasted text contains no U+2060. Current code looks intentional, but converter tests do not exercise Godot default-input ordering.
- Directly inspected a representative subset, not every one of the 140 PNGs. No fresh interactive runtime was permitted or performed. No claim of universal overlap-free crowds, all-world correctness, or performance acceptance is made.
- Existing art, controls and layout require no redesign for this finding. Only the report file was written; no simplification or production-code change was made.
