"""P7 independent cold history and section-10 windfall timing rules."""
from observation_rules_p6 import chill_window


def intervention_visible(stem: str, kind: str, elapsed: float, observed: list[str]) -> bool:
    """Dust ends at 0.6 s; held pile drop ends at 0.8 s (GOAL section 10)."""
    if stem.endswith('_panel_log'):
        return 'panel' in observed
    if kind != 'windfall':
        return kind in observed
    if elapsed < 0:
        return False
    dust = 'windfall_dust' if elapsed < .6 else 'windfall_dust_ended'
    drop = 'windfall_drop' if elapsed < .8 else 'windfall_drop_ended'
    opposite_dust = 'windfall_dust_ended' if elapsed < .6 else 'windfall_dust'
    opposite_drop = 'windfall_drop_ended' if elapsed < .8 else 'windfall_drop'
    return dust in observed and drop in observed and opposite_dust not in observed and opposite_drop not in observed
