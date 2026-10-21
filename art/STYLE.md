# WorldSim B — one colony, one visual language

요약: 32px 정면 탑다운, 어두운 흙과 돌, 차분한 녹색, 따뜻한 나무·가죽을 사용한다. 사람과 사건이 가장 먼저 보이고 땅은 가장 조용해야 한다. 물체에는 1px `#141210` 외곽선, 재질에는 3–4단계 명암만 쓴다. 모든 생성물은 동일 팔레트·격자·외곽선 검사를 거친다. 가구별 첫 승인 이미지를 이후 생성의 참조로 첨부하며, 생성 성공을 승인으로 간주하지 않는다.

## Camera and grid

Straight top-down, camera looking down. No perspective, isometric projection or three-quarter view. Tall trees and hut walls may expose a south-facing sliver occupying 10–15% of height, never a dominant front elevation.

The base tile is 32×32 px. Standard boxes are 32×32; tall 32×48 with 16 px north overhang; 64×64; and 96×96. Tree exceptions are the requested 48×48 small and 64×64 large crowns. Upright people occupy 32×32, adults about 28 px high and children 24 px high. South sleep occupies 32×16. Anchors describe ground contact independently of the transparent padding. Never resize a body/layer independently to fit a box.

## Line, light and hierarchy

Object silhouettes have a single pixel dark `#141210` outline. Ground and stockpile-floor tiles have none. Light is upper-left. Small cast shadows extend 1–2 px down-right in a darker ground tone. Materials use 3–4 discrete values: no gradients, blur, anti-aliasing or dithering (except ground-edge transitions). Quantization alone does not prove compliance with these visual rules.

Read order: people/events, items/work spots, walls/floors, ground. Dark earthy ground, muted desaturated greens, warm wood and hide. The hearth is the only saturated hot-orange source. People carry the brightest skin/tunic values. Existing six household colours are retained as a deliberate readability convention, not claimed to be historical dyes.

## Materials

Hide, bark, round logs with visible end-grain, packed earth, river stones, thatch/branches and woven reeds. Stone age: stone tools with wood handles, no metal. Generated garments use ochre, charcoal and bone; six legacy household tints are applied only by deterministic composition as the documented exception above.

## Palette

`palette/worldsim_b.json` is the sole 48-colour master. Every nontransparent final asset pixel, including partially transparent derived sprites, must be in it. The palette build records input SHA-256 values, weighted Lab clustering and nearest-source-pixel snapping. Required semantic colours count within 48: outline, four greyscale tint channels, source hair/tunic colours and four skin colours (one sourced, three explicitly trial-derived).

No smoothing is used at reduction or display. A sprite may fail visually even when its palette score is 100%.

## Prompt template

```
Subject: {one object, material, distinguishing readable silhouette, required state}
Target: {width}×{height} px in {grid cells}; generate at integer enlargement
       (1024×1024 for a 32×32 subject at 32×; 512×512 at 16×).
Camera: straight overhead top-down; only 10–15% south face on tall objects.
Line/light: 1 target-pixel #141210 outline; upper-left light; 3–4 flat values
            per material; hard pixel clusters, no anti-aliasing.
Palette words: dark earthy, muted olive, warm wood/hide, quiet ground;
               {family-specific palette words or exact mannequin channels}.
Background: perfectly solid #FF00FF magenta.
Composition: single object centred; generous clear margin; no text,
             no watermark, no scene.
Negative: isometric, perspective, three-quarter view, gradient, blur,
          photograph, 3D render, metal, modern materials, extra objects.
Reference: {first accepted family reference attached}; match this exact style.
```

Ground exception: request a seamless tileable top-down 256×256 texture per material, no object outline. Test 4×4 repeats before accepting any crop. Frame-sheet exception: explicitly declare cells, directions and pose order; attach the accepted mannequin. People are modest neutral grey mannequins with a plain loincloth, bald, flat skin `#9A9A9A`, loincloth `#6E6E6E`; hair `#B0B0B0`, tintable top `#C0C0C0`. A layer variant preserves the exact same frame positions and body geometry.

## Reference and acceptance discipline

First accepted tree, building and person become immutable family references attached to every later prompt in that family. Before acceptance they are candidates only. Keep full prompts, returned dimensions, tool outcomes, raw originals, source hashes and rejected alternatives outside Git under `/Users/rexxa/work/arttrial-data/`. Record each rejection in one sentence. Never substitute code-drawn objects for failed generation.

Each generated environment asset receives at least two candidates. Select using pipeline checks plus direct visual reading; record both. Ground alone may fall back to the legacy sheet after two failed seam attempts, explicitly disclosed. Blueprints, canopy shadows and bank geometry are permitted deterministic derivatives. Synthetic circles/squares/mannequins belong only to tests, never the final art set.

Layers share the mannequin's crop and scale transform. Difference extraction uses ΔE >6 and a 1px close on the mask; new mask pixels copy real variant RGB. Reject drift rather than mistaking moved skin for clothing. Reference ΔE is a descriptive colour-distribution statistic, not proof of matched shapes or style.
