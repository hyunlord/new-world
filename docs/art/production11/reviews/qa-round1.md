# Part 5 independent captured-evidence QA — round 1

Verdict: **NEEDS WORK / bounded evidence review complete**. Review HEAD: `dad5c5e092163de32c1e2810122ea07921f5ed6b`. This is retrospective visual review of existing evidence, not a third blind recognition round and not fresh exact-HEAD runtime QA. No Godot, cargo, browser, gate, or captures were run. No source was edited. The visual-qa skill direct-image inspection principles were applied within the assigned read-only lane; no recursive reviewers or new captures were launched.

## Direct visual findings

- Names and live inspector: EN Ikimu / KO 이키무 #25 are visible with age, village, position, state and three need bars. Content has a coherent hierarchy and readable glyphs. KO life/records heading reaches the bottom of the initial viewport while EN takes more vertical room. These captures show visibility, not actual selection by a user.
- Deceased record: EN Nitiso #0 explicitly says Deceased, founder, day 23 and old age. Family and temperament remain, and the live portrait/need bars are absent. This is evidence for distinct record presentation, not proof the family button was physically clicked.
- EN family row truncates `Mother: Nitiso · Deceas…`; the death status is not fully readable, unlike the father row. Source fixes should preserve the full state through wrapping or wider layout in a later authorized change.
- Chronicle/person history contains underlined person and village names, giving visible link affordance. EN Chronicle entries and KO 연대기 are readable; the newest visible rows correspond to days 59/58/57. No raw localization keys were seen in the viewed regions.
- KO right-hand life events split names into isolated syllables (`이키`/`무`, `티`/`미가`, `마`/`비토`, `유`/`바마`, `시`/`모와`). This weakens name recognition. Left Chronicle also wraps parent names. Text is clipped at scroll viewport edges in both locales; these are real scroll screenshots, not evidence that omitted text is absent from data.
- EN world scene still labels the food/firewood stockpile in Korean (`식량 174 장작 20/39`). This is a concrete localization defect independent of the chronicle accuracy score.
- Activity Z4 frames visibly contain bright cyan, purple, yellow, blue and orange symbols above agents. Symbols are prominent but in the crowded seed 20261018 day60 10:00 village, bodies, shelters and nearby icons overlap heavily. The day30 15:00 frame likewise has multiple icons in one dense cluster. Correct per-agent association and recognition of every one of the seven kinds is not established by these screenshots.
- Z1 day60 14:00 shows two village labels with numeric activity summaries above the dense settlements. Counts and colored miniature icons are visible; the reviewed original-size village crops are extremely small. Existing blind recognition results are the appropriate acceptance evidence, not a new informed interpretation after keys were available.

## Functional evidence boundary

Read `scripts/test/art/test_inspector_life_p11.gd`: its FakeWorld test directly emits the dead-parent button signal, expands 12 to 15 life lines, invokes living-person link handling and Escape, and checks cache/reset behavior in EN/KO. Existing `p1-test-inspector_life_p11-attempt2.log` reports `INSPECTOR_LIFE_P11 checks=72 failures=0`. These are useful synthetic behavior checks, not real mouse hit testing or integrated world navigation.

Read `scripts/test/art/test_chronicle_panel_p11.gd`: FakeWorld verifies latest60 paging, once-per-tick reads, panel toggling, cursor progression and reset through direct calls. It does not prove actual keyboard C dispatch or person/village click navigation. The viewed screenshots prove underlining and current displayed states only. Real-input link navigation, scrolling all content, last12 expansion via a real click, Escape through actual input dispatch, reset/reselect against a running world and active simulation responsiveness remain **NOT_VERIFIED by this lane**. No physical interaction was performed.

## Provenance and logs

All 11 viewed external PNGs were byte-compared with their corresponding archived entries: identical. All 268 activity manifest payloads were rehashed: zero SHA256 mismatches. Life chronicle sidecar explicitly identifies capture HEAD `327239189218223c0564bd9389c0d7fa7339a935`, game_v24, debug profile, seed20261010, tick85080 and bridge SHA256 `aa6da070ff17861f8dc21d8f6203643dec3d8c44a8d94b90525c388eb89cdd8a`. It is not a capture of review HEAD.

Activity day60 Z4 and day60 14:00 Z1 sidecars identify the same debug bridge SHA with no release SHA. Their corresponding launch.json records exit0, empty validation_errors, capture-contract success and artifact hashes; launch logs identify Godot4.6/Metal M4 Max and renderer/camera initialization. Launch-time pose_oracle_checked=false is preserved; subsequent pose verification is separately reported in the final oracle output, not rewritten into launch records. Existing life-attempt2.log likewise records renderer/camera initialization, not a click trace.

A live filesystem hash comparison against the life chronicle source map found changes in capture_identity_p11.gd, capture_running_identity_p11.gd, capture_framings_p11.gd, agent_activity.gd, agent_frame_choices_p10.gd, settlement_labels.gd and the debug bridge file. Other present listed sources matched. Thus existing life UI evidence must retain original source provenance; it does not establish all current rendering behavior or the currently built bridge. This comparison is not a new build or run.

## Gates retained without reinterpretation

- **H7 NOT_VERIFIED**: only12/14 eligible C crops, counts[1,2,2,2,1,2,2], missing one TendFire and one Explore. Original >=10/14 and no kind0/2 remain. Diagnostic rounds are0/12 and1/12, both recognition failures. Several fully sampled kinds stay0/2. D is2/6 FAIL then5/6 PASS. D second-round success does not complete C. The two allowed rounds remain exhausted; this review produced no new guesses, crops or scores.
- **H8 FAIL**: archived independent non-inference review is44 PASS /1 FAIL /0 NOT_VERIFIED among45 statements. P3-S07 wrote 오후디 where subject54 is 오후티. Preserve this reviewer transcription failure; do not mislabel it as simulation corruption or erase it through informed rereading. Captured KO screen visibly says 오후티, consistent with the retained comparison.
- The five departure causes remain N/A in the recorded real interval with no Departed events. No synthetic fixture establishes their actual screen appearance.
- Debug captures, synthetic tests and pose checks do not prove release performance, fresh exact-HEAD interaction, seven-kind blind recognition or complete visual acceptance.

## Images directly opened with view_image

Paths are relative to `/Users/rexxa/work/art-production11-evidence`. Hashes below identify the actual files opened; archive identity was also checked.

| Image | SHA256 | Archive byte match |
|---|---|---|
| `life-attempt2/living_dead_parent_en_top.png` | `e6941b5c98e025f319dc9369a64a4bbf827b1605d48219a9ccfc5c21af809409` | yes |
| `life-attempt2/living_dead_parent_ko_top.png` | `a59e11f15f575f137792740ecd32de7c8a5ea33b57c3e83e2fce645c2da2c26a` | yes |
| `life-attempt2/dead_parent_en_top.png` | `d1b7778e2791380fecfc948e4406c49965d1100646d24d537d86938aa1e76b22` | yes |
| `life-attempt2/chronicle_en.png` | `2167d53e114e5e5a57615ad334ca5166dcf4a31e1e22ff1962dfb6e51c88a304` | yes |
| `life-attempt2/chronicle_ko.png` | `265d0d51260fb0e52195870ac95ccdd0bd95fce73355293d5f5cabfffd165713` | yes |
| `life-attempt2/living_dead_parent_en_scroll_556.png` | `39f84f4f85493cbe24e45110cc0e2ca97eb8f2d3b4c72dd8655078186c36c129` | yes |
| `activity-attempt3/s20261018_d60_1000_z4/capture.png` | `5505ed3bcf2a8aeaac4f45f91dec4f1df01b8f514426dee4d758fda13d647a7b` | yes |
| `activity-village-extras-round1/s20261018_d60_1400_z1/capture.png` | `9d36da89def726bdf4eff3fe7ccf10bb5909eb9fb8e9185ccb1bd4e9d8cde40b` | yes |
| `activity-attempt3/s20261010_d30_1500_z4/capture.png` | `67541502a25e8e3e2446e7c2042cedf01214943553fde2eaaf77a068b1b35a89` | yes |
| `activity-village-blind-round2/blind/village_01.png` | `be13138f8241c2e69e1033d49dbcda212e30f45c5f1d1eb2d8334df7f19c8733` | yes |
| `activity-village-blind-round2/blind/village_02.png` | `326a5fa76071f5255cdd82031c1aec3c61e1348bbc02426700b2615539f0029c` | yes |
