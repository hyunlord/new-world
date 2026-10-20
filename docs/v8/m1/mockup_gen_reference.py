# Reference only: the PIL script that drew the V8 view-B mock-up (docs/images/v8_target_view_b.png).
# usage: python3 mockup_gen_reference.py <out_dir> <path to Galmuri11.ttf>
"""WorldSim V8 target-screen mockups (composition/scale/density study, not final art).

Builds one 96x64 settlement map and renders it in two styles:
  B = Dwarf-Fortress-like top-down grid (dark earthy, 1 person = 1 tile)
  A = current D3 3/4 style shrunk to 1 person = 1 tile
Outputs 1920x1080 PNGs.
"""
import random, math, sys
from PIL import Image, ImageDraw, ImageFont

OUT = sys.argv[1]
FONT = sys.argv[2]
T = 16
W, H = 96, 64
rng = random.Random(7)

# ------------------------------------------------------------------ map
# codes: g grass, G dark grass, d dirt, e packed earth floor, w water, s shallow
# R rock (unmined), F mined floor, W wood wall, S stone wall, D door, P plank floor
# t tree, b bush(berry), o boulder, f farm, r reed
grid = [['g'] * W for _ in range(H)]
obj = {}       # (x,y) -> object name
items = {}     # (x,y) -> item stack
zones = []     # (x0,y0,x1,y1,kind)
desig = {}     # (x,y) -> 'dig'|'chop'|'build'

def setg(x, y, c):
    if 0 <= x < W and 0 <= y < H:
        grid[y][x] = c

def value_noise(seed, scale):
    r = random.Random(seed)
    gw, gh = W // scale + 2, H // scale + 2
    g = [[r.random() for _ in range(gw)] for _ in range(gh)]
    def f(x, y):
        fx, fy = x / scale, y / scale
        x0, y0 = int(fx), int(fy); tx, ty = fx - x0, fy - y0
        tx, ty = tx * tx * (3 - 2 * tx), ty * ty * (3 - 2 * ty)
        a = g[y0][x0] * (1 - tx) + g[y0][x0 + 1] * tx
        b = g[y0 + 1][x0] * (1 - tx) + g[y0 + 1][x0 + 1] * tx
        return a * (1 - ty) + b * ty
    return f

n1, n2 = value_noise(3, 9), value_noise(5, 4)
NOISE = [[0.65 * n1(x, y) + 0.35 * n2(x, y) for x in range(W)] for y in range(H)]
# dark grass patches (smooth clusters)
for y in range(H):
    for x in range(W):
        if NOISE[y][x] > 0.58:
            grid[y][x] = 'G'

# rock hill (east/north-east)
for y in range(H):
    for x in range(W):
        edge = 60 + 5 * math.sin(y * 0.21) + 3 * math.sin(y * 0.53 + 1.0)
        if y < 44 + 4 * math.sin(x * 0.3) and x > edge:
            grid[y][x] = 'R'

# river
for y in range(H):
    cx = 19 + 3 * math.sin(y * 0.12) + 2 * math.sin(y * 0.31)
    for x in range(int(cx) - 2, int(cx) + 3):
        setg(x, y, 'w')
    setg(int(cx) - 3, y, 's'); setg(int(cx) + 3, y, 's')
    if rng.random() < 0.4:
        setg(int(cx) + 4, y, 'r') if grid[y][min(W-1,int(cx)+4)] in 'gG' else None

# forests (west bank + south)
for _ in range(560):
    if rng.random() < 0.5:
        x, y = rng.randrange(0, 14), rng.randrange(H)
    elif rng.random() < 0.6:
        x, y = rng.randrange(20, 60), rng.randrange(50, H)
    else:
        x, y = rng.randrange(W), rng.randrange(H)
    if grid[y][x] in 'gG' and (x, y) not in obj:
        obj[(x, y)] = 'tree'

def clear(x0, y0, x1, y1):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            obj.pop((x, y), None); items.pop((x, y), None)

# settlement core clearing
clear(24, 18, 60, 47)
# worn ground: smooth falloff around the settlement centre
for y in range(18, 48):
    for x in range(24, 60):
        dist = math.hypot((x - 41) / 15.0, (y - 32) / 11.0)
        if grid[y][x] in 'gG' and NOISE[y][x] + 0.5 * (1 - dist) > 0.86:
            grid[y][x] = 'd'

def path(a, b):
    (x0, y0), (x1, y1) = a, b
    n = max(abs(x1 - x0), abs(y1 - y0))
    for i in range(n + 1):
        x = round(x0 + (x1 - x0) * i / n + 0.6 * math.sin(i * 0.5)); y = round(y0 + (y1 - y0) * i / n)
        for dx in (0, 1):
            if 0 <= x + dx < W and grid[y][x + dx] in 'gG':
                grid[y][x + dx] = 'd'

# communal hearth plaza
for y in range(29, 36):
    for x in range(36, 43):
        grid[y][x] = 'e'
obj[(39, 32)] = 'hearth'
for p in [(37, 31), (41, 31), (37, 34), (41, 34), (39, 30), (39, 35)]:
    obj[p] = 'log_seat'

def hut(x0, y0, w, h, door, wall='W', floor='e'):
    for y in range(y0, y0 + h):
        for x in range(x0, x0 + w):
            edge = x in (x0, x0 + w - 1) or y in (y0, y0 + h - 1)
            grid[y][x] = wall if edge else floor
    grid[door[1]][door[0]] = 'D'

hut(28, 21, 6, 5, (30, 25)); obj[(29, 22)] = 'bed'; obj[(32, 22)] = 'bed'; obj[(29, 23)] = 'bed'; obj[(32, 23)] = 'jar'
hut(35, 21, 6, 5, (37, 25)); obj[(36, 22)] = 'bed'; obj[(39, 22)] = 'bed'; obj[(36, 23)] = 'jar'; obj[(39, 23)] = 'bed'
hut(42, 21, 7, 5, (45, 25), floor='P'); obj[(43, 22)] = 'bed'; obj[(47, 22)] = 'bed'; obj[(43, 23)] = 'bed'; obj[(47, 23)] = 'jar'; obj[(45, 22)] = 'hearth_s'
hut(28, 38, 6, 5, (30, 38)); obj[(29, 40)] = 'bed'; obj[(32, 40)] = 'bed'; obj[(29, 41)] = 'jar'; obj[(32, 41)] = 'bed'
# blueprint hut (planned)
for y in range(38, 43):
    for x in range(35, 41):
        if x in (35, 40) or y in (38, 42):
            desig[(x, y)] = 'build'

# outdoor stockpile zone
zones.append((44, 29, 50, 34, 'stock'))
stock_items = ['log', 'log', 'log', 'stone', 'stone', 'flint', 'berries', 'berries', 'hide', 'hide', 'stone', 'log', 'reeds', 'grain', 'grain']
for y in range(29, 35):
    for x in range(44, 51):
        if rng.random() < 0.72:
            items[(x, y)] = rng.choice(stock_items)

# workshop row
obj[(52, 28)] = 'knap'; obj[(52, 31)] = 'rack'; obj[(52, 34)] = 'wood_ws'; obj[(54, 31)] = 'kiln'
items[(53, 28)] = 'flint'; items[(53, 34)] = 'log'; items[(55, 32)] = 'pot'; items[(55, 30)] = 'pot'
zones.append((51, 27, 56, 35, 'work'))

# farm plot (early agriculture)
for y in range(40, 45):
    for x in range(44, 53):
        grid[y][x] = 'f'
zones.append((44, 40, 52, 44, 'farm'))

# berry bushes east of the river
for _ in range(16):
    x, y = rng.randrange(25, 31), rng.randrange(6, 58)
    if grid[y][x] in 'gGd' and (x, y) not in obj:
        obj[(x, y)] = 'bush'
for _ in range(10):
    x, y = rng.randrange(W), rng.randrange(H)
    if grid[y][x] in 'gG' and (x, y) not in obj:
        obj[(x, y)] = 'boulder'

# cave dug into the hill: corridor + rooms
def mine(x0, y0, x1, y1):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if grid[y][x] == 'R':
                grid[y][x] = 'F'

entry_y = 30
ex = next(x for x in range(W) if grid[entry_y][x] == 'R')
# make sure the rock mass is solid around the cave so rooms sit inside it
for y in range(entry_y - 10, entry_y + 10):
    for x in range(ex + 2, ex + 30):
        if 0 <= x < W:
            grid[y][x] = 'R'; obj.pop((x, y), None)
mine(ex, entry_y, ex + 16, entry_y + 1)           # main corridor
cx0 = ex + 5
mine(cx0 + 2, entry_y - 1, cx0 + 2, entry_y - 1)
mine(cx0, entry_y - 7, cx0 + 6, entry_y - 2)    # storeroom
zones.append((cx0, entry_y - 7, cx0 + 6, entry_y - 2, 'stock'))
for y in range(entry_y - 7, entry_y - 1):
    for x in range(cx0, cx0 + 7):
        if rng.random() < 0.7:
            items[(x, y)] = rng.choice(['grain', 'berries', 'pot', 'pot', 'hide', 'stone', 'flint'])
for i in range(3):                                # sleeping chambers south of corridor
    bx = ex + 4 + i * 4
    mine(bx + 1, entry_y + 2, bx + 1, entry_y + 2)
    mine(bx, entry_y + 3, bx + 2, entry_y + 5)
    obj[(bx, entry_y + 5)] = 'bed'; obj[(bx + 2, entry_y + 5)] = 'bed'; obj[(bx + 2, entry_y + 3)] = 'jar'
mine(ex + 17, entry_y - 3, ex + 21, entry_y + 4)  # mining face
for y in range(entry_y - 4, entry_y + 6):         # dig designations ahead of the face
    for x in range(ex + 22, ex + 25):
        if grid[y][x] == 'R':
            desig[(x, y)] = 'dig'
ore = set()
for _ in range(18):
    x, y = ex + 22 + rng.randrange(0, 6), entry_y - 5 + rng.randrange(0, 11)
    if 0 <= x < W and grid[y][x] == 'R':
        ore.add((x, y))
items[(ex + 19, entry_y)] = 'ore'; items[(ex + 18, entry_y + 2)] = 'stone'; items[(ex + 20, entry_y - 2)] = 'stone'
# worn paths: hut doors -> hearth -> stockpile -> cave
for a, b in [((30, 26), (38, 30)), ((37, 26), (39, 29)), ((45, 26), (41, 30)), ((30, 37), (37, 35)),
             ((43, 32), (44, 32)), ((51, 31), (ex, entry_y)), ((47, 35), (48, 39)), ((42, 36), (40, 46)), ((26, 32), (35, 32))]:
    path(a, b)

# chop designations at the south forest edge
for (x, y), o in list(obj.items()):
    if o == 'tree' and 46 <= y <= 52 and 30 <= x <= 44 and rng.random() < 0.5:
        desig[(x, y)] = 'chop'

# ------------------------------------------------------------------ people
HOUSE = [(176, 132, 70), (150, 74, 52), (96, 116, 70), (82, 98, 128), (190, 176, 140)]
HAIR = [(40, 30, 24), (70, 46, 30), (110, 74, 40), (30, 26, 26), (200, 196, 190)]
SKIN = [(196, 146, 110), (170, 120, 86), (140, 96, 66), (214, 172, 136)]
people = []
def person(x, y, carry=None, child=False, elder=False, act=None):
    walk = lambda: None
    if not (0 <= x < W and 0 <= y < H):
        return
    if grid[y][x] in 'RwWS':
        return
    people.append(dict(x=x, y=y, h=rng.choice(HOUSE), hair=(200, 196, 190) if elder else rng.choice(HAIR[:4]),
                       skin=rng.choice(SKIN), carry=carry, child=child, elder=elder, act=act,
                       sel=False))

for p in [(38, 31), (40, 31), (38, 34), (40, 34), (37, 33), (41, 33)]:
    person(*p, elder=rng.random() < 0.3)
person(39, 34, child=True); person(36, 32, child=True)
person(52, 29, act='knap'); person(53, 31, act='rack'); person(53, 35, act='wood'); person(54, 30, act='kiln')
for x, y in [(46, 41), (48, 42), (50, 43), (45, 43)]:
    person(x, y, act='farm')
person(43, 31, carry='log'); person(47, 35, carry='berries'); person(58, 30, carry='stone'); person(55, 26, carry='hide')
person(ex + 2, entry_y, carry='grain'); person(ex + 7, entry_y + 1, carry='stone'); person(ex + 19, entry_y, act='mine'); person(ex + 20, entry_y - 2, act='mine')
person(cx0 + 2, entry_y - 5); person(ex + 5, entry_y + 4)
for x, y in [(33, 49), (37, 48), (41, 50)]:
    person(x, y, act='chop')
for x, y in [(22, 14), (23, 26), (21, 40)]:
    person(x, y, carry='berries')
person(31, 23); person(37, 22, child=True); person(45, 23, elder=True); person(30, 40)
person(18, 33, act='fish'); person(36, 40, act='build'); person(39, 42, act='build')
for x, y in [(34, 28), (44, 27), (27, 33), (57, 39), (49, 37), (33, 36)]:
    person(x, y, child=rng.random() < 0.25)
people[8]['sel'] = True   # the knapper, shown in the detail panel
animals = [('deer', 5, 20), ('deer', 6, 22), ('deer', 4, 23), ('wolf', 2, 58), ('wolf', 4, 60)]

# ------------------------------------------------------------------ painters
def noise_fill(img, base, var, density, rr, colors=None):
    px = img.load()
    for y in range(T):
        for x in range(T):
            if rr.random() < density:
                c = rr.choice(colors) if colors else tuple(max(0, min(255, base[i] + rr.randint(-var, var))) for i in range(3))
                px[x, y] = c + (255,)

def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c)

class Style:
    pass

def make_style(name):
    s = Style()
    s.name = name
    if name == 'B':
        s.pal = dict(g=(58, 74, 46), G=(48, 62, 40), d=(84, 70, 52), e=(98, 82, 62), w=(36, 58, 86), s=(52, 78, 96),
                     R=(92, 86, 80), F=(70, 64, 60), W=(112, 80, 50), S=(120, 114, 106), D=(84, 58, 36), P=(120, 92, 60),
                     f=(88, 66, 44), r=(58, 74, 46))
        s.outline = (20, 18, 16)
    else:
        s.pal = dict(g=(96, 116, 74), G=(84, 104, 66), d=(128, 110, 84), e=(146, 124, 92), w=(52, 102, 150), s=(84, 132, 160),
                     R=(130, 126, 118), F=(108, 100, 94), W=(140, 98, 58), S=(150, 144, 134), D=(104, 70, 40), P=(160, 122, 80),
                     f=(118, 90, 60), r=(96, 116, 74))
        s.outline = (26, 22, 20)
    s.cache = {}
    return s

def ground_tile(s, c, x, y):
    key = (c, (x * 7 + y * 13) % 4)
    if key in s.cache:
        return s.cache[key]
    rr = random.Random(hash(key) & 0xffff)
    base = s.pal[c]
    im = Image.new('RGBA', (T, T), base + (255,))
    d = ImageDraw.Draw(im)
    if c in 'gGr':
        noise_fill(im, base, 10, 0.18, rr)
        for _ in range(3):
            gx, gy = rr.randrange(1, 15), rr.randrange(2, 15)
            d.line([(gx, gy), (gx, gy - 2)], fill=shade(base, 1.25))
    elif c in 'de':
        noise_fill(im, base, 8, 0.25, rr)
        if rr.random() < 0.5:
            d.point((rr.randrange(T), rr.randrange(T)), fill=shade(base, 0.7))
    elif c == 'f':
        for yy in range(1, T, 4):
            d.line([(0, yy), (T - 1, yy)], fill=shade(base, 0.72))
            d.line([(0, yy + 1), (T - 1, yy + 1)], fill=shade(base, 1.12))
        for xx in range(2, T, 5):
            d.line([(xx, 6), (xx, 3)], fill=(110, 140, 60)); d.point((xx - 1, 3), fill=(150, 170, 80)); d.point((xx + 1, 4), fill=(150, 170, 80))
            d.line([(xx + 2, 14), (xx + 2, 11)], fill=(110, 140, 60))
    elif c in 'ws':
        noise_fill(im, base, 4, 0.2, rr)
        for _ in range(2):
            wx, wy = rr.randrange(1, 12), rr.randrange(2, 14)
            d.line([(wx, wy), (wx + 3, wy)], fill=shade(base, 1.35))
    elif c == 'F':
        noise_fill(im, base, 5, 0.15, rr)
        d.line([(0, 0), (T - 1, 0)], fill=shade(base, 0.85)); d.line([(0, 0), (0, T - 1)], fill=shade(base, 0.85))
    elif c == 'P':
        for xx in range(0, T, 4):
            d.line([(xx, 0), (xx, T - 1)], fill=shade(base, 0.7))
        d.point((rr.randrange(T), rr.randrange(T)), fill=shade(base, 0.6))
    s.cache[key] = im
    return im

def rock_tile(s, x, y, exposed_below, exposed, is_ore):
    base = s.pal['R']
    rr = random.Random(x * 991 + y * 31)
    im = Image.new('RGBA', (T, T), base + (255,))
    d = ImageDraw.Draw(im)
    if not exposed:
        dark = shade(base, 0.5 if s.name == 'B' else 0.72)
        im.paste(dark + (255,), (0, 0, T, T))
        noise_fill(im, dark, 10, 0.35, rr)
        if rr.random() < 0.35:
            x0, y0 = rr.randrange(2, 12), rr.randrange(2, 12)
            d.line([(x0, y0), (x0 + rr.randint(-3, 3), y0 + 3), (x0 + rr.randint(-2, 4), y0 + 5)], fill=shade(dark, 0.6))
        if rr.random() < 0.2:
            x0, y0 = rr.randrange(2, 13), rr.randrange(2, 13)
            d.point((x0, y0), fill=shade(dark, 1.5)); d.point((x0 + 1, y0), fill=shade(dark, 1.25))
    else:
        # cut blocks
        noise_fill(im, base, 7, 0.2, rr)
        d.line([(0, 7), (T - 1, 7)], fill=shade(base, 0.75))
        d.line([(5, 0), (5, 7)], fill=shade(base, 0.75)); d.line([(11, 8), (11, 15)], fill=shade(base, 0.75))
        d.line([(0, 0), (T - 1, 0)], fill=shade(base, 1.3))
        if exposed_below:
            d.rectangle([0, 11, T - 1, T - 1], fill=shade(base, 0.55))
            d.line([(0, 11), (T - 1, 11)], fill=shade(base, 0.4))
    if is_ore:
        for _ in range(4):
            ox, oy = rr.randrange(2, 13), rr.randrange(2, 12)
            d.rectangle([ox, oy, ox + 1, oy], fill=(78, 150, 118)); d.point((ox, oy + 1), fill=(150, 104, 60))
    return im

def wall_tile(s, c, below_open):
    base = s.pal[c]
    im = Image.new('RGBA', (T, T), base + (255,))
    d = ImageDraw.Draw(im)
    if c == 'W':
        for xx in range(0, T, 3):
            d.line([(xx, 0), (xx, T - 1)], fill=shade(base, 0.72))
        d.line([(0, 0), (T - 1, 0)], fill=shade(base, 1.3))
        d.line([(0, 4), (T - 1, 4)], fill=shade(base, 0.6))
    elif c == 'D':
        d.rectangle([3, 0, 12, T - 1], fill=shade(base, 0.6))
        d.rectangle([4, 1, 11, T - 1], fill=(46, 32, 22))
    if below_open and c != 'D':
        d.rectangle([0, 12, T - 1, T - 1], fill=shade(base, 0.55))
    d.rectangle([0, 0, T - 1, T - 1], outline=s.outline)
    return im

def sprite(s, name, rr):
    """object sprites drawn into a T x T (or taller for A) RGBA image"""
    o = s.outline
    if name == 'tree':
        if s.name == 'B':
            im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
            g = rr.choice([(46, 84, 44), (40, 76, 46), (56, 90, 40)])
            d.ellipse([0, 0, 15, 15], fill=g, outline=o)
            d.ellipse([3, 2, 10, 9], fill=shade(g, 1.25))
            d.point((5, 4), fill=shade(g, 1.5)); d.ellipse([7, 8, 13, 13], fill=shade(g, 0.8))
            return im, 0
        im = Image.new('RGBA', (T, T + 8), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        g = rr.choice([(60, 120, 56), (52, 108, 60), (74, 128, 50)])
        d.rectangle([6, 14, 9, 23], fill=(96, 64, 40), outline=o)
        d.ellipse([0, 0, 15, 16], fill=g, outline=o); d.ellipse([3, 2, 10, 9], fill=shade(g, 1.3))
        return im, -8
    if name == 'bush':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        g = (58, 96, 52) if s.name == 'B' else (72, 120, 60)
        d.ellipse([2, 3, 13, 14], fill=g, outline=o)
        for _ in range(4):
            bx, by = rr.randrange(4, 11), rr.randrange(5, 12)
            d.point((bx, by), fill=(190, 40, 50)); d.point((bx + 1, by), fill=(220, 70, 70))
        return im, 0
    if name == 'boulder':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        c = (118, 114, 108) if s.name == 'B' else (150, 146, 138)
        d.ellipse([2, 4, 14, 14], fill=c, outline=o); d.ellipse([4, 5, 9, 9], fill=shade(c, 1.2))
        return im, 0
    if name in ('hearth', 'hearth_s', 'kiln'):
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        st = (110, 104, 98)
        if name == 'kiln':
            d.ellipse([1, 1, 14, 14], fill=(130, 80, 56), outline=o); d.ellipse([5, 5, 10, 10], fill=(40, 26, 20))
            d.point((7, 7), fill=(255, 160, 60)); d.point((8, 8), fill=(255, 120, 40))
            return im, 0
        d.ellipse([1, 1, 14, 14], fill=st, outline=o); d.ellipse([4, 4, 11, 11], fill=(50, 36, 28))
        d.polygon([(7, 4), (10, 9), (8, 11), (5, 9)], fill=(240, 130, 40)); d.polygon([(7, 6), (9, 9), (7, 10), (6, 9)], fill=(255, 220, 110))
        return im, 0
    if name == 'log_seat':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        d.rectangle([2, 6, 13, 10], fill=(120, 84, 52), outline=o); d.ellipse([11, 6, 14, 10], fill=(170, 130, 90), outline=o)
        return im, 0
    if name == 'bed':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        d.rectangle([2, 1, 13, 14], fill=(170, 150, 96), outline=o); d.rectangle([3, 2, 12, 5], fill=(200, 186, 150))
        d.rectangle([3, 7, 12, 13], fill=rr.choice([(130, 90, 60), (110, 96, 70), (140, 110, 80)]))
        return im, 0
    if name == 'jar':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        d.ellipse([4, 3, 11, 13], fill=(150, 96, 62), outline=o); d.rectangle([6, 2, 9, 4], fill=(120, 76, 50), outline=o)
        return im, 0
    if name == 'knap':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        d.ellipse([2, 4, 13, 13], fill=(126, 120, 112), outline=o); d.rectangle([6, 6, 9, 9], fill=(60, 64, 76))
        for p in [(2, 2), (13, 3), (12, 13), (1, 12)]:
            d.point(p, fill=(80, 84, 96))
        return im, 0
    if name == 'rack':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        d.rectangle([1, 2, 14, 13], outline=(110, 76, 44)); d.rectangle([3, 4, 12, 11], fill=(176, 140, 100), outline=o)
        return im, 0
    if name == 'wood_ws':
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        d.rectangle([1, 4, 14, 11], fill=(134, 96, 60), outline=o); d.line([(3, 7), (12, 7)], fill=(170, 130, 90))
        d.rectangle([10, 2, 12, 5], fill=(150, 150, 156), outline=o)
        return im, 0
    return None, 0

ITEM_COL = dict(log=(120, 82, 50), stone=(130, 126, 120), flint=(70, 76, 92), berries=(170, 40, 50), hide=(176, 140, 100),
                reeds=(150, 160, 90), grain=(208, 176, 90), pot=(156, 96, 62), ore=(78, 150, 118))

def item_sprite(s, name, rr):
    im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im); o = s.outline
    c = ITEM_COL[name]
    if name == 'log':
        for i, yy in enumerate((5, 8, 11)):
            d.rectangle([2 + i, yy, 13 - i, yy + 2], fill=c, outline=o); d.point((13 - i - 1, yy + 1), fill=(176, 136, 96))
    elif name in ('stone', 'ore', 'flint'):
        for bx, by in ((3, 8), (8, 8), (5, 4)):
            d.rectangle([bx, by, bx + 4, by + 3], fill=c, outline=o)
            if name == 'ore':
                d.point((bx + 2, by + 1), fill=(150, 104, 60))
    elif name in ('berries', 'grain'):
        d.ellipse([2, 5, 13, 14], fill=(140, 104, 60), outline=o); d.ellipse([4, 4, 11, 9], fill=c)
        d.point((6, 6), fill=shade(c, 1.4))
    elif name == 'hide':
        d.polygon([(3, 4), (12, 3), (14, 9), (11, 13), (4, 12), (2, 8)], fill=c, outline=o)
    elif name == 'reeds':
        for xx in range(4, 12, 2):
            d.line([(xx, 3), (xx + 1, 13)], fill=c)
        d.line([(3, 9), (12, 9)], fill=(110, 80, 50))
    elif name == 'pot':
        d.ellipse([4, 4, 11, 13], fill=c, outline=o); d.rectangle([6, 3, 9, 5], fill=shade(c, 0.8), outline=o)
    n = rr.randrange(3, 30)
    return im, n

def person_sprite(s, p):
    o = s.outline
    if s.name == 'B':
        # front-facing figure that fills the tile, with a ground shadow
        im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
        ch = p['child']
        d.ellipse([3, 13, 12, 15], fill=(0, 0, 0, 90))
        top = 0 if not ch else 4
        hw = 6 if not ch else 5
        hx = 8 - hw // 2
        d.rectangle([hx - 1, top, hx + hw, top + 5], fill=p['skin'], outline=o)          # head
        d.rectangle([hx - 1, top, hx + hw, top + 1], fill=p['hair']); d.point((hx - 1, top + 2), fill=p['hair']); d.point((hx + hw, top + 2), fill=p['hair'])
        d.point((hx + 1, top + 3), fill=o); d.point((hx + hw - 2, top + 3), fill=o)
        by = top + 6; bh = 5 if not ch else 3
        d.rectangle([3, by, 12, by + bh], fill=p['h'], outline=o)                     # tunic
        d.line([(4, by + 1), (11, by + 1)], fill=shade(p['h'], 1.3))
        d.rectangle([5, by + bh + 1, 6, min(15, by + bh + 3)], fill=shade(p['skin'], 0.8)); d.rectangle([9, by + bh + 1, 10, min(15, by + bh + 3)], fill=shade(p['skin'], 0.8))
        if p['carry']:
            c = ITEM_COL[p['carry']]
            d.rectangle([10, by - 2, 15, by + 2], fill=c, outline=o)
        if p['act'] in ('mine', 'chop', 'knap', 'build', 'wood'):
            d.line([(1, by - 4), (2, by + 4)], fill=(170, 124, 80)); d.rectangle([0, by - 5, 3, by - 4], fill=(170, 170, 180), outline=o)
        if p['act'] == 'farm':
            d.line([(1, by - 3), (2, by + 6)], fill=(170, 124, 80))
        if p['act'] == 'fish':
            d.line([(12, by), (2, by - 7)], fill=(170, 124, 80))
        return im, 0
    # A: D3 3/4, a bit taller than the tile
    im = Image.new('RGBA', (T, T + 6), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
    sc = 0.75 if p['child'] else 1.0
    top = 1 if not p['child'] else 6
    d.rectangle([5, top, 10, top + 5], fill=p['skin'], outline=o)
    d.rectangle([5, top, 10, top + 1], fill=p['hair'])
    d.point((7, top + 3), fill=o); d.point((9, top + 3), fill=o)
    bh = int(8 * sc)
    d.rectangle([4, top + 6, 11, top + 6 + bh], fill=p['h'], outline=o)
    d.rectangle([5, top + 7 + bh, 6, top + 9 + bh], fill=p['skin'], outline=o); d.rectangle([9, top + 7 + bh, 10, top + 9 + bh], fill=p['skin'], outline=o)
    if p['carry']:
        d.rectangle([10, top + 5, 14, top + 9], fill=ITEM_COL[p['carry']], outline=o)
    return im, -6

def animal_sprite(s, kind):
    im = Image.new('RGBA', (T, T), (0, 0, 0, 0)); d = ImageDraw.Draw(im); o = s.outline
    c = (150, 110, 70) if kind == 'deer' else (110, 110, 116)
    d.ellipse([2, 6, 12, 12], fill=c, outline=o); d.rectangle([11, 3, 14, 7], fill=c, outline=o)
    if kind == 'deer':
        d.line([(12, 3), (11, 0)], fill=(90, 70, 50)); d.line([(14, 3), (15, 0)], fill=(90, 70, 50))
    else:
        d.point((13, 4), fill=(230, 210, 80))
    return im

# ------------------------------------------------------------------ render map
def render_map(s):
    img = Image.new('RGBA', (W * T, H * T), (0, 0, 0, 255))
    rr = random.Random(11)
    open_c = set('gGdewsFPfrD')
    for y in range(H):
        for x in range(W):
            c = grid[y][x]
            below = grid[y + 1][x] if y + 1 < H else 'R'
            if c == 'R':
                exposed = any(0 <= x + dx < W and 0 <= y + dy < H and grid[y + dy][x + dx] in open_c
                              for dx in (-1, 0, 1) for dy in (-1, 0, 1))
                tile = rock_tile(s, x, y, below in open_c, exposed, (x, y) in ore and exposed or ((x, y) in ore and s.name == 'B'))
            elif c in 'WSD':
                tile = wall_tile(s, c, below in open_c and below not in 'WSD')
            else:
                tile = ground_tile(s, c, x, y)
            img.paste(tile, (x * T, y * T), tile)
    # zones
    zl = Image.new('RGBA', img.size, (0, 0, 0, 0)); zd = ImageDraw.Draw(zl)
    zc = dict(stock=(230, 200, 90), work=(110, 170, 220), farm=(140, 200, 100))
    for x0, y0, x1, y1, k in zones:
        c = zc[k]
        zd.rectangle([x0 * T, y0 * T, (x1 + 1) * T - 1, (y1 + 1) * T - 1], fill=c + (28,), outline=c + (150,))
    img = Image.alpha_composite(img, zl)
    # objects/items sorted by y
    draws = []
    for (x, y), name in obj.items():
        spr, oy = sprite(s, name, random.Random(x * 17 + y))
        if spr is not None:
            draws.append((y, x, spr, oy))
    for (x, y), name in items.items():
        spr, n = item_sprite(s, name, random.Random(x * 5 + y * 3))
        draws.append((y - 0.1, x, spr, 0))
    for p in people:
        spr, oy = person_sprite(s, p)
        draws.append((p['y'] + 0.5, p['x'], spr, oy))
        if p['sel']:
            sel = Image.new('RGBA', (T, T), (0, 0, 0, 0)); ImageDraw.Draw(sel).rectangle([0, 0, T - 1, T - 1], outline=(255, 230, 120, 255))
            draws.append((p['y'] + 0.6, p['x'], sel, 0))
    for kind, x, y in animals:
        draws.append((y + 0.5, x, animal_sprite(s, kind), 0))
    draws.sort(key=lambda t: t[0])
    for y, x, spr, oy in draws:
        img.alpha_composite(spr, (int(x) * T, int(round(y - 0.5 if y % 1 else y)) * T + oy) if False else (int(x) * T, int(math.floor(y + 0.01)) * T + oy))
    # designations
    dl = Image.new('RGBA', img.size, (0, 0, 0, 0)); dd = ImageDraw.Draw(dl)
    for (x, y), k in desig.items():
        X, Y = x * T, y * T
        if k == 'dig':
            for i in range(-T, T, 4):
                dd.line([(X + max(0, i), Y + max(0, -i)), (X + min(T - 1, i + T - 1), Y + min(T - 1, T - 1 - i))], fill=(240, 200, 60, 150))
            dd.rectangle([X, Y, X + T - 1, Y + T - 1], outline=(240, 200, 60, 200))
        elif k == 'chop':
            dd.line([(X + 3, Y + 3), (X + 12, Y + 12)], fill=(240, 200, 60, 230), width=2)
            dd.line([(X + 12, Y + 3), (X + 3, Y + 12)], fill=(240, 200, 60, 230), width=2)
        elif k == 'build':
            dd.rectangle([X + 1, Y + 1, X + T - 2, Y + T - 2], fill=(120, 180, 240, 70), outline=(140, 200, 255, 200))
    img = Image.alpha_composite(img, dl)
    return img

# ------------------------------------------------------------------ UI (drawn at logical 1x, scaled x2)
def font(sz=12, bold=False):
    return ImageFont.truetype(FONT.replace('Galmuri11.ttf', 'Galmuri11-Bold.ttf') if bold else FONT, sz)

UI_BG = (24, 21, 19, 236); UI_LINE = (70, 62, 54, 255); TXT = (226, 214, 190); DIM = (150, 138, 118); GOOD = (150, 200, 110); BAD = (220, 110, 90); GOLD = (232, 196, 110)

def text(d, xy, s, fill=TXT, f=None):
    d.text(xy, s, font=f or font(), fill=fill)

def bar(d, x, y, w, frac, col):
    d.rectangle([x, y, x + w, y + 5], fill=(50, 44, 38), outline=UI_LINE)
    d.rectangle([x + 1, y + 1, x + 1 + int((w - 2) * frac), y + 4], fill=col)

def ui_layer(lw, lh, full=True):
    ui = Image.new('RGBA', (lw, lh), (0, 0, 0, 0)); d = ImageDraw.Draw(ui); d.fontmode = '1'
    # top bar
    d.rectangle([0, 0, lw, 17], fill=UI_BG); d.line([(0, 17), (lw, 17)], fill=UI_LINE)
    cx = 6
    for k, c in [('3년 차 · 늦봄 · 맑음', TXT), ('인구 42', TXT), ('식량 312   장작 140   돌 86   부싯돌 24   토기 11', DIM)]:
        text(d, (cx, 3), k, c); cx += d.textlength(k, font=font()) + 18
    text(d, (lw - 250, 3), '시대', DIM)
    text(d, (lw - 222, 3), '신석기 전환기', GOLD)
    bar(d, lw - 140, 6, 70, 0.38, GOLD)
    text(d, (lw - 60, 3), '속도 2배', TXT)
    if not full:
        return ui
    # right detail panel
    px = lw - 226
    d.rectangle([px, 18, lw, lh], fill=UI_BG); d.line([(px, 18), (px, lh)], fill=UI_LINE)
    y = 24
    text(d, (px + 8, y), '도로아', GOLD, font(12, True)); text(d, (px + 54, y), '31세 · 강가네 · 돌깎이', DIM); y += 16
    text(d, (px + 8, y), '지금: 부싯돌 몸돌 다듬는 중 (61%)', TXT); y += 18
    text(d, (px + 8, y), '욕구', DIM); y += 14
    for k, v, c in [('배부름', 0.72, GOOD), ('기운', 0.41, GOLD), ('따뜻함', 0.88, GOOD), ('어울림', 0.23, BAD)]:
        text(d, (px + 12, y), k, TXT); bar(d, px + 64, y + 4, 140, v, c); y += 13
    y += 4
    text(d, (px + 8, y), '기술', DIM); y += 14
    for k, v in [('돌 다듬기', '7  숙련'), ('채집', '4'), ('불 다루기', '3'), ('흙 빚기', '2  배우는 중')]:
        text(d, (px + 12, y), k, TXT); text(d, (px + 100, y), v, DIM); y += 13
    y += 4
    text(d, (px + 8, y), '아는 것', DIM); y += 14
    for k, c in [('간석기 갈기', TXT), ('토기 굽기 — 아라에게 배우는 중', GOLD), ('녹색 돌? (정체 모름)', DIM)]:
        text(d, (px + 12, y), k, c); y += 13
    y += 4
    text(d, (px + 8, y), '기질', DIM); y += 14
    text(d, (px + 12, y), '새것 좋아함 · 겁 적음 · 고집 셈', TXT); y += 17
    text(d, (px + 8, y), '관계', DIM); y += 14
    for k in ['남편 카룬 (사이 좋음)', '딸 미에 (4세)', '바투와 사이 나쁨 — 덤불 다툼']:
        text(d, (px + 12, y), k, TXT); y += 13
    y += 4
    text(d, (px + 8, y), '최근 생각', DIM); y += 14
    for k, c in [('+5  새 움막이 아늑하다', GOOD), ('+3  첫 토기가 깨지지 않았다', GOOD), ('−4  어제 화덕가 말다툼', BAD), ('−2  손이 시리다', BAD)]:
        text(d, (px + 12, y), k, c); y += 13
    # bottom-left alerts
    ax, ay = 6, lh - 74
    d.rectangle([ax, ay, ax + 300, lh - 22], fill=UI_BG, outline=UI_LINE)
    for i, (k, c) in enumerate([('다라가 동굴 끝에서 녹색 돌을 찾았다', GOLD), ('남서쪽 숲에 늑대 두 마리', BAD), ('곡식 밭 첫 이삭이 여물었다', GOOD)]):
        text(d, (ax + 6, ay + 4 + i * 15), k, c)
    # bottom command bar
    d.rectangle([0, lh - 18, px - 1, lh], fill=UI_BG); d.line([(0, lh - 18), (px, lh - 18)], fill=UI_LINE)
    cx = 8
    for k in ['파기', '베기', '짓기', '구역', '작업 배정', '기술', '사람들', '연대기']:
        text(d, (cx, lh - 15), k, TXT); cx += d.textlength(k, font=font()) + 16
    return ui

def compose_near(s, cam_x, cam_y, fname):
    LW, LH = 960, 540
    m = render_map(s)
    view = m.crop((cam_x * T, cam_y * T, cam_x * T + LW, cam_y * T + LH))
    canvas = Image.new('RGBA', (LW, LH), (0, 0, 0, 255)); canvas.alpha_composite(view)
    canvas.alpha_composite(ui_layer(LW, LH, True))
    canvas.resize((LW * 2, LH * 2), Image.NEAREST).convert('RGB').save(f'{OUT}/{fname}')
    return m

def compose_far(m, fname):
    canvas = Image.new('RGBA', (1920, 1080), (14, 12, 11, 255))
    canvas.alpha_composite(m, ((1920 - m.width) // 2, 1080 - m.height - 8))
    ui = ui_layer(960, 540, False).resize((1920, 1080), Image.NEAREST)
    canvas.alpha_composite(ui)
    canvas.convert('RGB').save(f'{OUT}/{fname}')

sB = make_style('B')
CAM = (25, 14)
mB = compose_near(sB, *CAM, 'mock_B_near.png')
compose_far(mB, 'mock_B_far.png')
sA = make_style('A')
compose_near(sA, *CAM, 'mock_A_near.png')
print('people', len(people), 'cave entry x', ex)
