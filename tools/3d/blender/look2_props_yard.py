"""Append-only yard props, in metre units with Blender -Y as the front.

Build with Blender's tools/3d/blender/look2_props.py.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import math

import common as C
import look2_props_shapes as H


def cart(m: C.Model) -> None:
    """Pegged plank bed, splayed rails, spoked wheels and a pair of draw shafts."""
    for x in (-.42, -.21, 0, .21, .42):
        m.box("wood", (.19, 1.35, .09), loc=(x, .2, .66), jitter=.008)
    for x in (-.5, .5):
        H.log(m, (x, -.4, .58), (x, .85, .58), .055)
        H.log(m, (x, -.35, .59), (x * .8, -1.4, .45), .04)
        for y in (-.36, .78):
            H.log(m, (x, y, .64), (x * 1.04, y, 1.12), .04)
            H.lash(m, (x, y, .89), .05)
        for z in (.83, 1.03):
            m.box("wood", (.055, 1.28, .13), loc=(x, .2, z), jitter=.006)
    H.log(m, (-.84, .28, .42), (.84, .28, .42), .055)
    for x in (-.76, .76):
        points = [(x, .28 + math.cos(i * math.tau / 12) * .38,
                   .42 + math.sin(i * math.tau / 12) * .38) for i in range(13)]
        m.tube("wood_dark", points, [.055] * 13, segs=5)
        m.rod("wood", (x - .09, .28, .42), (x + .09, .28, .42), .08, segs=8)
        for i in range(6):
            a = math.tau * i / 6
            m.rod("wood", (x, .28, .42), (x, .28 + math.cos(a) * .36,
                  .42 + math.sin(a) * .36), .025, segs=5)
    H.basket(m, (-.2, .4, .71), h=.27, r=.22)
    H.pot(m, (.24, .2, .71), h=.36, r=.17)


def hay(m: C.Model) -> None:
    """Three tied sheaves with a ragged straw silhouette."""
    for x, y, angle in ((-.23, -.12, -.15), (.2, .09, .16), (0, .18, -.04)):
        for i in range(9):
            a = math.tau * i / 9
            m.rod("thatch", (x + math.cos(a) * .19, y + math.sin(a) * .19, .035),
                  (x + math.cos(a) * .08 + angle, y + math.sin(a) * .08,
                   .65 + .06 * (i % 3)), .045, segs=5, r2=.018)
        H.lash(m, (x + angle * .55, y, .4), .13)


def woodpile(m: C.Model) -> None:
    """Raised stack of split timber under a little weather cover."""
    for x in (-.65, .65):
        H.log(m, (x, -.5, .075), (x, .5, .075), .075)
    for level, count in enumerate((6, 5, 4)):
        for i in range(count):
            x = (i - (count - 1) / 2) * .22
            H.log(m, (x, -.52, .22 + level * .19), (x, .5, .22 + level * .19), .105)
    for x in (-.76, .76):
        H.log(m, (x, .45, 0), (x, .45, 1.05), .045)
    H.thatch(m, (1.8, 1.35, .35), .86)


def tool_rack(m: C.Model) -> None:
    """Tied timber stand with recognizable axe, rake and spade silhouettes."""
    for x in (-.75, .75):
        H.log(m, (x, -.25, 0), (x, 0, 1.3), .045)
        H.log(m, (x, .28, 0), (x, 0, 1.3), .04)
        H.lash(m, (x, 0, 1.2), .065)
    H.log(m, (-.9, 0, 1.12), (.9, 0, 1.12), .05)
    for x in (-.45, 0, .45):
        H.log(m, (x, -.2, .04), (x + .13, -.045, 1.22), .022)
    m.box("stone", (.19, .065, .14), loc=(-.32, -.045, 1.2), taper=.7)
    m.box("wood_dark", (.35, .07, .055), loc=(.1, -.05, 1.2))
    for x in (-.06, .02, .1, .18, .26):
        m.rod("wood", (x, -.05, 1.2), (x, -.17, 1.24), .014, segs=4)
    m.box("wood_dark", (.23, .06, .3), loc=(.45, -.18, .15), taper=.7)


def grain_baskets(m: C.Model) -> None:
    """Nested vessels and a basket of grain for a granary or drying yard."""
    H.basket(m, (-.25, 0, 0), h=.48, r=.29)
    H.basket(m, (.33, .13, 0), h=.3, r=.2)
    H.pot(m, (.1, -.3, 0), h=.33, r=.17)
    for i in range(6):
        a = i * math.tau / 6
        m.ball("thatch", .08, loc=(-.25 + math.cos(a) * .13, math.sin(a) * .13, .39),
               scale=(1, 1, .5), subdiv=1)


PROPS = {
    "yard_cart": (cart, (2, 3)),
    "yard_hay": (hay, (1.2, 1.2)),
    "yard_woodpile": (woodpile, (2, 1.6)),
    "yard_tool_rack": (tool_rack, (2, 1)),
    "yard_grain_baskets": (grain_baskets, (1.2, 1.2)),
}
