# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.materials.preview STYLE MATERIAL OUTPUT_DIRECTORY.
"""Render real resolved textures through straight, corner and room layouts."""
from __future__ import annotations

import json
import sys
from pathlib import Path

from PIL import Image

from art.compositor.geometry import perimeter
from art.compositor.walls import Door, render_walls
from art.materials.render import load_surface, load_textures
from art.materials.resolve import read_catalog, resolve


def render_layouts(root: Path, style_id: str, material_id: str, output: Path) -> tuple[Path, ...]:
    """Drive the same public renderer used by boards; unknown ids use defaults."""
    resolved = resolve(read_catalog(root), style_id, material_id)
    textures = load_textures(root, resolved)
    layouts = (
        ("ew", frozenset((x, 0) for x in range(-1, 4))),
        ("ns", frozenset((0, y) for y in range(-1, 4))),
        ("corner", frozenset({(0, 0), (1, 0), (0, 1)})),
        ("tee", frozenset({(-1, 0), (0, 0), (1, 0), (0, 1)})),
        ("cross", frozenset({(-1, 0), (0, 0), (1, 0), (0, 1), (0, -1)})),
        ("isolated", frozenset({(0, 0)})),
        ("room", perimeter((5, 4))),
    )
    output.mkdir(parents=True, exist_ok=True)
    paths: list[Path] = []
    for name, cells in layouts:
        doors = (Door((2, 3), textures.door_open),) if name == "room" else ()
        rendered = render_walls(cells, textures, doors)
        path = output / f"{name}.png"
        rendered.image.save(path, optimize=True)
        paths.append(path)
    floor = resolve(read_catalog(root), None, material_id, kind="floor")
    load_surface(root, floor).save(output / "floor.png", optimize=True)
    textures.door_closed.save(output / "door_closed.png", optimize=True)
    textures.door_open.save(output / "door_open.png", optimize=True)
    _ = (output / "resolved.json").write_text(resolved.model_dump_json(indent=2) + "\n")
    for path in paths:
        with Image.open(path) as image:
            if image.getbbox() is None:
                message = f"Empty rendered layout: {path}"
                raise ValueError(message)
    return tuple(paths)


def main() -> int:
    if len(sys.argv) != 4:
        print("Usage: python3 -m art.materials.preview STYLE MATERIAL OUTPUT_DIRECTORY", file=sys.stderr)
        return 2
    paths = render_layouts(Path(__file__).resolve().parents[2], sys.argv[1],sys.argv[2],Path(sys.argv[3]))
    print(json.dumps({"rendered": [str(path) for path in paths], "count": len(paths)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
