# Blind read evidence

## Round 1: **3/10 FAIL** (threshold 8/10)

Fresh reviewer `/root/blind_scene_first`, no inherited conversation. Only ten numbered 2× nearest-neighbour scene crops were provided. Complete answers were frozen before scoring. Historical M1-ART baseline: **7/10**, supplied by the trial contract and not rerun here.

| # | Target | Verbatim answer | Point | Reason |
|---|---|---|---:|---|
| 1 | hut | A square timber enclosure with stacked pale bundles along its sides and a central stone ring; a pale shape extends left from the ring. Its identity is uncertain. | 0 | Timber enclosure identified, but no dwelling/hut identification; uncertainty retained. |
| 2 | lean_to | A yellow hide or cloth stretched over a wooden frame. | 1 | Stretched hide on wooden frame describes the lean-to shelter structure; accepted descriptive synonym. |
| 3 | hearth | A burning campfire surrounded by stones. | 1 | Campfire surrounded by stones directly identifies hearth. |
| 4 | stockpile_logs | A brown gridded field with clustered yellow plants or bundles and two people. The central person appears to hold a tool across their shoulders; the activity is uncertain. | 0 | Gridded field and yellow plants/bundles do not identify log stockpile. |
| 5 | bare_logs | Three upright yellow bundles, possibly harvested grain. | 0 | Grain interpretation does not identify round logs. |
| 6 | carry | A pale-clothed person holding a long implement across or above their shoulders. The implement and activity are uncertain. | 0 | Shoulder implement without log/load identification does not identify log carrying. |
| 7 | chop | A red-clothed person raising a tool beside a large round target or log cross-section. Possibly preparing to strike it. | 1 | Raised tool beside log with preparation to strike identifies chopping action. |
| 8 | child | A purple-clothed person standing with a diagonal pale object across their body; activity uncertain. | 0 | Generic person, no child identification. |
| 9 | elder | A pink-clothed person with a pale diagonal object and an upright dark object beside their head; activity uncertain. | 0 | Generic person, no elder identification. |
| 10 | sleep | A gray stone ring containing dark brown material, with a pale elongated object extending left. Possibly a hearth, but the object and activity are uncertain. | 0 | Hearth/elongated object, no sleeping person identification. |

The lean-to answer receives one descriptive-synonym point for stretched hide on a wooden frame. Under a stricter requirement for the word shelter, the score would be 2/10; neither interpretation passes. Generic people do not earn age or activity points.

Frozen first-round images, hashes, answers and scoring: `/Users/rexxa/work/arttrial-data/people/boards/round1_frozen/`. The one permitted revision round is complete; Round2 and its separate fresh-reviewer score are recorded below.

## Variation: **10/24 FAIL** (threshold20/24)

A separate fresh reviewer `/root/blind_variation` received exactly the six preregistered3× sprites without labels. The sample was rendered directly from frozen board inputs before the remaining unrelated board keys were available; this does not establish completeness or uniqueness of all24 people.

| Anonymous # | Board index | Verbatim answer | Sex / age / carry / tool points |
|---|---:|---|---|
| 1 | 17 | 1. Sex: uncertain; age group: adult; carrying an item: no; visible tool: none. | [0, 0, 1, 1] |
| 2 | 12 | 2. Sex: uncertain; age group: adult; carrying an item: no; visible tool: none. | [0, 0, 1, 1] |
| 3 | 18 | 3. Sex: uncertain; age group: adult; carrying an item: uncertain; visible tool: uncertain. | [0, 0, 0, 0] |
| 4 | 0 | 4. Sex: uncertain; age group: adult; carrying an item: yes; visible tool: axe. | [0, 1, 1, 1] |
| 5 | 22 | 5. Sex: uncertain; age group: elder; carrying an item: uncertain; visible tool: uncertain. | [0, 1, 0, 0] |
| 6 | 19 | 6. Sex: uncertain; age group: child; carrying an item: no; visible tool: none. | [0, 0, 1, 1] |

All six sex answers were uncertain. Child/elder bodies share sexes by design; no field was excluded or resampled. The woodcutter sample correctly communicated adult, carrying and axe. Frozen input pixels, provenance and verbatim response: `/Users/rexxa/work/arttrial-data/people/boards/variation_first_frozen/`.

## Round2: **3/10 FAIL** (threshold8/10)

Fresh reviewer `/root/blind_scene_second`, no inherited conversation; same neutral instruction, same registered scene positions and2× crop bounds. Alternate logs, shoulder-log layer and matching sleeper/child/elder silhouettes were the only scene-asset changes. This exhausts the one permitted revision round.

| # | Target | Verbatim answer | Point | Reason |
|---|---|---|---:|---|
| 1 | hut | A wooden enclosure with pale bundles along its sides and a central stone-ringed dark object. Its purpose is uncertain. | 0 | Enclosure identified, but dwelling purpose explicitly uncertain. |
| 2 | lean_to | A yellow animal hide stretched on a wooden drying frame. | 0 | Explicit drying-frame interpretation does not identify a shelter. |
| 3 | hearth | A burning campfire surrounded by stones. | 1 | Direct campfire/stone-ring identification. |
| 4 | stockpile_logs | A blue-clothed person carrying a pole across their shoulders, with bundles or containers at either end. | 0 | Describes carrier but not stockpile or stored logs. |
| 5 | bare_logs | A horizontal log with a pale split or cut end; no visible activity. | 1 | Horizontal log and cut end directly identify timber. |
| 6 | carry | A pale-clothed person carrying a shoulder pole with loads at both ends. | 0 | Carrying is recognized, but shoulder pole/loads do not identify logs; strict target point withheld. |
| 7 | chop | A red-clothed person raising an axe beside a large round log cross-section, apparently preparing to chop. | 1 | Axe raised by a log, preparing to chop directly identifies action. |
| 8 | child | A purple-clothed person seen from behind; no clear activity. | 0 | Generic person, no child identification. |
| 9 | elder | A brown-clothed figure leaning forward with something near their hands; the object and activity are uncertain. | 0 | Lean recognized, but no elder identification. |
| 10 | sleep | A wooden enclosure with pale bundles and a central stone ring containing a dark object; no definite activity visible. | 0 | Enclosure/hearth, no sleeping person identification. |

The first reviewer described stretched hide on a frame without an alternate purpose, accepted descriptively as shelter. The second explicitly identified a drying frame, so it is a miss. Number6 recognizes carrying but not logs: awarding a generic-carrying point would give4/10, still below8/10. No further revision or reviewer reroll is performed.

Frozen sources, hashes, responses and scores: `/Users/rexxa/work/arttrial-data/people/revision_scene/`. Both rounds score below the supplied historical7/10; neither proves the generation method improves readability.
