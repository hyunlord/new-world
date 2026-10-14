# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by ingame_qa_observed_p8.py
"""P8 observer timing follows the explicitly revised windfall ring duration."""
from observation_rules_p6 import chill_window


def intervention_visible(stem: str, kind: str, elapsed: float, observed: list[str]) -> bool:
    """Ring remains full for1s, fades through1.3s; pile drop still ends at0.8s."""
    if stem.endswith('_panel_log'):
        return 'panel' in observed
    if kind != 'windfall':
        return kind in observed
    if elapsed < 0:
        return False
    dust = 'windfall_dust' if elapsed < 1.3 else 'windfall_dust_ended'
    drop = 'windfall_drop' if elapsed < .8 else 'windfall_drop_ended'
    opposite_dust = 'windfall_dust_ended' if elapsed < 1.3 else 'windfall_dust'
    opposite_drop = 'windfall_drop_ended' if elapsed < .8 else 'windfall_drop'
    return dust in observed and drop in observed and opposite_dust not in observed and opposite_drop not in observed
