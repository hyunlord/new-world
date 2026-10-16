# Converter v4 — Part 1 evidence

`pixel_normalize_v4.py` adds palette-first source-block mode pooling without changing v3, raw images, installed assets, or Production 1 reports. `measure_cell_v4(image)` returns unchanged v3 recovery metadata. `fit_cell_v4(image, palette, factor, icon_canvas=False, measurement=None)` returns `FitResult(native, sprite, median_sample, metrics)`. Factor planning and automatic guards are separate in `v4_contracts.py`; converter output is never visual acceptance.

## Literal test investigation

The unchanged regression uses NumPy seed 91, eight palette colours, native 19×33, scale 5.9 and phase 1.7. Initial red: 66.9642857% nearest-neighbour agreement (required ≥80%). Investigation was bounded to three hypotheses in one correction round:

1. Grid estimation error: substituting known period 5.9 while keeping the old modular phase made agreement worse, 60.4910714%; this was diagnostic only and is not shipped.
2. Phase origin: `_blocks` starts at `phase - period`. Keeping the old modular phase while changing period therefore moves the physical first boundary. Preserve `origin = phi - p` and pass `sampling_phi = origin + p/f`. This is the specified same physical origin. Agreement becomes **84.375%**, output **16×28**. The fixture, palette, seed and ≥80% threshold are unchanged.
3. Mode versus nearest: the remaining differences are expected from majority pooling; neither nearest resampling nor a nearest-colour agreement optimisation is used by the converter.

A regression asserts `phi-p == sampling_phi-p'` on both axes and checks factor 1 against the v3-quantized synthetic fixture. This equality is not a claim that mode pooling and median quantization match on all real images. The dark-outline test checks all four original native borders before adding a new outline; it is not satisfied by the newly added border.

## Measurement and size boundaries

Metrics retain native sizes, detected periods, new periods, original phases, sampling phases, physical origins, factor, outlined canvas dimensions, common-opaque median ΔE76, keyed-out count, palette membership, binary alpha, semantic counts and spill diagnostics. Spill fringe repair can produce a colour outside the palette, so a final nearest-Lab palette projection enforces membership; changed texels are explicitly counted and warned. Fidelity is measured after this projection.

All targets refer to `native`, before outline. U1 pads a fitting result to exactly 16×16; `sprite` is the separate outlined canvas (potentially 18×18). An oversized pooled U1 raises a conversion failure instead of clipping or silently changing the prescribed factor. Physical partial edge blocks can exceed object maxima by one texel; geometry guards retain their exact maxima and reject those candidates.

Read-only real-source smoke: V_a1_01@1 goes 22×36 →17×28 at f=28/36, ΔE76=9.5901 (colour warning); N3_01@1 goes21×17 →13×10 at f=12/21, ΔE76=7.2466 and fails the 12×12 geometry guard. Neither was accepted or installed. Source hashes and all measurements are in `converter_evidence.json`.

## Verification

- `python3 -m unittest discover -s tools/art_pipeline -p 'test_normalize_v*.py'`: **14/14 pass**, including unchanged v3 tests and six v4 tests.
- `python3 -m unittest discover -s tools/art_pipeline -p 'test_*.py'`: **77/77 pass**, including contracts tests.
- `python3 -m compileall -q tools/art_pipeline/pixel_normalize_v4.py tools/art_pipeline/test_normalize_v4.py`: pass.
- `git diff --check`: pass. Ruff and basedpyright executables are not available; no dependency was added.

Old v3 literal harmonic/JPEG limitations are unchanged; passing its existing suite does not retroactively establish those stronger Production 1 requirements. Visual acceptance, all-raw salvage, animation selection, and final runtime validation are later parts.
