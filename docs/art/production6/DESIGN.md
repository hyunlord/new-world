# Art Pass 6 presentation tokens

Inherits the project-owned D3 design system in `docs/art/production/DESIGN.md` without changing prior evidence.

## Palette
Frost roof #D8E8F0, frost underside/cold HUD #A8C0D0, ash #8A8A8A; night tint RGB(.80,.88,1.0) strength .12 maximum naturally, .18 only for an intervention cold snap. Rain #9FB4C8 alpha .5; gifts #7CC46A; spoil/rot #8A6A3A. Existing D3 production palette and nearest sprite sampling remain.

## Typography
Locale-only Korean/English. Inherit Galmuri11 24px and Galmuri9 20px; concise localized text buttons are the optional-icon fallback.

## Spacing and placement
Existing 16px world tiles, native pixel props min7x5, 1px outline; 12px HUD margins, 8px panel padding, 2px border. God's hand bottom-right, seven buttons in a compact grid, collapsed recent log.

## Motion
Frost fade .4 real seconds; doorway breath period2.4s, tint ease1s; morning memory90 game minutes, final30 fading. Rain120 screen px/s; windfall.6s; resource flash1.2s. All are read-only render state.

## Components
Reuse ArtUI panel styling; overlay frost and carry props share their owner's transforms/z order. Keep daylight modulation and clear the UI memory on reset.

## Surface verification
Native Godot windowed captures with the shared lock; headless scene tests exercise controls and effect lifecycles. This is not a browser surface, so web-only tooling and Lighthouse are inapplicable.
