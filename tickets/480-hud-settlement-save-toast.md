# T-480: HUD Settlement Info + Save Toast

## Action: DISPATCH (Codex)
## Files: scripts/ui/hud.gd
## Depends: T-410

### Changes:
1. Add settlement_manager reference to init()
2. Pop label: show settlement breakdown if settlements exist
   - "Pop:87 (S1:52 S2:35)" format
3. Save/Load toast notification:
   - Connect to SimulationBus.ui_notification signal
   - Show centered label at top for 2 seconds, then fade
   - "Game Saved!" / "Game Loaded!" style

### Implementation:
- _settlement_manager: RefCounted
- _toast_label: Label (centered, large font, auto-hide timer)
- _toast_timer: float (countdown in _process)
