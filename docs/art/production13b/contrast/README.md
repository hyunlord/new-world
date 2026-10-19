# Actual framebuffer badge contrast

**Coverage PARTIAL: 12 of 21 kind/background combinations measured.** All seven badge types have grass measurements; Craft, HearthCircle and SitByFire have dirt measurements; Craft and SitByFire have shelter measurements. No contrast pass threshold was invented.

| Activity | Grass | Dirt | Shelter |
|---|---:|---:|---:|
| TendFire | 3.3384:1 | NOT_MEASURED | NOT_MEASURED |
| Craft | 5.7405:1 | 3.4895:1 | 1.5297:1 |
| Repair | 2.2294:1 | NOT_MEASURED | NOT_MEASURED |
| HearthCircle | 3.0239:1 | 1.3714:1 | NOT_MEASURED |
| Explore | 4.9532:1 | NOT_MEASURED | NOT_MEASURED |
| Play | 2.7327:1 | NOT_MEASURED | NOT_MEASURED |
| SitByFire | 5.2206:1 | 3.1734:1 | 1.3912:1 |

[Visual sample review sheet](review-contact.png) · [Complete values, coordinates, RGB, masks and hashes](contrast.json) · [Frozen sample specification](samples.json)

The source images are current Pass13b windowed capture PNGs from the three base captures and supplementary attempts 1/2, with paths and SHA-256 in the report. Two full screenshots and all selected native-resolution crops were visually inspected. The first two available attempt3 capture contact sheets were also inspected; they did not establish the missing material combinations. No additional Godot capture was launched for contrast.

All 12 samples passed the existing capture schema/calendar validator and independent pose checker. Each sampled holder is unique, at Z4, icon-visible and inside the viewport; inherited draw-order geometry gives exactly zero foreign-sprite icon coverage. The draw extents are 80×80 pixels, except Craft at 88×88. These are source sidecar draw extents, not inferred opaque pixel segmentation.

Fill masks contain every matching actual main-colour framebuffer pixel inside the inspected badge rectangle. The matching RGB values were observed in those framebuffer images and their histograms; no source palette value was substituted. Highlight, plate and transparent pixels are excluded. Every selected background consists of a preserved 5×5 pixel patch: visible green open ground, flat tan settlement dirt, or a recognisable sloped shelter roof. Adjacent visible material is an explicit proxy for the material hidden under a badge. This does not reconstruct occluded terrain or establish that all 21 combinations occur naturally. The sample review text explains each classification.

The marked contexts were inspected after measurement: red masks follow the pictogram main fill; cyan marks fall on the described material. Source screenshots remain unchanged. Each full-frame binary mask and native-resolution review crop is retained alongside the report. Context offsets, every sampled RGB and coordinates, and input/output hashes are preserved. Tool exits 0. Black/white WCAG reference was previously checked at exactly 21:1.

Some visible badges belong to holders with `until_tick=-1`. Their rendered textures remain legitimate framebuffer contrast evidence. **This does not make those holders eligible for stage-B crops:** its unchanged active-holder rule is independent, and no stage-B selection was modified.

The tool reports channel-wise median RGB followed by standard sRGB linearisation and relative-luminance contrast. Shelter samples illustrate why a bright pictogram can have low contrast to a bright roof even though the dark badge plate aids recognition; this measurement deliberately follows the requested fill-versus-habitat definition. It is not fill-versus-plate contrast.

Nine combinations remain NOT_MEASURED: TendFire dirt/shelter, Repair dirt/shelter, HearthCircle shelter, Explore dirt/shelter, Play dirt/shelter. No distant unrelated surface or source-palette proxy was used to fill these cells. Overall contrast coverage must remain PARTIAL in H8 reporting.
