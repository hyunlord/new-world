# TRIAL2b crowd count

Fresh image-only reviewer: **60 people**; rendered body-bound intersection truth: **60**; absolute relative error **0%**, within the preregistered ±10% bar. This establishes countability only, not activity recognition.

Native640×360 crop `(64,128,704,488)` from the actual200-person1× window capture; no scaling or image modification. Count partial people, sleepers and children; exclude props. The crop and rule were fixed before the reviewer received the image. Ground-truth bodies use the captured rig bounds; this is a geometry count, with visual occlusion a possible limitation.

Raw evidence: `/Users/rexxa/work/arttrial2-data/trial2b/crowd-truth.json`, `crowd-blind-answer.json`, `crowd-blind-crop.png`, and `captures-round2/camp-200-1/runtime.json`. The reviewer was not given population totals, metadata, source, prior results or the answer key.

Crop SHA256: `3aa4da4853207c137ea389be4b4a32b7e063acd13d9eeb996cbc77854045bfe0`.

Verbatim answer:

```json
{
  "count": 60,
  "confidence": "high",
  "reasoning": "Counted visible human figures directly across six horizontal bands, recording 10 figures in each band. Included horizontal sleeping figures, smaller children, and figures clipped by the bottom edge. Excluded the vertical circular props and handheld items. Count is based only on the supplied 640x360 image."
}
```
