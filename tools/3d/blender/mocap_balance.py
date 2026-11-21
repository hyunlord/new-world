"""Small corrective pelvis translations for a locked stance."""

from collections.abc import Mapping
from typing import Final

import numpy as np

from mocap_kinematics import FloatArray
from mocap_spec import Plausibility
from mocap_support import DYNAMIC_MARGIN_M, GROUND_CONTACT_M, centre_of_mass, contact_candidates, support_shift


# The VAT runs the checks again on the decoded post-hand-IK mesh. Target a
# small inward reserve for rigid-foot prediction and upper-limb contact drift.
INBOARD_RESERVE_M: Final = .007
# Feet stay put when the pelvis shifts, so COM moves less than the pelvis.
# The actual knee/torso pose is evaluated again after every correction.
COM_RESPONSE: Final = .80


def stationary_offset(
    names: tuple[str, ...],
    heads: FloatArray,
    tails: FloatArray,
    soles: Mapping[str, FloatArray],
    settings: Plausibility,
    *,
    phase: float,
    scale: float,
) -> FloatArray:
    """Return a neutral-metre XY pelvis correction around the given planted feet."""
    points, heights = contact_candidates(
        np.array([phase]), names, heads[None] * scale, tails[None] * scale,
        {side: sole[None] * scale for side, sole in soles.items()}, settings,
    )
    contacts = points[0, heights[0] <= GROUND_CONTACT_M]
    centre = centre_of_mass(names, heads, tails)[:2] * scale
    dynamic = any(first <= phase <= last for first, last in settings.dynamic)
    if not len(contacts):
        return np.zeros(2)
    if not dynamic:
        # Leave a reserve even when the predicted COM is just inside the hull.
        radial = contacts - contacts.mean(axis=0)
        radius = float(np.linalg.norm(radial, axis=1).max())
        if radius > INBOARD_RESERVE_M:
            contacts = contacts - radial * (INBOARD_RESERVE_M / radius)
    offset = support_shift(centre, contacts, DYNAMIC_MARGIN_M - INBOARD_RESERVE_M if dynamic else 0.)
    return offset / (scale * COM_RESPONSE)
