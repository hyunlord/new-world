# Registered reading-test procedure

Status: preregistered protocol, now executed. Frozen verbatim answers and scores are in `BLIND_RESULTS.md`. Original rules and sample remain unchanged.

## Ten scene crops

Render the final accepted scene with `art/pipeline/boards.py`. Extract its ten
registered scene-local rectangles, then enlarge exactly 2× with nearest-neighbour.
Record the scene hash, each rectangle, anonymous identifier and image hash before
review. The reviewer receives the anonymous images, without source paths, answer
labels, generation prompts, the target mock-up, or this document. Use a new agent
with `fork_turns="none"`; no previous generator or selection reviewer can serve.

Neutral reviewer instruction: “For each numbered crop, describe the main object
or person and any visible activity. Say uncertain when the pixels do not support
a confident identification. Return one answer per number.”

Freeze the complete response verbatim before joining the answer key. Score one
point for each recognizable subject; ordinary synonyms count. The ten targets
are hut, lean-to, hearth, stockpile with logs, logs on bare ground, person carrying
logs, person chopping, child, elder, and sleeping person. A generic person without
the required activity or age does not earn the corresponding point. Record the
reason for every synonym acceptance and every miss. Threshold: **8/10**.

One failed-asset revision round is permitted. If used, preserve the first scene,
crops and answers, identify exactly which assets changed, and rerun with a different
fresh reviewer. Report both scores. The historical code-drawn **7/10** is supplied
baseline evidence, not a new result from this protocol.

## Six person composites

Inputs were frozen at `2026-10-21T13:19:00.331651+00:00` in external
`people/board_attributes.json`, SHA-256
`d2153a372f56e9859211be52c7fd39b2f4a4b330519c32bffaabb5613436dc9c`.
`random.Random(20261021).sample(range(24), 6)` selected zero-based indices
**17, 12, 18, 0, 22, 19**. Do not resample or change inputs after seeing results.

Send those six complete sprites at the board's exact nearest-neighbour 3× scale,
with neutral background and anonymous numbers, without their attribute labels.
Use another fresh reviewer, isolated from the scene test and generation history.
Ask for sex, age group (child / adult / elder), whether carrying an item, and which
tool is visible (or none), separately for every image. Preserve uncertainty.

Freeze the verbatim response before scoring against the original inputs. Each
field earns one point, for a fixed denominator of 24; uncertain or unreadable
fields earn zero. A tool answer must name its type or an unambiguous synonym;
“tool” alone does not earn the point. A shared child/elder body does not encode
sex reliably, and that limitation remains a miss rather than an excluded field.
Threshold: **20/24**. Check all 24 complete sprites for pairwise pixel inequality
separately; colour differences count as distinct pixel sets but do not by
themselves prove perceptual variety.

## Publication order

Only after each response is frozen may the anonymous identifiers be paired with
answers. The labelled `arttrial_crops.png` is assembled after the test. Save raw
responses, input hashes, key and scoring records externally; commit a concise
verbatim result and the final comparison images. Mechanical checks, palette
membership and parent selection judgments cannot substitute for either score.
