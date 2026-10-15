# Part 1 API contract audit

Compared the Stage A implementation at `8de4073adbd152ecdf91ab28a1a348085e87a20f` with the unchanged base `adfdecfece79ab019964729319389fd602685fab`. A separate read-only compatibility audit inspected the changed existing Rust structures and FFI collectors.

- The only new fields in existing public structures are `AgentRules.routine` and `SocietyRules.family`, the two fields authorized by section 5.2.
- `AgentTracer.routine` is private and `AgentTracer` is crate-visible. New activity components, observation types and getters do not extend existing external structures.
- `AgentSnapshotRow` retains 15 fields and the existing snapshot dictionary retains 17 keys. `AgentDetailRow` and `get_agent_detail` retain their exact nine fields/keys.
- `Tallies` retains 33 fields; `FissionRecord` retains 14. Neither existing structure was modified.
- Observer output extends `people.routine`, `agents.adult_ticks`, `agents.routine` and `meta.pass17`. These are trace outputs, not public Rust structure fields or existing FFI dictionaries.
- No tracked or untracked changes exist under the prohibited `rust/crates/sim-test/src/worldmodel/` path.
- The actual Godot getter smoke checks the nine-key detail dictionary and the new five-array activity dictionary. On the still-live `game_v20`, the activity arrays are empty, as required. The nonempty live activity join is checked after the final scene switch.

This is a bounded implementation audit, not one of the two fresh Part 8 review rounds.
