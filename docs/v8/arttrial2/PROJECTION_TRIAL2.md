# ART-TRIAL2 PROJECTION — frozen production evidence

Final snapshot **2026-10-22T01:32:33.796440+00:00**, 79 measured production calls and 116 review calls plus one negative control, as detailed in [COST.md](COST.md). **Successful v1.0 production generations, hours and expected judge rejects: NOT_ESTIMABLE.** Two ground assets are now accepted; every other environment family still has zero accepted examples. The unchanged first-pass evidence is preserved in `/Users/rexxa/work/arttrial2-data/accounting-firstpass.json`; updated measurements are in `accounting-final.json` alongside it.

## Prescribed inventory and fixed attempted workload

GOAL8 prescribes **344 asset units**. For unconditional attempted-workload accounting only, allocate one candidate per unit, three material calls per autotile set, and one separate call per person part/overlay. This defines **364 calls**, not a forecast for accepting 344 assets. Every failed attempt still consumes its call. Multiplying those fixed calls by observed family minimum/maximum brackets produces arithmetic scenarios, not confidence intervals, throughput estimates or finite successful-production bounds.

| v1.0 family | Asset units | Fixed candidate call budget | Observed-bracket arithmetic, h | Measurement basis |
|---|---:|---:|---:|---|
| Ground | 10 | 10 | 0.075–0.205 | 12 calls; 27.075–73.761s/call |
| Trees / plants | 30 | 30 | 0.181–0.478 | 18 calls; 21.747–57.397s/call |
| Buildings | 20 | 20 | 0.113–1.287 | 12 calls; 20.417–231.612s/call |
| Walls / floors / fences | 10 | 30 | 0.174–0.437 | 6 calls; 20.828–52.439s/call |
| Items | 60 | 60 | 0.438–3.799 | 10 calls; 26.279–227.948s/call |
| Animals | 24 | 24 | NOT_MEASURED | No corresponding production family measured |
| People parts / overlays | 70 | 70 | 0.345–0.960 | 12 calls; 17.760–49.375s/call |
| UI icons | 80 | 80 | NOT_MEASURED | No corresponding production family measured |
| Portrait pool | 40 | 40 | NOT_MEASURED | No corresponding production family measured |

The six measured-category rows allocate **220 calls**, yielding **1.327–7.166 hours of summed observed brackets** for this hypothetical attempt budget. The other **144 calls** (24 animals, 80 icons, 40 portraits) lack corresponding production measurements, so even whole-inventory attempted time is **NOT_MEASURED**. Do not substitute portrait-anchor review times or environment means. Shared concurrent brackets include queue/observation time and are not isolated image-model service time.

The actual people batch used one 14-part sheet plus 11 overlays/equipment: 12 calls exposing 25 parts. It does not establish that all 70 future parts fit sheets or can be accepted. The table deliberately defines a separate-call workload. Similarly, three autotile source calls define one candidate set, without guaranteeing seamlessness or acceptance. Reviews, retries, processing, rig work, runtime verification and integration are outside this fixed candidate workload.

## Full-production yield remains unavailable

First snapshot: environment 39 calls, 38 generated assets, numeric eligibility 1/38, combined accepted 0. Frozen production: 59 environment calls for the same 38 generated assets, latest numeric eligibility 2/38, combined accepted now 2 after confirmed numeric/model passes (2/38 generated = 5.26%; 2/40 including derived ghosts = 5%). All-production calls grew from 56 to 79. All 196 actual image-tool calls remain charged to the measurement count regardless of output usability; currency amounts were not exposed. The first calibration failed. V2 combined calibration passed, but its model rejected 0/3 negatives; all negative rejections came from numeric checks. Later evidence does not rewrite the first-pass result.

The requested successful projection requires measured tries and minutes **per accepted asset**. Ground alone has a measured 12/2 = 6 calls per accepted asset and 515.650/2 = 257.825 summed bracket seconds per accepted asset. Applying those two samples to 10 ground units gives an arithmetic point scenario of 60 calls and 0.716 summed-bracket hours, not a predictive interval or guarantee. Across all environment spending the observed ratio is 59/2 = 29.5 calls per accepted asset. Every other required family lacks accepted samples, so full successful-generation range, successful-hour range and expected judge rejects are **NOT_ESTIMABLE**; currency is **NOT_EXPOSED**. The retry cap limits spending before stopping, not the number of tries that guarantees success. There is no finite successful-production bound from these data. Accepted examples in each relevant family and actual workflow timings would be necessary for a later defensible range.
