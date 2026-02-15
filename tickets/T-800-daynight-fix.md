# T-800: Day/Night Transition Fix + N Key Toggle [Critical]

## Problem
- 1x speed: day cycles in 2.4s (too fast)
- 10x speed: color flickers rapidly (eye strain)
- Night too dark (Color(0.4, 0.4, 0.6))

## Changes
1. **main.gd**: Add lerp interpolation for day/night color
   - Slow lerp (0.3 * delta), even slower at high speed (0.05 * delta when >3x)
   - Store `_current_day_color` state variable
2. **main.gd**: Softer night colors
   - Night: Color(0.75, 0.75, 0.85) instead of Color(0.4, 0.4, 0.6)
   - Sunset: Color(0.95, 0.9, 0.85)
   - Dawn: Color(0.9, 0.9, 0.95)
3. **main.gd**: N key to toggle day/night on/off
   - `_day_night_enabled: bool = true`
   - When off: `world_renderer.modulate = Color.WHITE`

## Files
- scenes/main/main.gd

## Done
- [ ] No flicker at 10x speed
- [ ] N key toggles day/night
- [ ] docs/CONTROLS.md: N key added
- [ ] docs/VISUAL_GUIDE.md: color values updated
- [ ] docs/GAME_BALANCE.md: lerp speed noted
- [ ] CHANGELOG.md updated
