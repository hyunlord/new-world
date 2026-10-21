"""Native-pixel trial boards from supplied accepted assets, without fallback art."""
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from textwrap import wrap
from typing import Final

from PIL import Image, ImageDraw, ImageFont

from .compose import Person, resolve
from .models import Box, PipelineError, Size

REQUIRED_ENV: Final[Mapping[str, Size]] = {
    'hut': (96, 96), 'lean_to': (32, 48), 'hearth_0': (32, 32),
    'stockpile_floor_corner': (32, 32), 'stockpile_floor_edge': (32, 32),
    'stockpile_floor_interior': (32, 32), 'log_pile_1': (32, 32),
    'log_pile_2': (32, 32), 'log_pile_3': (32, 32), 'stump': (32, 32),
    'ground_grass_0': (32, 32), 'ground_dirt_0': (32, 32),
    'water_shallow_0': (32, 32), 'water_deep_0': (32, 32),
    'bank_w': (32, 32), 'bank_e': (32, 32),
    'tree_oak_s': (48, 48), 'tree_birch_s': (48, 48), 'tree_pine_s': (48, 48),
    'tree_oak_l': (64, 64), 'tree_birch_l': (64, 64), 'tree_pine_l': (64, 64),
}
PERSON_POSITIONS: Final[Mapping[str, Size]] = {
    'carry_1': (272, 320), 'carry_2': (320, 368),
    'chop_1': (88, 448), 'chop_2': (352, 480),
    'child': (192, 320), 'elder': (384, 256), 'sleeper': (144, 208),
    'adult_1': (240, 240), 'adult_2': (192, 416), 'adult_3': (304, 544),
}
TREE_POSITIONS: Final = ((16, 16), (112, 32), (240, 16), (368, 48), (16, 144),
                        (384, 144), (16, 304), (400, 368), (16, 560), (144, 576),
                        (288, 624), (400, 656), (32, 736), (176, 816), (352, 864))


@dataclass(frozen=True, slots=True)
class PersonSprite:
    person: Person
    image: Image.Image


@dataclass(frozen=True, slots=True)
class Placement:
    name: str
    rect: Box


@dataclass(frozen=True, slots=True)
class SceneResult:
    image: Image.Image
    placements: tuple[Placement, ...]
    crops: Mapping[str, Box]


def board_font(path: Path | None = None) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    if path is not None:
        return ImageFont.truetype(str(path), 14)
    return ImageFont.load_default(size=14)


def _paste(canvas: Image.Image, image: Image.Image, position: Size) -> Box:
    x, y = position
    if x < 0 or y < 0 or x + image.width > canvas.width or y + image.height > canvas.height:
        raise PipelineError('Placement would clip an asset')
    canvas.alpha_composite(image.convert('RGBA'), position)
    return x, y, x + image.width, y + image.height


def render_scene(env: Mapping[str, Image.Image], people: Mapping[str, PersonSprite]) -> SceneResult:
    """Fixed preregistered 20x30-tile layout; crop coordinates are scene-local."""
    for name, size in REQUIRED_ENV.items():
        if name not in env or env[name].size != size:
            raise PipelineError(f'Missing or wrong-size environment sprite: {name}')
    if set(people) != set(PERSON_POSITIONS):
        raise PipelineError('Scene requires exactly the ten declared person roles')
    for role, sprite in people.items():
        if sprite.image.size != resolve(sprite.person).box:
            raise PipelineError(f'Person frame size mismatch: {role}')
    if any(people[role].person.carry != 'logs' for role in ('carry_1', 'carry_2')):
        raise PipelineError('Both carriers must carry logs')
    if any(people[role].person.job != 'woodcutter' or people[role].person.state != 'work'
           for role in ('chop_1', 'chop_2')):
        raise PipelineError('Both chopping roles require axe work poses')
    if people['child'].person.age >= 15 or people['elder'].person.age < 50 or people['sleeper'].person.state != 'sleep':
        raise PipelineError('Child, elder and sleeper roles must match their attributes')
    scene = Image.new('RGBA', (640, 960))
    placements: list[Placement] = []
    for row in range(30):
        for column in range(20):
            name = 'ground_grass_0'
            if 2 <= column <= 13 and 4 <= row <= 18:
                name = 'ground_dirt_0'
            if column == 15:
                name = 'bank_w'
            elif column in (16, 18):
                name = 'water_shallow_0'
            elif column == 17:
                name = 'water_deep_0'
            elif column == 19:
                name = 'bank_e'
            _ = _paste(scene, env[name], (column * 32, row * 32))
    for row in range(4):
        for column in range(4):
            edge_x, edge_y = column in (0, 3), row in (0, 3)
            kind = 'corner' if edge_x and edge_y else 'edge' if edge_x or edge_y else 'interior'
            name = f'stockpile_floor_{kind}'
            image = env[name]
            turns = {(0, 0): 0, (3, 0): 3, (3, 3): 2, (0, 3): 1}.get((column, row), 0)
            if kind == 'edge':
                turns = 0 if row == 0 else 1 if column == 0 else 2 if row == 3 else 3
            image = image.rotate(90 * turns)
            placements.append(Placement(name, _paste(scene, image, (288 + column * 32, 320 + row * 32))))
    objects: list[tuple[str, Size]] = [('hut', (128, 160)), ('lean_to', (256, 160)),
        ('lean_to', (320, 176)), ('lean_to', (384, 192)), ('hearth_0', (224, 288)),
        ('log_pile_3', (352, 320)), ('log_pile_2', (384, 384)), ('log_pile_1', (160, 480)),
        ('stump', (112, 448)), ('stump', (376, 480))]
    trees = ('tree_oak_s', 'tree_birch_l', 'tree_pine_s', 'tree_oak_l', 'tree_birch_s', 'tree_pine_l')
    objects.extend((trees[index % len(trees)], position) for index, position in enumerate(TREE_POSITIONS))
    for name, position in sorted(objects, key=lambda item: (item[1][1] + env[item[0]].height, item[1][0])):
        placements.append(Placement(name, _paste(scene, env[name], position)))
    for role, position in PERSON_POSITIONS.items():
        placements.append(Placement(f'person:{role}', _paste(scene, people[role].image, position)))
    crops: dict[str, Box] = {
        'hut': (120, 152, 232, 264), 'lean_to': (248, 152, 296, 216),
        'hearth': (216, 280, 264, 328), 'stockpile_logs': (280, 312, 424, 456),
        'bare_logs': (152, 472, 200, 520), 'carry': (264, 312, 312, 360),
        'chop': (80, 440, 152, 488), 'child': (184, 312, 232, 360),
        'elder': (376, 248, 424, 296), 'sleep': (136, 200, 184, 232),
    }
    return SceneResult(scene, tuple(placements), crops)


def ab_crop_rect(image: Image.Image, origin: Size | None = None) -> Box:
    """Expose the exact recorded native crop; center by default, never pad or resize."""
    x, y = origin if origin is not None else ((image.width - 640) // 2, (image.height - 960) // 2)
    if x < 0 or y < 0 or x + 640 > image.width or y + 960 > image.height:
        raise PipelineError('Historical crop must lie entirely inside its source image')
    return x, y, x + 640, y + 960


def render_ab(
    scene: Image.Image, baseline: Image.Image, mockup: Image.Image,
    *, crop_origins: tuple[Size | None, Size | None] = (None, None),
) -> Image.Image:
    """Preserve three native sources plus explicit B/C crop origins for auditability.

    Crop origins are an independent presentation control; defaults center each
    historical image. Call ab_crop_rect with the same origins to persist metadata.
    """
    if scene.size != (640, 960) or any(image.width < 640 or image.height < 960 for image in (baseline, mockup)):
        raise PipelineError('A/B requires native 640x960 scene and source images at least that size')
    canvas = Image.new('RGBA', (1920, 1080), (20, 18, 16, 255))
    labels = ('A / generated trial / native 32px', 'B / historical M1 screenshot / original pixels', 'C / target mock-up / original pixels')
    draw = ImageDraw.Draw(canvas)
    rects = ((0, 0, 640, 960), ab_crop_rect(baseline, crop_origins[0]), ab_crop_rect(mockup, crop_origins[1]))
    for index, (image, rect) in enumerate(zip((scene, baseline, mockup), rects, strict=True)):
        _ = _paste(canvas, image.crop(rect), (index * 640, 60))
        draw.text((index * 640 + 16, 20), labels[index], fill='white', font=board_font())
    draw.text((16, 1040), 'Historical panels have different entity placement; this is not a controlled replay.', fill='white', font=board_font())
    return canvas


def render_people(people: Sequence[PersonSprite], font_path: Path | None = None) -> Image.Image:
    """Label exactly 24 distinct complete composites at nearest-neighbour 3x."""
    if len(people) != 24:
        raise PipelineError('Person board requires exactly 24 composites')
    if len({(sprite.image.size, sprite.image.convert('RGBA').tobytes()) for sprite in people}) != 24:
        raise PipelineError('Person board contains identical composites')
    actual = {(resolve(sprite.person).body, sprite.person.direction) for sprite in people}
    expected = {(body, direction) for body in ('adult_m', 'adult_f', 'child', 'elder') for direction in ('south', 'east')}
    if not expected.issubset(actual):
        raise PipelineError('Person board must include every body in south and east')
    canvas = Image.new('RGBA', (1600, 1440), (30, 32, 30, 255))
    draw = ImageDraw.Draw(canvas)
    font = board_font(font_path)
    for index, sprite in enumerate(people):
        appearance = resolve(sprite.person)
        if sprite.image.size != appearance.box:
            raise PipelineError('Person board frame size does not match attributes')
        person = sprite.person
        x, y = index % 4 * 400, index // 4 * 240
        scaled = sprite.image.resize((sprite.image.width * 3, sprite.image.height * 3), Image.Resampling.NEAREST)
        _ = _paste(canvas, scaled, (x + 152, y + 8))
        label = (f'{index + 1:02} id={person.agent_id} {person.sex} age={person.age} {person.direction} '
                 f'{person.state} household={person.household_id} skin={person.skin} hair={appearance.hair} '
                 f'hair_rgb={appearance.hair_color} top={person.top} beard={person.beard} head={person.head} tool={appearance.tool} carry={person.carry}')
        lines = wrap(label, width=45)
        if len(lines) > 7:
            raise PipelineError('Attribute label exceeds readable board cell')
        draw.multiline_text((x + 10, y + 112), '\n'.join(lines), fill='white', font=font, spacing=2)
    return canvas
