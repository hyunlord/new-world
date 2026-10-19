# Pre-redesign activity measurement evidence

Inspected completed baseline `/Users/rexxa/work/art-production13-evidence/identity/six/A` read-only, including P1/P3 images via image viewer. No Godot/cargo or image edits. **Existing sidecars are insufficient for the complete section6.1 measurement.** They preserve useful screenshots but do not provide per-icon Z4 geometry or matched unobscured terrain samples.

## Actual evidence

- P1.png is1920×1080, SHA256 `f778623c1fff62578cdd1db096e75cdac9b39d3f1398b1efa85558d781e84e79`; P1.json records zoom4, seed20261010, tick40560. It visibly contains purple HearthCircle, yellow Play and cyan SitByFire icons above a tightly overlapping settlement. Grass, dirt and shelter art exist in frame; this alone does not identify the pixels behind each icon.
- P1/P2/P4/P5/P6 JSON keys include census/camera/tick/zoom/clock but no entities/icon rectangles. P1/P2/P5/P6 have zoom4; P4 zoom1. P3 JSON lacks zoom, but its matching `soc_d33_0949_FV_all.entities.json` explicitly records zoom[1,1], framebuffer_scale1, viewport1920×1080. P3.png and `soc_d33_0949_FV_all.png` are byte-identical.
- That P3 sidecar has1416 entity rows including10 activity_icon rows with rect/cue_rect/frame_size/region/sprite_id and visibility. **All10 have shown=false and visible=false** at Z1. Their16×16/18×18 rectangles cannot establish visible Z4 icon dimensions. Visible village summary icons in P3 are a different presentation and must not substitute.
- Pixel inspection of P1, limited to x650..1249/y120..789, finds exact main-color connected components. Purple RGB(214,163,224) has two isolated head components `[996,184,1016,220)` and `[1032,184,1052,220)`, each496 pixels,20×36 bounds. Yellow RGB(243,216,107) at `[864,204,920,256)` is2016 pixels,56×52 bounds. Cyan RGB(121,213,205) includes `[868,348,916,380)` at1120 pixels,48×32 bounds. These are **measured fill components**, not full icon bounds (outlines/highlights and disconnected silhouette pieces excluded); do not report them as complete badge or texture sizes.

## Source prediction, not capture measurement

Existing `activity_icons_p11.gd` generates16×16 textures with OUTLINE211C25, highlightFFF2D4, nearest filtering. `agent_activity.gd` applies scale1, offset(-8,-16); camera zoom4 predicts64×64 texture canvas. Craft U1_04 PNG is18×18, predicting72×72 if its inherited transform is scale1. These predictions agree qualitatively with P1 but still require actual transformed full geometry, clipping and overlap checks. Blank pixels count toward texture canvas, not pictogram dimensions. Source already includes highlight pixels; there is no rounded badge.

## Missing evidence and minimal next capture

Before editing icon art, use the existing activity capture harness at Z4 on the Part0/last-accepted Part1 build (fresh reset, exact tick), which can supply activity kind, agent identity, actual icon screen rect, frame size, texture/atlas region, visibility and capture transform. Retain original PNG+JSON hashes and associate each sample with seed/tick/person/kind. Primary scenes: live day30 at10:00 and15:00, seed20261018 day60 at10:00; supplement until all seven kinds are represented, preserving absent-kind outcomes.

For each kind record complete texture bounds and opaque pictogram bounds separately, with any clipping/occlusion. Obtain explicit annotated grass/dirt/shelter background patches from the same frame and record pixel rectangles and why they represent that terrain. A matched same-frame icon-hidden render would directly expose the underlay, but must preserve simulation, camera and draw clock exactly; otherwise use visible adjacent patches and label this approximation. Do not sample a shelter roof and call it dirt, or infer invisible under-icon terrain from nearby colors. The baseline sidecars lack terrain segmentation/underlay, so no three-background sample or contrast ratio is claimed here.

Once sample provenance exists, convert sRGB channels to linear values (`c/12.92` for c<=0.04045, otherwise `((c+0.055)/1.055)^2.4`), derive relative luminance0.2126R+0.7152G+0.0722B, and contrast `(max(Lfill,Lbackground)+0.05)/(min(...)+0.05)`. Predeclare whether background median means per-channel median RGB then luminance or median per-pixel luminance; use consistently before/after and report it. Retain source-main fill and actual screenshot fill if tint/modulation changes it. Badge85% alpha must be evaluated after compositing for the redesigned result. All seven×three background results, actual Z4 sizes and unavailable samples must remain explicit.

Status: partial baseline image inspection complete; full7-kind sizes and grass/dirt/shelter contrast **NOT_MEASURED**, awaiting targeted capture. No arbitrary background samples were invented.
