# Blind review 1

Status: decoded-sequence proxy; actual timed playback NOT OBSERVED.

Only the 24 numbered GIFs were used. No route names, answer key, source assets, or other reviews were read. All 164 decoded frames were visually inspected in temporal order, preserving each GIF frame’s native 104×112 (1x) or 208×224 (2x) dimensions. Temporal panels add frame-index/duration labels outside the frames.

Files 05 and 09 at both scales contain one 1000 ms frame each. Every other file contains eight frames, alternating 120 and 130 ms, totaling 1000 ms. Duration metadata was inspected; perceived motion timing was not observed.

Confidence values are subjective 0–1, ordered activity / object / top / hair. They are not accuracy scores.

| GIF | Activity | Object | Top | Hair | Confidence |
|---|---|---|---|---|---|
| 01_1x.gif | idle | none | furcloak | longtied | 0.76 / 0.77 / 0.84 / 0.8 |
| 01_2x.gif | idle | none | furcloak | longtied | 0.82 / 0.83 / 0.9 / 0.86 |
| 02_1x.gif | walking | none | furcloak | short | 0.88 / 0.88 / 0.85 / 0.85 |
| 02_2x.gif | walking | none | furcloak | short | 0.94 / 0.94 / 0.91 / 0.91 |
| 03_1x.gif | idle | none | hidewrap | longtied | 0.69 / 0.51 / 0.82 / 0.8 |
| 03_2x.gif | idle | none | hidewrap | longtied | 0.75 / 0.57 / 0.88 / 0.86 |
| 04_1x.gif | carrying | log | hidewrap | short | 0.93 / 0.93 / 0.72 / 0.81 |
| 04_2x.gif | carrying | log | hidewrap | short | 0.99 / 0.99 / 0.78 / 0.87 |
| 05_1x.gif | sleeping | none | none | bald | 0.93 / 0.92 / 0.75 / 0.84 |
| 05_2x.gif | sleeping | none | none | bald | 0.99 / 0.98 / 0.81 / 0.9 |
| 06_1x.gif | idle | none | hidewrap | longtied | 0.83 / 0.86 / 0.84 / 0.78 |
| 06_2x.gif | idle | none | hidewrap | longtied | 0.89 / 0.92 / 0.9 / 0.84 |
| 07_1x.gif | carrying | log | hidewrap | unknown | 0.93 / 0.93 / 0.78 / 0.42 |
| 07_2x.gif | carrying | log | hidewrap | unknown | 0.99 / 0.99 / 0.84 / 0.48 |
| 08_1x.gif | idle | none | hidewrap | longtied | 0.64 / 0.51 / 0.8 / 0.82 |
| 08_2x.gif | idle | none | hidewrap | longtied | 0.7 / 0.57 / 0.86 / 0.88 |
| 09_1x.gif | sleeping | none | none | bald | 0.93 / 0.92 / 0.75 / 0.84 |
| 09_2x.gif | sleeping | none | none | bald | 0.99 / 0.98 / 0.81 / 0.9 |
| 10_1x.gif | idle | none | furcloak | short | 0.82 / 0.87 / 0.87 / 0.87 |
| 10_2x.gif | idle | none | furcloak | short | 0.88 / 0.93 / 0.93 / 0.93 |
| 11_1x.gif | walking | none | hidewrap | short | 0.89 / 0.88 / 0.87 / 0.87 |
| 11_2x.gif | walking | none | hidewrap | short | 0.95 / 0.94 / 0.93 / 0.93 |
| 12_1x.gif | carrying | log | furcloak | unknown | 0.93 / 0.93 / 0.8 / 0.42 |
| 12_2x.gif | carrying | log | furcloak | unknown | 0.99 / 0.99 / 0.86 / 0.48 |

## Visual evidence

- 01, both scales: Fixed leg pose with small body bob; shaggy layered torso and visible tied hair at the side.
- 02, both scales: Alternating foot placement reads as stepping; shaggy torso and short tousled hair.
- 03, both scales: Small bob without clear swing or step. Smooth patchy drape and side ponytail. Brown shoulder-side form is ambiguous but no clear axe blade is readable.
- 04, both scales: Large tied horizontal logs dominate the silhouette; feet step; smooth irregular clothing visible below load.
- 05, both scales: Side-lying curled body with head resting on bent arms; exposed upper torso and round bare-looking scalp.
- 06, both scales: Rear-facing fixed stance with bob; smooth broad drape and low side hair tie.
- 07, both scales: Horizontal bound logs; smooth cloth visible below. Load hides most head/hair contour, so hairstyle is uncertain.
- 08, both scales: Spread feet maintain their pose while the figure bobs. Smooth drape and tied side hair. Shoulder-side brown shape lacks a readable axe blade or swing.
- 09, both scales: Single curled side-lying pose with arms under head, bare-looking chest and scalp.
- 10, both scales: Same leg arrangement through ordered sequence with small vertical bob; shaggy layered torso and tousled short hair.
- 11, both scales: Feet visibly alternate stride; plain patchy hide drape and short tousled hair.
- 12, both scales: Large bound logs across torso; shaggy pointed hem below load. Head partly hidden and hairstyle not readable.

## Limitations

Only decoded temporal sequence and GIF duration metadata observed; actual timed moving playback was unavailable. Native pixel dimensions preserved in panels. This is a proxy readability answer, not a playback-validated animation judgement.

The 1x and 2x versions were inspected together, so scale answers are paired observations rather than independent observer trials. No scoring against an answer key was performed. No Godot instance was launched. Temporary panels remain outside the repository.
