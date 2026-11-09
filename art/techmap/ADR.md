# A4 ADR — Standalone knowledge, era and chapter illustrations

## Context

The content bible names four eras and concrete knowledge nodes, while the current UI uses text cards. The art branch predates those game-side panels. This session is authorized to produce candidate art and import drafts only. Existing imported sprites use the concept05 ink-and-tone language, but world base anchors and texture-density calibration do not describe UI layout.

## Decision

Create the new family under `art/style05/techmap/`, with the catalogue and fixed production contract under `art/techmap/`. Use 64px icons, 264x88 era vignettes and 520x260 chapter illustrations, transparent backgrounds and center anchors. Normalize mechanically without repainting. Keep exact raw provenance outside the repository and retain prompt/check/review summaries in the manifest. Generate the 167 concretely named nodes rather than inventing IDs to satisfy an approximate target. Use semantic chapter IDs to preserve the legacy chapter 6 conflict. Four knowledge states reuse canonical artwork through proposed UI treatments.

Candidate selection requires structural checks and independent visual comparison. The historical item numeric judge remains unchanged and its failures remain visible as diagnostics. General vision review is explicitly named; it is not the historical image-generation-model review. Candidate selection does not assert owner approval, UI integration or runtime verification.

## Alternatives considered

- Reuse world item anchors and height bands: rejected because centered UI icons are not objects placed at world feet.
- Add bitmap consumers or apply imports here: rejected because the brief assigns this work to C2/C3.
- Invent additional technology IDs to reach 190: rejected because the source tables do not name them.
- Paint separate stable/unknown/inert assets: rejected because UI states should preserve subject identity and localization.
- Recolor or sharpen generated art in code to clear checks: rejected because it conceals weak generations and drifts from original ink.

## Consequences

C2/C3 must pin the selected art commit, add explicit import rows and metadata, implement knowledge/header/chapter consumers, resolve chapter numbering, and capture the real game. A4 can verify file quality and coverage without importing or launching the game. Seasons, count buckets and depletion do not apply to this UI family.

## Date

2026-11-10

## Source

Issue https://github.com/hyunlord/new-world/issues/690; `/Users/rexxa/work/art-batch/BRIEF.md`; `/Users/rexxa/work/c2-data/GOAL.md`; source art `f182769fe06b1cb60105e4d0bf127b6524277838`; UI contract `4a44336e7782f2b27615d942b5cd54dbfee89406`; `art/techmap/STYLE.md`; `docs/v8/artbatch_a4/DECISIONS.md`.
