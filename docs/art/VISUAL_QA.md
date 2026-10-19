# Visual QA: review artifacts complete; production acceptance withheld

The requested surface is an offline art comparison with static Korean HUD composites. No browser, live UI interaction or game rendering claim is made.

|Check|Evidence|Verdict|
|---|---|---|
|Scene content and view|All nine noon/night/overview images directly inspected; narrow vertical river and east-bank camp present; night layouts retain noon relationships|Review-ready, generated grid scale remains approximate|
|Three palettes|Median cut, ΔE merge, hue/L* ordering; palette.hex and labelled swatches|31 / 48 / 19 colours|
|HUD construction|Reusable palette roles and nine-slice primitive in presentation.py; binary font masks, 12px/10px native fonts and nearest 2x|Pass for static composition|
|Korean precision|All six 1920x1080 composites inspected by lead and independent visual reviewer|No missing required content, clipping or overlap found|
|UI night isolation|Reviewer compared resource/feed/inspector interior crops between noon/night JPEGs|Byte-identical interiors; time/night text intentionally differs|
|HUD text/fill contrast|hud_metrics.json; linear-sRGB WCAG ratios|D1 7.00, D2 7.66, D3 6.04|
|Raw sheets|All selected and rejected sheets inspected; initial D2/D3 crossed grid boundaries|Replaced once each; selected sheets have zero edge contact|
|Sprite normalization|Actual CLI against all selected PNGs; contact sheets inspected|48/48 structural passes|
|Readability|1x/4x strips inspected and measured without corrective recoloring|D1/D2 fail; D3 passes 0.15 threshold|
|Semantic fidelity|Raw/normalized contacts show losses in berries, flame, faces, tools|Needs manual production correction|
|Comparison board|Three rows, six required columns opened and checked|Pass; read original files for fine text|

Review round 1 found a mechanical defect: majority downsampling could remove a thin bottom feature while the surviving silhouette remained above the baseline. The normalizer now trims sampled transparent bounds and aligns retained pixels before outlining; validation requires nonzero baseline contact. A failing regression established the defect, then nine tests passed. Independent round 2 repeated the original case and inspected all 48 outputs: fixed. No more review loops or image calls are needed for this comparison package.

Representative do/don't evidence: use D3/readability.png as the measured separation example; do not equate D1/D2 attractive F2 frames with their failing normalized readability. Do not treat the reddish berries in any raw contact cell as proof that exact 3/2/1/0 semantics survived palette quantization.

Scope remains art-only. Generation returned successful files, but tool model identity was not exposed. Scene texture density, animated movement, child/adult relative scale, native game camera, all terrain backgrounds, runtime performance and owner direction choice remain outside acceptance.
