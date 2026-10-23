# POLISH2 frozen face-variation plan

Frozen before portrait generation on 2026-10-24. Machine-readable authority:
`FACE_VARIANTS.json`, schema version 1. `route_i/MANIFEST.json` is authoritative:
**16 adult + 18 child = 34 unique accepted looks**, each with six variants (204).
The goal's “17 adult” count conflicts with the source inventory; no new source
identity is invented. The two missing adult long-tied looks explicitly use their
bun fallback's complete variant set, giving coverage for 36 logical looks.

## Adult table (every accepted adult look)

| ID | Skin | Hair | Age | Features | Men's beard |
|---|---|---|---|---|---|
| v1 | S1 light | H2 dark brown | young | narrow face, freckles | clean |
| v2 | S2 mid | H1 black | adult | broad nose, heavy brows | full |
| v3 | S3 deep | H3 auburn | adult | narrow face, scar | short |
| v4 | S1 light | H4 grey | older 55–70 | round face, freckles, age lines | full |
| v5 | S2 mid | H3 auburn | older 55–70 | broad nose, scar, age lines | short |
| v6 | S3 deep | H2 dark brown | young | round face, heavy brows | clean |

Women's beard is `none`. H4 is older-only. Skin targets do not encode identity,
ethnicity or ability; they are appearance axes only.

## Child table (every accepted child look)

| ID | Skin | Hair | Features |
|---|---|---|---|
| v1 | S1 light | H1 black | round face, freckles |
| v2 | S2 mid | H2 dark brown | narrow face, heavy brows |
| v3 | S3 deep | H3 auburn | broad nose, freckles |
| v4 | S1 light | H3 auburn | narrow face, scar |
| v5 | S2 mid | H1 black | broad nose, heavy brows |
| v6 | S3 deep | H2 dark brown | round face, scar |

All children retain child anatomy and no beard; no grey hair. Clothing and
hairstyle remain the original look's. Portraits add the described facial details;
the tiny sprites receive colour transforms only, never repainted features.

## Targets and transform contract

Targets live in `art/style05/palette_05.json:character_variant_targets`. Existing
judging-palette arrays are untouched. S1=(224,189,142), S2=(181,141,97),
S3=(113,74,47); H1=(44,32,20), H2=(83,66,45), H3=(139,73,43), H4=(160,155,140).
D65 sRGB ↔ CIELAB, one fixed `target Lab − source look Lab` shift per region and
variant, shared by all ten frames. Alpha, protected outline and all unmasked
pixels retain their exact original bytes. No resampling or outline recolouring.

- Baked: `route_i/<resolved-look>/<v1..v6>/<S|E>/<pose>.png`.
- Masks: `variant_masks/<resolved-look>/<S|E>/<pose>_{skin,hair}.png`.
- M-ART2 records: `variant_transforms.json` (source anchors, shifts, masks, hashes).
- Original `route_i/<look>/<S|E>/` assets and historical manifests remain intact.

TEMPLATE_V3 fixes the body canvas and contact geometry but does **not** define
face/head/forearm boxes and explicitly says it has no semantic masks. POLISH2
therefore records a separate reproducible anatomy-box detector and per-frame
box evidence; it does not attribute new boxes to the old template. Coverage is
reported against the recorded boxes, with transparent/outline pixels reported
separately. Source silhouettes can occlude forearms; missing evidence is a fail,
not an assumed pass.

## Person assignment

`h = (agent_id × 2654435761) mod 2^32`. Ordinary adults and children select from
their six variants at index `h mod 6` (zero-based). Adults aged ≥50 select from
ordered older pool `[v4,v5]` at index `h mod 2`. Child looks always use their child
pool. Fallback resolution happens before selection. This pure rule is implemented
in `art/compositor/face_variants.py`; no simulation or Godot code changes here.

## Predeclared review samples

Distinctness/owner board: take the first two lexically sorted accepted looks
from each of adult F, adult M, child F, child M (8 looks; all six variants each).
Identity: all six variants from owner-sample indices 0,1,2,4,6,7 (36 unique pairs),
covering both sexes, both life stages, all skin/hair/age/beard/feature axes.
Lists are frozen in `FACE_VARIANTS.json`. Randomized review packets must withhold
variant IDs, descriptions and answer mappings from reviewers where applicable.
