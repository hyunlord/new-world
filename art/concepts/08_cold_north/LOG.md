# 08 cold_north — generation log

Built-in session image generator. Originals byte-copied, never processed. Frozen reference: `S1_2.png`.

Total generations: 12; selected scene/portrait assets: 8; valid cutouts: 1/2. Summed tool wall time: 307.1 seconds.

## Consistency
S1/S2/S4 share faceted dry-brush medium, stark upper-left lighting and no separate ink contour; consistency risk: low.

## Production fit
Rig parts need isolated generation with fixed overhead anatomy and matching brush-scale; long blue ground shadows must be kept separate from body pieces. Strong rocks and directional grass can reveal tile repetition, so seamlessness needs actual edge testing. For 200 assets, freeze neutral rough-material samples and light direction; S1 reference alone can overconstrain compositions.

## Limitations
C2 one-shot magenta request failed: natural ground and four logs returned. Chroma key cannot extract a valid woodcutter; do not fake a two-person readability crop or retry beyond the once-only instruction. S7 people span roughly 64–90 px rather than exactly 64. Square outputs are 1254², including requested 512² portrait; originals retained at native size. One portrait camera rejection; no map-camera rejection.

## S1_1 — rejected

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-e0828e4a-a4bb-4f11-8d61-8f00a48d2a31.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 25.323 s
- Reference: `None`
- Camera/style/content verdict: Camera overhead and content correct, but photoreal surface detail overwhelms intended illustrated dry-brush medium.
- SHA-256: `44cc206fa629685f88775c15460dd9d8285ebcec61cd525160828f7632b37d1a`

```text
Use case: stylized-concept. Generate one 1024x1024 illustrated game art subject: a stone-age woman with a reed basket of berries, seen from directly above, standing on packed earth. True vertical overhead camera; see crown of head, shoulders, basket and feet; foreshortened body, not an eye-level or isometric figure. Full body centered with ample natural earth margin. Stone-age hide clothing, no metal or dyed cloth. No writing. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S1_2 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-aa655f03-bd2e-4a80-9f94-97da2d8d7bfb.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 23.676 s
- Reference: `None`
- Camera/style/content verdict: Frozen reference: visible broad painted facets, cool ground and long blue shadow; vertical view with crown, shoulders and feet; reed basket and berries clear.
- SHA-256: `355ae64664572fcb318172ec409c034ab2bfa46bd5cbe93f244529340b9b85b2`

```text
Use case: stylized-concept. Generate one 1024x1024 illustrated game art subject: a stone-age woman with a reed basket of berries, seen from directly above, standing on packed earth. True vertical overhead camera; see crown of head, shoulders, basket and feet; foreshortened body, not an eye-level or isometric figure. Full body centered with ample natural earth margin. Stone-age hide clothing, no metal or dyed cloth. No writing. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
CRITICAL MEDIUM: visibly hand-painted 2D illustration, broad coarse opaque strokes and flat simplified shapes, with rough broken drybrush edges. No photographic microdetail, no 3D render, no realistic skin pores. Blue slate shadows and ash grey earth. Simplify the crown, limbs and basket into clearly painted masses.
```

## S2_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S2_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-d63203c9-5681-49e6-aedc-63424b4cd15d.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 22.334 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Three log ends and stone axe distinct; overhead crown and feet; same painted facets, grey ground and long lower-right blue shadow as S1.
- SHA-256: `7910d516a6f6c4fdfda2cfaa1eeb44946b6b83f8400157c96a84c38c8d8ad0ca`

```text
Use case: stylized-concept. Generate one 1024x1024 image of a stone-age man with a stone axe held in one hand and exactly THREE logs balanced on his other shoulder, seen from directly above on packed earth. Full body centered with natural earth margin; clearly see crown of head, shoulders, three cut log ends, axe and feet. Rough simple hide clothing. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S3_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S3_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-faa0d634-59c3-4445-9216-f1ad0be3c8ee.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 25.811 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Elder with stick and small child legible from overhead; reference brushwork and shadow retained.
- SHA-256: `c6fe5bef75fa8ce098e6d90deee65de433bd75c4e1db63cd5e3f4d41873e8fdf`

```text
Use case: stylized-concept. Generate one 1024x1024 image: a stone-age elder leaning on a wooden walking stick and one child beside them, seen directly overhead on packed earth. Exactly two people, entire bodies visible, crown of heads and feet, foreshortened by vertical overhead camera. Simple undyed hide clothing. Natural ground margin. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S4_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S4_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-e8683bde-2439-48b5-b158-df2fc029bbec.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 22.035 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Four bedrolls, central amber hearth, south opening and roofless square overhead hut are explicit. Rough painted texture and lighting consistent.
- SHA-256: `01f71ae249e497b0dae69aa31eb44d0738d087ea5e2c8967c0ce6a7869e754a4`

```text
Use case: stylized-concept. Generate one 1024x1024 image: a small 3x3-tile square log-walled hut from TRUE VERTICAL ABOVE, roof completely absent. All four wall tops form an axis-aligned square, not a diamond. Visible packed earth interior contains EXACTLY FOUR separate hide bedrolls, two along left wall and two along right wall, plus one small stone hearth with amber fire in center. Doorway is centered on SOUTH/bottom wall, clearly open. Ground around hut. No people. No roof, no perspective/isometric view, no grid lines. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S5_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S5_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-75676bc0-11c4-4cb9-9500-ae2213078b95.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 23.953 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Four required props read separately from overhead; shelter includes front-face cheat. Amber fire anchors cool rough ground.
- SHA-256: `ae8f27b45b4eaa50eada24126b11ba2d8b1e519846cd35118337a45b7f13b900`

```text
Use case: stylized-concept. Generate one 1024x1024 image of four stone-age camp props spaced apart on packed earth, seen from directly overhead: one lean-to shelter made of branches and hide, one stone hearth with amber fire, one stacked log pile, and one berry bush with red berries. All four props fully within frame, no people. Camera looks straight vertically down, no horizon, only minimal front-face cheat on taller objects. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S6_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S6_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-3b1297d3-4f8c-4b00-b978-d7d5dae2fc94.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 27.555 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Correct four overhead quadrants: shallow water, leaf litter, actual green meadow, bare rock. No snow substitution. Strong textures may reveal repetition.
- SHA-256: `c147f0812f57fc68fb5a67a175a26946aa6ab49eec0c5c453b14d642fcbff10d`

```text
Use case: stylized-concept. Generate one 1024x1024 square texture swatch split into exactly FOUR equal quadrants with no labels or border: top-left river bank with shallow clear water and river stones; top-right forest floor under broadleaf canopy with fallen leaves, soil and sparse roots; bottom-left GRASSY MEADOW with desaturated natural green grass, no snow; bottom-right bare hill rock. True top-down seamless-looking repeating ground textures, no horizon, no people, no buildings. These are flat surfaces rather than scenic perspective vignettes. Cold palette does not mean snow, ice, conifers or winter. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S7_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S7_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-c90981b9-fa76-453e-b977-4b3e843e04ad.png`
- Requested size: [1536, 1024]; actual: [1536, 1024]
- Wall time: 41.636 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Eight workers visibly countable, three shelters, roofless four-bed hut, river and broadleaf forest. Overhead scene retains slight front-face cheat; blue shadows less dominant amid grass. People approximately 64-90 px, scale imperfect.
- SHA-256: `2ea672bd4c601824d8896f0a428199fcfcd9565da4b96899661262e302da6426`

```text
Use case: stylized-concept. Generate one wide 1536x1024 image of the whole founding stone-age camp seen TRUE VERTICAL TOP-DOWN like a map. A person is about 64 pixels long at requested image scale. EXACTLY EIGHT small people at work with clear separation: two beside river gathering, two near stockpile carrying logs, one tending hearth, one gathering berries, two beside hut working hides. One roofless square log hut with four bedrolls and south doorway, exactly THREE lean-to shelters, one central outdoor stone hearth burning amber, one stockpile of logs/stones, river bank along left edge, broadleaf forest edge along top. Natural packed earth clearing and green meadow grass, no snow, no conifers. All people from above showing head crown/shoulders/feet, not side-on. No text, borders, grid, montage. Wide landscape composition. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S8_1 — rejected

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S8_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-a93a5ae9-9fff-4a15-9f53-0522220353ce.png`
- Requested size: [512, 512]; actual: [1254, 1254]
- Wall time: 27.706 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Portrait remains elevated looking down; intended three-quarter front at eye level not achieved.
- SHA-256: `ab7941aa8f1ed7bce5e439d81995e7a1de54abb664346a9e34f6bdbf4dfd1213`

```text
Use case: stylized-concept. Generate one square 512x512 portrait for an inspector panel: three-quarter FRONT view of the stone-age woodcutter man's face and shoulders. This is the sole subject intentionally using a front camera. Dark brown tied-back hair, weathered masculine face, short beard, rough undyed hide shoulder clothing and simple bone necklace; no modern fittings. Cool unobtrusive natural backdrop. Same broad painted facets and upper-left stark lighting as reference; no axe/logs required in face portrait. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
```

## S8_2 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/S8_2.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-fe75fa01-4912-4adb-a6d9-2b340679346c.png`
- Requested size: [512, 512]; actual: [1254, 1254]
- Wall time: 22.594 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Eye-level three-quarter face/shoulder portrait corrected; painted facets and cool backdrop match reference.
- SHA-256: `a928845f4ed085ea66b83de097d7ef2a08ce19583f7460e0434041aab51bda2e`

```text
Use case: stylized-concept. Generate one square 512x512 portrait for an inspector panel: three-quarter FRONT view of the stone-age woodcutter man's face and shoulders. This is the sole subject intentionally using a front camera. Dark brown tied-back hair, weathered masculine face, short beard, rough undyed hide shoulder clothing and simple bone necklace; no modern fittings. Cool unobtrusive natural backdrop. Same broad painted facets and upper-left stark lighting as reference; no axe/logs required in face portrait. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI. CAMERA OVERRIDE: portrait camera must be eye-level, horizontally looking at face, not overhead. Eye line sits near upper third. Show front and side of face, NO top of hair crown, shoulders horizontal. Natural cool backdrop is abstract, not ground viewed downward.
```

## C1_1 — kept

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/C1_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-1fb2cb69-ba5a-49fe-89bc-0ef0395dea0f.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 21.455 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: One required extra generation, overhead full figure and basket on clean magenta, no ground patch or shadow.
- SHA-256: `9a0e5c4b1ce03fd12834fda6366979ce31738e2ec510394c01f450abc57fdf73`

```text
Use case: stylized-concept. Generate one 1024x1024 illustrated game art subject: a stone-age woman with a reed basket of berries, seen from directly above, standing on packed earth. True vertical overhead camera; see crown of head, shoulders, basket and feet; foreshortened body, not an eye-level or isometric figure. Full body centered with ample natural earth margin. Stone-age hide clothing, no metal or dyed cloth. No writing. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
CRITICAL MEDIUM: visibly hand-painted 2D illustration, broad coarse opaque strokes and flat simplified shapes, with rough broken drybrush edges. No photographic microdetail, no 3D render, no realistic skin pores. Blue slate shadows and ash grey earth. Simplify the crown, limbs and basket into clearly painted masses.
Additional cut-out variant: isolated on solid #FF00FF background. Replace ALL earth and scenery with uniform flat magenta, no ground patch, no drop shadow. Preserve full overhead woman, basket of berries and feet. Keep generous magenta margin.
```

## C2_1 — rejected

- Original: `/Users/rexxa/work/artconcept-data/08_cold_north/C2_1.png`
- Source: `/Users/rexxa/Library/Application Support/orca/codex-accounts/54be5844-95fd-4c46-925c-75327ccd186a/home/generated_images/01a15f4d-fa4e-7460-86a2-704535f5a589/exec-84e5fb85-9c80-4b94-801f-3424f9141ce1.png`
- Requested size: [1024, 1024]; actual: [1254, 1254]
- Wall time: 23.025 s
- Reference: `/Users/rexxa/work/artconcept-data/08_cold_north/S1_2.png`
- Camera/style/content verdict: Required one-shot magenta cutout failed: model retained natural earth and shadow and rendered four logs. Chroma key cannot extract it; preserve failure, do not retry beyond exactly-once constraint.
- SHA-256: `5d168135876bdde6cdbff8973a1427315d1c01e1f05622afcda43cd96e4a6abb`

```text
Use case: stylized-concept. Generate one 1024x1024 image of a stone-age man with a stone axe held in one hand and exactly THREE logs balanced on his other shoulder, seen from directly above on packed earth. Full body centered with natural earth margin; clearly see crown of head, shoulders, three cut log ends, axe and feet. Rough simple hide clothing. Input image 1 is the frozen S1 STYLE reference, not a composition to copy. Match this exact illustrated style, same lighting and material treatment. True straight-down vertical map camera for all subjects except portrait; never isometric or eye level. No text, watermark, UI, metal, dyed fabric or modern objects. STYLE: Illustrated painterly 2D with cool desaturated colours, long blue-grey shadows from stark upper-left light, rough stone, bark and hide textures, and restrained dry-brush marks. Keep people brighter than their ground and preserve readable tools. The hearth is the warmest point wherever present, a small concentrated amber contrast. Coldness is a palette and lighting interpretation only: do not add snow, ice, winter clothing, conifers or any content absent from the subject prompt. Preserve the exact camera: straight top-down map subjects with only a slight front-face cheat for tall objects. No photorealism, pixel art, text, watermark or UI.
Additional cut-out variant: isolated on solid #FF00FF background. Replace ALL earth and scenery with uniform flat magenta, no ground patch, no drop shadow. Preserve full overhead man, stone axe, exactly three logs and feet. Keep generous magenta margin.
```

## S7 reinspection before assembly

Re-opened the original 1536×1024 `S7_1.png` without processing. Crown-to-foot body spans are approximately 58–70 px: upper-left worker y≈195–255, stockpile worker y≈247–307, central carrier y≈260–329, hearth worker y≈445–505, right worker y≈438–496, bottom gatherer y≈789–849. These are manual visual estimates, not segmentation measurements. The earlier 64–90 px estimate included carried items and should not be read as body height. The requested approximately 64 px person scale is met. Eight people and three shelters remain countable; camera is overhead, with front-face cheat on shelter volumes. Keep S7_1; no additional generation.
