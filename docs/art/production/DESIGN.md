# D3 production design system

## Direction
Bold minimal D3 only. Binding decisions in BRIEF sections3,4,6,7 and12. Ground uses transformed D3 ramps; no D1 or D2 assets.

## Palette
Production palette: tools/art_pipeline/production_palette.json,116 colours. Native actor colours remain unchanged; world night modulation never affects CanvasLayer UI.

## Typography
Body Galmuri11 24px, small numbers Galmuri9 20px, headings Galmuri11-Bold24px. No antialiasing,hinting,subpixel positioning or mipmaps. Korean default, Locale-only text.

## Geometry
World tile16px, texel1worldpx, nearest, zoom1/2/3/4/6/8. Sprite anchor feet/base. Shared ysortz4, shadowsz3. UI margin12,separation16,panelpadding12x8,border2.

## Panels
Flat warm dark background rgba(.12,.10,.08,.96), earth border(.42,.32,.22,1), warm foreground(.95,.88,.72,1). Part6 completes layout.

## Interactions
Ppause,Ttracking,1–4speed,Llanguage. Selected-only goal lines/trails. Needglyph at75 or above. Defaultzoom4.

## Verification
Headless node/data arithmetic plus real windowed captures and A1–A12 checks. No headless pixel claims.
