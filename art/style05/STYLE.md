# Style 05 bible v2: ink and tone

Status: visual rules grounded in the six locked originals; numeric measurements are recorded below; image-model calibration is in progress. This document does not certify any candidate as accepted.

## Authority and medium

Use [anchors/ANCHORS.md](anchors/ANCHORS.md) and the byte identities in [anchors/manifest.json](anchors/manifest.json). The source direction is [concept 05](../concepts/05_ink_and_tone/CONCEPT.md); its [log](../concepts/05_ink_and_tone/LOG.md) preserves camera failures and retries.

Fine controlled brown-black ink sits over restrained earth-coloured washes, like an observed natural-history plate. Outer contours are stronger than interior marks. Bark uses broken longitudinal marks, reeds thin parallel strands, stone irregular cracks and small tonal facets, hide sparse broken edges and fur tufts. Retain a broad base tone plus a distinct shadow tone. The originals contain mottling and local variation; do not describe them as mathematically two-colour images. Avoid continuous dense cross-hatching over every surface.

The person and portrait have particularly clear contour-to-detail separation. The ground original is visibly denser than the sparse-detail prompt intended. Use its materials and colours without letting ground detail outrank a person or work action. Hierarchy: people/events, then items/work spots, then walls/floors, then ground.

## Measured control sheet

Measurements must come from the locked image bytes, with method, normalization, masking and per-anchor results recorded in `docs/v8/arttrial2/JUDGE_CALIBRATION.md`. Do not fill these cells with visual guesses.

| Control | Current evidence | Use |
|---|---|---|
| Mean ink stroke width, px normalized to 1024² | 2.1401–2.2429 px; mean 2.1961 px (six anchor means; edge-constrained dark-ridge estimator) | Match fine contours; distinguish the estimate from Canny edge density. Preserve scene aspect ratio and report scale normalization. |
| 24 dominant colours in Lab | [`palette_05.json`](palette_05.json), [`palette_05.png`](palette_05.png): measured Lab centroids of 24 most populated Lab bins | Judging reference only; never quantize generated images. |
| Lab histogram distance threshold | χ² ≤ 0.1327304646; each fold rebuilds its palette from only the other five anchors | Every held-out anchor must pass against the other five. |
| Canny luminance edge-density range | 0.1722552910–0.4108764802 (anchor extrema ×0.7/×1.3; 256 px longest-edge comparison) | Fraction of edge pixels in evaluated area; same preprocessing for candidates. |
| Mean local saturation range | 0.4352504445–0.5352829956; mean 9×9 local HSV saturation | Wash proxy, not recovered paint opacity. |

Wash appearance: a light-to-moderate colour veil leaves ink readable; broad lit skin and hide planes remain quieter than hair, fur and bark. **Physical paint opacity is not recoverable from flattened RGB originals.** Do not invent an alpha percentage; use measured saturation and direct anchor comparison to control the rendered wash. Alpha for a composited sprite, shadow or UI wash is a separate implementation parameter.

The visible palette spans sepia and charcoal contours, parchment/bone highlights, sage/olive vegetation, slate-grey rock and water, and amber fire. Berries can have small dark blue/red accents. Fire remains the warmest scene accent. The numeric 24-colour sample must retain actual measured values instead of converting these words into an invented palette.

## Light, camera and scale

Upper-left light; cast shadows generally fall down/right. The hut and scene show local amber hearth illumination without replacing the common daylight direction. Keep a consistent light model across independently generated pieces.

Map subjects use a vertical overhead camera: crown, shoulders, hands and feet on people; plan geometry for buildings; no horizon or perspective convergence. A slight front-face cheat is allowed on tall walls and props to explain thickness. The hut shows wall tops and restrained inner wall faces; the lean-to roof hides its floor. Do not broaden that allowance into a frontal shelter or isometric scene. Only the inspector portrait uses a three-quarter front view.

One tile is 64 px at 1× and a composed adult is approximately 56 px tall. Review at 0.5×, 1× and 2×. The scene anchor's people were recorded as roughly 70–90 px in the concept log: that image anchors the style, not compliant game scale. Do not inherit its size error.

## Background and framing

Cut-outs: uniform solid `#FF00FF`, no ground plane, no magenta within the subject, entire object and held items inside frame with margin. This trial's cut-out requirement supersedes the concept study's natural-ground default. Chroma-key removal and recorded edge clean-up are permitted; painted contour replacement is not.

Ground: full-frame seamless tileable natural material, no magenta, vignette, frame, label or isolated object. Verify a 3×3 repeat and edge-difference metric. A seamless-looking anchor is not proof of seamlessness.

## Reference attachments and prompt template

Every generation, including a retry or edit, attaches at least two distinct locked originals: the nearest family anchor plus `scene.png`. Ground/plants use ground; buildings/walls use hut; small props/items use props; rig parts/overlays use person; portraits use portrait. The scene is always the shared lighting and material reference. For scene generation attach scene plus the nearest subject anchor.

```text
Create [subject, exact count, material, footprint and required working size].
I have attached TWO LOCKED STYLE ANCHORS: [nearest anchor filename] and
scene.png. Match their ink medium, contour/detail balance, restrained washes,
palette and upper-left lighting. They are style references, not content to copy.
Illustrated 2D with fine controlled ink line work, restrained colour washes and
two-tone shading, like a carefully observed natural-history plate. Describe
bark, reed, stone and hide through selective marks, never dense all-over
hatching. Give major silhouettes slightly stronger ink than internal detail;
keep people distinct from the ground. Fire is the warmest point wherever present.
CAMERA: straight 90-degree overhead, crown-dominant people and architectural
plan geometry; only a slight front-face cheat for tall objects.
[Cut-out: isolated on solid #FF00FF; no ground or magenta on the subject;
all parts separated, uncropped and with margin. OR ground: full-frame seamless
tileable material with no border, no vignette and no isolated props.]
[Family constraints: south opening/work side, exact item count, separated rig
parts, neutral tint masks, tile connections, or portrait camera exception.]
Stone-age hide, bark, logs, reeds, stone, bone, ochre and charcoal.
No isometric view, perspective horizon, photo, photorealism, 3D render,
pixel art, gradient blur, dense all-over hatching, metal, modern items,
text, lettering, watermark or UI.
```

The first labelled rig-parts sheet is the sole task-directed text exception: use only required part labels in disposable margins, never on the extracted body parts. Record their removal as cropping. Do not copy labels into the runtime sprites.

## Acceptance

Run numeric checks and a separate fresh-context visual judge shown both anchors and the candidate. Require medium, line, light and palette each ≥4/5, a one-sentence explanation and numeric PASS. Neither signal alone certifies acceptance. Record the actual reviewer mechanism; a general vision review is not silently equivalent to the requested image-model judge. Calibration requires six anchor passes and rejection of woodcut, painterly and photo-like controls. Missing evidence stays PENDING/NOT_VERIFIED.

## 한국어 요약

고정 앵커 6장의 가는 잉크 외곽선, 절제한 흙빛 담채, 좌상단 광원을 유지한다. 지도는 수직 탑다운이며 높은 물체의 작은 앞면만 허용한다. 초상화만 정면 3/4 시점이다. 모든 생성과 재시도에 해당 가족 앵커와 캠프 장면 앵커를 함께 첨부한다. 바닥 원본은 세부가 조밀하고 캠프 원본의 인물 크기도 목표보다 크므로, 이를 게임 크기·가독성 합격 근거로 쓰지 않는다. 타일은 64 px, 성인은 약 56 px이다. 선폭·Lab 24색·담채 대리 지표는 위 표에 실제 측정값을 기록했으며, 팔레트 양자화나 외곽선 강제 보정은 금지한다. 수치 판정과 별도 시각 판정이 모두 통과해야 채택한다.
