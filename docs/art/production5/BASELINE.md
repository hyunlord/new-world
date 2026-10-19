# Part 0 baseline

Merged exact simulation `fe60de21249ad46ad2e2d9ca420d896066277c82` cleanly in `34b9d4b4f0e7323197125f208d3002244b9f40fe`.

Full gate: 1941 Rust tests, 243 Python tests, all 31 Godot art scene tests, clippy, strict, bridge, smoke pass. See `gates/p0/results.tsv`.

Fresh headless WorldSimNode reports `colds` (24 entries/24 agents) and `wood_targets` (present; zero settlements at startup); `gates/p0/live_snapshot.log`. Source world_node.rs live constructor and reset call game_v12 at lines181/228. This proves bridge availability, not winter UI appearance.
