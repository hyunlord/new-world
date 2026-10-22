# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline4.pawn_gen OUTPUT (tests: python3 -m pytest art/pipeline4)
"""Generator contract tests: inventory, determinism and rigid frame registration."""
from dataclasses import replace
from collections import Counter
import numpy as np
from .pawn_gen import PawnLook, iter_inventory, render_layer, registered


def test_factorized_adult_inventory():
    # Given the adult look dimensions. When enumerated as layers. Then no combinations.
    assert Counter(kind for kind,_,_ in iter_inventory(False)) == {'body':168,'hair':72,'top':24,'item':36}


def test_identical_inputs_produce_identical_pixels():
    # Given a fixed look/spec. When rasterized twice. Then RGBA and mask bytes match.
    look = PawnLook(item='stone_axe')
    first = render_layer('body',look)
    second = render_layer('body',look)
    assert all(a.tobytes()==b.tobytes() for a,b in zip(first,second,strict=True))


def test_body_motion_is_registered_idle():
    # Given fixed idle geometry. When every animation state renders. Then only registration changes.
    look = PawnLook()
    idle,_ = render_layer('body',look)
    for state in ('walk_a','walk_b','work_a','work_b','carry','sleep'):
        animated = replace(look,state=state)
        image,_ = render_layer('body',animated)
        assert np.array_equal(np.asarray(image),np.asarray(registered(idle,animated)))
