# ADR 0068 — Optional generated UI art with stable portrait selection

## Context

The 3D-only view retains functional but mostly textual UI panels. Part U of M3D-IMG requests warm, clean painterly portraits and illustrations, plus item icons matching the existing 3D models. Art generation and GPU-gated local rendering proceed independently of UI integration, so missing files must not make an existing panel unusable. The simulation, numeric content and saves are outside this presentation pass.

The inventory target is 322 generated images (72 people, seven animals, ten chapters, five yearly goals, 18 cards, 166 knowledge entries, 38 unique building/station IDs and six key-art options), plus a conditional target of 119 GLB-rendered item icons. These are planned counts, not proof that generation has completed.

## Decision

- Keep optional images for existing surfaces in `assets/ui/generated/`, addressed by family and canonical content ID. Keep the seven animal portraits, five yearly-goal illustrations and selected title art in external data while those surfaces do not exist. Use the small `scripts/ui/generated_art.gd` helper for cached texture lookup and image-node setup. Missing or invalid paths return null, and callers preserve their existing presentation.
- Select person portraits in the UI from entity bits, life stage, sex and era wardrobe with a deterministic four-look variant. Check snapshot column types and bounds before reading them. Share the hide wardrobe for the early paleolithic/neolithic eras; bronze and iron use their respective wardrobe families. Do not advance a simulation RNG or store a new identity field.
- Connect existing panels only. Keep generated image nodes non-interactive, preserve localized text and command behavior, and scale art to the existing containers. A missing animal inspector or title/loading surface requires a planner question rather than a new unsolicited panel. Reuse the existing startup boot splash for selected loading art, with the PNG format required by Godot4.6 and aspect-preserving fit; keep its normal startup duration.
- Add `ItemIcons.ui_texture_of` for optional icons rendered from existing GLBs. It falls back to the established item texture path. Leave the loose-world `texture_of` seam intact so a UI asset cannot change world piles or their established tests.
- Use cloud image generation for authored art. Item icons come from the models, not image generation. Any local render or capture requires the shared GPU gate check to pass. Keep prompt/adoption manifests, alternate key art, contact sheets and raw evidence outside the repository in `/Users/rexxa/work/art-img-data/ui/`.
- Work and push only `v8/art-ui`. The planner merges this branch into a later gated publish. Implementation and focused verification are complete; this decision records no full-gate or publication result.

## Alternatives considered

- Put generated UI art into simulation content or saves: rejected because portrait variety and decorative illustration do not change mechanics or persistent identity.
- Replace the shared item texture provider globally: rejected because it also serves loose-world presentation; a UI-only seam is the smaller boundary.
- Require every requested art file before showing a panel: rejected because it would turn independent image-generation failures into UI failures.
- Introduce new animal/title/loading panels in this art pass: deferred to the planner under the brief's missing-surface rule.

## Consequences

The art set can be generated and adopted in batches while each existing surface remains usable. Deterministic selection is view-only, and missing files have a visible fallback rather than a simulation dependency. Generated asset coverage, actual captures, branch push receipts and remaining missing surfaces must be reported separately; target counts do not establish completion. No Rust code, content quantities, bridge schema or save format changes are required.

## Date

2026-11-16

## Source

Owner task: Part U of `/Users/rexxa/work/art-img-data/BRIEF_IMG.md`, branch `v8/art-ui`; external `UI_PLAN.md` and `asset_inventory.json` in `/Users/rexxa/work/art-img-data/ui/`. Existing contracts: ADR 0016 (view-independent simulation) and ADR 0033 (content identities are data).
