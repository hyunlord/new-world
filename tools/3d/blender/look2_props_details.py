"""Craft construction details for the existing station identities.

Run via Blender's look2_props.py; no renderer or simulation calls.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import math

import common as C
import look2_props_shapes as H


def detail(m: C.Model, identity: str) -> None:
    """Add load-bearing joins and tool marks, keeping the same shared materials."""
    match identity:  # noqa: MATCH_OK — authored registry strings reject unknown IDs below.
        case "lean_to":
            for x in (-.85, .85):
                H.lash(m, (x, -.55, 1.56), .083)
        case "wood_bench" | "pottery_bench" | "casting_bench":
            for x in (-.6, .6):
                m.rod("wood_dark", (x, -.24, .12), (x, .24, .6), .025, segs=5)
            for x in (-.68, -.32, .32, .68):
                m.box("wood_dark", (.025, .025, .012), loc=(x, -.22, .778))
        case "drying_rack":
            for x in (-.85, .85):
                H.lash(m, (x, 0, 1.47), .07)
            for x in (-.6, -.42, -.25, .1, .28, .45):
                m.rod("thatch", (x, -.043, 1.48), (x, -.043, 1.39), .012, segs=4)
        case "loom":
            for x in (-.8, .8):
                H.lash(m, (x, .25, 1.79), .075)
            for z in (1.2, 1.3, 1.42):
                m.rod("thatch", (-.68, -.15 + math.sin(.22) * z - .048, z),
                      (.68, -.15 + math.sin(.22) * z - .048, z), .009, segs=4)
        case "storage_shelf":
            for x in (-.85, .85):
                for z in (.3, .85, 1.4):
                    H.lash(m, (x, -.22, z), .058)
            m.rod("wood_dark", (-.82, .245, .24), (.82, .245, 1.55), .03, segs=5)
        case "well_body":
            for x in (-1.1, 1.1):
                H.lash(m, (x, 0, 1.65), .095)
                for side in (-1, 1):
                    H.log(m, (x, side * .6, 1.85), (x, 0, 2.32), .045)
            for x in (-.15, -.09, -.03, .03, .09, .15):
                m.rod("thatch", (x, 0, 1.6), (x + .035, 0, 1.6), .092, segs=10)
        case "clay_oven":
            for x in (-.14, .14):
                m.box("clay", (.085, .13, .2), loc=(x, -.37, .2), jitter=.008)
            m.box("stone", (.37, .22, .05), loc=(0, -.36, .095), jitter=.01)
        case "hearth" | "hearth_unlit":
            for x, y in ((-.39, -.12), (.37, -.16), (0, .34)):
                H.log(m, (x, y, .05), (x * .18, y * .18, .92), .027)
            H.lash(m, (0, .01, .84), .06)
        case "bedroll":
            for x in (-.55, -.3, -.05, .2, .45, .7):
                m.rod("leather", (x, -.35, .055), (x + .035, -.31, .055), .009, segs=4)
        case "bee_skep":
            for z, radius in ((.4, .279), (.55, .26), (.69, .21)):
                H.rim(m, (0, 0, z), ("thatch", radius, .015))
        case "knapping_spot" | "grinding_stone" | "retting_pit" | "anvil" | "door":
            pass
        case "smelter" | "bloomery" | "kiln" | "forge" | "charcoal_clamp":
            pass
        case _:
            raise AssertionError("Unregistered station detail: " + identity)


def furnace_courses(m: C.Model, dimensions: tuple[float, float, float]) -> None:
    """Thin stepped clay rings show a hand-built shaft without extra materials."""
    height, lower, upper = dimensions
    for course in range(1, 5):
        t = course / 5
        radius = lower + (upper - lower) * t
        m.cyl("clay", radius + .012, .035, loc=(0, 0, height * t),
              segs=12, r2=radius, caps=False)
