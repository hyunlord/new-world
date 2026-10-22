# Independent blind readability review 2

Only numbered GIFs were inspected. No answer key, route identities, source assets, or other reviewer output was read. No scoring against a key was performed.

**Method limitation:** All decoded frames were viewed in temporal order at native 1x/2x pixel dimensions with duration labels. Actual timed playback was **not observed**. No Godot launched. Every eight-frame GIF alternates 120/130 ms for a 1000 ms cycle; 05 and 09 contain one 1000 ms frame at each scale. Confidence is subjective, per field, from 0 to 1.

| GIF | Activity | Object | Top | Hair | Confidence: activity/object/top/hair |
|---|---|---|---|---|---|
| 01_1x.gif | idle | none | furcloak | longtied | 0.87 / 0.87 / 0.8 / 0.78 |
| 01_2x.gif | idle | none | furcloak | longtied | 0.94 / 0.94 / 0.87 / 0.85 |
| 02_1x.gif | walking | none | furcloak | short | 0.86 / 0.9 / 0.83 / 0.85 |
| 02_2x.gif | walking | none | furcloak | short | 0.93 / 0.97 / 0.9 / 0.92 |
| 03_1x.gif | idle | axe | hidewrap | longtied | 0.46 / 0.72 / 0.78 / 0.82 |
| 03_2x.gif | idle | axe | hidewrap | longtied | 0.53 / 0.79 / 0.85 / 0.89 |
| 04_1x.gif | carrying | log | hidewrap | short | 0.91 / 0.92 / 0.67 / 0.78 |
| 04_2x.gif | carrying | log | hidewrap | short | 0.98 / 0.99 / 0.74 / 0.85 |
| 05_1x.gif | sleeping | none | none | bald | 0.92 / 0.91 / 0.59 / 0.83 |
| 05_2x.gif | sleeping | none | none | bald | 0.99 / 0.98 / 0.66 / 0.9 |
| 06_1x.gif | idle | none | hidewrap | longtied | 0.87 / 0.88 / 0.85 / 0.72 |
| 06_2x.gif | idle | none | hidewrap | longtied | 0.94 / 0.95 / 0.92 / 0.79 |
| 07_1x.gif | carrying | log | hidewrap | unknown | 0.91 / 0.92 / 0.68 / 0.28 |
| 07_2x.gif | carrying | log | hidewrap | unknown | 0.98 / 0.99 / 0.75 / 0.35 |
| 08_1x.gif | chopping | axe | hidewrap | longtied | 0.48 / 0.76 / 0.8 / 0.78 |
| 08_2x.gif | chopping | axe | hidewrap | longtied | 0.55 / 0.83 / 0.87 / 0.85 |
| 09_1x.gif | sleeping | none | none | bald | 0.92 / 0.91 / 0.59 / 0.83 |
| 09_2x.gif | sleeping | none | none | bald | 0.99 / 0.98 / 0.66 / 0.9 |
| 10_1x.gif | idle | none | furcloak | short | 0.89 / 0.9 / 0.83 / 0.86 |
| 10_2x.gif | idle | none | furcloak | short | 0.96 / 0.97 / 0.9 / 0.93 |
| 11_1x.gif | walking | none | hidewrap | short | 0.88 / 0.9 / 0.84 / 0.86 |
| 11_2x.gif | walking | none | hidewrap | short | 0.95 / 0.97 / 0.91 / 0.93 |
| 12_1x.gif | carrying | log | furcloak | unknown | 0.91 / 0.92 / 0.72 / 0.28 |
| 12_2x.gif | carrying | log | furcloak | unknown | 0.98 / 0.99 / 0.79 / 0.35 |

## Per-GIF observations and limits

- **01_1x.gif**: Feet retain their pose; slight body bob. Shaggy layered top and side ponytail visible. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **01_2x.gif**: Feet retain their pose; slight body bob. Shaggy layered top and side ponytail visible. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **02_1x.gif**: Legs alternate extended positions; shaggy top and compact hair silhouette. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **02_2x.gif**: Legs alternate extended positions; shaggy top and compact hair silhouette. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **03_1x.gif**: Upright tool remains near shoulder with small changes; no clear chopping arc or follow-through. Primary activity read is standing idle holding an axe. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **03_2x.gif**: Upright tool remains near shoulder with small changes; no clear chopping arc or follow-through. Primary activity read is standing idle holding an axe. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **04_1x.gif**: Large bound horizontal logs obscure chest; alternating legs and pointed hide hem remain visible. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **04_2x.gif**: Large bound horizontal logs obscure chest; alternating legs and pointed hide hem remain visible. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **05_1x.gif**: Single curled reclining pose with head on hands; torso looks bare, though monochrome outlines make clothing uncertain. Method: all 1 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **05_2x.gif**: Single curled reclining pose with head on hands; torso looks bare, though monochrome outlines make clothing uncertain. Method: all 1 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **06_1x.gif**: Rear-facing stationary stance, smooth draped top; rounded tied hair shape projects to the side. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **06_2x.gif**: Rear-facing stationary stance, smooth draped top; rounded tied hair shape projects to the side. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **07_1x.gif**: Logs dominate silhouette. Smooth lower top visible; most hair and any tie are occluded. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **07_2x.gif**: Logs dominate silhouette. Smooth lower top visible; most hair and any tie are occluded. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **08_1x.gif**: Repeated small tool and hand angle changes suggest chopping, but no large swing or contact cue; idle with axe remains plausible. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **08_2x.gif**: Repeated small tool and hand angle changes suggest chopping, but no large swing or contact cue; idle with axe remains plausible. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **09_1x.gif**: Single curled reclining pose with head on hands; torso looks bare, though monochrome outlines make clothing uncertain. Method: all 1 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **09_2x.gif**: Single curled reclining pose with head on hands; torso looks bare, though monochrome outlines make clothing uncertain. Method: all 1 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **10_1x.gif**: Fixed feet with small bob; shaggy top and short tousled hair visible. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **10_2x.gif**: Fixed feet with small bob; shaggy top and short tousled hair visible. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **11_1x.gif**: Alternating strides, plain diagonal hide folds and compact tousled hair. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **11_2x.gif**: Alternating strides, plain diagonal hide folds and compact tousled hair. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **12_1x.gif**: Logs obscure upper body and hair detail; shaggy pointed lower garment remains visible. Method: all 8 decoded frame(s), native dimensions [104, 112], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
- **12_2x.gif**: Logs obscure upper body and hair detail; shaggy pointed lower garment remains visible. Method: all 8 decoded frame(s), native dimensions [208, 224], duration metadata inspected. Limitation: static sequence, no timed playback; small features and occlusion can remain ambiguous.
