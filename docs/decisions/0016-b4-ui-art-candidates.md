# B4 decision record — UI candidates extend the A4 family

## Context

ART-BATCH round 2 requests jobs, alerts, needs/moods, decision illustrations, steward rules, building categories and map legend art. The art checkout predates the live UI, so names must be read from a pinned game commit without editing it. A4 established a production-scale ink-and-tone UI family and transparent export tooling.

## Decision

Reuse A4's exact normalizer, its tests and its comparison-sheet tool, with no new package dependency. Author 148 canonical images in five bounded lanes. Record existing runtime names separately from proposed art variants. Canonical icons are 64x64; decision vignettes 520x260; centers are UI anchors with null footprints. All source images come from the built-in image generator with actual reference attachments and preserved alpha. Every selected candidate has an immutable attempt record, normalized export, sidecar, contact-sheet comparison and visual review.

Needs use distinct adequate/critical silhouettes; era-specific work and card variants have explicit material permissions. The no-mood sentinel remains absence. Generic selectable states share canonical art through unapplied UI treatment recipes. After base coverage and variants, each lane regenerates 10–20 percent selected from its weakest contact-sheet candidates and retains the alternatives.

## Alternatives considered

- New normalization framework: rejected because A4 already supplies tested alpha/canvas/center semantics.
- One image per runtime state or locale: rejected because selection/disabled/KO/EN are UI presentation and labels; semantic variants are authored separately.
- Guessing new sim IDs for smelting, raids or future steward rules: rejected because these are not present at the pinned runtime commit.
- Applying game imports here: rejected because C2/C3 own runtime integration.

## Consequences

Importer drafts need a full immutable art commit and explicit UI metadata. Current UI image consumers and proposed binding groups still require C2/C3 integration. Structural/semantic/general-vision evidence is not the historical generator judge and does not confer owner approval. Raw/evidence data stays outside the repository.

## Date

2026-11-10

## Source

`/Users/rexxa/work/art-batch/BRIEF.md`; `BRIEF2.md`; content bible issue #690; `art/ui_b4/runtime_inventory.json` (game commit `4a44336e7782f2b27615d942b5cd54dbfee89406`); A4 `art/techmap/STYLE.md` and tools at `386248ffbab91993960145f59fbc3636890cea63`; ADR 0018 on the game branch.
