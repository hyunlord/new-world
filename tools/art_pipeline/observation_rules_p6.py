"""Pure rules for independent FFI cold history and transient-effect observations."""
from __future__ import annotations


def chill_window(now: int, last_cold: int, first_clear: int, duration: float) -> tuple[bool, bool]:
    """Return definitely/possibly eligible given the sampled clear-time interval."""
    if last_cold < 0 or first_clear <= last_cold or now < first_clear:
        return False, False
    return now - last_cold <= duration, now - first_clear <= duration


def intervention_visible(stem: str, kind: str, elapsed: float, observed: list[str]) -> bool:
    """A panel only proves its explicit shot; windfall must cease after 0.6 s."""
    if stem.endswith('_panel_log'):
        return 'panel' in observed
    if kind == 'windfall':
        if elapsed < 0:
            return False
        return ('windfall' in observed) if elapsed < .6 else ('windfall' not in observed and 'windfall_ended' in observed)
    return kind in observed
