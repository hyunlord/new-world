# Locale Key Fix — 45 missing keys synced from JSON to fluent

## Goal

45 keys exist in `localization/ko/*.json` and `localization/en/*.json` but are missing
from the fluent source files (`localization/fluent/{ko,en}/messages.ftl`). Since the
compiler uses `source_format: fluent`, these keys are silently dropped from the compiled
output and `key_registry.json`, causing raw key display in-game.

## Current State

The keys have already been appended to the fluent files and the compiler has been re-run.
The changes are unstaged in the working tree. This pipeline run validates correctness.

## Changes (already in working tree)

- `localization/fluent/ko/messages.ftl`: +45 keys appended (Korean translations)
- `localization/fluent/en/messages.ftl`: +45 keys appended (English translations)
- `localization/compiled/ko.json`: regenerated (4978 keys, was 4934)
- `localization/compiled/en.json`: regenerated (4978 keys, was 4934)
- `localization/key_registry.json`: regenerated (4978 keys, was 4934)

## Key examples

| Key | ko | en |
|-----|----|----|
| UI_OVERLAY_AUTHORITY | 권위 | Authority |
| UI_DOOR | 문 | Door |
| UI_WALL | 벽 | Wall |
| UI_TILE_INFO | 타일 정보 | Tile Info |
| ROOM_ROLE_CRAFTING | 공방 | Crafting |
| RECIPE_STONE_AXE | 돌도끼 | Stone Axe |

## Verification

1. **Anti-circular**: `UI_OVERLAY_AUTHORITY` must appear in compiled/ko.json as "권위" (not raw key)
2. **Registry stable**: existing key IDs unchanged (e.g. `ACE_DOMESTIC_VIOLENCE: 0` still 0)
3. **No regression**: all existing keys still present in compiled output
4. **Visual Verify**: in-game overlay bar shows "권위" instead of "UI_OVERLAY_AUTHORITY"
5. **Locale check passes**: `bash tools/harness/locale_check.sh` reports 0 critical P2 issues for these 45 keys
