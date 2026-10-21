# ART-CONCEPT decisions

- D01: The pass-specific contract overrides the generic branch and documentation checklist: only art/concepts/** and docs/v8/artconcept/**, branch v8/art-concept, no merge. Base verified as 3183e77dfbe6373f1c3d3e0d08a1121fb546b694.
- D02: Preserve pre-existing .gitignore, .ignore and docs/art/ changes without staging them.
- D03: Use the session built-in image generation tool, one call per asset. Preserve original resolution and all attempts outside git. No CLI/API substitution.
- D04: Follow all eight original concepts without replacement. Owner selection remains open.
- D05: Readability uses separate forest and meadow crops per concept, each with two people (S1 and S2); a 256-square 4-by-4 tiled patch repeats to cover the required 640-by-360 crop. People are 56 px high. Record this geometric interpretation before testing.
- D06: Generic code/runtime tests do not prove image quality; verify image dimensions, provenance, board sizes and fresh blind review. No game runtime changes are authorized.
- D07: The built-in tool returns 1254×1254 square originals despite 1024/512 requests; preserve those exact bytes. Scene output is 1536×1024. Record requested/actual sizes instead of secretly resizing originals.
- D08: Camera failures stay rejected after the maximum three attempts. Their best diagnostic preview may occupy a clearly marked board slot; it is never counted as an accepted subject. No extra generation beyond caps.
- D09: 08 C2 failed the required one-shot magenta request. Do not substitute AI removal, an opaque ground square, a new generation, or a one-person crop. Both 08 readability scores remain NOT_VERIFIED. 05/06 C2 contain four logs; retain only as explicitly content-mismatched diagnostic readability inputs.
- D10: The board assembler keys magenta only, rejects non-magenta inputs, and uniformly scales each whole sprite using manually estimated crown-to-foot height. No despill, outlines, quantization, block reduction or repainting. The manual anatomical estimate is approximate, recorded externally.
- D11: Exactly 12 boards are named in §5 (8 concept boards and 4 comparison boards). The early execution ledger's 13-board count was arithmetic error, corrected without changing scope.
- D12: Scope forbids modifying generic CHANGELOG, ROADMAP, HISTORY, architecture and skills. This pass's decisions, checks, original provenance and limitations live inside docs/v8/artconcept. Local read-only docs check is supplementary, not image acceptance.
- D13: Structured visual verdicts are stored with external originals and LOG.md, not .omx state, because this pass explicitly restricts repository writes to its two directories.
