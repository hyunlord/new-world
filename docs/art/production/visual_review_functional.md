# Functional visual review A

Verdict: **REVISE — functional installation verified, visual acceptance remains partial.**

Read-only inspection of existing actual Godot captures; no Godot process or new capture was launched. Inspected `shots/a_d2_1200_F2.png`, `a_d2_2300_F2N.png`, `a_d2_1200_F1.png`, `a_d2_1200_F4.jpg`, and `a_calib_z4.jpg`, their available entity sidecars, renderer sources, shared theme, `ingame_qa.md`, and integrated gate results.

## Located findings

1. **A5 fails: villagers are hidden by the forest and two measured actors miss minimum contrast.** In the noon village image, tree crowns occupy most of the central interaction area; multiple villagers show only a head or a narrow body fragment. The QA records median contrast 0.2378 but minimum 0.0548 against required 0.10, actor 0 at 0.0884, and eight occluded/unmeasurable actors. The passing median cannot establish individual readability. This is visible in `shots/a_d2_1200_F2.png`, not solely a metric failure. Ground is calmer but vegetation coverage remains a readability limitation.

2. **A11 fails: activity is not visually distinguishable at the requested scales.** The actual noon and calibration images show repeated frontal figures holding a red chest-level shape. Different activity silhouettes are insufficiently distinct; the saved blind labels score 16.67% at both 4× and 2× against 80%/60% requirements. `ingame_qa.md` additionally lists build, drink, sleep and child as absent from the observation set. Their absence must remain unobserved, not pass. The real scene installation is useful, but is not accepted production-quality animation communication.

3. **Full gate remains red in the inspected evidence.** `gates/integrated/results.tsv` records `cargo_test=101`. Its log ends with four failures in `harness_fix_inspector_overlap` pinning the replaced inspector offset/width (204→124, 280→360). These may qualify for documented section-4.4 re-pins, but the current evidence does not permit a green-branch or all-tests-passing claim. Strict GDScript, smoke, Python and listed headless art checks have exit 0 in this same record.

## Verified strengths and boundaries

- This is a real scene composition rather than an offline mockup: renderer source reads agent, building, construction, terrain and resource snapshots; captured sidecars include live snapshot/entity rectangles and zoom. No simulation writes were found in the inspected art renderers or HUD counter paths.
- Shared `assets/ui/worldsim_theme.tres` and `scripts/ui/panels/art_ui.gd` provide Galmuri typography, square dark panels, shared spacing and palette-derived colors. The Korean noon/night HUD is readable. The inspector is separate from the news panel and leaves the central gameplay area clear in the inspected F4 frame.
- Night visibly darkens the world while panel text retains its brightness; the hearth has a warm local pool. The overview preserves the full map and sparse interface, though individual activities are too small to identify by this review.
- Calibration shows crisp native silhouettes, separate north/south shelter occlusion cases and visible palette variation. Automated framebuffer evidence reports exact ramp agreement for all twelve known variants at both calibration zooms, zero mixed checker pixels, and correct y-sort probe results. This supports technical pixel/ramp integration, not action recognition.
- Full capture accounting is reported as 92/92; this reviewer visually opened five representative surfaces, not every frame. Automated A8/A9 UI/metadata checks and performance p95 15.266 ms are reported by the QA pipeline, not independently rerun here.

Two asset correction rounds are already exhausted. Preserve the present result as an honest partial delivery with A5/A11 failures visible; do not relabel visual acceptance as PASS on the strength of format, calibration or median contrast results.
