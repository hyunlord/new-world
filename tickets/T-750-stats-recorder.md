# T-750: StatsRecorder System

**Priority:** Critical | **Status:** Open

## Description
New SimulationSystem (priority=90, tick_interval=50) that records population/resource/job snapshots.
Required by T-753 (Stats Panel).

## Implementation
- New file: `scripts/systems/stats_recorder.gd`
- Extends SimulationSystem base
- Records: tick, pop, food/wood/stone totals, job counts
- MAX_HISTORY = 200 (= 10000 ticks)
- Registered in main.gd

## Done Definition
- [ ] StatsRecorder records snapshots every 50 ticks
- [ ] History capped at 200 entries
- [ ] Registered in simulation engine
- [ ] docs/SYSTEMS.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
