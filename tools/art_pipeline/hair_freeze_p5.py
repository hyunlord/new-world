# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by merge_p5_selection after installer dry-run.
"""Freeze pass-five approved envelopes without altering pass-four evidence."""
from hair_freeze_p4 import HairFreezeRequest, freeze_selected_hair as freeze
from install_v4 import InstallPlan


def freeze_selected_hair(request: HairFreezeRequest, plan: InstallPlan) -> int:
    """Use the same fingerprint gates with a distinct overriding registry."""
    return freeze(request, plan, production_pass=5)
