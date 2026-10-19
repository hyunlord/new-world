# Art Pass 13 presentation contract

## Direction
Retain the project-owned D3 world, native Godot pixel art and existing activity meanings. This extends production/DESIGN.md and the historical production11/activity/DESIGN.md; GOAL section6 supersedes old activity geometry and palette. No browser framework, dependency or unrelated UI redesign.

## Palette
Rounded badge: #211C25, alpha0.85. Exactly two opaque pictogram tones: per-kind main below and shared highlight #FFF2D4. Source originals stay untouched. Seven main hues have at least50degrees spacing (last-to-first60); metrics must calculate actual quantized RGB hue rather than accept planned values.

| Activity | Planned hue degrees | Main |
|---|---:|---|
| TendFire | 20 | #F5A076 |
| Craft | 70 | #E0F576 |
| Repair | 220 | #76A0F5 |
| HearthCircle | 270 | #B576F5 |
| Explore | 120 | #76F576 |
| Play | 320 | #F576CA |
| SitByFire | 170 | #76F5E0 |

## Typography
Retain Galmuri11 regular/bold24px for names and Galmuri9 count20px. Name scale may change only after Part3 actual glyph distinctness evidence. Existing UI_ACTIVITY_* localized names; tooltip adds only UI_TOOLTIP_PERSON_ACTIVITY_FMT.

## Geometry
Nearest filtering and integer source pixels. Pictogram canvas16×16 except Craft accepted18×18 shape; add2sourcepixels of badge margin each side. Badge corners round on the integer grid. World icon anchor centered horizontally and bottom-aligned above existing head offset. At Z4 the texture extents should be80×80 or88×88, but those predictions are not framebuffer measurements. Pictogram silhouettes exclude the shared badge for XOR/union>=25% over all21pairs. Summary uses same textures at2×, with rowheight at least44px to fitlargestbadge. Preserve existing summary count ordering and visibility gates.

## Components and states
TendFire log-on-flame; Craft originalU1_04 silhouette recolored to two tones onbadge; Repair hammer/plank; HearthCircle threeheads/flame; Explore footprints/path; Play ball/motionarc; SitByFire seatedfigure/flame. Distinct silhouettes carry meanings independentofcolor. World andsummary shareonecachedtexture API. Tooltip requires same visible person0.4s, immediatehideonmouse-out/noactivity; no selectionmutation.

## Motion
No new simulation or animation. All existing timers and activity decisions remain live. Tooltip delay0.4s only.

## Verification
Measure beforeediting. Current user prohibits allGUI windows: audited CLI-Anything headless checks permitted. Preserve prechange captures and exactsourcechecksums. Headlesstexture/transform metrics and generated icon sheet can support sourcechecks; labelthem explicitly. Fresh StageA reader gets shuffled actualZ4-sized icons/names only; frozenfirstresult and atmostone failingiconredesign. StageB needs realgamecropswithlegend andfreshreader, plusseparatenolegendreader. Do not claim missingrealcapture orperformanceevidence fromheadless output. No Lighthouse/browser checks applytothisGodotproduct.
