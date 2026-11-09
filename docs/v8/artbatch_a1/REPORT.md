# A1 animal candidate delivery — 2026-11-10

Completed 196 new frames (184 core + 12 optional winter reindeer), plus 20 byte-identical deer/wolf references: 20 appearance sets and 216 delivered PNGs. All new frames passed independent visual review (90–94); the complete family board passed at 91. Owner approval and game import remain separate.

Asset pin: `326822482b33edd1ac3bd4acefd0061f557756d6`.

- Sources: 62 built-in image generation calls, 492 requested frame slots, 196 accepted, 296 rejected. Malformed rejected sheets count their requested cells; accepted counts are actual verified PNGs. See `production_summary.json` for per-family counts and provenance.
- Art: `art/animal/batch_a1/adopted/`; aggregate `animal_manifest.json`; native/2× comparisons in `art/animal/batch_a1/boards/`.
- Verification: 26 source manifests / 196 frames reproduced from original RGBA; 216 delivery frames checked for complete grids, alpha, anchors, hashes and exact source bytes; 11 tests, default Ruff/basedpyright and static audit pass. Delivered PNG total 837,385 bytes.
- Import: `import_drafts/` pins the asset SHA; verified binding codes 0–5 only. The unchanged pinned importer resolved actual committed blobs for all 216 frames; 66 bound pose IDs had no gaps. This was a read-only plan, with no game writes. Unknown IDs and age/coat selection remain pending.
- Simplification: reuse existing splitting and normalization; one uniform upright scale per source sheet; exact-byte final copies; no dependencies or runtime systems added.
- Corrections: old horse base acceptance revoked and replaced; stale generator-record hashes reconciled after raw/prompt/reference checks; checker now rejects changed generation evidence.
- Limits: individual key poses are not animation cycles. Numeric projection reports do not certify camera angle or precise stroke ratios. Pig wash is slightly flatter and kid source contour stronger, accepted at native size.
- Existing docs-check failures remain historical docs binaries and prototype UIDs. No runtime/game/full gate was run on this art-only branch. `rust/`, `scripts/`, game assets and other worktrees are unchanged.

The complete Korean report and raw evidence are external at `/Users/rexxa/work/art-batch/A1/REPORT.md`. Rejected normalized attempts remain under that evidence root. C2/C3 own runtime import, selectors, play verification and owner review.
