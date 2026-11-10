# ADR 0016 (art line) — Keep era people compatible with adopted Route I

## Context

ART-BATCH B1 requests people through the eras, wardrobe and held-tool swaps,
all appearance-grid ages/sexes, portraits, variants and a regeneration pass.
The brief refers to a cut-out rig, but its named TEMPLATE_V3 and the final
ADR 0018 adoption record on the game line specify complete Route I figures.
The original layered rig is historical. The current game importer uses one
frame source pattern and one portrait source pattern for every look; job
state chooses a pose globally and does not select a held-tool variant.

## Decision

Produce seven wardrobe tops as complete ten-frame S/E sheets for all twelve
age/sex/hair combinations. Reuse the existing ownership, uniform scale,
sleep normalization and structural checks. Produce six portrait identities
per new look, including the existing elder variant scheme. Generate extra
work/load states as complete figures, with explicit pending selector bindings.
Keep every source and rejected/superseded attempt outside the repository.

Use a small B1 adapter to accept the new top IDs without changing historical
production schemas or ledgers. Candidate selection requires the unchanged
structural checks and a saved visual review at native/2× size. Preserve the
legacy source-resolution numerical style diagnostics without relabelling a
failure or a missing historical model score as a pass.

The import handoff must retain the existing 34 adopted looks and two logical
fallbacks, append top axes in an exact documented order, and distinguish a
file-import plan from runtime mask/tint/selector integration. No simulation,
renderer or live import configuration is changed in this art pass.

## Alternatives considered

- Revive the historical part rig: rejected because it contradicts the adopted
  character standard and does not match the current importer.
- Cut and recolour existing figures: rejected by TEMPLATE_V3 and because
  clothing/tool seams would damage the original owned pixels.
- Encode tools as appearance tops: rejected because work state is not identity
  and would expand or scramble the bridge's appearance grid.
- Claim a separate source root is immediately importable: rejected because
  the importer cannot override its source pattern per look.

## Consequences

The candidate set is larger than a layered rig, but every frame follows the
same checked geometry and remains directly inspectable. C2/C3 still owns
garment-code selection, masks and measured face transforms, tool-state
selection, resource warmup and actual game captures. An art selection is not
owner approval or game integration. Existing game-side ADR numbering is not
changed by this art-line-local decision.

## Date

2026-11-10

## Source

`art/batch_b1/STYLE_CONTRACT.md`, `art/batch_b1/CATALOG.json`,
`art/character/TEMPLATE_V3.md`; content bible `hyunlord/new-world#690`;
game reference `4a44336e7782f2b27615d942b5cd54dbfee89406`:
`assets/v8art/ADDING_ART.md`, `tools/v8/art_import.py`,
`rust/crates/sim-bridge/data/appearance.ron`, and final ADR 0018 adoption note.
