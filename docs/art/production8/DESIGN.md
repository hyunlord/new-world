# Art Pass 8 presentation system

## Intent and inheritance
Retain project-owned D3 art, production7/DESIGN.md and production/DESIGN.md. Native Godot rendering only. A quiet news panel, readable destinations and village pressure are additive presentation of game_v19 state. GOAL.md is the frozen acceptance contract.

## Colour
Stable village-number palette #E0B060, #7FB0D8, #B88AD0, #8CC07A repeats unchanged. Marker, label, selected-member outline and pressure meter use the same village colour. Marker outline uses existing dark outline. Morning chill outline #CFE3F5 at alpha0.8. Spoiled food particles #6B4E2A; rotten wood multiply #5A4A3A; light gift particles reuse the existing light dust colour.

## Typography and localization
Inherit Galmuri small HUD font and 12 screen-pixel village labels. Labels keep the existing dark backing and 5x3px padding. All player text uses Locale, en/ko Fluent + compiled JSON + registry in the same Part. Village arguments are stable presentation numbers. Plain eating never says famine.

## Geometry
World tile16px; native sprites unchanged. Edge arrows9x9px with1px outline; point from viewport centre to target, bounded inside visible world area. Founding distance is measured in tiles. Onscreen site uses existing stake exclusively. Pressure indicator3 segments beside labels at zoom>=2 and in selected inspector. Walking overlap correction max3screen px perpendicular to direction; deterministic AgentId; never onto water/buildings.

## Components and states
Feed classes: protected, ordinary, routine. Routine records retained invisibly; 20:00 per-village daily summaries. Protected visible retention>=240game ticks; pause freezes age; 4line capacity, oldest protected displaced only by protected overflow and +N counter. Full key-class table added by Part1.
Pressure P/theta thirds = calm/uneasy/ready; absent or zero theta hidden. One debug layer flag controls labels, meter, trail, marker and selected outlines for A30. Simulation and camera frozen at identical tick.

## Motion
Morning chill persists90game minutes after cleared cold for awake villagers, final30minutes fade; breath size1.5x. Food spoil6particles/1.2s with pile tint; wood rot4grey particles/1.2s with pile darken; gift6falling light particles. Windfall ring full alpha1.0s then fade0.3s. Effects target the nearest pile to the selected village formation tile.

## Verification
Natural live A1–A25, separate natural society A18 and A26–A32, labelled intervention A24/A33. Exact source/capture manifests, raw evidence outside git. A30 alternating50frame blocks,300on+300off, process brackets -1000/+1000 plus viewport CPU; GPU separate, vsync restored. Frozen blind answer files and fresh independent observers. Thresholds are never weakened; FAIL/N/A are retained. No image generation planned.

## News panel

Reuse existing ArtUI panel, existing theme and four-row typography. The counter uses the same Label styling below the rows. Daily summaries use20px text within the existing540px row so stored/eaten values remain readable. No new visual palette or assets. Protected news uses simulation ticks: 240 minimum protection and 480 tick expiry. Source event ticks and first observation ticks remain separate. Food summaries use observed ordinary stock changes; explicit casts and fission transfers are separately logged.

## Full feed key to class table

| Key | Class | Visible eligible | Village argument position (zero-based) |
|---|---|---|---|
| HUD_FEED_DEATH_STARVATION | protected | yes | none |
| HUD_FEED_DEATH_DEHYDRATION | protected | yes | none |
| HUD_FEED_DEATH_COMBAT | protected | yes | none |
| HUD_FEED_DEATH_OLD_AGE | protected | yes | none |
| HUD_FEED_DEATH_EXPOSURE | protected | yes | none |
| HUD_FEED_FISSION_DEPARTED | protected | yes | 1 |
| HUD_FEED_SETTLEMENT_FOUNDED | protected | yes | 0 |
| HUD_FEED_FISSION_CANCELLED | protected | yes | none |
| HUD_FEED_SETTLEMENT_ABANDONED | protected | yes | 0 |
| HUD_FEED_SETTLEMENT_FORMED | protected | yes | 0 |
| HUD_FEED_GOD_CAST_FMT | protected | yes | 0 |
| HUD_FEED_SETTLEMENT_DISSOLVED | protected legacy alias, no current producer | yes if explicitly pushed | 0 |
| HUD_FEED_AGENT_BORN | ordinary | yes | none (population) |
| HUD_FEED_SITE_STARTED | ordinary | yes | none (construction ID) |
| HUD_FEED_SITE_COMPLETED | ordinary | yes | none (construction ID) |
| HUD_FEED_DAY_SUMMARY_FMT | ordinary | yes | 0 |
| HUD_FEED_STOCKPILE_DEPOSIT | routine | never | 0 |
| HUD_FEED_STOCKPILE_CONSUME | routine | never | 0 |
| HUD_FEED_MORE_FMT | presentation counter, not an event | counter only | none |

Store the same explicit mapping in `DESIGN.md` and code. Unknown keys should not silently become protected. Counter creation must not append feed events.

### Protected history counter

The four visible rows expire after480simulation ticks; minimum240tick protection remains unchanged. The +N counter represents all additional protected history not currently visible, including expired rows, until world reset. It is not an unread indicator: no acknowledgement or history-reader feature is claimed. Ordinary and routine history never contributes to this counter. This preserves A31 representation across widely spaced captures without keeping stale rows onscreen.
