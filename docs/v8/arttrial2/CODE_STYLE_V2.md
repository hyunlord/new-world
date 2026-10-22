# Code companion runtime review V2

**Own visual verdict: the four examples are legible on the calmer ground, but do not yet belong comfortably beside the generated ink-and-wash art. Fixes remain listed; no new style-judge pass is claimed.**

Inspected actual round-two `camp-24-1/capture.png` and `camp-200-1/capture.png` under `/Users/rexxa/work/arttrial2-data/trial2b/captures-round2/`. In the clear lower-right region of the 24-person image, the dashed stockpile square, blueprint frame, selection rectangle and thin progress bar are all visible. The 200-person shot has actors crossing these examples and is less suitable for style comparison.

Also inspected `/Users/rexxa/work/arttrial2-data/proto-code-companion-crop.png` and its JSON provenance. This is a **historical** crop from `proto-perf-24-v2/capture.png`, not round two; its dense grass cannot represent current ground. Current appearance is assessed from the actual round-two images above. Their hashes are recorded in `RUNTIME_V2.md`.

| Runtime finding | Concrete fix |
|---|---|
| Long edges and regularly spaced dashes look like clean UI geometry beside irregular generated contours. The small midpoint jitter is barely perceptible at 1×. | Preserve the bounded ±0.5 px jitter requirement while distributing it across line segments; rejudge beside generated props at native scale. |
| The blueprint is an empty schematic frame, visibly less descriptive than the detailed hut. | Add sparse interior plan marks consistent with the blueprint role; retain restrained ink and wash. |
| Current source uses 2.0 px lines and a wash alpha of 0.25, matching active `TRIAL2B_GOAL.md` §4.3. | No parameter correction required; judge the visual fit independently. |

`art/proto/Companion.gd` matches the run-recorded SHA `4381fd75fc2ed80f2d28069d0f57f1988f2924e58390752d98f3cbc6debe752c`. This review made no code or asset edits. The older 1.5 px / 15% specification is withdrawn and is not an active violation. A listed fix supports the goal's C8 reporting alternative; it is not equivalent to a judge pass.

Final all-walking `camp-walk-24-1` and `camp-walk-200-1` screenshots were also directly inspected. Source/hash parity holds, and the companion appearance and verdict are unchanged.
