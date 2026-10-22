# Producing compatible characters

This is the Trial3 production procedure, not an acceptance certificate. Read
TEMPLATE.md and LAYER_CONTRACT.md before creating assets. Originals, tool output,
renders and logs belong in `/Users/rexxa/work/arttrial3-data/`. Keep accepted
working PNGs <=1.5MB each, GIFs <=2MB, total added art <=40MB.

## Add a body

1. Generate its exact template guide with `art/pipeline3/build_guides.py`.
2. Route A: edit the guide using person_v2 and scene as style references. Request
   a bald grey-skinned adult in a plain loincloth, magenta background, exactly
   on the boxes. Preserve the full canvas; never auto-crop/recenter the figure.
3. Key magenta, Lanczos downscale to working size, clean residual alpha only.
   Make the region mask, and retain every operation and input SHA256 in sidecar.
4. Run `python3 -m art.pipeline3.check_layer <sidecar.json>`. Failed layers go
   only to diagnostics; no taste judging and no normal boards.
5. Edit accepted S idle to E/N. Edit each direction to walk_a/b and work_a/b.
   Copy the idle head and torso back if needed, logging the exact rectangles.
   Draw sleep separately on96x64, head left, center48,36.
6. Derive child/elder using registered geometry; log transform parameters,
   recheck them against their own template. Derivation does not confer a pass.

Route B requires an authenticated Higgsfield workspace (or existing Meshy key),
rigged GLB and source clips, plus Blender4.x. Preserve GLB/clip provenance. Fit
the camera and model to the guide, not a post-render image warp. Render separate
object-id layers, masks from material ids, 3-band toon, 2px authoring Freestyle,
dark#141210 and paper-grain compositor. Record every manual Blender adjustment.
If the tools cannot run, mark B unavailable; do not substitute procedural dolls.

## Add an overlay

Author the top or hair directly on an accepted direction, full canvas. For tops,
ask for flat#C0C0C0 primary cloth; preserve silhouette ink and wash. Require
opaque interior, >=95% torso coverage and declared collar/cloak bounds. If
overlay-only fails, the goal permits declared mask-box extraction and hole fill
of the dressed render; it never permits image subtraction. Log the source and
box. Cloth masks use3/4, hair uses2. Check every body/frame combination.

## Add an item

Generate the object alone on magenta, with a visible connected silhouette.
Declare its grip pixel (or shoulder pixel for carry). Keep the full template
canvas. Region5 identifies the item. The checker places the anchor and rejects
clipping and gaps. Test simultaneous item/carry combinations too.

## Release evidence

Keep attempt counts across retries: A <=3 per layer, B <=3 render setups.
Never reset a budget by renaming a failed candidate. Stop a route below50%
structural pass after its tries. Run the route manifest through check_set and
retain CHECKS JSON/Markdown and all rejected-layer failure panels.

Only structurally passing layers enter the lab, GIF readability tests and style
review. Use two fresh blind reviewers per route, 12 GIF combos at each zoom,
and the exact registered scoring thresholds. One revision round is allowed;
rerun with fresh reviewers. Still-image scores cannot prove motion readability.
Measure200 walking for60 seconds in the actual graphical camp; headless results
are not P1. Report missing measurements explicitly. Recommend routes by the
registered S1/R1/P1 rule, with T1 and K1 tie-breaking, and leave selection to owner.

## Reproduce this trial

```bash
python3 -m pytest art/pipeline3/test_structure.py -q
python3 -m art.pipeline3.check_set art/character/route_a/manifest.json --output art/character/route_a
python3 -m art.pipeline3.capture_lab art/character/route_a/manifest.json /absolute/external/output all --allow-incomplete
python3 -m art.pipeline3.cost_report --frozen
```

The current manifest intentionally exits nonzero with NOT_COMPLETE: south fur
cloak was rejected at the three-attempt cap. `--allow-incomplete` captures only
the passing supplied subset and never converts minimum-set failure into PASS.
Omit that flag for a real complete-route release. All accepted originals and
derivations are SHA-bound; rerun checks after any sidecar or pixel change.

GIF stores timing in10ms units: eight samples alternate120/130ms for one
second, exactly8fps on average. Static sleep may be losslessly merged into one
1000ms frame. All full grid pages remain external when their aggregate would
exceed the40MB art budget; publish representative pages with the exhaustive
page index. Static or decoded-frame review is not timed-playback acceptance.
