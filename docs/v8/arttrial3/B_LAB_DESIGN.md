# Route B browser evidence design

## 1. Atmosphere and identity
A quiet, anonymous specimen viewer. It displays the emitted GIF bytes at natural
pixel dimensions; labels identify specimens only and never reveal activities.
This scoped document is the design contract for `browser_lab.*`.

## 2. Color
Tokens derive from `art/style05/palette_05.json` RGB entries: paper #e0bd8e
(index 21), ink #18110a (20), border #7d6950 (8), meadow #8a8242 (17),
forest #48442d (19). Ground switching changes the surrounding tile only:
opaque ground baked into a GIF remains unchanged. No asset recoloring.

## 3. Typography
System sans; body 16px/1.5; title 24px/1.25; specimen labels 14px/1.5.
No web fonts. Text wraps naturally; labels remain anonymous.

## 4. Spacing and layout
4px base: gap 16px, inset 16px, controls 8px. Flexible wrapping specimen grid.
Tiles keep the image's natural width/height, with overflow scroll on narrow
viewports rather than resize the evidence. Native scale is mandatory.

## 5. Components
Toolbar: labeled native selects for 1x/2x source set and surrounding ground,
and a labeled checkbox for tile outlines. Keyboard focus uses 2px ink outline.
Specimen: figure + native img + anonymous caption. Loading/error status uses
role=status. Missing images are reported, never substituted.

## 6. Motion and interaction
Only actual browser GIF decoding animates specimens; no seeking, canvas,
frame extraction, animation replacement or CSS transform. Playwright captures
at a target 8Hz and records actual monotonic timestamps and screenshot hashes.
The essential test animation remains enabled; no decorative motion exists.

## 7. Depth and surface
Border-only panels; no shadows, gradients, decorative icons or overlays.

## Evidence boundaries
The Python command publishes a local copy of the 24 supplied GIFs, SHA256,
dimensions and source paths. It fails before publication if any is missing.
Capture saves one viewport sequence per specimen at each source zoom, plus
375/768/1280px overview captures. Timing is measured, not assumed from target
8Hz. Distinct screenshot hashes show visible changes but do not score activity.
A static GIF is recorded as STATIC_OR_UNOBSERVED, not called animated.
The optional 200-image/60-second rAF probe is browser scheduling evidence only;
it is not Godot P1. Original Godot lab owns combinations and actual ground.
Final readability and visual approval belong to independent reviewers.

## Running
```
CHROMIUM_EXECUTABLE='/path/to/installed/chromium' /opt/homebrew/bin/python3 \
  -m art.pipeline3.browser_lab /Users/rexxa/work/arttrial3-data/b/lab/gifs \
  /Users/rexxa/work/arttrial3-data/b/browser --capture
```
`CHROMIUM_EXECUTABLE` is optional when Playwright's expected browser is installed.
The installed Python package expects Chromium 1208; this machine instead has
Chromium 1248, so the explicit executable override is required here.
Serve output with `python3 -m http.server --directory OUTPUT` for manual review;
opening HTML as a file cannot fetch its manifest. `--seconds` defaults to 3 per
specimen; `--crowd` adds the separately labeled browser scheduling probe.
Synthetic harness fixtures, when used, live in `b/browser-harness-fixture/`
and must never be presented as Route B render or readability evidence.

## Actual layer composite evidence
`b_lab_composites.py MANIFEST CHECKS A_LAB OUTPUT` requires the checker report's
input hashes to match the actual B manifest, sidecars and rendered pixels.
Failure leaves the complete evidence set labeled `DIAGNOSTIC_REJECTED`; it never
writes or overrides CHECKS. These are Pillow composites, not a Godot admission.

The compositor places already-positioned material passes without translating
prop anchors. The only added drawing is the lab shadow, centred at (32,88),
28×8 pixels. Original 1× PNG pixels are alpha-composited without correction;
2× uses nearest scaling. GIF palette conversion is limited to GIF transport;
all original RGB composite frames remain PNG. Idle and sleep use their actual
single rendered poses and therefore may show no visible motion.

The twelve samples use A's original activity schedule and available wardrobe
attributes, with bare sleep recorded as no top/hair. The 48 upright grid choices
are 2 tops × 2 hairs × axe present/absent × logs present/absent × 3 directions.
Both grounds are rendered, and pages stay at or below 1200×900 without shrinking
sprites. A/B comparison uses preserved A stills beside matched B samples.
Private answer keys stay outside the browser-served output directory; only
anonymous GIF copies are published by the browser harness.

### Idle playback clarification
The existing lab (§5.1, `CharacterLab.gd`) requires the idle two-frame bob.
Final composites therefore translate the complete idle figure plus its contact
shadow upward by one pixel on alternate frames. This is lab composition motion,
not a repaired render; source RGBA remains byte-identical. Sleep alone stays
static. Review page captions and alt text show only 01–12, not route names.
Screenshot contact pages crop actual measured browser screenshots at the DOM
image bounds; original full screenshots and timestamps remain the primary proof.

## Actual run verification (2026-10-22)
Final evidence is `b/lab/` and `b/browser/`, entirely `DIAGNOSTIC_REJECTED`
because the final checker passed 0/34 layers. Composite publication is not lab
acceptance. All 119 checker input hashes matched before composition and after
output creation. There are 24 GIFs, 192 original composite PNG frames, complete
48-combination coverage for each ground/zoom pair, and exact nearest 2× frames.
All owner board pages are at most 1200×900.

Chromium 156.0.8078.4 played all 24 actual GIFs: 576 full-page screenshots,
72 screenshot contact pages, zero JavaScript errors. Mean sample interval was
125.131ms; maximum was 159.181ms. Twenty GIFs showed visible change. Samples
05 and 09 at both scales were static sleep poses, recorded without pretending
animation. Activity/attribute/style scores were not assigned by the builder.

The anonymous reviewer packet is outside the browser and lab directories at
`/Users/rexxa/work/arttrial3-data/anonymous-playback-20261022/`. It contains only
numbered browser screenshot/contact pages and a neutral description. Public
browser manifests omit source filesystem paths; detailed provenance remains
in the non-reviewer lab evidence. Reviewers must not receive the private key.

Python compilation, JavaScript syntax, the programming no-excuse audit and
`git diff --check` passed. Fresh independent readability/visual review is still
required. Browser measurements do not establish Godot P1.
