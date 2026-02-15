# T-900: UI Font Size Constants + TICK_MINUTES

## Status: TODO

## Description
Add UI_FONT_* constants to GameConfig. Change TICK_HOURS=1 to TICK_MINUTES=15.
Adjust all time-based decay rates, tick intervals, and age thresholds.

## Changes
- game_config.gd: UI_FONT constants, TICK_MINUTES, adjusted rates/intervals/ages
- simulation_engine.gd: get_game_time() with TICK_MINUTES
- stats_recorder.gd: tick_interval 50→200

## Done
- [ ] Constants added and verified
- [ ] docs/ updated
