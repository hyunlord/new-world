# A3 item adoption decisions — 2026-11-10

## Context

ART-BATCH requests missing item artwork across four eras and count/state variants,
with no game import or runtime changes. The art checkout predates the live
`assets/v8art` integration. Its existing ENV1/POLISH sources are nevertheless the
sources declared by the pinned integration configuration.

## Decision

1. Reuse ENV1's 64px centered icon contract, anchor (32,32), fit inside 58px,
   and its ordinal 1/3/9-copy layouts. Preserve source alpha. These are item icons,
   not newly calibrated STRUCT world props. Footprint is one tile.
2. Preserve full prompts, tool records and raw images externally. Commit selected
   PNGs, per-item provenance, the complete adoption manifest and curated sheets.
   Null seeds mean the image service did not expose a seed.
3. Keep numerical style and whole-image lighting measurements unchanged and
   visible. They are diagnostics alongside direct comparison, not automatic
   adoption: four of six imported raw references also fail this style calibration.
   Final 64px style scoring fails all six references because line extraction is
   resolution-dependent. Raw-source scoring remains the recorded measurement.
4. A numeric exception requires an actual visual comparison. Whole-image colour
   gradients can be dominated by dark metal/light wood, a dark vessel interior,
   biological colour patterns or count layout. Preserve both the numeric failure
   and the separate visual decision. Fish and goat-hide lighting direction is
   indeterminate, not proven upper-left. Their silhouettes, local tone and palette
   remain compatible with the reference family.
5. Reject semantic errors: acorn-like hazelnuts, axe-like hoes, modern garment
   necklines, an anachronistic sword guard, or unreadable defining metal parts.
   Retain all failed attempts. Do not adopt an unresolved weak candidate merely
   to make the catalogue count complete.
6. The iron spear and steel edge remain withheld after the image service blocked
   their correction batch. No accepted replacements or runtime bindings are
   claimed. The original error and prompts remain external evidence.
7. Use an item-manifest adapter retaining legacy geometry for later import. The
   current importer ignores adjacent sidecars and otherwise loses the centered
   anchor. Future simulation codes and the general filled/empty selector remain
   explicit integration work; do not invent IDs or unsupported binding fields.

## Alternatives considered

- STRUCT ground anchor and 4–24px sizing: rejected for this family because the
  actually imported inventory icons use centered 64px canvases.
- Alter calibration until every candidate passes: rejected; it would obscure
  existing evidence and confuse visual acceptance with instrument output.
- Strip faint alpha or repaint pixels to force geometry: rejected; normalization
  reuses the existing source-preserving pipeline.
- Add runtime sidecar support: unnecessary for this art-only handoff; existing
  `meta_sources.item` can consume a merged geometry adapter.

## Consequences

Some narrow objects and seed varieties require labels at32px. Count images are
ordinal indicators, not literal inventories. Empty states have matching artwork
but no general runtime selector. Art adoption is neither game integration nor
owner play approval. All non-passing diagnostics and the two withheld identities
are visible in the manifest/review record and handoff report.

## Date

2026-11-10.

## Source

`/Users/rexxa/work/art-batch/BRIEF.md`; content bible issue690;
`STYLE.md`; `MANIFEST.json`; `review.json`; `drafts/README.md`;
integration commit `4a44336e7782f2b27615d942b5cd54dbfee89406`.
