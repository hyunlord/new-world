# TRIAL4 pawn raster pipeline

Run `python3 -m art.pipeline4.pawn_gen /absolute/output/directory`, then
`python3 -m art.pipeline3.check_set /absolute/output/directory/manifest.json --output /absolute/checks`.
Run regression tests with `python3 -m pytest art/pipeline4 -q`.

`pawn_spec.py` contains geometry, palette indices, anchors and motion;
`ink_params.json` contains ink tuning. `ink.py` hashes both exact files and uses
that SHA256 as the grain seed. `pawn_gen.py` owns rasterization orchestration,
layer inventory and the shared composition transforms. The spec is data only.
No image generation services, image editing or simulation changes are used.

Environment measured: Python3.14, Pillow12.2.0, numpy2.4.3. Cairo and skia were
not used. Base palette values are exact palette05 entries. Flat-band thresholds
are support quantiles, so shadow/highlight target35%/15% of each primitive's
area. Grain is uniform multiplicative ±8%; the contour is an internal2px
stroke derived from the union alpha. Detail strokes are1px, at most4hair and
3cloth. Rendering occurs at4× with Lanczos downsampling.

The canonical male torso is rasterized once. Female/elder and child torso
widths use the same registered image transform as their tops. Child hair uses
18/22 horizontal and16/20 vertical scale about the head centre. All body
frames are rigid registrations of idle; masks follow the same transforms.

The top hem ends at y81: its three-pixel margin below the upper70% target
prevents the mandatory Lanczos kernel's negative lobe from making opaque
body pixels translucent at that target boundary. The neckline extends over
the base of the neck to cover actual BODY alpha within the target band.

`compose_layers` consumes stored assets. `item_placement` performs animation
on a padded canvas and reports whether alpha is lost outside the final canvas.
The browser and checker share this placement, including shoulder logs and
head-left sleep. `compose` is a convenience raster preview, not acceptance
evidence for the stored inventory.

The generated inventory contains636 layers:504bodies (168adult plus168child
and168elder),72hair,24tops,36items. Optional none variants are explicitly
empty layers. Sidecars and manifest are deterministic. `GENERATION.json`
contains wall-clock and size measurements and is deliberately excluded from
byte-identity comparison because elapsed time is a measurement.

Implementation diagnostics are retained externally. Early prototype output
iterations were overwritten before diagnostic preservation was requested;
their previews and tool errors survive, but no byte provenance is claimed for
those overwritten intermediates. `implementation-attempt-before-collar`,
`initial`, and `baseline` preserve subsequent failed implementation candidates.

The final renderer applies the template's canonical BODY support envelope to
TOP/HAIR before PNG/mask serialization, with no coordinate-specific repair.
This is a shared layer construction operation, recorded in sidecars, authorized
because independently Lanczos-filtered overlapping shapes can create a tiny
halo outside the same underlying silhouette. Alpha-support cutoff remains8;
no near-opaque alpha saturation or relaxed structural threshold is used.
The checker still requires top alpha at least BODY alpha over the upper band.

Candidate freeze: `d9133b411b0cd935ac253d3a2bddd1a2dadfe2befc8a186069c1f60780dc4b50`
(spec file bytes plus ink JSON bytes). Full initial candidate generation:
22.652258415939286seconds,636layers, total layer PNG1,741,698bytes,
smallest104bytes (empty), largest3715bytes. Raw timings, PNGs, masks, sidecars,
and checks live in `/Users/rexxa/work/arttrial4-data/candidate-final` and
`candidate-final-checks`. Initial implementation attempts are enumerated in
external `implementation-attempts.json`; the freeze does not conceal them.
