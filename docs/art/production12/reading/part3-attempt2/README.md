# Attempt 2 independent reference audit

Source: `c46e66bd128aa271772caba5f9317de860d222db`. Six capture results exit0, errors empty. Audit found **0 data/provenance/hash/scroll-range issues**. This is not the fresh reader's acceptance verdict.

- 140 original images are 1920×1080. Every PNG hash matches its sidecar; every anonymous reader copy matches the private map and original hash. No image was changed or rescaled by this audit.
- All sidecar heads and loaded bridge hashes match registration. Bridge SHA256: `65be186ddffe90bba99a491432d7e41bc63abb431f4350b09561b6420ea19a3c`.
- All six fresh references contain activities, agents, buildings, full chronicle pages, clock, persons, porters, relations, settlements, society, tallies and world_info. Living selected Life subjects additionally contain agent_detail. Dead person0 correctly has no living detail.
- All selected person-chronicle rows occur exactly in their own full chronicle. All 18 section/language scroll groups begin at0, cover their recorded content range without gaps and reach the bottom. This range test does not prove absence of partly clipped lines or glyph ambiguity.
- No raw CHRON_/INSPECTOR_/LIFE_ key or unexpanded printf placeholder occurs in captured Life/chronicle text.
- EN/KO semantic facts match raw records: child25/dead parent0, departures47 cause1 and45 cause2 at84840, settlement2 from1 at84993, porter69 food10 aid sender2→receiver1 at120960, quarrel23 with57 village1/2 at31769, estrangement1/2 at34560. Birth/partner/child roles and display days match the raw rows. Text is materially the same historical narrative as attempt1; pixels and reader performance must be judged independently.

`audit.json` preserves raw selected event records, reference hashes, per-image checks, all scroll intervals and mapped-image checks. `scoring-reference.json` supplies selected agents/persons/activity/place, clocks, population, tallies, stocks and buildings without depending on the renderer's prose. `audit.py` reproduces the automated checks; it does not open reader answers.

Current activity truth: live25 kind6 at(23,18); multi45 kind6 at(3,62),47 kind3 at(6,63); crowd1 person1 kind3 at(23,18), crowd2 person29 kind6 at(41,33), crowd3 person30 kind1 at(24,21). Porter69 has no daily activity row. Its raw carried_foods is10; it is outbound to village1.

Limits retained: no kind18/21 exists in these raw worlds at capture ticks; group deliveries and reconciliation remain unexercised. W-multi has departure1/2; crowd2 also contains cause3 without a selected Life capture. Wood/exchange/failed-return and other-role quarrel Life are not covered. The resource snapshot is still absent: a claimed HUD berry total would need another raw artifact, not an inferred pass. Sidecar text is not proof of pixel readability, target-label correctness, or lack of clipped lines.

No reader answers accessed or reader contacted. No sources, builds, Godot, cargo or commits touched. Attempt1 preserved.

## Separate resource replay corroboration

`resource-replay-audit.json`: PASS. Replay at c08076dd uses the same bridge and exactly equal source manifests as the c46e66bd captures. All six original reference hashes match; clock, tallies and world_info compare exactly under recursive numeric normalization (integers/floats by decimal value, booleans distinct; no tolerance). Berry totals independently summed from raw resource kind0 amounts<255 are live48, multi54, estranged40, crowd1 30, crowd2 104, crowd3 40.

The replay’s original clock/tallies equality flags remain false in the preserved replay artifact and are copied into the audit. JSON numeric semantic equality does not recover original GDScript numeric runtime types; int/float dictionary typing is consistent with the mismatch. No raw flag or original capture was overwritten. This supplies later corroboration for berry claims, not original capture-time resource evidence. No rerun was performed and no reader answers were opened.
