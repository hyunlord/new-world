# Part 0 live-world census

Source merge: `0caabf5fd1c62b9d3c99845daab1805e79799db7`. Debug bridge built locally; no renderer changes. Source `world_node.rs:193` builds live `game_v22`; stress builder remains `game_v19` at line 185. Runtime census: `live-day30-1000.json`.

Seed 20261010, displayed day 30 at 10:00 is tick 41880 (day_index 29, clock offset applied). Population 29; settlements 1; lit fires 0. Seven state-tag-4 agents: TendFire 1, Craft 0, Repair 0, HearthCircle 3, Explore 0, Play 1, SitByFire 2. Unknown activity kinds: 0. Five family/temperament/detail getter samples are preserved in the raw JSON. Probe elapsed 200.007 seconds, zero failures.

This establishes bridge outputs, not visual correctness. Original six framing census and replacement selection are still pending.
