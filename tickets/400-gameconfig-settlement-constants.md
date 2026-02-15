# T-400: GameConfig Settlement/Migration Constants

## Action: DIRECT
## Files: scripts/core/game_config.gd

## Changes
Add constants for settlement and migration systems:
- SETTLEMENT_MIN_DISTANCE: int = 25 (min distance between settlements)
- MIGRATION_TICK_INTERVAL: int = 200
- MIGRATION_MIN_POP: int = 40 (minimum population before migration checks)
- MIGRATION_GROUP_SIZE_MIN: int = 3
- MIGRATION_GROUP_SIZE_MAX: int = 5
- MIGRATION_CHANCE: float = 0.05 (5% per check when pop > threshold)
- MIGRATION_SEARCH_RADIUS_MIN: int = 30
- MIGRATION_SEARCH_RADIUS_MAX: int = 80
- SETTLEMENT_BUILD_RADIUS: int = 15
- BUILDING_MIN_SPACING: int = 2
