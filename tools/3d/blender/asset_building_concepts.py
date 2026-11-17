"""Visual concepts for inert templates whose sparse pieces cannot expand into a game site."""
import math

KINDS = {'shrine', 'chief_hall', 'trading_post', 'market', 'warehouse'}


def concept(m, template, stage):
    """Fill the authored footprint with a distinguishable concept, never an executable plan."""
    w, h = template['footprint']
    x, y = w / 2 - 0.35, h / 2 - 0.35
    m.box('earth', (w - 0.15, h - 0.15, 0.10), loc=(0, 0, 0.05))
    if stage == 'foundation':
        for sx in (-1, 1):
            for sy in (-1, 1):
                m.box('stone', (0.4, 0.4, 0.22), loc=(sx * x, sy * y, 0.11))
        return
    top = 1.1 if stage == 'half' else 2.2
    for sx in (-1, 1):
        for sy in (-1, 1):
            m.rod('wood', (sx * x, sy * y, 0), (sx * x, sy * y, top), 0.10, segs=6)
    if stage == 'frame':
        return
    match template['id']:
        case 'shrine':
            for n in range(9):
                a = 2 * math.pi * n / 9
                m.ball('stone', 0.27, loc=(math.cos(a) * 1.15, math.sin(a) * 1.15, 0.20), scale=(0.8, 0.8, 1.5), subdiv=1)
            m.box('clay', (0.9, 0.7, 0.45), loc=(0, 0, 0.325))
            m.box('stone', (1.1, 0.9, 0.12), loc=(0, 0, 0.61))
            m.cyl('horn', 0.16, 0.55, loc=(0, 0, 0.67), segs=5, r2=0.05)
        case 'trading_post' | 'market':
            m.box('wood', (w - 0.7, 0.65, 0.15), loc=(0, -y + 0.25, 0.85))
            for sx in (-1, 1):
                m.box('wood_dark', (0.14, 0.5, 0.8), loc=(sx * (x - 0.3), -y + 0.25, 0.4))
            for n in range(3):
                px = (n - 1) * 0.7
                m.box('wood', (0.55, 0.50, 0.4), loc=(px, y - 0.45, 0.25))
                m.box('wood_dark', (0.55, 0.04, 0.07), loc=(px, y - 0.72, 0.3))
                m.ball('fruit' if template['id'] == 'market' else 'metal', 0.13, loc=(px, -y + 0.25, 1.0), subdiv=1)
            if template['id'] == 'market':
                for sx in (-1, 1):
                    m.box('cloth', (0.5, 0.04, 0.75), loc=(sx * x, -y, top - 0.3))
        case 'chief_hall' | 'warehouse':
            material = 'clay' if template['id'] == 'chief_hall' else 'stone'
            for level in range(max(1, round(top / 0.25))):
                z = 0.125 + level * 0.25
                for side in (-1, 1):
                    for block in range(4):
                        py = -(h - 0.4) / 2 + (block + 0.5) * (h - 0.4) / 4
                        m.box(material, (0.28, (h - 0.4) / 4 - 0.025, 0.235),
                              loc=(side * x, py, z), jitter=0.009)
                for block in range(4):
                    px = -(w - 0.4) / 2 + (block + 0.5) * (w - 0.4) / 4
                    m.box(material, ((w - 0.4) / 4 - 0.025, 0.28, 0.235),
                          loc=(px, y, z), jitter=0.009)
                for side in (-1, 1):
                    for block in range(2):
                        px = side * (0.9 + (block + 0.5) * (w - 2.2) / 4)
                        m.box(material, ((w - 2.2) / 4 - 0.025, 0.28, 0.235),
                              loc=(px, -y, z), jitter=0.009)
            if template['id'] == 'chief_hall':
                for side in (-1, 1):
                    m.rod('wood_dark', (side * 0.9, -y - 0.05, 0), (side * 0.9, -y - 0.05, top), 0.13)
                m.rod('wood', (-1.1, -y - 0.05, top), (1.1, -y - 0.05, top), 0.13)
                m.rod('horn', (-0.32, -y - 0.18, top), (0.32, -y - 0.18, top + 0.30), 0.06)
            else:
                for side in (-1, 1):
                    m.rod('wood_dark', (side * 0.86, -y - 0.04, 0.1),
                          (side * 0.86, -y - 0.04, top), 0.09, segs=7)
                m.rod('wood', (-1.04, -y - 0.04, top), (1.04, -y - 0.04, top), 0.10, segs=7)
                for px in (-1, 0, 1):
                    m.box('wood', (0.7, 0.6, 0.5), loc=(px, 0.65, 0.3))
                    m.ball('thatch', 0.3, loc=(px, -0.35, 0.35), scale=(0.8, 0.8, 1.2), subdiv=1)
        case _: raise AssertionError(template['id'])
