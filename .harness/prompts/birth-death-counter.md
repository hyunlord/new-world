# Birth/Death Counter Fix

## Problem
`stats_total_births` and `stats_total_deaths` are always 0 despite population changes.
Birth/Death events are pushed to `event_store` but counters are never incremented.

## Fix
Added `resources.stats_total_births += 1` after Birth event push in biology.rs.
Added `resources.stats_total_deaths += 1` after Death event push in biology.rs.

## Verification
1. Birth counter: after 4380 ticks, `stats_total_births > 0`
2. Death counter: after 4380 ticks, `stats_total_deaths >= 0`
3. Consistency: `initial_pop + total_births - total_deaths == current_alive_count`
4. Anti-circular: Birth event count in event_store == stats_total_births
