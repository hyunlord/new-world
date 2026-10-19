# Frozen factual reading: part 2 scoring

**117/117 independently corroborated atomic text claims correct; 14 additional screen matches remain NOT_VERIFIED against independent data.** Six name readings match the visible strings exactly, with zero visible name errors. Picture-only activity recognition is **NOT_ESTABLISHED**, not 3/3.

The answer SHA-256 matches its pre-scoring freeze: `a1615ec056fbfbc8327b845a45ee862200c8e610439b62bfbde7727280ce6ec8`. Freeze time: 2026-10-19T13:11:11.128064+00:00. All six mapped PNG hashes were verified and all six full images visually inspected. No blind-reader contact or revised answers.

[Atomic claim ledger and source hashes](part2-score.json) retains every text claim, raw activity row, exact reference pointer, picture interpretation and artifact observation. Reference files are each capture directory's `reference-data.json`, using `reference_worlds[0]`; screenshot sidecars supply rendered-context evidence only.

## Counts

| Image | Total atomic text claims | Independently correct | Incorrect | Screen-only | Name |
|---|---:|---:|---:|---:|---|
| image_0001.png | 22 | 19 | 0 | 3 | Baho |
| image_0002.png | 21 | 19 | 0 | 2 | 바호 |
| image_0003.png | 23 | 21 | 0 | 2 | Simowa |
| image_0004.png | 22 | 20 | 0 | 2 | 시모와 |
| image_0005.png | 22 | 19 | 0 | 3 | Wano |
| image_0006.png | 21 | 19 | 0 | 2 | 와노 |

One assertion per scalar field; each position/target coordinate pair counts once. Spring and season-day count separately. EN/KO readings count separately, including repeated facts. Names are a separate six-reading result. Picture-only interpretations and artifact observations do not inflate the fact numerator or denominator.

## Independently verified activity identities

| Selected subject | Entity bits | Raw activity kind | Place | Until tick | Capture tick |
|---|---:|---|---|---:|---:|
| 1 | 4294967297 | 3 | (23, 18) | 41960 | 41880 |
| 29 | 4294967326 | 6 | (40, 35) | 85116 | 85080 |
| 30 | 8589934612 | 1 | (24, 21) | 42244 | 42180 |

These are respectively HearthCircle, SitByFire and Craft, joined by `agents.agent_ids → agents.ids → activities.ids`. All activities remain active beyond the capture tick. The reader correctly reads the corresponding activity **text**, but explicitly describes the image-only interpretation as plausible/inferred/uncertain. Agreement with the hidden data cannot retroactively convert that response into confident picture recognition.

The activity target is `activities.place_x/place_y`, not the generic agent target coordinates: these agents have `target_kinds=8` and generic `target_xs/ys=-1`. Raw activity-place coordinates corroborate the target displayed in the inspector.

## Unverified independent-data claims

All six construction-zero readings match the pixels, but the saved reference worlds contain no construction snapshot. All six berry-count readings match the pixels, but the reference worlds omit berry-resource counts. The two explicit daytime readings also match the pixels and have corroborated hours 10/15, but the raw reference contains no daytime flag; no unrecorded threshold was assumed. These 14 claims remain SCREEN_MATCH_ONLY_NOT_VERIFIED, not incorrect, and are excluded from the 117 denominator.

Names Baho/바호, Simowa/시모와 and Wano/와노 visibly match all six screenshots. Raw snapshots corroborate selected identities but contain no independent generated name strings; the name result is exact screenshot transcription, not an invented separate name oracle. No confusion pairs observed.

## Per-image review

### image_0001.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-crowd1-attempt1/crowd1_en.png`.

Text ledger:

| Claim | Reader value | Result |
|---|---|---|
| day | 30 | VERIFIED_CORRECT |
| hour | 10 | VERIFIED_CORRECT |
| season | Spring | VERIFIED_CORRECT |
| season_day | 2 | VERIFIED_CORRECT |
| population | 29 | VERIFIED_CORRECT |
| shelters | 4 | VERIFIED_CORRECT |
| food | 135 | VERIFIED_CORRECT |
| firewood | 12 | VERIFIED_CORRECT |
| cold | 0 | VERIFIED_CORRECT |
| births | 11 | VERIFIED_CORRECT |
| deaths | 7 | VERIFIED_CORRECT |
| agent_id | 1 | VERIFIED_CORRECT |
| life_stage | Elder | VERIFIED_CORRECT |
| age_days | 46 | VERIFIED_CORRECT |
| home_village | 1 | VERIFIED_CORRECT |
| position | [23, 18] | VERIFIED_CORRECT |
| activity | HearthCircle | VERIFIED_CORRECT |
| activity_target | [23, 18] | VERIFIED_CORRECT |
| carried_food | 0 | VERIFIED_CORRECT |
| under_construction | 0 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| berries | 30 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| daytime | True | SCREEN_MATCH_ONLY_NOT_VERIFIED |

Picture-only claims:

- Selected gray-haired person above central hearth/log, yellow ring visible, among several people: visually consistent with supplied PNG and selected sprite bounds; no independent gray-hair semantic field.
- Seated/resting/sitting by hearth is explicitly plausible; INFERENCE, not an exact HearthCircle identification.
- Exact hand action unresolved: reader limitation retained.

Artifact observations (visual observations, not additional simulation facts):

- No split name: consistent with inspected PNG.
- No raw key: consistent with inspected PNG.
- No percent artifact: consistent with inspected PNG.
- No joiner artifact: consistent with inspected PNG.

### image_0002.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-crowd1-attempt1/crowd1_ko.png`.

Text ledger:

| Claim | Reader value | Result |
|---|---|---|
| day | 30 | VERIFIED_CORRECT |
| hour | 10 | VERIFIED_CORRECT |
| season | Spring | VERIFIED_CORRECT |
| season_day | 2 | VERIFIED_CORRECT |
| population | 29 | VERIFIED_CORRECT |
| shelters | 4 | VERIFIED_CORRECT |
| food | 135 | VERIFIED_CORRECT |
| firewood | 12 | VERIFIED_CORRECT |
| cold | 0 | VERIFIED_CORRECT |
| births | 11 | VERIFIED_CORRECT |
| deaths | 7 | VERIFIED_CORRECT |
| agent_id | 1 | VERIFIED_CORRECT |
| life_stage | Elder | VERIFIED_CORRECT |
| age_days | 46 | VERIFIED_CORRECT |
| home_village | 1 | VERIFIED_CORRECT |
| position | [23, 18] | VERIFIED_CORRECT |
| activity | HearthCircle | VERIFIED_CORRECT |
| activity_target | [23, 18] | VERIFIED_CORRECT |
| carried_food | 0 | VERIFIED_CORRECT |
| under_construction | 0 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| berries | 30 | SCREEN_MATCH_ONLY_NOT_VERIFIED |

Picture-only claims:

- Ringed gray-haired person in central group, seated near hearth: visual observation consistent with PNG.
- No distinct crafting/carrying action visible: uncertainty/negative visual observation, not a claim the simulation cannot craft or carry.
- Hearth interpretation remains INFERENCE, not an exact HearthCircle identification.

Artifact observations (visual observations, not additional simulation facts):

- No artifact observed: consistent with inspected PNG.

### image_0003.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-crowd2-attempt1/crowd2_en.png`.

Text ledger:

| Claim | Reader value | Result |
|---|---|---|
| day | 60 | VERIFIED_CORRECT |
| hour | 10 | VERIFIED_CORRECT |
| season | Spring | VERIFIED_CORRECT |
| season_day | 4 | VERIFIED_CORRECT |
| population | 92 | VERIFIED_CORRECT |
| shelters | 14 | VERIFIED_CORRECT |
| food | 518 | VERIFIED_CORRECT |
| firewood | 79 | VERIFIED_CORRECT |
| cold | 0 | VERIFIED_CORRECT |
| births | 98 | VERIFIED_CORRECT |
| deaths | 33 | VERIFIED_CORRECT |
| agent_id | 29 | VERIFIED_CORRECT |
| life_stage | Elder | VERIFIED_CORRECT |
| age_days | 57 | VERIFIED_CORRECT |
| home_village | 1 | VERIFIED_CORRECT |
| position | [40, 35] | VERIFIED_CORRECT |
| activity | SitByFire | VERIFIED_CORRECT |
| activity_target | [40, 35] | VERIFIED_CORRECT |
| carried_food | 0 | VERIFIED_CORRECT |
| villages | 2 | VERIFIED_CORRECT |
| village1_map_count | 42 | VERIFIED_CORRECT |
| under_construction | 0 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| berries | 96 | SCREEN_MATCH_ONLY_NOT_VERIFIED |

Picture-only claims:

- Selected elder/dense central group/side-facing seated appearance: visually consistent; age and selected identity separately verified in text ledger.
- Large activity icons overlap people; specific icon association uncertain: visible overlap and reader limitation retained.
- Resting inferred from posture; exact fire interaction not independently clear: INFERENCE, not confident SitByFire recognition.

Artifact observations (visual observations, not additional simulation facts):

- No text artifact observed: consistent with inspected PNG.
- Substantial icon/crowd overlap: consistent with inspected PNG.

### image_0004.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-crowd2-attempt1/crowd2_ko.png`.

Text ledger:

| Claim | Reader value | Result |
|---|---|---|
| day | 60 | VERIFIED_CORRECT |
| hour | 10 | VERIFIED_CORRECT |
| season | Spring | VERIFIED_CORRECT |
| season_day | 4 | VERIFIED_CORRECT |
| population | 92 | VERIFIED_CORRECT |
| shelters | 14 | VERIFIED_CORRECT |
| food | 518 | VERIFIED_CORRECT |
| firewood | 79 | VERIFIED_CORRECT |
| cold | 0 | VERIFIED_CORRECT |
| births | 98 | VERIFIED_CORRECT |
| deaths | 33 | VERIFIED_CORRECT |
| agent_id | 29 | VERIFIED_CORRECT |
| life_stage | Elder | VERIFIED_CORRECT |
| age_days | 57 | VERIFIED_CORRECT |
| home_village | 1 | VERIFIED_CORRECT |
| position | [40, 35] | VERIFIED_CORRECT |
| activity | SitByFire | VERIFIED_CORRECT |
| activity_target | [40, 35] | VERIFIED_CORRECT |
| carried_food | 0 | VERIFIED_CORRECT |
| villages | 2 | VERIFIED_CORRECT |
| under_construction | 0 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| berries | 96 | SCREEN_MATCH_ONLY_NOT_VERIFIED |

Picture-only claims:

- Dense hearth group and side-facing gray-haired seated appearance: visual observations consistent with PNG.
- Selected map identity partly obscured by icons: reader association uncertainty, not a contradictory claim of geometric selected-body occlusion.
- Resting plausible, not clear isolated action: INFERENCE, not confident SitByFire recognition.

Artifact observations (visual observations, not additional simulation facts):

- No text artifact observed: consistent with inspected PNG.
- Icon overlap: consistent with inspected PNG.

### image_0005.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-crowd3-attempt1/crowd3_en.png`.

Text ledger:

| Claim | Reader value | Result |
|---|---|---|
| day | 30 | VERIFIED_CORRECT |
| hour | 15 | VERIFIED_CORRECT |
| season | Spring | VERIFIED_CORRECT |
| season_day | 2 | VERIFIED_CORRECT |
| population | 28 | VERIFIED_CORRECT |
| shelters | 4 | VERIFIED_CORRECT |
| food | 161 | VERIFIED_CORRECT |
| firewood | 46 | VERIFIED_CORRECT |
| cold | 0 | VERIFIED_CORRECT |
| births | 11 | VERIFIED_CORRECT |
| deaths | 8 | VERIFIED_CORRECT |
| agent_id | 30 | VERIFIED_CORRECT |
| life_stage | Adult | VERIFIED_CORRECT |
| age_days | 9 | VERIFIED_CORRECT |
| home_village | 1 | VERIFIED_CORRECT |
| position | [24, 21] | VERIFIED_CORRECT |
| activity | Craft | VERIFIED_CORRECT |
| activity_target | [24, 21] | VERIFIED_CORRECT |
| carried_food | 0 | VERIFIED_CORRECT |
| under_construction | 0 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| berries | 40 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| daytime | True | SCREEN_MATCH_ONLY_NOT_VERIFIED |

Picture-only claims:

- Hands-forward person/portrait and yellow hammer-like icons: visually consistent with PNG; selected-sprite association explicitly uncertain.
- Crafting/making: INFERENCE despite agreeing with independently verified Craft activity.
- Specific object unreadable: limitation retained; no specific-item claim to score.

Artifact observations (visual observations, not additional simulation facts):

- No split name: consistent with inspected PNG.
- No raw key: consistent with inspected PNG.
- No percent artifact: consistent with inspected PNG.
- No joiner artifact: consistent with inspected PNG.
- Icon crowding: consistent with inspected PNG.

### image_0006.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-crowd3-attempt1/crowd3_ko.png`.

Text ledger:

| Claim | Reader value | Result |
|---|---|---|
| day | 30 | VERIFIED_CORRECT |
| hour | 15 | VERIFIED_CORRECT |
| season | Spring | VERIFIED_CORRECT |
| season_day | 2 | VERIFIED_CORRECT |
| population | 28 | VERIFIED_CORRECT |
| shelters | 4 | VERIFIED_CORRECT |
| food | 161 | VERIFIED_CORRECT |
| firewood | 46 | VERIFIED_CORRECT |
| cold | 0 | VERIFIED_CORRECT |
| births | 11 | VERIFIED_CORRECT |
| deaths | 8 | VERIFIED_CORRECT |
| agent_id | 30 | VERIFIED_CORRECT |
| life_stage | Adult | VERIFIED_CORRECT |
| age_days | 9 | VERIFIED_CORRECT |
| home_village | 1 | VERIFIED_CORRECT |
| position | [24, 21] | VERIFIED_CORRECT |
| activity | Craft | VERIFIED_CORRECT |
| activity_target | [24, 21] | VERIFIED_CORRECT |
| carried_food | 0 | VERIFIED_CORRECT |
| under_construction | 0 | SCREEN_MATCH_ONLY_NOT_VERIFIED |
| berries | 40 | SCREEN_MATCH_ONLY_NOT_VERIFIED |

Picture-only claims:

- Hands-forward portrait and hammer-like icons: visually consistent observations.
- Crafting/making: INFERENCE; selected person/action not cleanly isolated from still image.
- Specific crafted item unreadable: limitation retained; no item claim scored.

Artifact observations (visual observations, not additional simulation facts):

- No text artifact observed: consistent with inspected PNG.
- Icon crowding: consistent with inspected PNG.

## Limit

This slice verifies crowd-screen factual transcription only. It does not establish picture-only identification of the three activities, prove zero perceived crowd clutter, or pass the separate stage-B activity test. No source files were modified, Godot launched, commits created or blind-reader messages sent.
