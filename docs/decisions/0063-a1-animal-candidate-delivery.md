# ADR 0063 — Deliver verified animal candidates separately from game import

## Context

The 2026-11-10 ART-BATCH brief assigns A1 the animal family on `v8/art-batch-a1`,
with raw evidence outside the repository and C2/C3 responsible for later import.
The content bible and brief expand the finite catalogue to 17 required appearance
sets / 184 S/E pose images. Winter reindeer adds an optional 12 images. Existing
deer/wolf supply 20 unchanged reference images. These numbers define coverage;
they do not establish visual acceptance or an owner decision.

The imported animal convention uses 96×80 upright canvases anchored at (48,72)
and 96×64 dead canvases anchored at (48,56). Its splitter and normalizer already
preserve whole figures, use one measured S-stand shoulder scale for upright poses,
and use a separate declared body-length scale for dead poses.

## Decision

- Reuse those geometry and ownership functions through thin A1 processing and
  verification tools. Record prompts, actual reference attachments, hashes,
  landmarks, transforms, anchors and footprints. An unexposed seed remains null.
- Fix the family line ratio, material mark budgets, wash, hatching and light in
  the family style rules. Review each selected pose at native scale and nearest
  2× alongside deer/wolf and other new animals. Numeric projection results are
  reports; they cannot prove the camera angle, anatomy or style.
- Treat a two-image attack sheet as two unaltered rectangular crops only when
  the center gutter contains no visible alpha ≥16. Keep every source RGBA pixel,
  including alpha-1 dust, and record the gutter maximum. The regular 2×5 splitter
  keeps its existing ownership path.
- Inspect returned alpha rather than trusting the generation request. Pig try1
  had background alpha 173–251 and no transparent pixels; it was rejected.
  Preserve failed originals and use generator revisions for background or visual
  repairs. No code erasing, recolouring or anatomical assembly is allowed.
- Require a complete catalogue matrix, successful structural/source checks and
  an explicit root selection decision before assembling the final pack. Copy PNG
  bytes exactly; preserve original sidecar provenance and append selection/review
  hashes. Use `ACCEPTED_ART_CANDIDATE`, with owner approval and runtime import
  separately pending. Include existing deer/wolf bytes without regenerating them.
- Keep import/binding rows as drafts. This branch does not modify simulation,
  rendering or another worktree. A pose key is not a complete animation cycle.

## Alternatives considered

- Accept a sheet because it looks transparent: rejected because alpha inspection
  found a nearly opaque background in a transparency-requested output.
- Erase faint gutter pixels or require exact-zero alpha: rejected because the
  former changes source pixels and the latter rejects invisible dust that the
  established alpha-16 contract permits. Lossless cropping preserves both.
- Shrink each pose independently until it fits: rejected because it conceals
  inconsistent anatomy/scale and weakens cross-pose continuity.
- Import directly into the game or declare owner adoption from automated checks:
  rejected because the brief assigns game integration to C2/C3 and a numeric
  structure pass is distinct from visual acceptance and owner approval.

## Consequences

The final candidate manifest can serve the existing data importer, with stable
species/facing/pose paths and explicit canvas/anchor metadata. Original and
rejected generations remain external; compact comparison boards are curated in
the art family. Full selection and delivery verification are still pending at
this decision's creation.

The existing art-branch docs check already fails on historical binaries under
`docs/` and missing prototype UIDs. Its baseline output is preserved externally;
this decision neither changes those files nor reports a game gate as passed.
The simulator registration table is unaffected by these offline tools.

## Date

2026-11-10

## Source

- `/Users/rexxa/work/art-batch/BRIEF.md`; `/Users/rexxa/work/c2-data/GOAL.md` §3.5.
- Content bible: https://github.com/hyunlord/new-world/issues/690.
- `art/animal/batch_a1/catalog.json`; `docs/v8/artbatch_a1/DECISIONS.md`.
- `art/animal/TEMPLATE_A1.md`; `art/pipeline_env1/animal_normalize.py`.
- `art/animal/batch_a1/process.py`, `check.py`, `adopt.py`, `test_process.py`.
- `/Users/rexxa/work/art-batch/A1/raw/pig/try1.verdict.json`;
  `/Users/rexxa/work/art-batch/A1/records/docs-check.txt`.
- Number follows the highest ADR observed on `origin/v8/main` (0062); this
  historical art checkout contains earlier ADRs only through 0015.
