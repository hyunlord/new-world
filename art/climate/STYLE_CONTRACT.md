# B2 frozen style and geometry contract

B2 extends the accepted concept-05 ink-and-tone art. These rules are fixed
before adoption; a failed candidate is regenerated or recorded as rejected.
The source and scope inventory is in [COVERAGE.md](COVERAGE.md). Per-image
manifests record the actual geometry, source references, generation attempt,
normalization and checks.

## Reference identity

- Current imported grounds are
  `art/style05/env/ground/{dark_grass,meadow,forest_floor,hill_rock,packed_earth,river_bank}.png`.
  The historical `art/style05/ground_v2/` candidates contain failed judgments
  and are not substituted for accepted references.
- Existing flora comes from `art/flora/sprites/`, `masks/` and `species.json`:
  oak, birch, pine and berry. Route-I people establish human scale; accepted
  ground, hut and plant anchors establish palette, illumination and projection.
- Freeze reference paths and SHA-256s per generation. Do not modify a reference
  to make a new candidate match. Complete source provenance is retained in the
  external `B2/references/` directory.

## Shared drawing language

| Property | Frozen target |
|---|---|
| World scale | 64px tile; accepted route-I adult visible height about 56px |
| Standing flora projection | Front three-quarter, about 20° down; visible base/root contact |
| Ground projection | Orthographic top-down; no horizon, raised island or baked standing face |
| Lighting | Upper left; lower-right shade, no conflicting second light |
| Contour | Warm brown-black or sepia ink; main-to-secondary line target **2:1** |
| Wash | Restrained natural earth, sage/olive, ochre and slate; broad readable masses |
| Hatch direction | `/`, in a group of **3** parallel strokes; do not cover every surface |
| Wood | **3** broken longitudinal marks per principal visible surface |
| Stone | **3** crack/facet groups per principal mass |
| Leaves | **3** vein accents per clump, with simple larger foliage masses |
| Reeds and grasses | **5** parallel bundle marks per principal group; avoid hairline noise |
| Soil | **3** quiet furrow/texture groups per tile-scale surface |

Mark counts and the 2:1 ratio are prompt and visual-review targets, not claimed
automatic measurements. Round-one A2's botanical controls and A3/A4 use `/`;
A1's animal contract describes the other diagonal. B2 deliberately follows the
nearest botanical A2 reference and does not claim all round-one families had an
identical hatch rule.

No photorealism, glossy 3D shading, pixel-art quantization, dense all-over
hatching, modern props, labels or decorative frames. Standing cutouts have no
ground mat, cast shadow, scenery or detached decorative leaves. Ground shadows
belong to later composition. Transparent output must retain real saved alpha;
hidden RGB in a preview is not itself a halo defect.

## Canvas, anchor and footprint

| Family | Canvas | Source anchor | Target visible height | Accepted height band |
|---|---|---|---|---|
| Small tree | 128×112 | (64,100) | 80px | 64–96px |
| Large tree | 192×192 | (96,180) | 160px | 128–192px |
| Existing reused stump | 64×48 | (32,40) | 20px | 14–24px |
| Standard shrub | 96×96 | (48,80) | 32px | 24–40px |
| Herb, cereal or tall reed | Declared before export in lane manifest | Declared in same record | Declared for its anatomy and human scale | Preregistered, not adjusted after failure |
| Ground | Declared by ground manifest | Declared tile origin convention | Full-frame texture | Evaluated at 64px tile scale |

Source canvas anchors are not world footprint dimensions. Every manifest
declares a footprint separately and records any overhanging canopy. Do not
inherit a ground tile anchor for a taller flora canvas. No candidate is accepted
by silently changing its declared geometry after inspecting a failed check.

Reuse `art/flora/normalize.py`'s generic `ProductionSpec`, `plan_transform` and
`apply_transform`: alpha-threshold crop with recorded fringe, aspect-preserving
Lanczos resize and transparent placement. No local pixel repainting, outline
thickening, sharpening or palette correction is used to rescue a weak source.
Historical batch CLI entrypoints have fixed old output directories and must not
be invoked blindly against new assets.

Paired deciduous winter forms use the reference summer transform. Record trunk
and root silhouette comparison, base alignment and any remaining discrepancy;
identical normalization parameters alone do not prove identical trunk anatomy.

## State and season semantics

Canonical summer foliage follows the accepted palette:
`#507A38` mid, `#759C4B` light and `#30532B` shade. Species-natural differences
such as olive's muted leaves need a stated family rationale and visual review,
not a quiet exemption from an applicable check.

| Existing seasonal operation | Parameters or semantics |
|---|---|
| Spring foliage | Canopy-only D65 Lab delta `[6,0,6]` |
| Summer | Identity `[0,0,0]`, no color-space round trip needed |
| Oak autumn | Canopy-only `[-8,35,15]` |
| Birch autumn | Canopy-only `[8,10,32]` |
| Generic deciduous autumn | `[0,18,25]`, unless a documented species override is selected |
| Deciduous winter | Generated bare frame, keeping the summer trunk/root identity |
| Pine autumn | Same foliage as summer |
| Pine winter | Evergreen foliage, delta `[-8,0,0]`, top-envelope snow depth3px, alpha≤0.70 |
| Existing berry winter | Delta `[-8,0,0]`, no existing snow cap |

Seasonal derivation through the existing renderer is explicitly part of this
pass's interpretation of the brief's “matching the existing seasons work.”
Derived outputs are disclosed as derivatives, never counted as fresh image
generations. `art/flora/seasons.py` preserves all original alpha and unmasked
RGB; foliage masks exclude trunk and dark ink. Pine snow uses at most178/255
alpha after quantization and only the top envelope, not lower interior holes.
Do not double-apply seasonal tints to already rendered seasonal exports.

Harvested is not winter. A picked berry, olive, apple or grape loses its fruit
while retaining a living leafy structure. Cut cereals and reeds lose tops.
Wood-harvest oak/birch/pine use stumps. Olive and pine stay evergreen in winter;
oak, birch, apple and grape have dormant bare forms. Each shrub's winter habit
is declared rather than inferred from the ambiguous old berry filename
`berry_bush_bare`, which means fruit-depleted and leafy.

Ground's existing renderer has day/night/winter/drought conditions. Winter
reduces chroma to35%, optionally composes a separate snow overlay with alpha≤0.6,
then applies the declared scene contrast cap. B2 four-season ground exports
follow those references while remaining proposed selector data. Record whether
a winter output contains baked snow or expects a separate layer; never add both
unintentionally. Mild winter does not automatically mean heavy snow.

## Numerical and visual checks

Use existing checks without silently changing their thresholds:

- Canvas and declared anchor agreement; alpha foreground exists; no alpha≥16
  clipping on the canvas border. Use the existing alpha≥128 connected-component
  check where a connected tree or shrub is expected. Natural multi-stem stands
  must declare their morphology and report component counts honestly.
- `art/compositor/projection_check.py`: base row within anchor±2, declared
  height band and upper-left-light gradients. Zero samples are inconclusive,
  not a pass. Human inspection also checks the camera and readable contact.
- Tree summer mask checks: foliage hue in the accepted palette range,
  saturation≥0.30, lower30%/central40% trunk-box mask≤2%, and foliage-cluster
  coverage≥85%. This tree anatomy test is not presented as a meaningful herb
  or reed test.
- Ground opposite-edge mean RGB difference≤18, each axis≤2× its internal
  neighboring-pixel difference, plus native64px repetition and edge-adjacency
  review. `mirror_blend` is the historical last resort after two seam failures,
  not an automatic preprocessing step.
- Distinguish source texture restraint from runtime scene restraint. Existing
  source ground-v2 limits use luminance std≤0.6×person and saturation≤0.35;
  the current scene painter uses mean≤0.32, std≤0.25×person and saturation≤0.22.
  Report the specific contract used, not a composite claim from different bars.
- Reuse deterministic world-coordinate edge sampling. Ordinary blend width12px,
  water falloff10px, displaced bank ink and neighbor continuity follow the
  existing edge recipe. Do not claim independently seamless edge tiles unless
  their joined boundaries were actually checked.

Numeric passes alone do not establish adoption. Keep every failed result and
inspect normalized PNGs at1× and nearest2×, on light and dark grounds, alongside
accepted references and neighboring B2 family members. Preserve the three-attempt
per-state generation budget from the art line; disclose any requested self-review
regeneration as an additional recorded attempt within that budget. Do not reset
an asset's attempt counter by renaming it.

The review freezes a first completed set, names the weakest10–20%, regenerates
those assets with targeted corrections, and compares before/after. The final
selection ledger records why each replacement was adopted or rejected. Reports
must separate generated originals, normalized exports, derived seasons/edges
and existing reused frames; no visual or game-integration claim is made before
its evidence exists.
