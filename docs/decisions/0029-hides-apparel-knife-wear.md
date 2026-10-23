# ADR 0029 — Hides, apparel and knife wear

Status: draft (M5 Part 0, 2026-10-23); final in M5 Part 7.

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
  and the warmest top first. A hide wrap scales cold exposure damage ×700 ‰, a fur cloak ×500 ‰; a fur cloak wearer
  works outdoors in a cold snap at the normal speed. A worn top is part of the ledger (`worn`); at a death it is left
  on the ground. The snapshot's apparel column gives the art its top.
- **Ledger**: Meat, CuredHide, HideWrap, FurCloak, Spear and WolfHide join the per-kind invariant (Hide already did);
  spoiled meat and broken knives and spears are consumed.

## Alternatives considered
- A knife slot beside the held tool: a person holds one tool (M2); a workshop tool keeps the scraper's axe in hand.
- Per-item durability on stacks: stacks merge by spec (M4 D6); wear stays on the held or workshop tool.

## Consequences
- Blades now have three sinks (axes, spears, knives); F8 measures knife breakage.

## Date
2026-10-23

## Source
docs/v8/m5/M5_GOAL.md §3 and §4; docs/v8/m5/M5_DECISIONS.md
