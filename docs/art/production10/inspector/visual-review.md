# Actual-capture visual review

Scope: read-only native `view_image` inspection of supplied actual 1920×1080 captures. This is a visual review, not a technical gate, runtime validation, animation test, blind identification score, or source audit. No blind crops or answer key were opened. No code or runtime was changed. Household rings were intentionally dropped and are not a mandatory failure.

Verdict convention: UI PASS means the visible text and panel layout have no observed clipping, tofu, raw localization keys, or overlapping text. Activity FAIL means the screenshot alone does not make individual activities consistently distinguishable; this does not invalidate technical rendering or data correctness.

| Actual capture | UI verdict | Activity readability | Evidence and limits |
|---|---|---|---|
| activities-part2-attempt2/s20261010_d30_1000_z1/capture.png | PASS | FAIL at overview scale | Korean HUD, village label, news heading, and intervention controls are readable. No text overflow/tofu/raw keys observed. Characters and props in the center are very small and overlap; specific activities cannot be consistently identified from this full frame. Gray off-map space occupies much of upper-left view. |
| activities-part2-attempt2/s20261010_d30_1000_z4/capture.png | PASS | LIMITED | Large sprites reveal standing and seated silhouettes. Dense center crowding overlaps tents, people, and hearth; individual task identities remain ambiguous. Village and stock labels are much smaller than HUD text, but readable on inspection. No visible panel text defect. |
| activities-part2-attempt2/s20261010_d30_1500_z1/capture.png | PASS | FAIL at overview scale | HUD values and Korean labels are legible without clipping. Overview scale and central crowding prevent reliable per-agent activity distinction. No temperament/family panel is open. |
| activities-part2-attempt2/s20261010_d30_1500_z4/capture.png | PASS | LIMITED | Standing/seated/horizontal poses and carried-looking forms are more visible than at z1. Center remains crowded with overlapping sprites and shelters; the horizontal silhouette does not establish a specific task or animation from one still. Stock label is small but legible. No tofu/raw key/visible clipped text. |

Day-60 captures and six EN/KO parent-panel captures: pending delivery at initial review. Four temperament bars and family rows cannot yet be evaluated from the activity images, because those panels are not open.

## Day-60 follow-up

| Actual capture | UI verdict | Activity readability | Evidence and limits |
|---|---|---|---|
| activities-part2-attempt2/s20261018_d60_1000_z4/capture.png | PASS | LIMITED; crowded center FAIL for consistent individual identification | Korean top HUD (including village count), news heading and intervention text remain readable without observed tofu/raw keys/clipped text. Stock text is legible but runs across a busy sprite cluster. The central settlement is substantially crowded: bodies, shelters and hearth overlap. Distinct poses exist, especially among isolated foreground figures, but the frame does not consistently reveal each person's activity. |
| activities-part2-attempt2/s20261018_d60_1000_z1/capture.png | PASS | FAIL at overview scale | Both settlements are visible and Korean settlement labels remain readable. HUD and intervention panel have no observed clipping/tofu/raw keys. Both settlement centers are tightly stacked with agents and shelters; tiny sprites preclude reliable per-agent activity identification. The second settlement is near the lower-right map boundary but its visible label is inside the image. |

All six activity images have now been inspected. Static UI legibility passes in this set; consistent individual activity recognition across these full frames does not. This is deliberately not a blind-test score.

## Final parent-panel EN/KO review

All six PNGs below were opened separately with native `view_image`. The actual screenshot was primary evidence; `parent.json` was then checked for matching selected-agent family data, temperament values and label rectangles. The 620px inspector is a scrolling viewport. Content beyond the viewport in a top-position capture is not claimed visible merely because the sidecar says `visible_in_tree`.

| Actual capture (under parent-part3-en-ko-attempt1/) | Verdict | Visible evidence and limits |
|---|---|---|
| parent-en-top.png | PASS for top inspector | Villager 5, Elder, age 53 days, Village 1, position, state, target, need labels and carrying amount are readable. “State: Resting by the fire” wraps across two lines naturally without overlap. Bottom begins the next section at the scrolling boundary; this frame alone does not show temperament/family. |
| parent-en-temperament.png | PASS | All four labeled bars (Curiosity, Caution, Warmth, Persistence) are simultaneously visible and separated. Family rows also fit: Female; Spouse: Villager 24; Mother: None; Father: None; Children: 1; Villager 30. No raw keys/tofu/overlap or horizontal clipping. Missing-parent rows use dim gray and have weaker contrast but are readable. |
| parent-en-family.png | PASS | Family values and final child button are fully inside the panel; four temperament bars remain visible above. This is the same scroll position as the temperament capture, so it is not independent coverage of a different scroll extent. |
| parent-ko-top.png | PASS for top inspector | 주민 5, 노인 · 나이 53일, 1번 마을, 위치, 상태, 목표, 허기/갈증/수면 and 식량 0개 운반 중 are readable. 기질 heading is at the lower edge; its rows are below this scroll view. No tofu/raw keys or clipped main-field text. |
| parent-ko-temperament.png | PASS | All four bars and labels 호기심 / 신중함 / 정 / 끈기 are visible. 가족, 여, 배우자: 주민 24, 모: 없음, 부: 없음, 자녀: 1명, 주민 30 all fit without line splitting/overlap/clipping. Missing-parent gray rows remain lower contrast than active rows. |
| parent-ko-family.png | PASS | The same complete family block and four bars are visible. Last child row is fully inside the panel. Same scroll position as KO temperament view, not a distinct viewport. No visible tofu/raw keys. |

### Values and geometry cross-check

- Selected agent is 5. `parent.json` records female=true, partner=24, mother=-1, father=-1, children=[30], matching the displayed family rows in both languages.
- Four bar fills visibly differ in the expected order: Warmth/정 longest, Curiosity/호기심 next, Caution/신중함 next, Persistence/끈기 shortest. Sidecar values are rd=736, ns=539, ha=340, p=222. Approximate fill lengths are consistent with about 74%, 54%, 34%, 22%.
- Exact numeric temperament values are **not printed in the UI**. Only relative bar amounts are visually readable; the exact values above are sidecar evidence, not screenshot-readable numbers.
- EN scroll offset is 503; KO is 472. In both lower views the four labels start at y=262/314/366/418 and the final child button is y=700..736, within the panel ending near y=744. Family rows fit together in the lower view.
- Static images cannot establish pointer hit targets, wheel behavior, relationship-click outcomes, dynamic updates, or animation. Any sidecar interaction records are separate evidence from this visual pass.

## Visual outcome

The twelve requested actual captures are reviewed. EN/KO inspector layout and four temperament bars/family rows pass this static visual inspection. No visible tofu or raw localization keys were found. Remaining visual-quality limits are dense settlement sprite overlap, tiny overview-scale activity detail, small world-space labels, and reduced contrast on disabled parent rows. Exact temperament numerals are absent from the display. No overall technical gate or blind recognition result is assigned here.
