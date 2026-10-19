# Part 5 final visual evidence audit — round 2

Reviewed candidate (Part4 freeze pending at review time): `1dc641b5cd161cf9683349f367194935e8213b3f`.
Verdict: **PASS for six-framing pixel identity; overall visual acceptance remains NEEDS WORK**. This is the second and final evidence-review round, not an additional blind recognition round. No runtime, build, browser, source edits or agents were launched.

## Independently checked identity

Opened all six actual candidate PNGs with view_image, under `p3-runtime/candidate1/B/P1.png` through `P6.png`. Independently decoded full-resolution A, A′ and B using Pillow RGBA, computed absolute signed channel differences with NumPy, counted pixels with any changed channel, and hashed full RGBA byte buffers. No masking, resizing, threshold or alpha omission. Compared complete sidecar draw_clock objects independently as well.

All six images are 1920x1080. All six A/A′ comparisons and all six A/B comparisons have **0 differing pixels / maximum channel difference0**. All three full draw_clock objects match for every framing. P5 independently satisfies its stricter exact-zero requirement. Results agree with the official `candidate1-comparison/pixels/comparison.json` and `draw-clock.log`.

Detailed independent machine evidence: `qa-round2-pixels.json`, including all 18 paths, PNG hashes, RGBA hashes, dimensions, metrics, and seed/tick/population. This verifies decoded pixels, not merely matching PNG containers.

## Actual scenes inspected

| Framing | Visible evidence | Scene assessment |
|---|---|---|
| P1 | Day29 12:00 spring, population29, shelters4, crowded settlement with activity icons | Valid rendered daytime scene; overlapping bodies/icons remain |
| P2 | Day23 02:00 winter night, population29, lit central fire and dark terrain | Valid night scene; not blank/failed rendering |
| P3 | Day33 09:49, population37, two villages, overview, news of13 departures and second village | Valid society overview with village labels and news |
| P4 | Day89 12:34, population66, two villages with30/36 members and activity-count labels | Valid later overview with dense villages |
| P5 | Day2 08:00, population500, shelters11, many tightly overlapping bodies | Valid stress framing; crowding is visible, not hidden by comparison masks |
| P6 | Day29 12:00, named inspector 바호 #1, age45, village, position, needs, carrying4 food | Valid inspector overlay; lower life/family content is outside this initial viewport |

Six candidate launch rows show exit0, artifacts present, log_errors none. Candidate sidecars are explicitly debug_build=true. They record loaded bridge SHA `0f7e3371662d12f8a0cf4e7ab0b57312afe50edbf89c89ec552267a621ac4afc`; A/A′ record `2bcac5929643387e37e3bb724ffc7e447d47e44508f2f1794b2e7e561240889d`. These are debug rendering checks, not release performance captures.

## H5 coverage and provenance limits

The existing running comparison reports PASS with no errors for W0/W200 (600 frames, manual advance every2 frames), W500_FROZEN (240 frames, no tick advance), and W500_2X (240 frames, advance every frame), using SimReads mutation routing and fixed_fps60. This report was inspected, not rerun. These deterministic manual schedules check sampled state/render identity; elapsed capture times and synthetic frame schedules do not measure real-time performance or prove production timing under natural scheduling.

The six-frame base provenance is explicitly supplemental and recorded during A at `2026-10-16T13:04:53.816974+00:00`, source HEAD `f0d04f8e5bb698385b4751f0e0ea9f401ebef5ac`. It is not per-launch before/after evidence for the early A images. `reference-validation.md` correctly distinguishes it from running-A per-launch snapshots. A/B sidecars do not themselves embed full source HEAD provenance; B's generic P3 manifest git_sha is empty. Candidate commit attribution relies on surrounding frozen-run records, and this lane does not silently upgrade those fields to embedded proof. Exact image equality does not erase this limit.

H5 identity evidence covers these six captured states and four scheduled running cases. It does not prove all cameras, all simulation outcomes, natural long-run behavior, release performance, user input dispatch or every life/chronicle scroll state.

## Prior visual findings and gates remain

Read-only git comparison `f0d04f8e..1dc641b5 -- scripts/ui localization` shows only `scripts/ui/agent_renderer.gd` and `scripts/ui/camp_seats.gd` changed in that scope. Names, life, records, chronicle/localization and activity summary UI are unchanged in this scope. No new UI fixes were presumed.

Round1 limitations remain: English stockpile labels in Korean; deceased-mother status ellipsis; Korean names split into isolated syllables; scroll-edge clipping; crowded icon/body association; tiny village summary symbols. The six identical frames demonstrate no new pixel difference in sampled states, not correction of these pre-existing defects. P6 only shows the top of a living inspector, so it does not refresh full EN/KO record/chronicle evidence.

Actual mouse/keyboard selection, links, scrolling, expansion and Escape remain NOT_VERIFIED by this audit. FakeWorld signal/function checks cannot replace that real-input evidence.

**H7 remains NOT_VERIFIED**: C12/14 eligible, diagnostic0/12 then1/12, missing TendFire/Explore samples and fully sampled kinds at0/2; D2/6 FAIL then5/6 PASS. No new blind guesses, scoring or replacement answers were produced.

**H8 remains FAIL**:44/45 independent non-inference statements correct, retained P3-S07 오후디 versus 오후티 transcription error. The prior five-cause departure N/A finding belongs to its recorded interval and is not replaced by P3's distinct society scene.

## Directly viewed B file hashes

| Image | PNG SHA256 | RGBA SHA256 |
|---|---|---|
| P1 | `f778623c1fff62578cdd1db096e75cdac9b39d3f1398b1efa85558d781e84e79` | `076d22bf7587b6901f51d4b2b9960b3ab62a3995cc72f8ca317965079dd3ec2e` |
| P2 | `8c87670976cd58d47a725a10bf0af86889893d32a716d5dd8c2bc0dfb660edf6` | `a480eddfd9a7560fce67f637a411da737faa3dc43c999871bf7c40f84035a54e` |
| P3 | `7dc0a43018042a978a9504261cdb5db494ca13845c815bf778f7440039c7b5ce` | `9c340775f950ec2b521735a3cd83d8f9960601f39d07b54a1b23c2a8de7f382e` |
| P4 | `d553274fc114d58a93482006d0a99d7fededb71efc701885da4579cda6ccf0a2` | `4410428535a6dfe90745db6b3b629d3c4cd29b7afd31ae4aa9e28d814259bbc4` |
| P5 | `7f99ecb1303926e3c40c560630427484156cb829ea6a84c629970ca467cb466a` | `439dc8420d24e72b763fc0048cb24909b6328bceb15aa2dc1482882408958a38` |
| P6 | `8dbb09f83a97cda0f113968f3689483fae6d55b81e6a859a1b0420330f626463` | `6618e40a8bd9fdac9db881e45982fed018140639213734f9768f60d688593ddb` |
