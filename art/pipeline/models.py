"""Typed image and atlas contracts shared by the trial pipeline."""
import json
from collections.abc import Callable
from dataclasses import dataclass
from typing import Final, Literal, TypeAlias, TypedDict

from PIL import Image

Size = tuple[int, int]
Box = tuple[int, int, int, int]
RGB = tuple[int, int, int]
AssetKind = Literal['object', 'ground', 'ghost', 'shadow', 'layer']

class PipelineError(ValueError):
    """A source or declared asset violates a pipeline precondition."""

@dataclass(frozen=True, slots=True)
class Asset:
    name: str
    image: Image.Image
    box: Size
    anchor: Size
    kind: AssetKind = 'object'
    min_component_pixels: int = 2
    outline_context: Image.Image | None = None

class AtlasMetadata(TypedDict):
    atlas: str
    size: list[int]
    tile_size: int
    palettes: dict[str, str | list[str]]
    tiles: dict[str, list[int]]
    boxes: dict[str, list[int]]
    anchors: dict[str, list[int]]
    kinds: dict[str, AssetKind]


Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
# With no custom hooks, the standard decoder returns only JSON primitive trees.
decode_json: Final[Callable[[str], Json]] = json.loads


def json_object(value: Json) -> dict[str, Json]:
    """Validate an object before indexing external JSON."""
    if not isinstance(value, dict):
        raise PipelineError('Expected JSON object')
    return value


def json_array(value: Json) -> list[Json]:
    """Validate an array before iterating external JSON."""
    if not isinstance(value, list):
        raise PipelineError('Expected JSON array')
    return value


def json_string(value: Json) -> str:
    """Validate a string before parsing external JSON fields."""
    if not isinstance(value, str):
        raise PipelineError('Expected JSON string')
    return value


def json_integer(value: Json) -> int:
    """Validate integer coordinates and counts, rejecting JSON booleans."""
    if type(value) is not int:
        raise PipelineError('Expected JSON integer')
    return value
