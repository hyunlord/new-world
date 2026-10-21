# ART-CONCEPT decisions

- D01: The pass-specific contract overrides the generic branch and documentation checklist: only art/concepts/** and docs/v8/artconcept/**, branch v8/art-concept, no merge. Base verified as 3183e77dfbe6373f1c3d3e0d08a1121fb546b694.
- D02: Preserve pre-existing .gitignore, .ignore and docs/art/ changes without staging them.
- D03: Use the session built-in image generation tool, one call per asset. Preserve original resolution and all attempts outside git. No CLI/API substitution.
- D04: Follow all eight original concepts without replacement. Owner selection remains open.
- D05: Readability uses separate forest and meadow crops per concept, each with two people (S1 and S2); a 256-square 4-by-4 tiled patch repeats to cover the required 640-by-360 crop. People are 56 px high. Record this geometric interpretation before testing.
- D06: Generic code/runtime tests do not prove image quality; verify image dimensions, provenance, board sizes and fresh blind review. No game runtime changes are authorized.
