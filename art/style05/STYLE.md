# Style 05 bible v2: ink and tone

Status: TRIAL2b rules amended 2026-10-22. Six original images remain locked. The earlier TRIAL2 model-prompt V2 calibration is historical and is distinct from the new TRIAL2b numeric judge v2. New calibration and acceptance are pending; this document does not certify candidates.

## Authority and medium

Use [anchors/ANCHORS.md](anchors/ANCHORS.md) and the byte identities in [anchors/manifest.json](anchors/manifest.json). The source direction is [concept 05](../concepts/05_ink_and_tone/CONCEPT.md); its [log](../concepts/05_ink_and_tone/LOG.md) preserves camera failures and retries.

Fine controlled brown-black ink sits over restrained earth-coloured washes, like an observed natural-history plate. Outer contours are stronger than interior marks. Bark uses broken longitudinal marks, reeds thin parallel strands, stone irregular cracks and small tonal facets, hide sparse broken edges and fur tufts. Retain a broad base tone plus a distinct shadow tone. The originals contain mottling and local variation; do not describe them as mathematically two-colour images. Avoid continuous dense cross-hatching over every surface.

The person and portrait have particularly clear contour-to-detail separation. The ground original is visibly denser than the sparse-detail prompt intended. Use its materials and colours without letting ground detail outrank a person or work action. Hierarchy: people/events, then items/work spots, then walls/floors, then ground.

## Historical TRIAL2 measured control sheet (superseded as an acceptance instrument)

The following measurements preserve the original record. Global Lab histogram thresholds are withdrawn for TRIAL2b; use family-specific calibration in `docs/v8/arttrial2/JUDGE_CALIBRATION.md` when verified. Do not apply this historical table to new acceptance decisions.

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

Amended 2026-10-22 (TRIAL2b, D11): the map retains a vertical overhead camera and plan geometry for buildings. A slight front-face cheat is allowed on tall walls and props. People instead use front three-quarter view from about 20° above, with visible face and chest. Feet touch the tile's lower third with a soft contact shadow. At 56 px, the head is approximately 30% of height; shoulders are wide, limbs simplified, hands mitts and feet wedges. A generated 2-px dark ink contour must make the figure readable at 48 px; do not force outlines in post-processing. Tunic/hide and hair each form a clear shape. South is front, east is side three-quarter, west is flipped east; north is optional. The axe head is head-sized; carried logs cross the shoulders and exceed body width. The old overhead-person rule is withdrawn, not an alternative camera.

One tile is 64 px at 1× and a composed adult is approximately 56 px tall. Review at 0.5×, 1× and 2×. The scene anchor's people were recorded as roughly 70–90 px in the concept log: that image anchors the style, not compliant game scale. Do not inherit its size error.

## Background and framing

Cut-outs: uniform solid `#FF00FF`, no ground plane, no magenta within the subject, entire object and held items inside frame with margin. This trial's cut-out requirement supersedes the concept study's natural-ground default. Chroma-key removal and recorded edge clean-up are permitted; painted contour replacement is not.

Ground: full-frame seamless tileable natural material, no magenta, vignette, frame, label or isolated object. Verify a 3×3 repeat and edge-difference metric. A seamless-looking anchor is not proof of seamlessness.

Amended 2026-10-22 (TRIAL2b, D12): calm, low contrast, soft wash, few small details, no flowers, no scattered stones; detail only at large scale. Ground alone permits a recorded global tone adjustment: luminance standard deviation ≤60% of the foreground-masked person anchor's, mean HSV saturation ≤0.35. Record raw and adjusted values and normalization/masking. Prompt seamless tiling first; mirror-blend is a disclosed last resort. Ground detail must remain below people and actions in the visual hierarchy.

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
CAMERA: map and buildings straight overhead; people front three-quarter from
20 degrees above with 30%-height heads, clear faces, simplified limbs and
strong silhouettes readable at 48px. Slight front-face cheat for tall objects.
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

고정 원본의 잉크와 절제한 담채, 좌상단 광원을 유지한다. TRIAL2b부터 지도는 탑다운, 사람은 약 20° 내려다본 정면 3/4이며 머리는 키의 약 30%다. 타일 64px, 성인 56px, 48px에서도 사람 실루엣이 보여야 한다. 바닥은 낮은 대비와 넓은 담채를 쓰고 인물 대비의 60%, 평균 채도 0.35 상한을 측정한다. 양자화·외곽선 강제 보정은 금지한다. 기존 전체 장면 Lab 분포 판정은 철회하며 가족별 v2 수치 판정과 별도 모델 판정을 모두 통과해야 채택한다.
