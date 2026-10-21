"""Deterministic paper-doll composition; asset lookup never invents missing art."""
from collections.abc import Mapping
from dataclasses import dataclass
from hashlib import blake2b
from typing import Final, Literal, NewType, assert_never

import numpy as np
from PIL import Image

from .outline import enforce_outline

AgentId = NewType('AgentId', int)
HouseholdId = NewType('HouseholdId', int)
type RGB = tuple[int, int, int]
type Body = Literal['adult_m', 'adult_f', 'child', 'elder']
type Direction = Literal['south', 'east', 'west']
type Frame = Literal['idle', 'walk_a', 'walk_b', 'work', 'sleep']
type Hair = Literal['short', 'long_tied', 'braided', 'bun', 'shaved_sides']
type Job = Literal['none', 'woodcutter', 'builder', 'farmer', 'miner', 'hunter', 'gatherer']
HAIRS: Final[tuple[Hair, ...]] = ('short', 'long_tied', 'braided', 'bun', 'shaved_sides')
HAIR_COLORS: Final[tuple[RGB, ...]] = ((42, 30, 22), (90, 58, 34), (138, 106, 58), (58, 42, 30))
SKINS: Final[tuple[RGB, ...]] = ((216, 168, 132), (185, 129, 95), (145, 92, 64), (104, 64, 46))
TUNICS: Final[tuple[RGB, ...]] = ((239, 226, 192), (111, 176, 232), (224, 96, 76), (80, 192, 168), (176, 136, 224), (232, 136, 176))
TINT_CHANNELS: Final[Mapping[str, tuple[int, ...]]] = {
    'body': (154, 110), 'top': (192,), 'hair': (176,), 'beard': (176,),
    'head': (192,), 'tool': (), 'carry': (),
}


@dataclass(frozen=True, slots=True)
class CompositionError(ValueError):
    detail: str


@dataclass(frozen=True, slots=True)
class MissingLayerError(CompositionError):
    key: str


@dataclass(frozen=True, slots=True)
class Person:
    agent_id: AgentId
    sex: Literal['male', 'female']
    age: int
    household_id: HouseholdId
    hairstyle: Hair | None = None
    skin: int = 0
    job: Job = 'none'
    carry: Literal['logs', 'berries', 'stone'] | None = None
    state: Frame = 'idle'
    direction: Direction = 'south'
    top: Literal['hide_wrap', 'fur_cloak', 'woven_tunic', 'leather_vest'] | None = 'hide_wrap'
    beard: Literal['short', 'long'] | None = None
    head: Literal['headband', 'fur_hat', 'hood'] | None = None

    def __post_init__(self) -> None:
        if self.age < 0 or self.agent_id < 0 or self.household_id < 0 or not 0 <= self.skin < len(SKINS):
            raise CompositionError('Age and IDs must be nonnegative; skin must index four tones')


@dataclass(frozen=True, slots=True)
class Appearance:
    person: Person
    body: Body
    hair: Hair
    hair_color: RGB
    skin_color: RGB
    tunic: RGB
    tool: str | None
    direction: Literal['south', 'east']
    flip: bool
    box: tuple[int, int]


def keyed_index(agent_id: AgentId, domain: str, count: int) -> int:
    """BLAKE2b keyed/domain-separated choice, independent of Python hash seed."""
    if count <= 0:
        raise CompositionError('Choice count must be positive')
    payload = f'{domain}:{agent_id}'.encode('ascii')
    return int.from_bytes(blake2b(payload, key=b'worldsim-arttrial-v1', digest_size=8).digest(), 'big') % count


def resolve(person: Person) -> Appearance:
    if person.age < 15:
        body: Body = 'child'
    elif person.age >= 50:
        body = 'elder'
    else:
        match person.sex:
            case 'male': body = 'adult_m'
            case 'female': body = 'adult_f'
            case _: assert_never(person.sex)
    match person.job:
        case 'none': tool = None
        case 'woodcutter': tool = 'axe'
        case 'builder': tool = 'hammer'
        case 'farmer': tool = 'hoe'
        case 'miner': tool = 'pick'
        case 'hunter': tool = 'spear'
        case 'gatherer': tool = 'basket'
        case _: assert_never(person.job)
    match person.direction:
        case 'south': direction, flip = 'south', False
        case 'east': direction, flip = 'east', False
        case 'west': direction, flip = 'east', True
        case _: assert_never(person.direction)
    match person.state:
        case 'sleep': direction, flip, box = 'south', False, (32, 16)
        case 'idle' | 'walk_a' | 'walk_b' | 'work': box = (32, 32)
        case _: assert_never(person.state)
    hair = person.hairstyle or HAIRS[keyed_index(person.agent_id, 'hair_style', 5)]
    color = (232, 232, 232) if person.age >= 50 else HAIR_COLORS[keyed_index(person.agent_id, 'hair_color', 4)]
    return Appearance(person, body, hair, color, SKINS[person.skin], TUNICS[person.household_id % 6], tool, direction, flip, box)


def layer_keys(appearance: Appearance) -> tuple[str, ...]:
    """Ordered exact lookup keys; only explicitly absent layers are omitted."""
    person = appearance.person
    suffix = f'{appearance.body}/{appearance.direction}/{person.state}'
    layers = [('shadow', 'base'), ('body', 'base'), ('top', person.top),
              ('hair', appearance.hair), ('beard', person.beard), ('head', person.head),
              ('tool', appearance.tool), ('carry', person.carry)]
    return tuple(f'{family}/{variant}/{suffix}' for family, variant in layers if variant is not None)


@dataclass(frozen=True, slots=True)
class LayerAtlas:
    layers: Mapping[str, Image.Image]

    def require(self, key: str) -> Image.Image:
        try:
            return self.layers[key]
        except KeyError as error:
            raise MissingLayerError('Requested generated frame is missing', key) from error


def tint(image: Image.Image, appearance: Appearance, *,
         channels: tuple[int, ...] = (154, 110, 192, 176),
         body_context: Image.Image | None = None) -> Image.Image:
    """Replace only exact reserved channels; retain all texture and alpha values."""
    source = np.array(image.convert('RGBA'), dtype=np.uint8)
    output = source.copy()
    replacements = {154: appearance.skin_color, 110: appearance.tunic,
                    192: appearance.tunic, 176: appearance.hair_color}
    for gray in channels:
        color = replacements[gray]
        matches = np.empty((image.height, image.width), dtype=np.bool_)
        _ = np.all(np.equal(source[:, :, :3], gray), axis=2, out=matches)
        selected = matches & (source[:, :, 3] > 0)
        output[selected, :3] = color
    if body_context is not None:
        if body_context.size != image.size:
            raise CompositionError('Tint body context size mismatch')
        context = np.array(body_context.convert('RGBA'), dtype=np.uint8)
        inherited = np.empty((image.height, image.width), dtype=np.bool_)
        _ = np.all(np.equal(source, context), axis=2, out=inherited)
        for gray in (154, 110):
            matches = np.empty((image.height, image.width), dtype=np.bool_)
            _ = np.all(np.equal(source[:, :, :3], gray), axis=2, out=matches)
            selected = inherited & matches & (source[:, :, 3] > 0)
            output[selected, :3] = replacements[gray]
    output[output[:, :, 3] == 0] = 0
    return Image.fromarray(output)


def compose_person(person: Person, atlas: LayerAtlas) -> Image.Image:
    appearance = resolve(person)
    keys = layer_keys(appearance)
    foreground = Image.new('RGBA', appearance.box)
    body = atlas.require(keys[1])
    shadow = atlas.require(keys[0]).convert('RGBA')
    if shadow.size != appearance.box:
        raise CompositionError(f'Shadow size mismatch: {keys[0]}')
    for key in keys[1:]:
        layer = atlas.require(key)
        if layer.size != appearance.box:
            raise CompositionError(f'Layer size mismatch: {key}')
        family = key.partition('/')[0]
        foreground.alpha_composite(tint(layer, appearance, channels=TINT_CHANNELS[family],
                                        body_context=body))
    output = shadow.copy()
    output.alpha_composite(enforce_outline(foreground))
    if appearance.flip:
        output = output.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    return output
