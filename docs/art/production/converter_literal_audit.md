# Converter v3 literal section 6.1 audit

Status: **NOT SATISFIED**. The ordinary tooling unit suite passing is not full section 6.1 acceptance.

Reproduce:

```bash
python3 tools/art_pipeline/audit_converter_literal.py
```

This command is intentionally separate from general regression discovery and exits **1** while the literal requirements remain unmet. Its detailed evidence is `converter_literal_audit.json`. No existing generation report was recalculated or overwritten by this audit; converter selection and production thresholds are unchanged.

## What is proven

All nine scale/phase combinations (5.3, 5.9, 6.4 × 0, 1.7, 3.1) recover exactly 20×28 from PNG and JPEG. PNG color recovery is 100% at ΔE76 <3. The reproducible fixture randomly assigns eight explicitly listed colors using seed91.

## What is not proven, and fails directly

Raw JPEG quality88, default4:2:0 chroma subsampling, recovers only35.71–60.00% of original texel colors within ΔE76 <3. The prior test quantized against the known original palette before comparison and therefore tested a stronger-input normalization pipeline, not raw JPEG reconstruction. That test is now named explicitly `test_png_recovery_and_known_palette_jpeg_normalization`; its original assertions remain unchanged. Known-palette matching reaches100%, but does **not** satisfy the raw recovery requirement.

Two bounded reconstruction experiments were considered without changing production code. Clustering inferred native colors with a supplied eight-cluster count reached83.0/66.1/72.0% with median/farthest initialization for the three scales at phase1.7. Thirty deterministic multi-start mean-clustering trials reached85.4/100/100%; the lowest scale still failed90%, and the cluster count itself is unavailable from arbitrary source art. Installing that heuristic would introduce unproven color assumptions rather than establish the requested guarantee. No guessed palette was installed.

## Harmonic counterexample

A20×28 grayscale palette sprite uses repeated columns40,45,200,205 and a7-value alternate-row difference, upscaled4.0 with phase1.7. The prescribed algorithm returns period7.9 and native10×15, not4.0 and20×28.

The specification's maximum-score rule and universal strong2s rejection conflict for this input. If alternate true boundaries have magnitudes H and L, the fundamental score is approximately(H+L)/2, while sampling only the strong boundaries at2s produces H−L because its midpoints hit the weak boundaries. For H>3L, the harmonic score is larger. Midpoint subtraction is useful but does not universally reject strong alternating edges.

The score, search grid, shared-period argmax and anisotropy conditions are retained exactly. Returning4 for this counterexample would need an additional assumption or selection rule not present in the prescribed argmax. The production-scale alternating-edge regression at5.9 remains useful but has been renamed to avoid calling it universal harmonic rejection; its2s harmonic lies outside the4–8 search interval.

## Production consequence

Lossless generation inputs avoid JPEG color degradation but not pixel-period ambiguity. Native-size and fidelity acceptance remain unchanged; source crops with a logical block period outside4–8 or ambiguous harmonic structure require source review/regeneration. Section6.1 is a declared art-QA failure, even when the general code gate is green.
