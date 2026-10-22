# Locked style 05 originals

These six accepted ART-CONCEPT originals are immutable style references. Do not crop, resize, recompress, retouch or replace these files. [manifest.json](manifest.json) records original paths, generation source paths, dimensions and full SHA-256 identities. Derived previews must use separate paths.

| Locked file | Source | Native size | What it anchors | Limitation retained |
|---|---|---|---|---|
| [person.png](person.png) | S1_2 | 1254×1254 | Crown-dominant map camera; stronger body contour; fine hair/basket ink; two-tone arm shading; upper-left light | Natural ground is not a cut-out; not a rig-parts source. |
| [hut.png](hut.png) | S4_1 | 1254×1254 | Roofless plan, south opening, wall thickness cheat, bark/rope/hide marks, amber hearth | Full composition is not an autotile or game-sized sprite. |
| [props.png](props.png) | S5_2 | 1254×1254 | Bark-roof occlusion, log end grain, stone/fire and berry detail; restrained earthy wash | Angled peripheral supports remain a mild face cheat, not permission for perspective buildings. |
| [ground.png](ground.png) | S6_1 | 1254×1254 | Water/forest/meadow/rock materials; olive, grey and earth ranges; botanical ink | Dense detail and four quadrants; seamlessness was not verified. |
| [scene.png](scene.png) | S7_3 | 1536×1024 | Shared material, wash and light across camp assets; hearth as warm focus; overhead layout | Concept log records people roughly 70–90 px rather than requested 64 px. |
| [portrait.png](portrait.png) | S8_1 | 1254×1254 | Face/hide/hair treatment, contour hierarchy, skin tones and wash; portrait continuity with map | Three-quarter front camera is portrait-only; requested size was not actual size. |

Every generation attaches the closest family anchor **and scene.png**. Both must be real image attachments, not only names in a prompt. For new scenes use scene plus a subject anchor. Record both paths and hashes in the asset sidecar. All retries keep these references; a previous candidate is an optional additional reference, never their replacement.

## Original-size exception and budget

The goal explicitly permits exactly these six raw 1024²+ accepted anchor originals in the repository. They therefore exceed the normal ≤1.5 MB per accepted working PNG limit. This exception does not extend to environment, person, autotile or comparison generation originals.

The six PNGs total **24,032,685 bytes** (24.033 decimal MB; about 22.919 MiB), measured from the locked files. The pass's ≤40 MB total `art/` growth limit still includes them. Conservatively interpreting that cap as 40,000,000 bytes leaves **15,967,315 bytes** before other new art files. Keep all other raw originals and rejects in `/Users/rexxa/work/arttrial2-data/`; commit only working-size accepted PNGs, boards, proto and documentation within the remaining budget.

These anchors establish reference provenance and appearance, not automatic judge calibration or production acceptance. See [STYLE.md](../STYLE.md) and `docs/v8/arttrial2/JUDGE_CALIBRATION.md` for measured controls and actual calibration status.
