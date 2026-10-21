"""Run real registered assets: python3 -m art.pipeline.registry MANIFEST OUTPUT_DIR."""
import json
import sys
from collections.abc import Mapping
from dataclasses import asdict, dataclass
from pathlib import Path

from PIL import Image

from .check import CheckRecord, check_asset, write_checks
from .models import Asset, AssetKind, Json, PipelineError, Size, decode_json
from .palette import load_palette
from .sheet import pack_sheet


@dataclass(frozen=True, slots=True)
class RegistryEntry:
    name: str
    image: Path
    box: Size
    anchor: Size
    kind: AssetKind
    family: str
    reference: Path
    outline_context: Path | None
    min_component_pixels: int

@dataclass(frozen=True, slots=True)
class Registry:
    palette: Path
    assets: tuple[RegistryEntry, ...]

@dataclass(frozen=True, slots=True)
class RegistryResult:
    passed: bool
    asset_count: int
    source_png_bytes: int
    sheet_png_bytes: int
    per_file_limit_bytes: int
    oversized_sources: tuple[str, ...]
    atlas_written: bool

@dataclass(frozen=True, slots=True)
class PaletteMetadata:
    legacy: Mapping[str, str | list[str]]
    channels: Mapping[str, str]
    skins: list[str]

def mapping(value: Json) -> Mapping[str, Json]:
    """Require an object at a JSON boundary."""
    if not isinstance(value, dict):
        raise PipelineError('Expected JSON object')
    return value

def text(value: Json) -> str:
    """Require a nonempty string at a JSON boundary."""
    if not isinstance(value, str) or not value.strip():
        raise PipelineError('Expected nonempty string')
    return value

def pair(value: Json) -> Size:
    """Require two integer coordinates, excluding JSON booleans."""
    if not isinstance(value, list) or len(value) != 2:
        raise PipelineError('Expected two integer coordinates')
    a, b = value
    if type(a) is not int or type(b) is not int:
        raise PipelineError('Expected integer coordinates')
    return a, b

def source_path(value: Json, base: Path) -> Path:
    """Resolve a required existing input, relative to its manifest."""
    path = Path(text(value))
    resolved = path if path.is_absolute() else base / path
    if not resolved.is_file():
        raise PipelineError(f'Missing input file: {resolved}')
    return resolved.resolve()

def parse_entry(value: Json, base: Path) -> RegistryEntry:
    """Validate metadata once before loading image content."""
    item = mapping(value)
    kinds: dict[str, AssetKind] = {'object': 'object', 'ground': 'ground',
                                  'ghost': 'ghost', 'shadow': 'shadow', 'layer': 'layer'}
    kind = kinds.get(text(item.get('kind')))
    if kind is None:
        raise PipelineError('Unsupported asset kind')
    box, anchor = pair(item.get('box')), pair(item.get('anchor'))
    if min(box) < 1 or not (0 <= anchor[0] <= box[0] and 0 <= anchor[1] <= box[1]):
        raise PipelineError('Box must be positive and anchor inside declared box')
    minimum = item.get('min_component_pixels', 2)
    if type(minimum) is not int or minimum < 1:
        raise PipelineError('Component minimum must be a positive integer')
    context = item.get('outline_context')
    return RegistryEntry(text(item.get('name')), source_path(item.get('image'), base),
                         box, anchor, kind, text(item.get('family')),
                         source_path(item.get('reference'), base),
                         None if context is None else source_path(context, base), minimum)

def load_registry(path: Path) -> Registry:
    """Reject empty registries, duplicate names, and missing required sources."""
    root = mapping(decode_json(path.read_text(encoding='utf-8')))
    values = root.get('assets')
    if not isinstance(values, list) or not values:
        raise PipelineError('Registry must contain a nonempty assets array')
    entries = tuple(parse_entry(value, path.parent) for value in values)
    if len({entry.name for entry in entries}) != len(entries):
        raise PipelineError('Duplicate registry asset names')
    return Registry(source_path(root.get('palette'), path.parent), entries)

def hex_color(value: Json) -> str:
    """Normalize legacy colour spelling after validating exactly six hex digits."""
    color = text(value).removeprefix('#').lower()
    if len(color) != 6 or any(character not in '0123456789abcdef' for character in color):
        raise PipelineError('Expected six-digit colour hex')
    return color

def color_list(value: Json) -> list[str]:
    """Require at least one source colour for legacy people metadata."""
    if not isinstance(value, list) or not value:
        raise PipelineError('Expected nonempty colour list')
    return [hex_color(color) for color in value]

def palette_metadata(path: Path) -> PaletteMetadata:
    """Keep legacy palettes separate from extra channels and skin variants."""
    root = mapping(decode_json(path.read_text(encoding='utf-8')))
    people = mapping(root.get('people'))
    skins = color_list(people.get('skins'))
    legacy: dict[str, str | list[str]] = {
        'skin': skins[0], 'hairs': color_list(people.get('hairs')),
        'tunics': color_list(people.get('tunics')),
        'elder_hair': hex_color(people.get('elder_hair')),
    }
    channels = {key: hex_color(value) for key, value in mapping(root.get('channels')).items()}
    return PaletteMetadata(legacy, channels, skins)

def read_rgba(path: Path) -> Image.Image:
    """Detach decoded pixels from a closed source file."""
    with Image.open(path) as image:
        return image.convert('RGBA')

def run_registry(manifest: Path, output: Path) -> RegistryResult:
    """Check every input before packing; retain failed reports without a new atlas.

    Output may already contain older atlas files; atlas_written in stats is the
    authority for this run. Failure never overwrites an older atlas with bad art.
    """
    registry = load_registry(manifest)
    colors = load_palette(registry.palette)
    palettes = palette_metadata(registry.palette)
    assets: list[Asset] = []
    records: list[CheckRecord] = []
    for entry in registry.assets:
        context = None if entry.outline_context is None else read_rgba(entry.outline_context)
        asset = Asset(entry.name, read_rgba(entry.image), entry.box, entry.anchor,
                      entry.kind, entry.min_component_pixels, context)
        assets.append(asset)
        result = check_asset(asset, colors, read_rgba(entry.reference))
        records.append(CheckRecord(result, entry.family, str(entry.image)))
    write_checks(records, output / 'CHECKS.md', final=True)
    checked = all(record.result.passed for record in records)
    limit = 1_500_000
    oversized = tuple(entry.name for entry in registry.assets if entry.image.stat().st_size > limit)
    sheet_bytes = 0
    if checked:
        sheet = pack_sheet(assets, palettes=palettes.legacy)
        sheet.image.save(output / 'sheet.png')
        sheet_bytes = (output / 'sheet.png').stat().st_size
        metadata = {**sheet.metadata, 'channels': dict(palettes.channels), 'skins': palettes.skins}
        _ = (output / 'sheet.json').write_text(json.dumps(metadata, indent=2) + '\n', encoding='utf-8')
    result = RegistryResult(checked and sheet_bytes <= limit, len(assets),
                            sum(entry.image.stat().st_size for entry in registry.assets),
                            sheet_bytes, limit, oversized, checked)
    _ = (output / 'registry_stats.json').write_text(json.dumps(asdict(result), indent=2) + '\n', encoding='utf-8')
    return result

def main() -> int:
    """CLI reports expected input failures and exits nonzero on any failed gate."""
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        result = run_registry(Path(sys.argv[1]), Path(sys.argv[2]))
    except (OSError, PipelineError, json.JSONDecodeError) as error:
        print(f'Registry rejected: {error}', file=sys.stderr)
        return 2
    print(json.dumps(asdict(result), indent=2))
    return 0 if result.passed else 1

if __name__ == '__main__':
    raise SystemExit(main())
