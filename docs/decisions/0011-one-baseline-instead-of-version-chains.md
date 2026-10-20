# ADR 0011 — One baseline instead of `game_vNN` chains

## Context
Each revitalize pass added a rules constructor `WorldRules::game_vNN` that called the previous one and changed a few fields (`game_v8 … game_v26`), plus matching worldgen configs and test scenes, so the live rules were spread over 19 chained functions.

## Decision
`WorldRules::baseline(seed)` and `WorldGenConfig::baseline(seed)` hold the live values as flat literals equal to `game_v26` (values drawn per world from `WorldVariety` stay calls). Older constructors and scenes are deleted; the probe keeps `game_v26` only as an alias of `baseline`. A test compares the baseline's Debug dump with the dumps captured at `v7-final-art`.

## Alternatives considered
Keep the chain (rejected: unreadable, and every new pass lengthened it); a RON rules file (deferred: data-driven rules may come with M2's content work).

## Consequences
New rule changes edit the baseline directly; behaviour changes are recorded in CHANGELOG/HISTORY, and identity runs compare against the previous baseline's data.

## Date
2026-10-20

## Source
docs/v8/V8_0_GOAL.md §8.1
