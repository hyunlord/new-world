"""Deterministic temporary tileset for the V8 M1 top-down view (style "B").

Purpose
    Draws every tile the M1 GDScript renderer needs into one RGBA atlas, in code,
    with the palettes and painting approach of the owner's chosen mock-up
    (docs/v8/m1/mockup_gen_reference.py -> docs/images/v8_target_view_b.png):
    dark earthy Dwarf-Fortress-like grid, 16 px tiles, 1 person = 1 tile.
    Visual hierarchy: people/events > items/work spots > walls/floors > ground;
    the ground tiles are the calmest (lowest-contrast) layer. No external art.
    These are placeholders until real art exists.

Usage
    python3 tools/v8/gen_temp_tiles.py [--out DIR]      # default DIR = assets/v8/temp
    Writes DIR/tiles.png (RGBA, 256 x 128) and DIR/tiles.json.

JSON schema (tiles.json)
    {
      "atlas": "tiles.png",
      "tile_size": 16,
      "size": [256, 128],
      "tiles": { "<name>": [x, y, w, h], ... }        # pixels in the atlas, keys sorted
    }
    Every 16x16 tile sits on the 16-px grid; the two 48x48 tiles (hut, hut_ghost)
    start on the grid too. `shadow` is 16x5 at the top of its own grid cell.

Tile notes for the renderer
    deadwood: a thin, dark grey twig with a soft outline, quieter than the
        log items so it does not read as a log waiting to be hauled.
    terrain (16x16, fully opaque): grass_0..3, grass_dark_0/1, dirt_0/1, water_0/1,
        shallow, hill, mountain, forest_floor_0/1. No edge tiles.
    nature / items / hearth / shelter: transparent background, drawn to fill the cell.
    hut (48x48): 3x3 log hut seen from above, door gap in the middle of the south
        wall (cell column 1 of the bottom row). hut_ghost: same outline in light
        greyscale, walls alpha 128 (0.5), floor alpha 48 (~0.19); modulate it
        green/red for valid/invalid placement.
    person_skin / person_hair / person_tunic: GREYSCALE layers (r == g == b);
        draw them in that order at the person's cell with `modulate` = skin, hair,
        tunic colour. Outlines and eyes are near-black so they stay dark after
        modulation. child_* layers: same idea, 12x12 content centred in the cell
        (content box x 2..13, y 2..13); the child head is as wide as the adult's
        (x 4..11) with the eyes two rows above the chin outline and no side locks,
        so the eyes never merge with the outline.
    shadow (16x5): soft dark ellipse, alpha <= 110. Draw it under the person with
        its top at cell y + 11 (adults) or y + 10 (children).
    icons: tool_axe, tool_hammer are drawn in the TOP-RIGHT QUARTER of the cell
        (x 8..15, y 0..7) so they can be drawn at the person's cell unmoved.
        carry_log is a bundle of two light, outlined logs in rows 0..6 of its
        tile; the renderer draws it after the person layers, 4 px (adults) or
        3 px (children) above the person's cell, so it rides on the head and
        covers only the hair, never the eyes.
    overlays: mark_chop (yellow X, opaque, with a 1-px dark outline of alpha 230);
        zone_tint (flat white, alpha 89 = 0.35; modulate for the zone colour);
        zone_edge (1-px white border, alpha 255, transparent inside; modulate);
        select (1-px bright yellow rectangle with heavier corners).

Determinism
    Each tile draws with its own random.Random(SEED * 1000 + k), k a fixed
    constant per tile (no hash(), no time). The PNG is saved without metadata,
    so the same Pillow version gives byte-identical output on every run.
"""
import argparse
import json
import random
from pathlib import Path

from PIL import Image, ImageDraw

T = 16
ATLAS_W, ATLAS_H = 256, 128
SEED = 7  # the mock-up's seed

# mock-up style B palette (mockup_gen_reference.py make_style('B'))
PAL = dict(g=(58, 74, 46), G=(48, 62, 40), d=(84, 70, 52), e=(98, 82, 62), w=(36, 58, 86),
           s=(52, 78, 96), R=(92, 86, 80), W=(112, 80, 50), D=(84, 58, 36), P=(120, 92, 60))
HILL = (72, 76, 54)
FOREST_FLOOR = (46, 52, 36)
OUTLINE = (20, 18, 16)
LOG = (120, 82, 50)
LOG_END = (176, 136, 96)
CARRY = (216, 170, 110)  # a carried log: much lighter than the log items, tunics and ground
CARRY_END = (246, 226, 182)
TRANSPARENT = (0, 0, 0, 0)


def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c)


def rgba(c, a=255):
    return tuple(c) + (a,)


def new(w=T, h=T, fill=TRANSPARENT):
    im = Image.new("RGBA", (w, h), fill)
    return im, ImageDraw.Draw(im)


def noise_fill(im, base, var, density, rr):
    """Speckle a tile with small random deviations of `base` (mock-up noise_fill)."""
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            if rr.random() < density:
                px[x, y] = tuple(max(0, min(255, base[i] + rr.randint(-var, var))) for i in range(3)) + (255,)


# ------------------------------------------------------------------ terrain (calm)
def grass(base, rr, blades=3):
    im, d = new(fill=rgba(base))
    noise_fill(im, base, 6, 0.18, rr)
    for _ in range(blades):
        gx, gy = rr.randrange(1, 15), rr.randrange(2, 15)
        d.line([(gx, gy), (gx, gy - 2)], fill=rgba(shade(base, 1.15)))
    return im


def dirt(base, rr):
    im, d = new(fill=rgba(base))
    noise_fill(im, base, 6, 0.25, rr)
    for _ in range(2):
        d.point((rr.randrange(T), rr.randrange(T)), fill=rgba(shade(base, 0.82)))
    return im


def water(base, rr, ripples=2):
    im, d = new(fill=rgba(base))
    noise_fill(im, base, 4, 0.2, rr)
    for _ in range(ripples):
        wx, wy = rr.randrange(1, 12), rr.randrange(2, 14)
        d.line([(wx, wy), (wx + 3, wy)], fill=rgba(shade(base, 1.3)))
    return im


def hill(rr):
    im, d = new(fill=rgba(HILL))
    noise_fill(im, HILL, 5, 0.2, rr)
    # two soft contour arcs: a raised, grassy-stony slope
    d.arc([1, 3, 14, 16], 200, 340, fill=rgba(shade(HILL, 0.84)))
    d.arc([3, 0, 12, 9], 200, 340, fill=rgba(shade(HILL, 1.12)))
    d.point((rr.randrange(2, 14), rr.randrange(9, 14)), fill=rgba((96, 92, 84)))
    return im


def mountain(rr):
    """Unmined rock top (mock-up rock_tile, not exposed), noise toned down."""
    dark = shade(PAL["R"], 0.5)
    im, d = new(fill=rgba(dark))
    noise_fill(im, dark, 7, 0.35, rr)
    x0, y0 = rr.randrange(3, 11), rr.randrange(2, 9)
    d.line([(x0, y0), (x0 + 2, y0 + 3), (x0 + 1, y0 + 5)], fill=rgba(shade(dark, 0.7)))
    d.point((x0 + 5, y0 + 2), fill=rgba(shade(dark, 1.35)))
    return im


def forest_floor(rr):
    im, d = new(fill=rgba(FOREST_FLOOR))
    noise_fill(im, FOREST_FLOOR, 5, 0.22, rr)
    for _ in range(4):  # leaf litter
        d.point((rr.randrange(T), rr.randrange(T)), fill=rgba((60, 52, 36)))
    return im


# ------------------------------------------------------------------ nature
def tree(variant, rr):
    im, d = new()
    g = [(46, 84, 44), (40, 76, 46), (56, 90, 40)][variant]
    if variant == 2:  # rounder, lumpier canopy: three overlapping crowns
        d.ellipse([0, 2, 10, 12], fill=g, outline=OUTLINE)
        d.ellipse([5, 0, 15, 10], fill=g, outline=OUTLINE)
        d.ellipse([3, 5, 14, 15], fill=g, outline=OUTLINE)
        d.ellipse([3, 4, 9, 10], fill=g)
        d.ellipse([6, 2, 11, 7], fill=shade(g, 1.25))
        d.ellipse([7, 9, 12, 13], fill=shade(g, 0.8))
        return im
    inset = 0 if variant == 0 else 1
    d.ellipse([inset, inset, 15 - inset, 15 - inset], fill=g, outline=OUTLINE)
    d.ellipse([3, 2, 10, 9], fill=shade(g, 1.25))
    d.point((5, 4), fill=shade(g, 1.5))
    d.ellipse([7, 8, 13, 13], fill=shade(g, 0.8))
    for _ in range(3):
        d.point((rr.randrange(4, 12), rr.randrange(4, 12)), fill=shade(g, 0.85))
    return im


def bush(berries, rr):
    im, d = new()
    g = (58, 96, 52) if berries else (64, 82, 50)
    d.ellipse([2, 3, 13, 14], fill=g, outline=OUTLINE)
    d.ellipse([4, 4, 9, 8], fill=shade(g, 1.2))
    if berries:
        for bx, by in ((4, 6), (9, 5), (6, 10), (10, 9), (7, 7)):
            d.point((bx, by), fill=(190, 40, 50))
            d.point((bx + 1, by), fill=(220, 70, 70))
    else:
        for bx, by in ((5, 7), (9, 6), (7, 11), (10, 10)):
            d.point((bx, by), fill=(96, 74, 52))
    return im


def deadwood(rr):
    """A thin, weathered grey twig: 1-px wood inside a soft dark rim (not near-black), so it
    stays quieter than the log items and does not read as a dropped log."""
    im, d = new()
    c = (86, 82, 74)
    rim = (46, 44, 40)
    d.line([(3, 12), (12, 6)], fill=rim, width=3)
    d.line([(7, 9), (6, 5)], fill=rim, width=3)
    d.line([(3, 12), (12, 6)], fill=c)
    d.line([(7, 9), (6, 5)], fill=shade(c, 0.9))
    d.point((12, 6), fill=(100, 96, 88))
    return im


# ------------------------------------------------------------------ items
def log_stack(rows):
    """rows: list of (x0, x1, y) logs, each 3 px tall, with a lit end grain."""
    im, d = new()
    for x0, x1, y in rows:
        d.rectangle([x0, y, x1, y + 2], fill=LOG, outline=OUTLINE)
        d.point((x1 - 1, y + 1), fill=LOG_END)
    return im


# ------------------------------------------------------------------ buildings
def hearth():
    im, d = new()
    d.ellipse([1, 1, 14, 14], fill=(110, 104, 98), outline=OUTLINE)
    d.ellipse([4, 4, 11, 11], fill=(50, 36, 28))
    d.polygon([(7, 4), (10, 9), (8, 11), (5, 9)], fill=(240, 130, 40))
    d.polygon([(7, 6), (9, 9), (7, 10), (6, 9)], fill=(255, 220, 110))
    return im


def shelter():
    """V7 lean-to from above: a sloped roof of branches propped on a ridge pole,
    open (dark) on the south side."""
    im, d = new()
    roof = (104, 82, 52)
    d.rectangle([1, 2, 14, 12], fill=roof, outline=OUTLINE)
    for x in range(3, 14, 3):  # branches running down the slope
        d.line([(x, 3), (x - 1, 11)], fill=shade(roof, 0.72))
    d.line([(1, 3), (14, 3)], fill=(132, 98, 60))          # ridge pole
    d.point((1, 2), fill=(150, 116, 74)); d.point((14, 2), fill=(150, 116, 74))
    d.rectangle([2, 12, 13, 14], fill=(36, 28, 22), outline=OUTLINE)  # open front
    for x in (4, 9):  # leaves on the roof
        d.point((x, 6), fill=(70, 92, 52)); d.point((x + 2, 9), fill=(70, 92, 52))
    return im


HUT_WALL = 6  # wall thickness in px


def hut_shape(d, rr, wall, wall_dark, wall_light, floor, outline, door_dark, floor_noise):
    """Draw the 48x48 hut geometry with the given colours (shared by hut and ghost)."""
    S, k = 48, HUT_WALL
    # floor
    d.rectangle([k, k, S - 1 - k, S - 1 - k], fill=floor)
    if floor_noise:
        for _ in range(110):
            x, y = rr.randrange(k, S - k), rr.randrange(k, S - k)
            d.point((x, y), fill=floor_noise[rr.randrange(len(floor_noise))])
    # inner face of the north wall (depth, like the mock-up's below_open shading)
    d.rectangle([k, k, S - 1 - k, k + 2], fill=wall_dark)
    # walls: north/south run logs east-west, west/east run them north-south
    d.rectangle([0, 0, S - 1, k - 1], fill=wall)
    d.rectangle([0, S - k, S - 1, S - 1], fill=wall)
    d.rectangle([0, 0, k - 1, S - 1], fill=wall)
    d.rectangle([S - k, 0, S - 1, S - 1], fill=wall)
    for y in (2, 4):
        d.line([(k, y), (S - 1 - k, y)], fill=wall_dark)
        d.line([(k, S - k + y), (S - 1 - k, S - k + y)], fill=wall_dark)
    for x in (2, 4):
        d.line([(x, k), (x, S - 1 - k)], fill=wall_dark)
        d.line([(S - k + x, k), (S - k + x, S - 1 - k)], fill=wall_dark)
    d.line([(0, 0), (S - 1, 0)], fill=wall_light)
    d.line([(0, S - k), (S - 1, S - k)], fill=wall_light)
    # corner posts
    for cx, cy in ((0, 0), (S - k, 0), (0, S - k), (S - k, S - k)):
        d.rectangle([cx, cy, cx + k - 1, cy + k - 1], fill=wall_light, outline=outline)
        d.point((cx + k // 2, cy + k // 2), fill=wall_dark)
    # segment joints every 16 px (one per grid cell, mock-up per-tile outline)
    for t in (16, 32):
        d.line([(t, 0), (t, k - 1)], fill=outline)
        d.line([(t, S - k), (t, S - 1)], fill=outline)
        d.line([(0, t), (k - 1, t)], fill=outline)
        d.line([(S - k, t), (S - 1, t)], fill=outline)
    # door gap in the south wall, middle cell
    d.rectangle([19, S - k, 28, S - 1], fill=door_dark)
    d.line([(18, S - k), (18, S - 1)], fill=outline)
    d.line([(29, S - k), (29, S - 1)], fill=outline)
    # outlines
    d.rectangle([0, 0, S - 1, S - 1], outline=outline)
    d.rectangle([k - 1, k - 1, S - k, S - k], outline=outline)
    d.rectangle([19, S - k, 28, S - k], fill=door_dark)  # reopen the inner outline at the door


def hut(rr):
    im, d = new(48, 48)
    floor = PAL["e"]
    hut_shape(d, rr, wall=PAL["W"], wall_dark=shade(PAL["W"], 0.68), wall_light=shade(PAL["W"], 1.25),
              floor=floor, outline=OUTLINE, door_dark=(46, 32, 22),
              floor_noise=[shade(floor, 0.9), shade(floor, 1.08), shade(floor, 0.95)])
    return im


def hut_ghost(rr):
    im, d = new(48, 48)
    a_wall = 128
    hut_shape(d, rr, wall=(225, 225, 225, a_wall), wall_dark=(190, 190, 190, a_wall),
              wall_light=(245, 245, 245, a_wall), floor=(210, 210, 210, 48), outline=(255, 255, 255, 200),
              door_dark=(160, 160, 160, 72), floor_noise=None)
    return im


# ------------------------------------------------------------------ people (greyscale layers)
G_OUT = (24, 24, 24, 255)


def grey(v, a=255):
    return (v, v, v, a)


def person_layers(child):
    """Returns (skin, hair, tunic) greyscale layers; mock-up person_sprite geometry."""
    skin, ds = new()
    hair, dh = new()
    tunic, dt = new()
    if not child:
        hx0, hx1, top = 4, 11, 0          # head box
        tx0, tx1, ty0, ty1 = 3, 12, 6, 11  # tunic box
        legs = [(5, 6), (9, 10)]
        ly0, ly1 = 12, 14
    else:
        # As wide as the adult head (the eyes need a skin pixel on both sides), but shorter.
        hx0, hx1, top = 4, 11, 2
        tx0, tx1, ty0, ty1 = 4, 11, 7, 10
        legs = [(5, 6), (9, 10)]
        ly0, ly1 = 11, 13
    # skin: head with outline + eyes, legs
    ds.rectangle([hx0, top, hx1, top + (5 if not child else 4)], fill=grey(232), outline=G_OUT)
    # Child eyes one row higher than before: a skin row separates them from the chin outline (they merged
    # into a dark "beard" bar on the last face row; player review round 1, #6).
    eye_y = top + (3 if not child else 2)
    ds.point((hx0 + 2, eye_y), fill=G_OUT)
    ds.point((hx1 - 2, eye_y), fill=G_OUT)
    for lx0, lx1 in legs:
        ds.rectangle([lx0, ly0, lx1, ly1], fill=grey(190))
    # hair: top of the head inside the outline, plus side locks (adults only: on the child they would sit
    # beside the eyes)
    dh.line([(hx0 + 1, top + 1), (hx1 - 1, top + 1)], fill=grey(235))
    if not child:
        dh.point((hx0 + 1, top + 2), fill=grey(215))
        dh.point((hx1 - 1, top + 2), fill=grey(215))
    dh.line([(hx0 + 1, top), (hx1 - 1, top)], fill=grey(200))  # hair covers the top outline
    # tunic: outlined body with a lit shoulder line
    dt.rectangle([tx0, ty0, tx1, ty1], fill=grey(212), outline=G_OUT)
    dt.line([(tx0 + 1, ty0 + 1), (tx1 - 1, ty0 + 1)], fill=grey(255))
    dt.line([(tx0 + 1, ty1 - 1), (tx1 - 1, ty1 - 1)], fill=grey(176))
    return skin, hair, tunic


def shadow():
    im, d = new(16, 5)
    d.ellipse([1, 0, 14, 4], fill=(0, 0, 0, 70))
    d.ellipse([3, 1, 12, 3], fill=(0, 0, 0, 105))
    return im


# ------------------------------------------------------------------ icons
def carry_log():
    """A bundle of two light logs with a dark outline in rows 0..6, carried on the head
    (the renderer draws it a few px above the person's cell)."""
    im, d = new()
    for x0, x1, y in ((1, 13, 0), (3, 15, 3)):  # upper log, lower log (they share an outline row)
        d.rectangle([x0, y, x1, y + 3], fill=CARRY, outline=OUTLINE)
        d.line([(x0 + 1, y + 1), (x1 - 3, y + 1)], fill=shade(CARRY, 1.12))
        d.rectangle([x1 - 2, y, x1, y + 3], fill=CARRY_END, outline=OUTLINE)
        d.point((x1 - 1, y + 2), fill=shade(CARRY_END, 0.8))
    return im


def tool_axe():
    """Top-right quarter (x 8..15, y 0..7): diagonal haft, wide pale stone blade."""
    im, d = new()
    d.line([(9, 6), (12, 1)], fill=OUTLINE, width=3)
    d.line([(9, 6), (12, 1)], fill=(176, 128, 80))      # haft
    d.polygon([(12, 0), (15, 1), (15, 6), (12, 3)], fill=(206, 210, 220), outline=OUTLINE)  # blade
    d.point((14, 3), fill=(240, 242, 248))
    return im


def tool_hammer():
    """Top-right quarter (x 8..15, y 0..7): upright haft, blocky dark stone head across it."""
    im, d = new()
    d.line([(11, 7), (11, 3)], fill=OUTLINE, width=3)
    d.line([(11, 7), (11, 3)], fill=(176, 128, 80))
    d.rectangle([8, 0, 15, 3], fill=(120, 116, 110), outline=OUTLINE)
    d.line([(9, 1), (14, 1)], fill=(170, 166, 158))
    return im


# ------------------------------------------------------------------ overlays
ACCENT = (250, 206, 60)


def outline_alpha(im, color):
    """Paint `color` on every transparent pixel 8-adjacent to an opaque one."""
    src = im.copy().load()
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            if src[x, y][3]:
                continue
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < im.width and 0 <= ny < im.height and src[nx, ny][3] == 255:
                        px[x, y] = color
                        break
                else:
                    continue
                break
    return im


def mark_chop():
    im, d = new()
    d.line([(3, 3), (12, 12)], fill=ACCENT + (255,), width=3)
    d.line([(12, 3), (3, 12)], fill=ACCENT + (255,), width=3)
    return outline_alpha(im, (24, 18, 10, 230))


def zone_tint():
    im, _ = new(fill=(255, 255, 255, 89))
    return im


def zone_edge():
    im, d = new()
    d.rectangle([0, 0, T - 1, T - 1], outline=(255, 255, 255, 255))
    return im


def select():
    im, d = new()
    c = (255, 230, 120, 255)
    d.rectangle([0, 0, T - 1, T - 1], outline=c)
    for cx, cy, sx, sy in ((0, 0, 1, 1), (15, 0, -1, 1), (0, 15, 1, -1), (15, 15, -1, -1)):
        d.line([(cx + sx, cy + sy), (cx + 3 * sx, cy + sy)], fill=c)
        d.line([(cx + sx, cy + sy), (cx + sx, cy + 3 * sy)], fill=c)
    return im


# ------------------------------------------------------------------ atlas
def build_tiles():
    """Returns [(name, col, row, image)] in a fixed order; col/row on the 16-px grid."""
    def rr(k):
        return random.Random(SEED * 1000 + k)

    adult = person_layers(child=False)
    kid = person_layers(child=True)
    rows = [
        # row 0: terrain
        [("grass_0", grass(PAL["g"], rr(1))), ("grass_1", grass(PAL["g"], rr(2))),
         ("grass_2", grass(PAL["g"], rr(3))), ("grass_3", grass(PAL["g"], rr(4), blades=2)),
         ("grass_dark_0", grass(PAL["G"], rr(5))), ("grass_dark_1", grass(PAL["G"], rr(6))),
         ("dirt_0", dirt(PAL["d"], rr(7))), ("dirt_1", dirt(PAL["d"], rr(8))),
         ("water_0", water(PAL["w"], rr(9))), ("water_1", water(PAL["w"], rr(10))),
         ("shallow", water(PAL["s"], rr(11), ripples=1)), ("hill", hill(rr(12))),
         ("mountain", mountain(rr(13))), ("forest_floor_0", forest_floor(rr(14))),
         ("forest_floor_1", forest_floor(rr(15)))],
        # row 1: nature, items, small buildings, icons
        [("tree_0", tree(0, rr(20))), ("tree_1", tree(1, rr(21))), ("tree_2", tree(2, rr(22))),
         ("bush_full", bush(True, rr(23))), ("bush_bare", bush(False, rr(24))), ("deadwood", deadwood(rr(25))),
         ("logs_s", log_stack([(3, 12, 7), (4, 11, 10)])),
         ("logs_m", log_stack([(4, 11, 3), (2, 13, 6), (3, 12, 9), (2, 13, 12)])),
         ("logs_l", log_stack([(1, 14, 2), (1, 14, 5), (2, 13, 8), (2, 13, 11), (3, 12, 13)])),
         ("hearth", hearth()), ("shelter", shelter()),
         ("carry_log", carry_log()), ("tool_axe", tool_axe()), ("tool_hammer", tool_hammer())],
        # row 2: people, shadow, overlays
        [("person_skin", adult[0]), ("person_hair", adult[1]), ("person_tunic", adult[2]),
         ("child_skin", kid[0]), ("child_hair", kid[1]), ("child_tunic", kid[2]),
         ("shadow", shadow()), ("mark_chop", mark_chop()), ("zone_tint", zone_tint()),
         ("zone_edge", zone_edge()), ("select", select())],
    ]
    out = []
    for r, row in enumerate(rows):
        for c, (name, im) in enumerate(row):
            out.append((name, c, r, im))
    # rows 3..5: the 48x48 buildings
    out.append(("hut", 0, 3, hut(rr(30))))
    out.append(("hut_ghost", 3, 3, hut_ghost(rr(30))))
    return out


def build_atlas():
    atlas = Image.new("RGBA", (ATLAS_W, ATLAS_H), TRANSPARENT)
    rects = {}
    for name, col, row, im in build_tiles():
        x, y = col * T, row * T
        atlas.paste(im, (x, y))
        rects[name] = [x, y, im.width, im.height]
    return atlas, rects


def to_json(meta):
    """Sorted keys, one tile rect per line (stable, diff-friendly)."""
    lines = ["{"]
    for k in sorted(k for k in meta if k != "tiles"):
        lines.append(f" {json.dumps(k)}: {json.dumps(meta[k])},")
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
    meta = {"atlas": "tiles.png", "tile_size": T, "size": [ATLAS_W, ATLAS_H],
            "tiles": dict(sorted(rects.items()))}
    (out / "tiles.json").write_text(to_json(meta))
    return out / "tiles.png", out / "tiles.json"


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    default = Path(__file__).resolve().parents[2] / "assets" / "v8" / "temp"
    ap.add_argument("--out", default=str(default), help="output directory (default assets/v8/temp)")
    args = ap.parse_args()
    png, js = write(args.out)
    print(f"wrote {png} and {js}")


if __name__ == "__main__":
    main()
