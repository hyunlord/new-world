"""Deterministic tile and sprite sheet for the V8 M1 colony view (M1-ART).

Purpose
    Draws every tile and sprite the M1 GDScript renderer needs into one RGBA sheet, in code, after the owner's
    chosen mock-up (docs/v8/m1/mockup_gen_reference.py -> docs/images/v8_target_view_b.png): a dark, earthy,
    Dwarf-Fortress-like top-down grid, 16 px tiles, 1 person = 1 tile. Replaces the M1 placeholder generator
    (gen_temp_tiles.py). No external art, no randomness outside the seeded per-sprite generators.

Craft rules (also in .claude/skills/worldsim-godot, "Pixel-art craft rules")
    1. Every object sprite has a 1-px dark outline OUTLINE (20, 18, 16) on its silhouette; ground tiles have none.
    2. Three to four values per material (shadow, base, light, highlight); no gradients, no anti-aliasing.
    3. Light from the top-left; cast shadows fall down-right, 1 px, in a darker ground colour, never black.
    4. The ground is the calmest layer: variation comes from clusters (the renderer's value noise), never from
       per-tile random colour, so each ground variant is quiet on its own.
    5. Nothing on the ground competes with a person in value contrast: people carry the brightest values
       (skin, tunic) and the heaviest outline.
    6. Text on the map sits on a dark backing box (drawn by the renderer, not here).

Usage
    python3 tools/v8/gen_tiles.py [--out DIR]           # default DIR = assets/v8
    Writes DIR/tiles.png (RGBA) and DIR/tiles.json.

JSON schema (tiles.json)
    {
      "atlas": "tiles.png",
      "palettes": {"elder_hair": hex, "hairs": [hex], "halo": hex, "skin": hex, "tunics": [hex]},
      "size": [w, h],
      "tile_size": 16,
      "tiles": { "<name>": [x, y, w, h], ... }        # pixels in the sheet, keys sorted
    }
    Every sprite starts on the 16-px grid; a sprite larger than 16 px occupies whole grid cells. The renderer's
    TileSet uses only the 16x16 ground tiles, each alone in its cell.

Sprites (names; see the painters for the exact pixels)
    ground 16x16, opaque, no outline:
        grass_0..2 (family g), grass_dark_0..2 (family G, ~10 darker), forest_floor_0..3, dirt_0..3,
        dirt_edge_0..1 (grass dithered on the NORTH side; the renderer turns them with the tile transform flags),
        hill_0..2, mountain_0..2 (cracked rock), mountain_top_0..2 (lit top edge, used where the tile above is not
        rock), mountain_cliff (dark rock face, used where the tile below is not rock), water_0..3 (deep),
        shallow_0..3, bank_1..15 (water touching land; the number is the 4-bit mask N=1 E=2 S=4 W=8 of land
        neighbours, a sand lip on each such side), bank_in_ne/nw/se/sw (land only on that diagonal: a sand corner),
        stock_floor (tamped earth of a stockpile zone).
    zone parts 16x16, transparent: stock_edge_n/e/s/w (1-px darker border on that side), stock_stake_nw/ne/sw/se
        (a small stake in that corner).
    canopy: tree_<species>_<size> 24x24 for species oak/birch/pine and size 0..2 (drawn at the tile minus 4 px, so
        neighbours overlap into a mass; at 1x zoom the renderer draws size 1 without jitter or shadow);
        tree_shadow 20x8 (alpha 90).
    nature 16x16: stump, bush_full, bush_bare, deadwood.
    items 16x16: logs_s (1-8), logs_m (9-16), logs_l (17+).
    buildings: hut 48x48 (log walls, south doorway, earth floor, 4 bedrolls, hearth stone; no roof), hut_ghost
        48x48 (blueprint blue, alpha 80, dashed outline), hut_ghost_place 48x48 (greyscale; the order tool tints it
        green or red), shelter 16x20 (lean-to; drawn 4 px above its tile), hearth_0/hearth_1 16x16 (two fire frames).
    people (greyscale layers r == g == b; the renderer modulates skin, hair, tunic, halo; the outline layer is
        drawn last, unmodulated, and carries the outline, eyes and belt):
        person_<layer>_<frame>, child_<layer>_<frame> with layer skin/hair/tunic/outline/halo and frame 0 (stand),
        1 (stride), sleep. Adults and children are 16x16 cells with the feet on row 15 (a child's figure is 8x10,
        a big head on a small body);
        the sleep frames are 16x8 (lying, head to the left), drawn centred on the tile.
        shadow 10x4 (alpha 110), drawn under the feet (cell x + 3, y + 13).
        carry_back / carry_front 16x16: three outlined logs across the shoulders; carry_back (two logs) is drawn
        before the person layers so the head covers it, carry_front (one log, rows 7..11) after them.
        tool_axe / tool_hammer 16x16 cells with a 5x7 tool at x 11..15, y 5..11 (the right hand).
    marks: mark_chop 16x16 cell, a 7x7 yellow axe glyph at the top-right (x 9..15, y 0..6); select (1-px yellow
        rectangle), hover (1-px white, alpha 120).
    order bar icons 12x12: icon_chop, icon_stockpile, icon_hut, icon_remove, icon_cancel.
    M2 temporaries (section "m2"; M-ART2 replaces them): boulder and flint_outcrop 16x16 (resource nodes),
        knapping_spot 16x16, wood_bench and drying_rack 32x16 (workshops, drawn at the footprint's top-left),
        item_firewood / item_stone / item_flint / item_blade / item_axe / item_hide 16x16 (ground stacks of the
        new item kinds), mark_quarry 16x16 (a 7x7 pick glyph at the top-right, like mark_chop), icon_build and
        icon_quarry 12x12 (order bar).
    M3 temporaries (section "m3"; M-ART2 replaces them), all 16x16 unless noted:
        log_wall_0..15 and stone_wall_0..15: wall autotiles; the number is the 4-bit mask of wall-or-door
        neighbours (N=1, E=2, S=4, W=8, as bank_*). Every tile is a 10-px band through the centre (rows/columns
        3..12) with an arm to each connected side that runs to the tile edge without an outline, so neighbours join.
        Mask -> shape: 10 (E+W) = horizontal run (courses run east-west, no post); 5 (N+S) = vertical run (courses
        run north-south, no post); 0 = an isolated post; 1/2/4/8 = an end (post + one arm); 3/6/9/12 = a corner,
        7/11/13/14 = a tee, 15 = a cross (post + arms). The post is a 12x12 square (x, y 2..13): a cut log end for
        log walls, a squared cornerstone for stone walls. Logs: two per course, lit on the north/west side;
        stone: grey blocks in a running bond.
        door_h (in an east-west wall: log-end jambs at x 0..2 and 13..15 joining the wall arms, a plank leaf
        between them) and door_v (the same turned for a north-south wall).
        floor_packed (packed earth, opaque, a little darker than dirt, tamped dashes), floor_rock (dug rock floor
        inside a hill, opaque grey chipped stone); both are ground-like (no outline).
        bedroll (a 14x8 hide with a straw pillow to the west at x 1..14, y 4..11, like the hut's bedrolls).
        ghost_wall / ghost_door / ghost_floor / ghost_bedroll: blueprint ghosts in hut_ghost's style (blueprint
        blue, translucent, the 3-on-2-off dashed tile outline): hatched block; jambs + leaf + swing arc; dotted
        tamping lines; the bedroll's plan outline. ghost_refused: ONE red overlay for a refused tile of any kind
        (red fill, red dashed outline, a cross), drawn in place of (or over) the kind's ghost.
        mark_dig (yellow corner brackets around the tile plus a 7x7 diagonal pick glyph at the top-right) and
        mark_deconstruct (a 7x7 yellow X at the top-right), outlined like mark_chop; icon_dig and
        icon_deconstruct 12x12 (order bar).
    M4 temporaries (section "m4"; M-ART2 replaces them), 16x16:
        ford (a river tile people wade: lighter shallow water with a band of pale gravel and three flat stepping
        stones across it, rows 5..10, lit top-left with a 1-px wet shadow down-right; opaque, ground-like, no
        outline; drawn over the terrain's water tile).
        clay_bank (a reddish-brown river-bank mound with a dug face; a resource node like boulder), pottery_bench (a
        low log bench with clay and a forming pot), grinding_stone (a flat slab with a worn hollow and a hand stone),
        item_clay / item_raw_pot / item_clay_pot / item_ground_axe (ground stacks of the M4 item kinds); all outlined
        objects with a transparent background, appended after `ford` so every older region stays where it was.

Determinism
    Each sprite draws with its own random.Random(SEED * 1000 + k), k a fixed constant per sprite (no hash(), no
    time). The PNG is saved without metadata, so the same Pillow version gives byte-identical output on every run.
"""
import argparse
import json
import math
import random
from pathlib import Path

from PIL import Image, ImageDraw

T = 16
SHEET_W = 512
SEED = 7  # the mock-up's seed
OUTLINE = (20, 18, 16)
TRANSPARENT = (0, 0, 0, 0)

# mock-up style B palette (mockup_gen_reference.py make_style('B')); the on-screen grass is exactly g.
PAL = dict(g=(58, 74, 46), G=(48, 62, 40), d=(84, 70, 52), e=(98, 82, 62), w=(36, 58, 86),
           s=(52, 78, 96), R=(92, 86, 80), W=(112, 80, 50))
FOREST_FLOOR = (44, 50, 34)
HILL = (72, 76, 54)
ROCK = (60, 57, 52)
SAND = (122, 108, 80)
WET_SAND = (92, 86, 70)
LOG_DARK, LOG, LOG_LIGHT = (86, 58, 34), (118, 82, 50), (150, 108, 66)
GRAIN, GRAIN_RING = (190, 152, 104), (140, 100, 62)
CARRY_DARK, CARRY, CARRY_LIGHT = (176, 130, 80), (214, 168, 108), (240, 206, 150)
ACCENT = (250, 206, 60)

# Household tunics (household id mod 6), hair (agent id mod 4), elder hair, skin, and the halo tint. Tunics are
#   light or saturated so people stand out from the earth and the canopy (no dirt-brown, no grass-olive, no yellow:
#   the marks are yellow). The tunic layer's base grey is 212, so the drawn colour is about 0.83 x these.
PALETTES = {
    "tunics": ["efe2c0", "6fb0e8", "e0604c", "50c0a8", "b088e0", "e888b0"],
    "hairs": ["2a1e16", "5a3a22", "8a6a3a", "3a2a1e"],
    "elder_hair": "e8e8e8",
    "skin": "d8a884",
    "halo": "fff0c8",
}


def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c[:3])


def rgba(c, a=255):
    return tuple(c[:3]) + (a,)


def new(w=T, h=T, fill=TRANSPARENT):
    im = Image.new("RGBA", (w, h), fill)
    return im, ImageDraw.Draw(im)


def speckle(im, base, var, density, rr, box=None):
    """Small random deviations of `base` on a fraction `density` of the pixels (calm ground texture)."""
    px = im.load()
    x0, y0, x1, y1 = box or (0, 0, im.width, im.height)
    for y in range(y0, y1):
        for x in range(x0, x1):
            if rr.random() < density:
                k = rr.randint(-var, var)
                px[x, y] = tuple(max(0, min(255, base[i] + k)) for i in range(3)) + (255,)


def outline_alpha(im, color=OUTLINE, a=255):
    """Paint `color` on every transparent pixel 8-adjacent to an opaque one (the silhouette outline)."""
    src = im.copy().load()
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            if src[x, y][3]:
                continue
            hit = False
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < im.width and 0 <= ny < im.height and src[nx, ny][3] == 255:
                        hit = True
            if hit:
                px[x, y] = rgba(color, a)
    return im


def from_ascii(rows, colors):
    """An image from equal-length strings; '.' is transparent, every other char maps through `colors`."""
    h, w = len(rows), len(rows[0])
    im = Image.new("RGBA", (w, h), TRANSPARENT)
    px = im.load()
    for y, row in enumerate(rows):
        assert len(row) == w, (y, row)
        for x, ch in enumerate(row):
            if ch != ".":
                px[x, y] = colors[ch]
    return im


# ------------------------------------------------------------------ ground (calm, opaque, no outline)
def grass(base, rr, blades):
    im, d = new(fill=rgba(base))
    speckle(im, base, 3, 0.10, rr)
    for _ in range(blades):
        gx, gy = rr.randrange(1, 15), rr.randrange(3, 15)
        d.line([(gx, gy), (gx, gy - 2)], fill=rgba(shade(base, 1.14)))
        d.point((gx + 1, gy), fill=rgba(shade(base, 0.9)))  # its 1-px shadow down-right
    for _ in range(rr.randint(1, 2)):
        d.point((rr.randrange(T), rr.randrange(T)), fill=rgba(shade(base, 0.86)))
    return im


def forest_floor(rr):
    im, d = new(fill=rgba(FOREST_FLOOR))
    speckle(im, FOREST_FLOOR, 3, 0.14, rr)
    for _ in range(5):  # leaf litter: brown specks, a few lighter
        x, y = rr.randrange(T), rr.randrange(T)
        d.point((x, y), fill=rgba((62, 52, 34)))
        if rr.random() < 0.4:
            d.point(((x + 1) % T, y), fill=rgba((74, 62, 40)))
    for _ in range(2):
        d.point((rr.randrange(T), rr.randrange(T)), fill=rgba((34, 40, 28)))
    return im


def dirt(base, rr):
    im, d = new(fill=rgba(base))
    speckle(im, base, 4, 0.18, rr)
    for _ in range(2):
        x, y = rr.randrange(1, 15), rr.randrange(1, 15)
        d.point((x, y), fill=rgba(shade(base, 1.18)))       # pebble
        d.point((x + 1, y + 1), fill=rgba(shade(base, 0.8)))  # its shadow
    return im


def dirt_edge(rr, k):
    """Worn earth with grass dithered over the north side (rows 0..4); rotated by the renderer."""
    im = dirt(PAL["d"], rr)
    px = im.load()
    g = PAL["g"]
    for y in range(5):
        for x in range(T):
            # rows 0-1 grass, then a thinning ordered dither with a per-variant phase
            keep = y < 2 or (y == 2 and (x + k) % 2 == 0) or (y == 3 and (x + 2 * k) % 3 == 0) or \
                (y == 4 and (x + 3 * k) % 5 == 0)
            if keep:
                v = g if rr.random() > 0.12 else shade(g, 0.9)
                px[x, y] = rgba(v)
    return im


def hill(rr):
    im, d = new(fill=rgba(HILL))
    speckle(im, HILL, 3, 0.14, rr)
    for _ in range(rr.randint(2, 3)):  # rock speckle: a lit stone with its shadow down-right
        x, y = rr.randrange(1, 13), rr.randrange(1, 13)
        d.rectangle([x, y, x + 1, y], fill=rgba((104, 100, 88)))
        d.point((x, y + 1), fill=rgba((88, 86, 76)))
        d.line([(x + 1, y + 1), (x + 2, y + 1)], fill=rgba(shade(HILL, 0.8)))
    for _ in range(2):
        gx, gy = rr.randrange(1, 15), rr.randrange(3, 15)
        d.line([(gx, gy), (gx, gy - 1)], fill=rgba(shade(HILL, 1.12)))
    return im


def mountain(rr, top=False):
    im, d = new(fill=rgba(ROCK))
    speckle(im, ROCK, 4, 0.22, rr)
    for _ in range(2):  # cracks: a dark line with a lit pixel on its upper-left lip
        x0, y0 = rr.randrange(2, 12), rr.randrange(3 if top else 1, 11)
        pts = [(x0, y0), (x0 + rr.randint(-1, 2), y0 + 2), (x0 + rr.randint(-1, 3), y0 + 4)]
        d.line(pts, fill=rgba(shade(ROCK, 0.66)))
        d.point((x0 - 1, y0), fill=rgba(shade(ROCK, 1.22)))
    d.point((rr.randrange(2, 14), rr.randrange(4, 14)), fill=rgba(shade(ROCK, 1.3)))
    if top:
        d.line([(0, 0), (T - 1, 0)], fill=rgba(shade(ROCK, 1.45)))
        d.line([(0, 1), (T - 1, 1)], fill=rgba(shade(ROCK, 1.25)))
        for x in range(0, T, 5):
            d.point((x + rr.randrange(3), 2), fill=rgba(shade(ROCK, 1.12)))
    return im


def mountain_cliff(rr):
    """The rock's south face where the tile below is open: the top half rock, the bottom a dark face."""
    im = mountain(rr)
    d = ImageDraw.Draw(im)
    face = shade(ROCK, 0.62)
    d.rectangle([0, 10, T - 1, T - 1], fill=rgba(face))
    d.line([(0, 10), (T - 1, 10)], fill=rgba(shade(ROCK, 0.48)))
    for x in range(1, T, 4):
        d.line([(x, 11), (x, 14)], fill=rgba(shade(face, 0.82)))
    return im


def water(base, rr, ripples):
    im, d = new(fill=rgba(base))
    speckle(im, base, 3, 0.12, rr)
    for _ in range(ripples):  # 2-px ripples: a light pair with a darker trough under it
        wx, wy = rr.randrange(1, 13), rr.randrange(2, 14)
        d.line([(wx, wy), (wx + 1, wy)], fill=rgba(shade(base, 1.32)))
        d.line([(wx + 1, wy + 1), (wx + 2, wy + 1)], fill=rgba(shade(base, 0.86)))
    return im


def bank(mask, rr):
    """Shallow water with a sand lip on every side whose 4-neighbour is land (mask bits N=1 E=2 S=4 W=8)."""
    im = water(PAL["s"], rr, 1)
    px = im.load()

    def lip(points_outer, points_inner):
        for x, y in points_outer:
            px[x, y] = rgba(SAND if (x + y) % 5 else shade(SAND, 1.1))
        for x, y in points_inner:
            if px[x, y][:3] != SAND and px[x, y][:3] != shade(SAND, 1.1):
                px[x, y] = rgba(WET_SAND)

    r = range(T)
    if mask & 1:
        lip([(x, y) for x in r for y in (0, 1)], [(x, 2) for x in r])
    if mask & 2:
        lip([(x, y) for y in r for x in (14, 15)], [(13, y) for y in r])
    if mask & 4:
        lip([(x, y) for x in r for y in (14, 15)], [(x, 13) for x in r])
    if mask & 8:
        lip([(x, y) for y in r for x in (0, 1)], [(2, y) for y in r])
    return im


def bank_inner(corner, rr):
    """Shallow water with a small sand corner where only the diagonal neighbour is land."""
    im = water(PAL["s"], rr, 1)
    px = im.load()
    for y in range(4):
        for x in range(4):
            if x + y <= 3:
                xx = x if "w" in corner else T - 1 - x
                yy = y if "n" in corner else T - 1 - y
                px[xx, yy] = rgba(SAND if x + y <= 2 else WET_SAND)
    return im


def stock_floor(rr):
    """Tamped earth: lighter than worn earth, with faint horizontal tamping streaks."""
    base = (94, 80, 60)
    im, d = new(fill=rgba(base))
    speckle(im, base, 3, 0.14, rr)
    for y in (3, 8, 12):
        x = rr.randrange(0, 6)
        d.line([(x, y), (x + rr.randrange(5, 9), y)], fill=rgba(shade(base, 1.07)))
    return im


def stock_edge(side):
    im, d = new()
    c = rgba((64, 52, 38))
    if side == "n":
        d.line([(0, 0), (T - 1, 0)], fill=c)
    elif side == "s":
        d.line([(0, T - 1), (T - 1, T - 1)], fill=c)
    elif side == "w":
        d.line([(0, 0), (0, T - 1)], fill=c)
    else:
        d.line([(T - 1, 0), (T - 1, T - 1)], fill=c)
    return im


def stock_stake(corner):
    """A small pale stake (2x3, outlined) one pixel inside the corner."""
    im, d = new()
    x = 2 if "w" in corner else T - 5
    y = 1 if "n" in corner else T - 6
    d.rectangle([x, y, x + 2, y + 4], fill=rgba(OUTLINE))
    d.line([(x + 1, y + 1), (x + 1, y + 3)], fill=rgba((150, 116, 72)))
    d.point((x + 1, y + 1), fill=rgba((186, 150, 100)))
    return im


# ------------------------------------------------------------------ canopy
SPECIES = {
    # name: (shadow, base, light, highlight)
    "oak": ((28, 56, 34), (44, 82, 44), (60, 102, 50), (84, 124, 60)),
    "birch": ((52, 80, 44), (72, 104, 54), (94, 126, 64), (122, 150, 78)),
    "pine": ((20, 44, 34), (30, 62, 44), (44, 82, 54), (66, 104, 64)),
}
CANOPY_SIZES = (16, 20, 23)  # diameter of the canopy mass inside its 24x24 sprite, per size variant


def _lumps(species, diam, rr, centre):
    """Lump circles (cx, cy, r) of a canopy of diameter `diam` around `centre`."""
    cx, cy = centre
    R = diam / 2.0
    lumps = [(cx, cy, R * 0.78)]
    n = 6 if species == "oak" else 8
    for i in range(n):
        a = 2 * math.pi * (i + rr.random() * 0.5) / n
        rr_ = R * (0.36 if species == "oak" else 0.3) * (0.85 + 0.3 * rr.random())
        dist = R - rr_ - 0.2
        lumps.append((cx + math.cos(a) * dist, cy + math.sin(a) * dist, rr_))
    # front (south) lumps last, so they overlap the ones behind them
    return sorted(lumps, key=lambda l: (l[1], l[0]))


def _disc(px, w, h, cx, cy, r, color, dx=0.0, dy=0.0, rad=None):
    rad = r if rad is None else rad
    for y in range(h):
        for x in range(w):
            if (x + 0.5 - cx - dx) ** 2 + (y + 0.5 - cy - dy) ** 2 <= rad * rad and \
                    (x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2 <= r * r:
                px[x, y] = rgba(color)


def canopy(species, diam, size, rr):
    """A 24x24 canopy: lumps shaded shadow -> base -> light -> highlight from the
    top-left, a darker underside, a 1-px outline."""
    im = Image.new("RGBA", (size, size), TRANSPARENT)
    px = im.load()
    sh, base, light, hi = SPECIES[species]
    c = (size / 2.0, size / 2.0)
    if species == "pine":
        R = diam / 2.0
        pts = 9

        def star(r_out, r_in, ox=0.0, oy=0.0, rot=0.0):
            poly = []
            for i in range(pts * 2):
                a = rot + math.pi * i / pts
                r = r_out if i % 2 == 0 else r_in
                poly.append((c[0] + ox + math.cos(a) * r, c[1] + oy + math.sin(a) * r))
            return poly

        rot = rr.random()
        d = ImageDraw.Draw(im)
        d.polygon(star(R - 0.5, R * 0.68, rot=rot), fill=rgba(sh))
        d.polygon(star(R * 0.82, R * 0.55, -0.6, -0.6, rot + 0.2), fill=rgba(base))
        d.polygon(star(R * 0.56, R * 0.36, -1.2, -1.2, rot + 0.4), fill=rgba(light))
        d.polygon(star(R * 0.28, R * 0.18, -1.4, -1.4, rot + 0.1), fill=rgba(hi))  # the lighter crown
    else:
        for (lx, ly, lr) in _lumps(species, diam, rr, c):
            _disc(px, size, size, lx, ly, lr, sh)
            _disc(px, size, size, lx, ly, lr, base, -0.9, -0.9, lr - 0.6)
            _disc(px, size, size, lx, ly, lr, light, -1.4, -1.4, lr * 0.55)
            if lr > 2.6:
                _disc(px, size, size, lx, ly, lr, hi, -1.6, -1.6, lr * 0.22)
        if species == "birch":  # trunk-white specks between the lumps
            for _ in range(max(2, int(diam / 5))):
                x, y = int(c[0] + rr.uniform(-diam / 3, diam / 3)), int(c[1] + rr.uniform(-diam / 3, diam / 3))
                if px[x, y][3] and px[x, y][:3] in (sh, base):
                    px[x, y] = rgba((214, 210, 196))
                    if y + 1 < size and px[x, y + 1][3]:
                        px[x, y + 1] = rgba((150, 146, 136))
    return outline_alpha(im)


def tree_shadow():
    im, d = new(20, 8)
    d.ellipse([0, 0, 19, 7], fill=(8, 14, 6, 90))
    return im


def stump(rr):
    im, d = new()
    d.ellipse([3, 4, 12, 13], fill=rgba((96, 66, 40)))       # bark rim (darker underside)
    d.ellipse([3, 3, 12, 11], fill=rgba(GRAIN))               # cut face
    d.ellipse([5, 5, 10, 9], outline=rgba(GRAIN_RING))
    d.point((7, 7), fill=rgba(GRAIN_RING))
    d.point((4, 4), fill=rgba((222, 190, 140)))
    im = outline_alpha(im)
    d = ImageDraw.Draw(im)
    for x, y in ((1, 13), (13, 12), (14, 4)):  # chips
        d.point((x, y), fill=rgba((200, 166, 116)))
    return im


def bush(berries, rr):
    im, d = new()
    g = (58, 96, 52) if berries else (66, 84, 52)
    d.ellipse([2, 3, 13, 14], fill=rgba(shade(g, 0.72)))
    d.ellipse([2, 2, 12, 12], fill=rgba(g))
    d.ellipse([4, 3, 8, 7], fill=rgba(shade(g, 1.22)))
    if berries:
        for bx, by in ((4, 8), (9, 5), (6, 11), (10, 9), (7, 7)):
            d.point((bx, by), fill=rgba((182, 36, 46)))
            d.point((bx, by - 1), fill=rgba((236, 96, 92)))
    else:
        for bx, by in ((5, 8), (9, 6), (7, 11), (10, 10)):
            d.point((bx, by), fill=rgba((92, 72, 50)))
    return outline_alpha(im)


def deadwood(rr):
    """A thin grey twig, outlined, quieter than the log items."""
    im, d = new()
    c = (98, 92, 82)
    d.line([(3, 12), (12, 7)], fill=rgba(c))
    d.line([(7, 10), (6, 6)], fill=rgba(shade(c, 0.86)))
    d.point((11, 7), fill=rgba((122, 116, 104)))
    return outline_alpha(im)


# ------------------------------------------------------------------ items
def _log(d, x0, x1, y, light=LOG_LIGHT, base=LOG, dark=LOG_DARK, grain=GRAIN, ring=GRAIN_RING):
    """A horizontal round log, 5 px tall (outline, light, base, dark, outline) with its end grain on the right."""
    d.rectangle([x0, y, x1, y + 4], fill=rgba(OUTLINE))
    d.line([(x0 + 1, y + 1), (x1 - 3, y + 1)], fill=rgba(light))
    d.line([(x0 + 1, y + 2), (x1 - 3, y + 2)], fill=rgba(base))
    d.line([(x0 + 1, y + 3), (x1 - 3, y + 3)], fill=rgba(dark))
    d.rectangle([x1 - 2, y + 1, x1 - 1, y + 3], fill=rgba(grain))
    d.point((x1 - 1, y + 2), fill=rgba(ring))


def log_pile(rows):
    """rows: (x0, x1, y) logs drawn back (north) to front (south)."""
    im, d = new()
    for x0, x1, y in rows:
        _log(d, x0, x1, y)
    return im


# ------------------------------------------------------------------ buildings
def hearth(frame):
    im, d = new()
    stone, stone_l, stone_d = (112, 106, 98), (140, 134, 124), (78, 74, 68)
    d.ellipse([1, 2, 14, 15], fill=rgba(stone_d))
    d.ellipse([1, 1, 14, 14], fill=rgba(stone))
    for x, y in ((3, 3), (7, 1), (11, 3), (2, 8), (12, 8)):
        d.point((x, y), fill=rgba(stone_l))
    for x, y in ((4, 12), (8, 13), (11, 11)):
        d.point((x, y), fill=rgba(stone_d))
    d.ellipse([4, 4, 11, 11], fill=rgba((46, 34, 28)))
    if frame == 0:
        d.polygon([(7, 3), (10, 8), (9, 10), (5, 10), (5, 7)], fill=rgba((236, 120, 36)))
        d.polygon([(7, 6), (9, 9), (6, 9)], fill=rgba((255, 214, 104)))
    else:
        d.polygon([(8, 3), (10, 7), (10, 10), (6, 10), (5, 8)], fill=rgba((236, 120, 36)))
        d.polygon([(8, 5), (9, 9), (6, 9), (7, 7)], fill=rgba((255, 214, 104)))
    d.point((7, 8), fill=rgba((255, 244, 200)))
    return outline_alpha(im)


def shelter():
    """A lean-to from above, 16x20: two poles above a hide roof, the dark opening to the south."""
    im, d = new(16, 20)
    hide, hide_l, hide_d = (152, 118, 78), (182, 146, 100), (114, 86, 56)
    for x in (4, 11):  # poles crossing above the ridge
        d.line([(x, 0), (x + (1 if x == 4 else -1), 5)], fill=rgba((126, 92, 56)))
    d.polygon([(2, 5), (13, 5), (15, 15), (0, 15)], fill=rgba(hide))
    d.line([(3, 6), (12, 6)], fill=rgba(hide_l))
    for x in (5, 8, 11):
        d.line([(x, 7), (x + (x - 8) // 3, 14)], fill=rgba(hide_d))  # seams
    d.line([(1, 14), (14, 14)], fill=rgba(hide_d))
    d.rectangle([3, 15, 12, 18], fill=rgba((34, 26, 20)))  # the opening
    d.line([(4, 16), (11, 16)], fill=rgba((52, 40, 30)))
    return outline_alpha(im)


HUT = 48
HUT_WALL = 7
# Top-left of the four 14x8 bedrolls inside the hut; the agent layer lays sleepers on them (agent_layer.gd BEDROLLS
#   = these minus (1, 4)).
HUT_BEDROLLS = ((9, 9), (25, 9), (9, 30), (25, 30))


def _hut_walls(d, wall_l, wall, wall_d, gap, grain, ring):
    """Two courses of logs on every side: north/south run east-west, west/east run north-south."""
    S, k = HUT, HUT_WALL
    for box in ([0, 0, S - 1, k - 1], [0, S - k, S - 1, S - 1], [0, 0, k - 1, S - 1], [S - k, 0, S - 1, S - 1]):
        d.rectangle(box, fill=gap)
    for y0 in (0, S - k):  # north, south
        for course in (1, 4):
            y = y0 + course
            d.line([(1, y), (S - 2, y)], fill=wall_l)
            d.line([(1, y + 1), (S - 2, y + 1)], fill=wall)
    for x0 in (0, S - k):  # west, east
        for course in (1, 4):
            x = x0 + course
            d.line([(x, k), (x, S - 1 - k)], fill=wall_l)
            d.line([(x + 1, k), (x + 1, S - 1 - k)], fill=wall)
    for x in (13, 31):  # staggered log joints
        d.line([(x, 1), (x, 2)], fill=wall_d)
        d.line([(x + 4, 4), (x + 4, 5)], fill=wall_d)
        d.line([(x + 2, S - 6), (x + 2, S - 5)], fill=wall_d)
    for cx, cy in ((0, 0), (S - k, 0), (0, S - k), (S - k, S - k)):  # crossing corners with end grain
        d.rectangle([cx, cy, cx + k - 1, cy + k - 1], fill=wall_d)
        for ex, ey in ((1, 1), (4, 4), (1, 4), (4, 1)):
            d.rectangle([cx + ex, cy + ey, cx + ex + 1, cy + ey + 1], fill=grain)
            d.point((cx + ex + 1, cy + ey + 1), fill=ring)


def hut(rr):
    im, d = new(HUT, HUT)
    S, k = HUT, HUT_WALL
    floor = PAL["e"]
    d.rectangle([k, k, S - 1 - k, S - 1 - k], fill=rgba(floor))
    speckle(im, floor, 4, 0.16, rr, (k, k, S - k, S - k))
    d.line([(k, k), (S - 1 - k, k)], fill=rgba(shade(floor, 0.74)))  # the walls' shadow on the floor
    d.line([(k, k), (k, S - 1 - k)], fill=rgba(shade(floor, 0.74)))
    d.line([(k + 1, k + 1), (S - 2 - k, k + 1)], fill=rgba(shade(floor, 0.86)))
    _hut_walls(d, rgba(LOG_LIGHT), rgba(LOG), rgba(LOG_DARK), rgba(OUTLINE), rgba(GRAIN), rgba(GRAIN_RING))
    # four bedrolls (2x2), lying east-west like the sleep pose: hides with a lighter pillow at the head (west)
    hides = [(150, 112, 72), (128, 96, 66), (140, 116, 82), (120, 90, 60)]
    for i, (bx, by) in enumerate(HUT_BEDROLLS):
        h = hides[i]
        d.rectangle([bx, by, bx + 13, by + 7], fill=rgba(OUTLINE))
        d.rectangle([bx + 1, by + 1, bx + 12, by + 6], fill=rgba(h))
        d.line([(bx + 1, by + 6), (bx + 12, by + 6)], fill=rgba(shade(h, 0.8)))
        for _ in range(4):
            d.point((bx + rr.randrange(5, 12), by + rr.randrange(2, 6)), fill=rgba(shade(h, 1.16)))
        d.rectangle([bx + 1, by + 1, bx + 3, by + 5], fill=rgba((206, 192, 156)))  # pillow
        d.line([(bx + 3, by + 1), (bx + 3, by + 5)], fill=rgba((172, 158, 124)))
    # a small hearth stone in the middle
    d.ellipse([20, 19, 27, 26], fill=rgba(OUTLINE))
    d.ellipse([21, 20, 26, 25], fill=rgba((116, 110, 102)))
    d.ellipse([22, 21, 25, 24], fill=rgba((52, 40, 32)))
    d.point((23, 22), fill=rgba((214, 112, 40)))
    d.point((24, 23), fill=rgba((176, 80, 30)))
    # the doorway: the middle third of the south wall, a packed threshold
    d.rectangle([19, S - k, 28, S - 1], fill=rgba(OUTLINE))
    d.rectangle([20, S - k, 27, S - 1], fill=rgba(shade(floor, 0.82)))
    d.line([(20, S - k), (27, S - k)], fill=rgba(shade(floor, 0.66)))
    for x in (17, 29):  # door posts: log ends
        d.rectangle([x, S - k + 1, x + 1, S - 2], fill=rgba(GRAIN))
        d.point((x + 1, S - 3), fill=rgba(GRAIN_RING))
    return im


def hut_ghost(blue):
    """The hut as a blueprint: hatched walls, the doorway, the bedrolls and hearth as faint plan lines, a dashed
    outline on both wall edges. `blue` False gives the greyscale ghost the order tool tints green or red."""
    im, d = new(HUT, HUT)
    S, k = HUT, HUT_WALL
    if blue:
        wall, floor, line, plan = (120, 176, 236, 80), (120, 176, 236, 30), (170, 214, 255, 235), (170, 214, 255, 120)
    else:
        wall, floor, line, plan = (225, 225, 225, 128), (210, 210, 210, 48), (255, 255, 255, 200), None
    d.rectangle([0, 0, S - 1, S - 1], fill=wall)
    d.rectangle([k, k, S - 1 - k, S - 1 - k], fill=floor)
    d.rectangle([20, S - k, 27, S - 1], fill=floor)  # doorway
    px = im.load()
    if blue:  # diagonal hatching across the walls, as on a plan
        for y in range(S):
            for x in range(S):
                in_wall = not (k <= x < S - k and k <= y < S - k) and not (20 <= x <= 27 and y >= S - k)
                if in_wall and (x + y) % 4 == 0:
                    px[x, y] = plan
        for bx, by in HUT_BEDROLLS:
            d.rectangle([bx, by, bx + 13, by + 7], outline=plan)
            d.line([(bx + 3, by + 1), (bx + 3, by + 6)], fill=plan)
        d.ellipse([20, 19, 27, 26], outline=plan)

    def dash(points):
        for i, (x, y) in enumerate(points):
            if not blue or i % 5 < 3:
                px[x, y] = line

    ring = [(x, 0) for x in range(S)] + [(S - 1, y) for y in range(1, S)] + \
        [(x, S - 1) for x in range(S - 2, -1, -1)] + [(0, y) for y in range(S - 2, 0, -1)]
    dash([p for p in ring if not (20 <= p[0] <= 27 and p[1] == S - 1)])
    inner = [(x, k - 1) for x in range(k - 1, S - k + 1)] + [(S - k, y) for y in range(k, S - k + 1)] + \
        [(x, S - k) for x in range(S - k - 1, k - 2, -1)] + [(k - 1, y) for y in range(S - k - 1, k - 1, -1)]
    dash([p for p in inner if not (20 <= p[0] <= 27 and p[1] == S - k)])
    for x in (19, 28):
        dash([(x, y) for y in range(S - k, S)])
    return im


# ------------------------------------------------------------------ people (greyscale layers + outline layer)
# Layer of each ASCII char: skin (s light, S shade, L legs), hair (h, H), tunic (T light, t base, u shade),
#   outline (o outline, e eye, b belt; fixed colours, never modulated).
LAYER = {"s": "skin", "S": "skin", "L": "skin", "h": "hair", "H": "hair", "T": "tunic", "t": "tunic", "u": "tunic",
         "o": "outline", "e": "outline", "b": "outline"}
VALUE = {"s": (232, 232, 232, 255), "S": (196, 196, 196, 255), "L": (184, 184, 184, 255),
         "h": (236, 236, 236, 255), "H": (204, 204, 204, 255),
         "T": (255, 255, 255, 255), "t": (214, 214, 214, 255), "u": (172, 172, 172, 255),
         "o": rgba(OUTLINE), "e": rgba(OUTLINE), "b": rgba((84, 58, 36))}

ADULT_TOP = [
    "....oooooooo....",
    "....ohhhhhho....",
    "....ohssssho....",
    "....osesseso....",
    "....ossssss" + "o....",
    "....oSssssSo....",
    "...oooooooooo...",
    "...oTTTTTTTTo...",
    "...otttttttto...",
    "...otttttttto...",
    "...obbbbbbbbo...",
    "...otttttttto...",
    "...ouuuuuuuuo...",
    "...oooooooooo...",
]
ADULT = {
    "0": ADULT_TOP + ["....oLLooLLo....", "....oooooooo...."],
    "1": ADULT_TOP + ["...oLLo..oLLo...", "...oooo..oooo..."],
}
CHILD_TOP = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "....oooooooo....",
    "....ohhhhhho....",
    "....osesseso....",
    "....oSssssSo....",
    ".....oooooo.....",
    ".....oTTTTo.....",
    ".....ouuuuo.....",
    ".....oooooo.....",
]
CHILD = {
    "0": CHILD_TOP + [".....oLooLo.....", ".....oooooo....."],
    "1": CHILD_TOP + ["....oLo..oLo....", "....ooo..ooo...."],
}
# Lying, head to the left, under a blanket in the tunic colour (no belt; eyes closed).
ADULT_SLEEP = [
    "................",
    ".ooooooooooooo..",
    "ohhsssoTTTTTTTo.",
    "ohhssssttttttto.",
    "ohhssssttttttto.",
    "ohhsssouuuuuuuo.",
    ".ooooooooooooo..",
    "................",
]
CHILD_SLEEP = [
    "................",
    "..ooooooooo.....",
    ".ohhssoTTTTo....",
    ".ohhssstttto....",
    ".ohhssstttto....",
    ".ohhssouuuuo....",
    "..ooooooooo.....",
    "................",
]


def figure_layers(rows):
    """{layer: image} for skin, hair, tunic, outline, plus the halo: a 1-px white ring outside the figure."""
    h, w = len(rows), len(rows[0])
    layers = {}
    for name in ("skin", "hair", "tunic", "outline"):
        layers[name] = from_ascii([
            "".join(ch if ch != "." and LAYER[ch] == name else "." for ch in row) for row in rows], VALUE)
    whole = from_ascii(rows, VALUE)
    halo = Image.new("RGBA", (w, h), TRANSPARENT)
    src, px = whole.load(), halo.load()
    for y in range(h):
        for x in range(w):
            if src[x, y][3]:
                continue
            if any(0 <= x + dx < w and 0 <= y + dy < h and src[x + dx, y + dy][3]
                   for dx in (-1, 0, 1) for dy in (-1, 0, 1) if abs(dx) + abs(dy) == 1):
                px[x, y] = (255, 255, 255, 255)
    layers["halo"] = halo
    return layers


def shadow():
    return from_ascii(["..xxxxxx..", "xxxxxxxxxx", "xxxxxxxxxx", "..xxxxxx.."], {"x": (0, 0, 0, 110)})


def carry_log(part):
    """Three light logs across the shoulders. "back": two logs drawn BEFORE the person (they pass behind the head
    and show on both sides of it); "front": one log drawn after the person, across the chest (rows 7..11)."""
    im, d = new()
    grain, ring = (246, 228, 188), (190, 150, 100)
    if part == "back":
        _log(d, 0, 15, 1, CARRY_LIGHT, CARRY, CARRY_DARK, grain, ring)
        _log(d, 1, 15, 4, CARRY_LIGHT, CARRY, CARRY_DARK, grain, ring)
    else:
        _log(d, 0, 14, 7, CARRY_LIGHT, CARRY, CARRY_DARK, grain, ring)
    return im


def tool_axe():
    im, d = new()
    d.rectangle([12, 6, 14, 12], fill=rgba(OUTLINE))
    d.line([(13, 7), (13, 11)], fill=rgba((176, 128, 80)))  # haft
    d.rectangle([11, 4, 15, 8], fill=rgba(OUTLINE))
    d.rectangle([14, 5, 14, 7], fill=rgba((206, 210, 220)))  # stone blade
    d.point((12, 5), fill=rgba((150, 152, 160)))
    d.rectangle([13, 5, 13, 6], fill=rgba((176, 128, 80)))
    return im


def tool_hammer():
    im, d = new()
    d.rectangle([12, 6, 14, 12], fill=rgba(OUTLINE))
    d.line([(13, 8), (13, 11)], fill=rgba((176, 128, 80)))
    d.rectangle([11, 4, 15, 8], fill=rgba(OUTLINE))
    d.rectangle([12, 5, 14, 7], fill=rgba((128, 124, 116)))
    d.line([(12, 5), (14, 5)], fill=rgba((172, 168, 158)))
    return im


# ------------------------------------------------------------------ M2 temporary sprites (M-ART2 replaces them)
STONE_D, STONE, STONE_L = (86, 82, 76), (118, 112, 104), (150, 144, 134)
FLINT_D, FLINT, FLINT_L = (52, 56, 66), (78, 84, 98), (120, 128, 146)
CORTEX = (196, 188, 168)


def boulder(rr):
    """A grey rounded rock filling most of its tile, lit top-left, outlined."""
    im, d = new()
    d.ellipse([1, 3, 14, 14], fill=rgba(STONE_D))
    d.ellipse([1, 2, 13, 12], fill=rgba(STONE))
    d.ellipse([3, 3, 8, 7], fill=rgba(STONE_L))
    d.line([(6, 9), (10, 7)], fill=rgba(STONE_D))  # a crack
    for _ in range(3):
        d.point((rr.randint(4, 11), rr.randint(4, 11)), fill=rgba(shade(STONE, 0.9)))
    return outline_alpha(im)


def flint_outcrop(rr):
    """Three dark blue-grey nodules with pale cortex, outlined."""
    im, d = new()
    for x0, y0, x1, y1 in ((1, 6, 7, 13), (6, 3, 13, 10), (8, 9, 14, 14)):
        d.ellipse([x0, y0 + 1, x1, y1], fill=rgba(FLINT_D))
        d.ellipse([x0, y0, x1 - 1, y1 - 1], fill=rgba(FLINT))
        d.point((x0 + 2, y0 + 1), fill=rgba(FLINT_L))
    for _ in range(4):
        d.point((rr.randint(3, 12), rr.randint(5, 12)), fill=rgba(CORTEX))
    return outline_alpha(im)


def knapping_spot():
    """A flat anvil stone with a hammerstone and flakes around it."""
    im, d = new()
    d.ellipse([2, 5, 12, 13], fill=rgba(STONE_D))
    d.ellipse([2, 4, 12, 11], fill=rgba(STONE))
    d.line([(4, 5), (9, 5)], fill=rgba(STONE_L))
    d.ellipse([10, 2, 14, 6], fill=rgba(shade(STONE, 0.85)))  # hammerstone
    d.point((11, 3), fill=rgba(STONE_L))
    for x, y in ((6, 7), (8, 8), (1, 13), (13, 12), (4, 14)):
        d.point((x, y), fill=rgba(FLINT_L))
    return outline_alpha(im)


def wood_bench():
    """A 32x16 split-log bench on two stone feet, with a blade and a haft on it."""
    im, d = new(32, 16)
    for x in (4, 24):
        d.rectangle([x, 10, x + 3, 14], fill=rgba(STONE_D))
        d.line([(x, 10), (x + 3, 10)], fill=rgba(STONE_L))
    d.rectangle([1, 4, 30, 10], fill=rgba(LOG_DARK))
    d.rectangle([1, 4, 30, 8], fill=rgba(LOG))
    d.line([(2, 5), (29, 5)], fill=rgba(LOG_LIGHT))
    d.line([(10, 7), (19, 7)], fill=rgba(GRAIN_RING))
    d.line([(20, 6), (26, 6)], fill=rgba((176, 128, 80)))  # a haft
    d.rectangle([7, 5, 9, 6], fill=rgba(FLINT_L))  # a blade
    return outline_alpha(im)


def drying_rack():
    """A 32x16 frame: two posts, a crossbar and two hanging hides."""
    im, d = new(32, 16)
    pole = (126, 92, 56)
    for x in (2, 28):
        d.rectangle([x, 1, x + 1, 14], fill=rgba(pole))
    d.line([(2, 2), (29, 2)], fill=rgba(shade(pole, 1.2)))
    for x0 in (6, 17):
        d.rectangle([x0, 3, x0 + 7, 11], fill=rgba((152, 118, 78)))
        d.line([(x0, 3), (x0 + 7, 3)], fill=rgba((182, 146, 100)))
        d.line([(x0 + 3, 5), (x0 + 3, 10)], fill=rgba((114, 86, 56)))
    return outline_alpha(im)


def item_firewood():
    """Split firewood: short pale billets in a small stack."""
    im, d = new()
    for x0, x1, y in ((4, 11, 5), (3, 12, 8), (5, 11, 11)):
        _log(d, x0, x1, y, (214, 168, 108), (176, 130, 80), (140, 100, 62))
    return im


def item_pile(base, light, dark, spots):
    """A small heap of lumps (stone, flint)."""
    im, d = new()
    for x0, y0 in ((3, 7), (8, 7), (5, 3), (10, 10), (2, 11)):
        d.ellipse([x0, y0 + 1, x0 + 4, y0 + 4], fill=rgba(dark))
        d.ellipse([x0, y0, x0 + 4, y0 + 3], fill=rgba(base))
        d.point((x0 + 1, y0 + 1), fill=rgba(light))
    for x, y in spots:
        d.point((x, y), fill=rgba(CORTEX))
    return outline_alpha(im)


def item_blade():
    """Three leaf-shaped flint blades side by side."""
    im, d = new()
    for x in (3, 7, 11):
        d.polygon([(x, 3), (x + 2, 6), (x + 1, 13), (x - 1, 6)], fill=rgba(FLINT))
        d.line([(x, 4), (x, 11)], fill=rgba(FLINT_L))
    return outline_alpha(im)


def item_axe():
    """A hafted stone axe lying diagonally."""
    im, d = new()
    d.line([(3, 13), (11, 4)], fill=rgba((176, 128, 80)), width=2)
    d.polygon([(9, 1), (14, 3), (13, 7), (9, 5)], fill=rgba(FLINT))
    d.line([(10, 2), (13, 3)], fill=rgba(FLINT_L))
    return outline_alpha(im)


def item_hide():
    """A stretched hide, tan."""
    im, d = new()
    d.polygon([(3, 3), (8, 2), (13, 4), (12, 9), (13, 13), (7, 12), (2, 13), (4, 8)], fill=rgba((152, 118, 78)))
    d.polygon([(5, 4), (8, 4), (10, 6), (8, 9), (5, 8)], fill=rgba((182, 146, 100)))
    return outline_alpha(im)


def mark_quarry():
    """A 7x7 pick glyph (yellow, outlined) at the cell's top-right, x 9..15, y 0..6 (like mark_chop)."""
    glyph = from_ascii([
        ".......",
        ".yyyy..",
        "y..y.y.",
        "...y...",
        "...y...",
        "...y...",
        ".......",
    ], {"y": rgba(ACCENT)})
    glyph = outline_alpha(glyph)
    im, _ = new()
    im.paste(glyph, (9, 0))
    return im


# ------------------------------------------------------------------ M3 temporary sprites (M-ART2 replaces them)
# Wall autotile: the 4-bit mask of wall-or-door neighbours (N=1, E=2, S=4, W=8). A wall is a 10-px band through the
#   tile centre (rows/columns 3..12, outline included) with an arm to every connected side; the arm reaches the
#   tile edge without an outline there, so neighbouring wall tiles join. Straight runs (mask 10 = E+W, mask 5 = N+S)
#   show only courses running along the wall; every other mask (0 isolated, 1/2/4/8 ends, corners, tees, the cross)
#   carries a 12x12 post (x, y 2..13) in the centre with its arms.
WALL_BAND = (3, 12)   # first and last row of a horizontal band (outline rows included)
WALL_POST = (2, 13)   # first and last row/column of the post square (outline included)
STONE_WALL_D, STONE_WALL, STONE_WALL_L, STONE_WALL_HI = (74, 72, 68), (112, 108, 102), (138, 134, 126), (160, 156, 146)
ROCK_FLOOR = (78, 75, 70)
PACKED = (76, 64, 48)
GHOST_BLUE = dict(fill=(120, 176, 236, 80), faint=(120, 176, 236, 30), line=(170, 214, 255, 235),
                  plan=(170, 214, 255, 120))
GHOST_RED = dict(fill=(236, 84, 72, 70), line=(255, 120, 104, 235), plan=(255, 140, 120, 200))
DOOR_WOOD, DOOR_WOOD_L, DOOR_WOOD_D = (150, 108, 66), (182, 138, 88), (104, 72, 42)


def wall_straight(mask):
    """'h' for a pure east-west run, 'v' for a pure north-south run, None where the tile carries a post."""
    return {10: "h", 5: "v"}.get(mask)


def wall_silhouette(mask):
    """The set of (x, y) a wall with neighbour mask `mask` covers."""
    a, b = WALL_BAND
    pts = set()

    def box(x0, y0, x1, y1):
        pts.update((x, y) for y in range(y0, y1 + 1) for x in range(x0, x1 + 1))
    if wall_straight(mask) is None:
        p0, p1 = WALL_POST
        box(p0, p0, p1, p1)
    else:
        box(a, a, b, b)
    if mask & 1:
        box(a, 0, b, b)
    if mask & 2:
        box(a, a, T - 1, b)
    if mask & 4:
        box(a, a, b, T - 1)
    if mask & 8:
        box(0, a, b, b)
    return pts


def wall_outline_pixels(pts):
    """Silhouette pixels next to a pixel outside it (pixels beyond the tile edge count as inside: arms continue)."""
    out = set()
    for x, y in pts:
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                nx, ny = x + dx, y + dy
                if 0 <= nx < T and 0 <= ny < T and (nx, ny) not in pts:
                    out.add((x, y))
    return out


def wall_region(mask, x, y):
    """Which part of wall tile `mask` the silhouette pixel (x, y) belongs to: 'h' (a course running east-west),
    'v' (north-south) or 'post'."""
    straight = wall_straight(mask)
    if straight:
        return straight
    p0, p1 = WALL_POST
    if p0 <= x <= p1 and p0 <= y <= p1:
        return "post"
    return "h" if WALL_BAND[0] <= y <= WALL_BAND[1] and (x < p0 or x > p1) else "v"


def _paint_wall(mask, rr, course, post):
    """Fill the silhouette of `mask`: course(along, across) -> colour for the arms (across 0..7 = the band's inner
    rows from its north/west side), post(x, y) -> colour inside the post; then the outline."""
    im, _ = new()
    px = im.load()
    pts = wall_silhouette(mask)
    a = WALL_BAND[0] + 1
    for x, y in pts:
        part = wall_region(mask, x, y)
        if part == "h":
            px[x, y] = rgba(course(x, y - a))
        elif part == "v":
            px[x, y] = rgba(course(y, x - a))
        else:
            px[x, y] = rgba(post(x, y))
    for x, y in wall_outline_pixels(pts):
        px[x, y] = rgba(OUTLINE)
    return im


def log_wall(mask, rr):
    """A log wall tile for neighbour mask `mask`: two logs per course along each arm (lit on the north/west side,
    staggered joints), a post showing a cut log end (bark rim, grain, a ring, a lit top-left)."""
    logs = [LOG_LIGHT, LOG, LOG, LOG_DARK] * 2
    joints = {(5, 0), (5, 1), (5, 2), (11, 4), (11, 5), (11, 6)}
    knot = rr.randrange(2, 13)

    def course(along, across):
        if (along % 16, across) in joints:
            return LOG_DARK
        if along % 16 == knot and across == 2:
            return GRAIN_RING
        return logs[across] if 0 <= across < 8 else LOG

    p0, p1 = WALL_POST
    c = (p0 + p1) / 2.0

    def post(x, y):
        r2 = (x - c) ** 2 + (y - c) ** 2
        if (x, y) == (p0 + 3, p0 + 3):
            return (222, 190, 140)
        if r2 <= 1.0:
            return GRAIN_RING
        if 6.0 <= r2 <= 9.5:
            return GRAIN_RING
        if r2 <= 20.5:
            return GRAIN
        return LOG_DARK
    return _paint_wall(mask, rr, course, post)


def stone_wall(mask, rr):
    """A dry-stone wall tile for neighbour mask `mask`: grey blocks in a running bond (two courses of 4 rows, blocks
    5 long, the second course offset), each lit on its top/left edge; the post is a squared cornerstone."""
    def course(along, across):
        row = across % 4
        k = (along + 2 * (across // 4)) % 5
        if row == 3 or k == 0:
            return STONE_WALL_D  # bed and head joints
        if row == 0 or k == 1:
            return STONE_WALL_L
        return STONE_WALL

    p0, p1 = WALL_POST
    chips = {(rr.randrange(p0 + 3, p1 - 2), rr.randrange(p0 + 3, p1 - 2)) for _ in range(3)}

    def post(x, y):
        if y == p0 + 1 or (x, y) == (p0 + 1, p0 + 1):
            return STONE_WALL_HI
        if x == p0 + 1:
            return STONE_WALL_L
        if x == p1 - 1 or y == p1 - 1:
            return STONE_WALL_D
        return shade(STONE_WALL, 0.88) if (x, y) in chips else STONE_WALL
    return _paint_wall(mask, rr, course, post)


def door(orientation):
    """A plank door in a wall. 'h' sits in an east-west wall: two jambs (log ends, x 0..2 and 13..15, the wall band's
    rows 3..12) that join the wall arms on both sides, and between them the closed leaf (vertical planks, rows
    5..10) with the ground showing at its two edges. 'v' is the same door turned for a north-south wall."""
    im, d = new()
    a, b = WALL_BAND
    for x0, x1, inner in ((0, 2, 2), (13, 15, 13)):
        d.rectangle([x0, a, x1, b], fill=rgba(LOG_DARK))
        d.line([(x0, a), (x1, a)], fill=rgba(OUTLINE))
        d.line([(x0, b), (x1, b)], fill=rgba(OUTLINE))
        d.line([(inner, a), (inner, b)], fill=rgba(OUTLINE))
        jx0, jx1 = (0, 1) if x0 == 0 else (14, 15)
        d.rectangle([jx0, a + 2, jx1, b - 2], fill=rgba(GRAIN))
        d.point((jx0, a + 2), fill=rgba((222, 190, 140)))
    d.line([(3, 5), (12, 5)], fill=rgba(OUTLINE))
    d.line([(3, 10), (12, 10)], fill=rgba(OUTLINE))
    d.rectangle([3, 6, 12, 9], fill=rgba(DOOR_WOOD))
    d.line([(3, 6), (12, 6)], fill=rgba(DOOR_WOOD_L))
    for x in (5, 8, 11):
        d.line([(x, 7), (x, 9)], fill=rgba(DOOR_WOOD_D))
    d.point((10, 8), fill=rgba((214, 190, 150)))  # the pull: a bone peg
    return im if orientation == "h" else im.transpose(Image.Transpose.TRANSPOSE)


def floor_packed(rr):
    """Packed earth: flat, a little darker than worn earth, with a few tamped marks (short light and dark dashes)."""
    im, d = new(fill=rgba(PACKED))
    speckle(im, PACKED, 2, 0.10, rr)
    for y in (2, 7, 12):
        x = rr.randrange(1, 8)
        d.line([(x, y), (x + rr.randrange(3, 6), y)], fill=rgba(shade(PACKED, 1.08)))
        d.line([(x + 1, y + 1), (x + rr.randrange(2, 4), y + 1)], fill=rgba(shade(PACKED, 0.92)))
    return im


def floor_rock(rr):
    """Dug rock floor inside a hill: grey chipped stone, chisel scars lit on their upper-left lip."""
    im, d = new(fill=rgba(ROCK_FLOOR))
    speckle(im, ROCK_FLOOR, 3, 0.16, rr)
    for _ in range(4):
        x, y = rr.randrange(1, 13), rr.randrange(1, 14)
        d.line([(x, y), (x + 2, y)], fill=rgba(shade(ROCK_FLOOR, 0.86)))
        d.point((x, y - 1), fill=rgba(shade(ROCK_FLOOR, 1.14)))
    return im


def bedroll(rr):
    """A hide-and-straw bedroll from above, lying east-west (head to the west) like the hut's bedrolls and the sleep
    frame: a 14x8 outlined hide at x 1..14, y 4..11, a straw pillow at the west end."""
    im, d = new()
    hide = (140, 106, 70)
    d.rectangle([1, 4, 14, 11], fill=rgba(OUTLINE))
    d.rectangle([2, 5, 13, 10], fill=rgba(hide))
    d.line([(2, 10), (13, 10)], fill=rgba(shade(hide, 0.8)))
    d.line([(5, 5), (13, 5)], fill=rgba(shade(hide, 1.16)))
    for _ in range(4):
        d.point((rr.randrange(6, 13), rr.randrange(6, 10)), fill=rgba(shade(hide, 1.12)))
    d.rectangle([2, 5, 4, 9], fill=rgba((196, 176, 112)))  # straw pillow
    d.point((3, 6), fill=rgba((226, 206, 140)))
    d.line([(4, 5), (4, 9)], fill=rgba((160, 140, 86)))
    return im


FORD_WATER = (66, 94, 106)
GRAVEL, GRAVEL_L, GRAVEL_D = (112, 104, 86), (132, 124, 104), (84, 80, 68)
FORD_STONE, FORD_STONE_L = (120, 114, 100), (138, 132, 116)


def ford(rr):
    """A ford: lighter shallow water (a little paler than shallow_*), a band of pale gravel across the tile (rows
    5..10, ragged edges) and three flat stepping stones on it, lit top-left with a 1-px wet shadow down-right. Opaque
    and ground-like (no outline); the band runs east-west and reaches both tile edges so neighbouring fords join."""
    im = water(FORD_WATER, rr, 1)
    px = im.load()
    for x in range(T):
        top = 5 + (1 if rr.random() < 0.3 else 0)
        bottom = 10 - (1 if rr.random() < 0.3 else 0)
        for y in range(top, bottom + 1):
            px[x, y] = rgba(GRAVEL)
        px[x, top - 1] = rgba(shade(FORD_WATER, 1.12))  # the wet rim where the gravel meets the water
        px[x, bottom + 1] = rgba(shade(FORD_WATER, 0.88))
    for _ in range(10):  # gravel grain: a light pebble with its shadow, a few dark ones
        x, y = rr.randrange(0, T - 1), rr.randrange(6, 10)
        px[x, y] = rgba(GRAVEL_L)
        px[x + 1, y + 1] = rgba(GRAVEL_D)
    d = ImageDraw.Draw(im)
    for x0 in (1, 6, 11):  # stepping stones, 4x3 each
        y0 = 6 + rr.randrange(0, 2)
        d.rectangle([x0 + 1, y0 + 1, x0 + 4, y0 + 3], fill=rgba(GRAVEL_D))  # shadow down-right
        d.rectangle([x0, y0, x0 + 3, y0 + 2], fill=rgba(FORD_STONE))
        d.line([(x0, y0), (x0 + 2, y0)], fill=rgba(FORD_STONE_L))
        d.point((x0, y0 + 1), fill=rgba(FORD_STONE_L))
    return im


# ------------------------------------------------------------------ M4 temporary sprites (M-ART2 replaces them)
CLAY_D, CLAY, CLAY_L = (112, 72, 50), (150, 98, 66), (184, 128, 90)
POT_D, POT, POT_L = (104, 58, 38), (146, 84, 52), (182, 116, 74)
RAW_D, RAW, RAW_L = (122, 104, 84), (156, 136, 110), (188, 168, 138)


def clay_bank(rr):
    """A river-bank clay bank: a low reddish-brown mound with a darker cut face (where it is dug) and wet streaks,
    lit top-left, outlined."""
    im, d = new()
    d.ellipse([1, 5, 14, 14], fill=rgba(CLAY_D))
    d.ellipse([1, 4, 14, 12], fill=rgba(CLAY))
    d.ellipse([3, 5, 8, 8], fill=rgba(CLAY_L))
    d.rectangle([9, 9, 12, 12], fill=rgba(shade(CLAY_D, 0.85)))  # the cut face
    d.line([(9, 9), (12, 9)], fill=rgba(CLAY_D))
    for _ in range(4):
        d.point((rr.randint(3, 12), rr.randint(6, 11)), fill=rgba(shade(CLAY, 1.12)))
    return outline_alpha(im)


def _pot(d, x0, y0, w, h, dark, base, light):
    """A round-bellied pot in a w x h box at (x0, y0): body, a narrow neck and a lit rim."""
    d.ellipse([x0, y0 + 2, x0 + w - 1, y0 + h - 1], fill=rgba(dark))
    d.ellipse([x0, y0 + 2, x0 + w - 2, y0 + h - 2], fill=rgba(base))
    d.rectangle([x0 + 2, y0, x0 + w - 3, y0 + 2], fill=rgba(base))
    d.line([(x0 + 2, y0), (x0 + w - 3, y0)], fill=rgba(light))
    d.point((x0 + 2, y0 + 4), fill=rgba(light))


def pottery_bench():
    """A low log bench with a lump of clay and a half-formed pot on it."""
    im, d = new()
    for x in (2, 11):
        d.rectangle([x, 11, x + 2, 14], fill=rgba(LOG_DARK))
    d.rectangle([1, 8, 14, 11], fill=rgba(LOG_DARK))
    d.rectangle([1, 8, 14, 10], fill=rgba(LOG))
    d.line([(2, 8), (13, 8)], fill=rgba(LOG_LIGHT))
    d.ellipse([2, 4, 6, 8], fill=rgba(CLAY))  # a lump of clay
    d.point((3, 5), fill=rgba(CLAY_L))
    _pot(d, 8, 1, 6, 8, RAW_D, RAW, RAW_L)
    return outline_alpha(im)


def grinding_stone():
    """A flat grinding slab with a hollow worn into it and a hand stone resting on it."""
    im, d = new()
    d.ellipse([1, 6, 14, 14], fill=rgba(STONE_D))
    d.ellipse([1, 5, 14, 12], fill=rgba(STONE))
    d.ellipse([4, 7, 11, 10], fill=rgba(shade(STONE, 0.86)))  # the worn hollow
    d.line([(3, 6), (8, 6)], fill=rgba(STONE_L))
    d.ellipse([9, 2, 14, 7], fill=rgba(STONE_D))  # the hand stone
    d.ellipse([9, 2, 13, 6], fill=rgba(STONE_L))
    return outline_alpha(im)


def item_clay():
    """Three lumps of wet clay."""
    return item_pile(CLAY, CLAY_L, CLAY_D, ())


def item_raw_pot():
    """Two unfired pots, pale grey-brown."""
    im, d = new()
    _pot(d, 1, 4, 7, 10, RAW_D, RAW, RAW_L)
    _pot(d, 8, 3, 7, 11, RAW_D, RAW, RAW_L)
    return outline_alpha(im)


def item_clay_pot():
    """Two fired pots, red-brown with a dark band."""
    im, d = new()
    for x0, y0 in ((1, 4), (8, 3)):
        _pot(d, x0, y0, 7, 11 - (y0 - 3), POT_D, POT, POT_L)
        d.line([(x0 + 1, y0 + 6), (x0 + 5, y0 + 6)], fill=rgba(POT_D))
    return outline_alpha(im)


def item_ground_axe():
    """A hafted axe with a smooth polished (ground) grey head, lying diagonally."""
    im, d = new()
    d.line([(3, 13), (11, 4)], fill=rgba((176, 128, 80)), width=2)
    d.polygon([(9, 1), (14, 2), (14, 7), (9, 5)], fill=rgba(STONE))
    d.line([(10, 2), (13, 2)], fill=rgba(STONE_L))
    d.line([(14, 3), (14, 6)], fill=rgba((196, 192, 182)))  # the ground edge
    return outline_alpha(im)


def _ghost_border(px, line):
    """The hut ghost's dash (3 on, 2 off) around the tile edge."""
    ring = [(x, 0) for x in range(T)] + [(T - 1, y) for y in range(1, T)] + \
        [(x, T - 1) for x in range(T - 2, -1, -1)] + [(0, y) for y in range(T - 2, 0, -1)]
    for i, (x, y) in enumerate(ring):
        if i % 5 < 3:
            px[x, y] = line


def ghost(kind):
    """Blueprint ghosts in the hut ghost's style (blueprint blue, translucent, dashed outline):
    wall = hatched block; door = jambs and the leaf with its swing arc (a plan's door symbol); floor = a faint fill
    with dotted tamping lines; bedroll = the bedroll's plan outline and pillow line."""
    g = GHOST_BLUE
    im, d = new()
    px = im.load()
    if kind == "wall":
        d.rectangle([0, 0, T - 1, T - 1], fill=g["fill"])
        for y in range(T):
            for x in range(T):
                if (x + y) % 4 == 0:
                    px[x, y] = g["plan"]
    elif kind == "door":
        d.rectangle([0, 0, T - 1, T - 1], fill=g["faint"])
        d.rectangle([0, 3, 2, 12], fill=g["fill"])
        d.rectangle([13, 3, 15, 12], fill=g["fill"])
        d.line([(3, 8), (12, 8)], fill=g["line"])          # the leaf, closed
        d.arc([-6, -1, 12, 17], 270, 360, fill=g["plan"])  # its swing to the north
        d.line([(3, 8), (3, 0)], fill=g["plan"])
    elif kind == "floor":
        d.rectangle([0, 0, T - 1, T - 1], fill=g["faint"])
        for y in (4, 8, 12):
            for x in range(2, 14, 2):
                px[x, y] = g["plan"]
    else:  # bedroll
        d.rectangle([0, 0, T - 1, T - 1], fill=g["faint"])
        d.rectangle([1, 4, 14, 11], outline=g["plan"])
        d.line([(4, 5), (4, 10)], fill=g["plan"])
    _ghost_border(px, g["line"])
    return im


def ghost_refused():
    """One red overlay for a refused tile of any blueprint kind (drawn in place of the kind's ghost, or over it):
    a translucent red fill, a red dashed outline and a cross."""
    g = GHOST_RED
    im, d = new(fill=g["fill"])
    px = im.load()
    for i in range(2, T - 2):
        px[i, i] = g["plan"]
        px[T - 1 - i, i] = g["plan"]
    _ghost_border(px, g["line"])
    return im


def mark_dig():
    """Dig designation on a hill tile: yellow corner brackets around the whole tile (the tile is to be dug out) and a
    7x7 diagonal pick glyph at the top-right (x 9..15, y 0..6), outlined like mark_chop and mark_quarry."""
    glyph = outline_alpha(from_ascii([
        ".......",
        "..yy...",
        "...yy..",
        "..y.y..",
        ".y...y.",
        "y......",
        ".......",
    ], {"y": rgba(ACCENT)}))
    im, d = new()
    for (x0, y0, dx, dy) in ((1, 1, 1, 1), (1, 14, 1, -1), (14, 14, -1, -1)):  # three corners; the glyph is the 4th
        d.line([(x0, y0), (x0 + 3 * dx, y0)], fill=rgba(OUTLINE))
        d.line([(x0, y0), (x0, y0 + 3 * dy)], fill=rgba(OUTLINE))
        d.line([(x0, y0), (x0 + 2 * dx, y0)], fill=rgba(ACCENT))
        d.line([(x0, y0), (x0, y0 + 2 * dy)], fill=rgba(ACCENT))
    im.paste(glyph, (9, 0), glyph)
    return im


def mark_deconstruct():
    """Deconstruct designation: a 7x7 crossed-out glyph (a yellow X, outlined) at the top-right (x 9..15, y 0..6)."""
    glyph = outline_alpha(from_ascii([
        ".......",
        ".y...y.",
        "..y.y..",
        "...y...",
        "..y.y..",
        ".y...y.",
        ".......",
    ], {"y": rgba(ACCENT)}))
    im, _ = new()
    im.paste(glyph, (9, 0))
    return im


# ------------------------------------------------------------------ marks and icons
def mark_chop():
    """A 7x7 axe glyph (yellow, outlined) at the cell's top-right, x 9..15, y 0..6."""
    glyph = from_ascii([
        ".......",
        "...yyy.",
        "..yyyy.",
        "...yy..",
        "..y.y..",
        ".y.....",
        ".......",
    ], {"y": rgba(ACCENT)})
    glyph = outline_alpha(glyph)
    im, _ = new()
    im.paste(glyph, (9, 0))
    return im


def frame_rect(color):
    im, d = new()
    d.rectangle([0, 0, T - 1, T - 1], outline=color)
    return im


def icon(kind):
    im, d = new(12, 12)
    if kind == "chop":
        d.line([(3, 10), (8, 3)], fill=rgba((176, 128, 80)), width=2)
        d.polygon([(6, 1), (10, 1), (10, 6), (8, 4)], fill=rgba((206, 210, 220)))
    elif kind == "stockpile":
        d.rectangle([1, 3, 10, 10], fill=rgba((150, 108, 64)))
        d.line([(1, 3), (10, 3)], fill=rgba((190, 146, 96)))
        d.line([(1, 6), (10, 6)], fill=rgba((112, 78, 46)))
        d.line([(1, 10), (10, 3)], fill=rgba((112, 78, 46)))
    elif kind == "hut":
        d.rectangle([1, 1, 10, 10], fill=rgba(LOG))
        d.rectangle([3, 3, 8, 8], fill=rgba(PAL["e"]))
        d.rectangle([4, 9, 7, 10], fill=rgba(PAL["e"]))
        d.line([(1, 1), (10, 1)], fill=rgba(LOG_LIGHT))
    elif kind == "build":
        d.rectangle([1, 6, 10, 10], fill=rgba(LOG))
        d.line([(1, 6), (10, 6)], fill=rgba(LOG_LIGHT))
        d.polygon([(1, 6), (5, 1), (10, 6)], fill=rgba((152, 118, 78)))
    elif kind == "quarry":
        d.ellipse([1, 5, 10, 11], fill=rgba(STONE))
        d.line([(2, 2), (8, 2)], fill=rgba((206, 210, 220)), width=2)
        d.line([(5, 3), (5, 8)], fill=rgba((176, 128, 80)))
    elif kind == "dig":  # a pick over a dark hole
        d.ellipse([1, 6, 10, 11], fill=rgba((46, 40, 34)))
        d.line([(3, 9), (9, 2)], fill=rgba((176, 128, 80)), width=2)
        d.line([(5, 1), (10, 5)], fill=rgba((206, 210, 220)))
    elif kind == "deconstruct":  # a log wall stub, crossed out
        d.rectangle([1, 4, 10, 8], fill=rgba(LOG))
        d.line([(1, 4), (10, 4)], fill=rgba(LOG_LIGHT))
        d.line([(2, 10), (9, 1)], fill=rgba((236, 96, 80)), width=2)
    elif kind == "remove":
        d.polygon([(1, 7), (6, 2), (10, 6), (5, 11)], fill=rgba((226, 150, 150)))
        d.polygon([(1, 7), (3, 5), (7, 9), (5, 11)], fill=rgba((236, 230, 220)))
    else:  # cancel
        d.line([(2, 2), (9, 9)], fill=rgba((236, 96, 80)), width=2)
        d.line([(9, 2), (2, 9)], fill=rgba((236, 96, 80)), width=2)
    return outline_alpha(im)


# ------------------------------------------------------------------ sheet
def build_sprites():
    """[(section, name, image)] in a fixed order."""
    def rr(k):
        return random.Random(SEED * 1000 + k)

    out = []

    def add(section, name, im):
        out.append((section, name, im))

    for i in range(3):
        add("ground", f"grass_{i}", grass(PAL["g"], rr(1 + i), 1 + i))
        add("ground", f"grass_dark_{i}", grass(PAL["G"], rr(4 + i), 3 - i))
    for i in range(4):
        add("ground", f"forest_floor_{i}", forest_floor(rr(10 + i)))
        add("ground", f"dirt_{i}", dirt(PAL["d"], rr(14 + i)))
    for i in range(2):
        add("ground", f"dirt_edge_{i}", dirt_edge(rr(18 + i), i))
    for i in range(3):
        add("ground", f"hill_{i}", hill(rr(20 + i)))
        add("ground", f"mountain_{i}", mountain(rr(23 + i)))
        add("ground", f"mountain_top_{i}", mountain(rr(26 + i), top=True))
    add("ground", "mountain_cliff", mountain_cliff(rr(29)))
    for i in range(4):
        add("ground", f"water_{i}", water(PAL["w"], rr(30 + i), 2))
        add("ground", f"shallow_{i}", water(PAL["s"], rr(34 + i), 1))
    for mask in range(1, 16):
        add("ground", f"bank_{mask}", bank(mask, rr(40 + mask)))
    for i, corner in enumerate(("ne", "nw", "se", "sw")):
        add("ground", f"bank_in_{corner}", bank_inner(corner, rr(60 + i)))
    add("ground", "stock_floor", stock_floor(rr(64)))
    for side in "nesw":
        add("zone", f"stock_edge_{side}", stock_edge(side))
    for corner in ("nw", "ne", "sw", "se"):
        add("zone", f"stock_stake_{corner}", stock_stake(corner))
    k = 70
    for species in ("oak", "birch", "pine"):
        for size, diam in enumerate(CANOPY_SIZES):
            add("canopy", f"tree_{species}_{size}", canopy(species, diam, 24, rr(k)))
            k += 1
    add("small", "tree_shadow", tree_shadow())
    add("small", "stump", stump(rr(90)))
    add("small", "bush_full", bush(True, rr(91)))
    add("small", "bush_bare", bush(False, rr(92)))
    add("small", "deadwood", deadwood(rr(93)))
    add("small", "logs_s", log_pile([(3, 12, 4), (2, 13, 7), (4, 11, 10)]))
    add("small", "logs_m", log_pile([(2, 12, 1), (3, 14, 4), (1, 12, 7), (2, 14, 10)]))
    add("small", "logs_l", log_pile([(1, 14, 0), (0, 15, 3), (2, 15, 6), (0, 14, 9), (1, 15, 11)]))
    add("small", "hearth_0", hearth(0))
    add("small", "hearth_1", hearth(1))
    add("small", "shelter", shelter())
    add("building", "hut", hut(rr(94)))
    add("building", "hut_ghost", hut_ghost(True))
    add("building", "hut_ghost_place", hut_ghost(False))
    for prefix, frames, sleep in (("person", ADULT, ADULT_SLEEP), ("child", CHILD, CHILD_SLEEP)):
        for frame, rows in list(frames.items()) + [("sleep", sleep)]:
            for layer, im in figure_layers(rows).items():
                add("person", f"{prefix}_{layer}_{frame}", im)
    add("small", "shadow", shadow())
    add("small", "carry_back", carry_log("back"))
    add("small", "carry_front", carry_log("front"))
    add("small", "tool_axe", tool_axe())
    add("small", "tool_hammer", tool_hammer())
    add("small", "mark_chop", mark_chop())
    add("small", "select", frame_rect((255, 230, 120, 255)))
    add("small", "hover", frame_rect((255, 255, 255, 120)))
    for kind in ("chop", "stockpile", "hut", "remove", "cancel"):
        add("small", f"icon_{kind}", icon(kind))
    # M2 temporaries (resource nodes, workshops, item kinds, the quarry mark and order icons).
    add("m2", "boulder", boulder(rr(95)))
    add("m2", "flint_outcrop", flint_outcrop(rr(96)))
    add("m2", "knapping_spot", knapping_spot())
    add("m2", "wood_bench", wood_bench())
    add("m2", "drying_rack", drying_rack())
    add("m2", "item_firewood", item_firewood())
    add("m2", "item_stone", item_pile(STONE, STONE_L, STONE_D, ()))
    add("m2", "item_flint", item_pile(FLINT, FLINT_L, FLINT_D, ((4, 8), (9, 12))))
    add("m2", "item_blade", item_blade())
    add("m2", "item_axe", item_axe())
    add("m2", "item_hide", item_hide())
    add("m2", "mark_quarry", mark_quarry())
    for kind in ("build", "quarry"):
        add("m2", f"icon_{kind}", icon(kind))
    # M3 temporaries (walls, doors, floors, bedroll, ghosts, the dig and deconstruct marks and order icons).
    for mask in range(16):
        add("m3", f"log_wall_{mask}", log_wall(mask, rr(100 + mask)))
    for mask in range(16):
        add("m3", f"stone_wall_{mask}", stone_wall(mask, rr(120 + mask)))
    add("m3", "door_h", door("h"))
    add("m3", "door_v", door("v"))
    add("m3", "floor_packed", floor_packed(rr(140)))
    add("m3", "floor_rock", floor_rock(rr(141)))
    add("m3", "bedroll", bedroll(rr(142)))
    for kind in ("wall", "door", "floor", "bedroll"):
        add("m3", f"ghost_{kind}", ghost(kind))
    add("m3", "ghost_refused", ghost_refused())
    add("m3", "mark_dig", mark_dig())
    add("m3", "mark_deconstruct", mark_deconstruct())
    for kind in ("dig", "deconstruct"):
        add("m3", f"icon_{kind}", icon(kind))
    # M4 temporaries (the river ford).
    add("m4", "ford", ford(rr(150)))
    # M4 Part 4: clay banks, the two new workshops and the new item kinds (after the ford: old regions keep their place).
    add("m4", "clay_bank", clay_bank(rr(151)))
    add("m4", "pottery_bench", pottery_bench())
    add("m4", "grinding_stone", grinding_stone())
    add("m4", "item_clay", item_clay())
    add("m4", "item_raw_pot", item_raw_pot())
    add("m4", "item_clay_pot", item_clay_pot())
    add("m4", "item_ground_axe", item_ground_axe())
    return out


def pack(sprites):
    """Shelf-pack on the 16-px grid: each sprite gets whole cells, each section starts a new shelf."""
    rects, x, y, shelf_h, section = {}, 0, 0, 0, None
    for sec, name, im in sprites:
        cw = -(-im.width // T) * T
        ch = -(-im.height // T) * T
        if sec != section or x + cw > SHEET_W:
            y += shelf_h
            x, shelf_h, section = 0, 0, sec
        rects[name] = [x, y, im.width, im.height]
        x += cw
        shelf_h = max(shelf_h, ch)
    return rects, y + shelf_h


def build_atlas():
    sprites = build_sprites()
    rects, height = pack(sprites)
    atlas = Image.new("RGBA", (SHEET_W, height), TRANSPARENT)
    for _, name, im in sprites:
        x, y, _, _ = rects[name]
        atlas.paste(im, (x, y))
    return atlas, rects


def to_json(meta):
    """Sorted keys, one tile rect per line (stable, diff-friendly)."""
    lines = ["{"]
    for k in sorted(k for k in meta if k != "tiles"):
        lines.append(f" {json.dumps(k)}: {json.dumps(meta[k], sort_keys=True)},")
    lines.append(' "tiles": {')
    names = sorted(meta["tiles"])
    for i, n in enumerate(names):
        lines.append(f"  {json.dumps(n)}: {json.dumps(meta['tiles'][n])}" + ("," if i < len(names) - 1 else ""))
    lines += [" }", "}", ""]
    return "\n".join(lines)


def write(out_dir):
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    atlas, rects = build_atlas()
    atlas.save(out / "tiles.png", format="PNG")
    meta = {"atlas": "tiles.png", "palettes": PALETTES, "size": list(atlas.size), "tile_size": T,
            "tiles": dict(sorted(rects.items()))}
    (out / "tiles.json").write_text(to_json(meta))
    return out / "tiles.png", out / "tiles.json"


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    default = Path(__file__).resolve().parents[2] / "assets" / "v8"
    ap.add_argument("--out", default=str(default), help="output directory (default assets/v8)")
    args = ap.parse_args()
    png, js = write(args.out)
    print(f"wrote {png} and {js}")


if __name__ == "__main__":
    main()
