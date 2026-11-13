# R2 — colour-only B2 handoff

B2 source: `403caffd81174d4ecdc00b79d3dd2c465f1ab236`.
Shipped reference source: `f182769fe06b1cb60105e4d0bf127b6524277838`.
This branch contains candidate art and additive drafts, never runtime imports.

## Results and boundaries

- 41 herb/shrub images corrected; 26 shipped-tree seasonal derivatives retained
  pixel-identically. These 67 remain active **art candidates**, carrying B2's
  original missing model-review qualification, not a new production approval.
- 12 olive proposals corrected but blocked: all fail the existing classified
  foliage coverage check; one also retains its pre-existing trunk-mask failure.
  Original-mask summer hue passes all12; this does not waive the other checks.
- Eight latest B2 ground tiles corrected for review only. All still fail the
  existing wrap-seam check; no ground, season or edge is promoted to active import.
  Seven also exceed the legacy source saturation ceiling0.35 when matched to
  the more saturated shipped grounds; the frozen ceiling is not relaxed.
- No regenerated images. B2 generation budgets and missing states stay closed.

## Recorded transform

`colour.py` transports independent HSV channels through monotone linear knots
at q0/q10/q50/q90/q100, with duplicate knots merged by their target mean.
Constant channels map to the reference median. Hue is circular with its cut at
cyan; this earth/foliage family does not cross that cut. Values are rounded once
back to8-bit RGB. Alpha and RGB under alpha0 are byte-identical; no pixel moves,
resampling, morphology, outline editing, seam blending or palette quantization.
The input SHA, reference SHA, source revision and implementation fully determine
all knots; raw before/after measurements are outside the repository in R2.

`targets.json` records shipped HSV q10/median/q90 controls before comparison.
Each output is checked against the corresponding channel quantiles with0.025
absolute tolerance (hue turns, S/V unit interval), allowing small-population
quantile interpolation and8-bit rounding. This is a new colour diagnostic, not
an override of B2's existing style, anatomy or seam gates. Maximum observed error
is0.022636. Canonical tree seasons use the existing masked Lab renderer as their
reference and need no correction. Olive uses a same-size oak seasonal control;
its differing anatomy and failed mask checks remain explicit, never exempted.

No exact shipped cereal/legume/flax/grape family exists on the pinned base.
Reeds supply the closest grass control; berry foliage supplies leafy herbs;
shipped bare oak supplies dormant stem colour, without turning winter stems
into green leaves. Winter evergreen shrubs retain the existing Lab(-8,0,0)
operation. Proxy choices are disclosed per target; they are not claims that new
species existed in the shipped set. Full/harvested/winter shapes remain B2's.
B2's tree seasonal colours are baked: do not apply their tint twice.

## Import and reproduction

`art_import.draft.json` pins the immutable output commit. Merge its67 `files`
additively; never append `proposed_files` or rejected ground reviews. All paths,
anchors and footprints are explicit. Sidecars retain B2 metadata separately from
R2 hashes. Existing19 catalog references and all inherited gaps remain intact.
`bindings.draft.json` preserves proposed ID/origin semantics and points at this
handoff; selector integration, species/material validation and runtime enum
verification remain C2/C3 work. No olive wood material is invented.

Preserve source bytes under `/Users/rexxa/work/art-batch/R2/baseline/` by extracting
only `art/climate` and `art/flora/climate` from the pinned B2 commit. Original
B2 ground diagnostics and rejected64px working tiles remain under
`/Users/rexxa/work/art-batch/B2/grounds/`. Then run from the repository root:

```sh
uv run --with numpy --with pillow --with pydantic --with scipy --with scikit-image python -m art.climate_colour.export <ASSET_COMMIT>
uv run --with numpy --with pillow --with pydantic --with scipy --with scikit-image python -m art.climate_colour.boards
uv run --with numpy --with pillow --with pydantic --with scipy --with scikit-image python -m art.climate_colour.verify
uv run --with numpy --with pillow --with pydantic --with pytest python -m pytest art/climate_colour -q
```

The final export always reads frozen B2 bytes; reruns never stack transforms.
Pass1 olive Lab correction and pass2 wrong-family routing are retained externally
and superseded by pass3. The three-attempt cap is not reset. Herb dormant palette
routing was corrected on its second colour pass. Boards show native1x and exact
nearest2x before/control/after, with light/dark backdrops, by climate; unassigned
proposed herb origins have their own sheet rather than fabricated memberships.
External Korean report: `/Users/rexxa/work/art-batch/R2/REPORT.md`.

The modules own colour transport, typed handoff parsing, reference selection,
export, contact sheets, or verification respectively. No new project dependency,
game code or runtime configuration was introduced. Export is235 non-comment
lines; split its ground lane if future scope grows.
