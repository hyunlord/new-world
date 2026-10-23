# ADR 0029 — Hides, apparel and knife wear

Status: accepted (M5, 2026-10-23; drafted in Part 0, final in Part 7 with the amendments below).

## Context
M2–M4 never consumed blades except to haft axes, so skill growth (M4 F8) had no subject. The art line's people wear
one of three tops (loincloth, hide wrap, fur cloak); the simulation never decided what anyone wore.

## Decision
- **Hide work** at the drying rack (gated by `hide_scraping`): `scrape_hide` (Hide 1 → CuredHide 1, 60 ticks, knap),
  `make_hide_wrap` (CuredHide 2 → HideWrap 1, 90 ticks), `make_fur_cloak` (droppable 8.5: WolfHide 2 → FurCloak 1,
  120 ticks; a killed wolf drops a WolfHide).
- **Knife = a FlintBlade used as a tool** that lasts 8 uses. A workshop recipe that needs a knife keeps one blade as the
  workshop's tool (supplied like an input, counted on its shelf by the ledger); each craft wears it one use; at 8 it
  breaks (ledger consumed, chronicle `ToolBroke`). Butchering is part of a kill: it wears the hunter's held blade one
  use (Meat 3), or yields Meat 2 when the hunter holds none.
- **Apparel**: `Apparel { top: HideWrap | FurCloak, since }` on a person who wears one (absent = loincloth). A standing
  order "모두 가죽 옷" (`colony.dress_all`, on by default; command `SetDressOrder`) gives an `Equip` job (Haul type) for a
  top on a stockpile to the person who most needs it — highest `Hardiness` (lowest cold tolerance), then `AgentId` —
  and the warmest top first. A hide wrap's ×700 ‰ (a fur cloak's ×500 ‰) acts on the cold hour itself (amended,
  D15); a fur cloak wearer works outdoors in a cold snap at the normal speed. A worn top counts as held in the
  ledger; at a death it is left on the ground. The snapshot's apparel column gives the art its top.
- **Ledger**: Meat, CuredHide, HideWrap, FurCloak, Spear and WolfHide join the per-kind invariant (Hide already did);
  spoiled meat and broken knives and spears are consumed.

## Alternatives considered
- A knife slot beside the held tool: a person holds one tool (M2); a workshop tool keeps the scraper's axe in hand.
- Per-item durability on stacks: stacks merge by spec (M4 D6); wear stays on the held or workshop tool.

## Consequences
- Blades now have three sinks (axes, spears, knives); F8 measures knife breakage.

## Amendments (as built, M5 DECISIONS D10, D15)
- The standing order is `ColonyState::dress_off` (false = on, the default); the hourly pass gives each free top on a
  stockpile cell to the neediest adult **free at that pass** (a busy person is served at a later pass).
- When the hearth is out on a winter night a wearer is cold that hour with chance `Top::cold_permille` (one keyed roll
  an hour, `STREAM_APPAREL`, the lineage key); a cold hour bites as for anyone. Cold hours, the mood's cold night and
  exposure damage all scale by the factor on average. Scaling only the damage (the draft) left cold events equal
  between clothed and unclothed people by construction.
- Held-out result, final: F5 19/32 (wraps arrive too slowly on twelve worlds: hides come one a kill and a wrap takes
  two), F8 0/32 (a world uses a knife 6–22 times in 48 days: one or two knives, not ten — the goal's sink is too small for
  its bar).

## Amendment M5.5 — one hide a wrap, four uses a knife (as built; M5.5 goal §3.1.4–5, DECISIONS M5.5 D12)
- `make_hide_wrap` takes **1** cured hide (the fur cloak keeps 2 wolf hides): 4 wraps need 4 kills, which 30 days of
  hunting give (M5: ~1 kill in 7 days at first, faster later).
- `knife_uses` 8 → **4**: with traders buying blades (ADR 0032) the knife and the trade are the blade sinks.
- Reason: M5's F5 (wraps) and F8 (knives) bars could not be met by the M5 numbers; the arithmetic is in the M5.5 goal.

## Date
2026-10-23

## Source
docs/v8/m5/M5_GOAL.md §3 and §4; docs/v8/m5/M5_DECISIONS.md
