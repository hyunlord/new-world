# WorldSim interface system

## 1. Atmosphere and identity

A legible settlement ledger around the living map. Preserve the existing warm, dark panels and Galmuri pixel text; village activity remains the visual focus. M-UX exposes actions and explanations without replacing the established art direction.

## 2. Color

Source of runtime tokens: `assets/ui/worldsim_theme.tres` and `scripts/ui/panels/art_ui.gd`.

| Role | Existing token/value | Use |
|---|---|---|
| Panel | Theme panel bg `Color(0.063, 0.043, 0.012, 0.92)` | Map overlays |
| Border | Theme panel border `Color(0.161, 0.149, 0.129, 1)` | Two-pixel frame |
| Text | Theme Label font_color `Color(0.933, 0.918, 0.824, 1)` | Body and controls |
| Heading | Theme Heading font_color `Color(0.745, 0.549, 0.400, 1)` | Panel titles |
| Success | Existing proficiency green `8ee07a` | Completed state |
| Warning | Existing proficiency amber `d8a050` | Action required |
| Mark | Existing proficiency marker `f2e6c8` | Focus/threshold |

Use theme resources rather than copying palette constants into each component. Alert severity colors and minimap category colors belong in their data registries. Do not encode content-specific colors in GDScript branches.

## 3. Typography

Galmuri11 body and Galmuri11-Bold headings at 24 design pixels; Galmuri9 `SmallNumber` at 20. Keep nearest filtering and the existing 1920×1080 design viewport. Labels use `Locale.ltr`, and new strings exist in Korean and English. Button shortcut names come from `assets/ui/controls.json`.

## 4. Spacing and layout

Base spacing 4 design pixels; panel margins horizontal 12, vertical 8; row gaps 8 or 16; icons 32. Use Containers for child geometry. Bottom-right controls remain visible, minimap 200×200 above them, right-edge alerts above the minimap, bottom-left learning card above the order bar. Help opens from the top-right. Preserve a clear map area and avoid overlap at screen-fitting window sizes. UI scale settings adjust the interface consistently.

## 5. Components

Shared panels use `PanelContainer` with the WorldSim theme; content uses VBox/HBox/MarginContainer. Buttons have localized labels, shortcut-bearing tooltips, visible selected state, keyboard focus and disabled state where appropriate. Alerts show severity/count and explanation/fix; repeated activation cycles causes. A lesson card has progress and next/skip/off controls. No modal traps or input passing through a button into the world.

## 6. Motion and interaction

Camera zoom retains the ¼×/½×/1×/2× ladder and interpolates for 120 ms, anchored at the cursor. Click selection and right-click cancellation happen only when a drag was not recognized. Two-finger pan and right-drag preserve the current order tool. On-screen controls call the same camera APIs as keyboard shortcuts. Minimap data refresh is at most 2 Hz; its camera rectangle tracks camera movement each frame. Performance must be measured in a rendered window.

## 7. Depth and surface

Use the existing two-pixel border and dark translucent surface. Preserve map contrast behind panels; no decorative shadows, gradients or new branded styles. New UI must be inspected in real Godot windows at a laptop-sized fitting window and the capped desktop size. Browser/Lighthouse checks do not apply to this native Godot interface.
