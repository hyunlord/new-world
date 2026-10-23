# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic --with typer python -m art.compositor.struct_render
"""STRUCT rendering reuses frozen ENV1 sources, foreground, tones, HUD and LOD."""
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
from dataclasses import replace
from pathlib import Path
from typing import Final, assert_never

import typer
from PIL import Image, ImageDraw
from pydantic import JsonValue

from art.compositor.assets import REPO, Catalog, load_catalog
from art.compositor.drawing import ForegroundPainter, site_overlay
from art.compositor.geometry import HUD_HEIGHT, WINDOW, Viewport
from art.compositor.ground import GroundPainter
from art.compositor.models import Overlay, Scene
from art.compositor.native_layers import DESTINATION
from art.compositor.placements import RenderPerson, Sprite, render_people
from art.compositor.render import (
    FONT,
    RenderInputs,
    RenderProduct,
    RenderSettings,
    sha256,
)
from art.compositor.struct_ground import paint_ground
from art.compositor.struct_layers import StructureTile, build_layout, structural_tiles
from art.compositor.tone import Condition, NightLight, night_value
from art.compositor.wall_style import CapStyle, WallParameters

DOCS: Final = REPO / "docs/v8/artstruct"
ENV1: Final = REPO / "docs/v8/artenv1"
SCENES: Final = REPO / "art/compositor/scenes"
EXTERNAL: Final = Path("/Users/rexxa/work/artstruct-data/renders")


def struct_catalog() -> Catalog:
    manifests = (ENV1 / "built_assets.json", ENV1 / "items_manifest.json", DOCS / "assets.json", DOCS / "ford_assets.json")
    items = DOCS / "item_assets.json"
    return load_catalog(manifests + ((items,) if items.exists() else ()))



def paint_structure(painter: ForegroundPainter, row: StructureTile) -> dict[str, JsonValue] | None:
    """Draw only the declared cap/face footprint and update real figure visibility."""
    viewport = painter.viewport
    if not viewport.visible(row.position, round(64 * viewport.zoom)):
        return None
    source = site_overlay(row.tile.image) if row.site else row.tile.image
    scaled = source.resize((round(source.width * viewport.zoom), round(source.height * viewport.zoom)), Image.Resampling.NEAREST)
    px, py = viewport.project(row.position)
    origin = (px - viewport.tile // 2, py - round(48 * viewport.zoom))
    painter.image.alpha_composite(scaled, origin)
    if painter.coverage is not None:
        painter.coverage.paste(0, origin, scaled.getchannel("A").point([255 if value >= 128 else 0 for value in range(256)]))
    return {"source": row.source, "cell": list(row.tile.cell), "origin": list(origin),
            "face_visible": row.tile.face_visible, "site": row.site}


def compose(inputs: RenderInputs, settings: RenderSettings, *, cap_style: CapStyle = "dark",
            wall_params: WallParameters | None = None) -> RenderProduct:
    """Re-compose, retaining original scene coordinates and declared actor identities."""
    scene, catalog = inputs.scene, inputs.catalog
    viewport = Viewport(scene.camp, settings.zoom)
    layout = build_layout(scene, inputs.overlay, catalog)
    ground = paint_ground(scene, layout, GroundPainter(catalog, viewport, settings.condition), wall_params)
    image = ground.image
    coverage = Image.new("L", WINDOW)
    painter = ForegroundPainter(image, viewport, catalog, settings.lod, coverage)
    actors = render_people(scene, inputs.overlay)
    labels = {actor.person.id: index + 1 for index, actor in enumerate(actors)}
    walls = structural_tiles(layout, catalog, (scene.width, scene.height), cap_style=cap_style, params=wall_params)
    layers: list[Sprite | RenderPerson | StructureTile] = [*layout.sprites, *actors, *walls]
    objects: list[dict[str, JsonValue]] = []
    people: list[dict[str, JsonValue]] = []
    for row in sorted(layers, key=lambda item: (item.position[1], item.position[0])):
        match row:
            case StructureTile():
                record = paint_structure(painter, row)
                if record is not None:
                    objects.append(record)
                continue
            case Sprite():
                # ENV1 animals already use +11; standing objects now share that base.
                placed = row if row.category == "animal" else replace(row, position=(row.position[0], row.position[1] + 11 / 64))
                record = painter.sprite(placed)
                if record is not None:
                    record["position"] = list(row.position)
                    record["projection_base_offset_px"] = 11
                    objects.append(record)
                continue
            case RenderPerson():
                record = painter.person(row, labels[row.person.id])
                if record is not None:
                    people.append(record)
                continue
        assert_never(row)
    coverage.paste(0, (0, 0, WINDOW[0], HUD_HEIGHT))
    coverage.paste(0, (0, WINDOW[1] - 23, WINDOW[0], WINDOW[1]))
    histogram = coverage.histogram()
    for person in people:
        person_id = person["id"]
        if not isinstance(person_id, int):
            message = "Person evidence lost its real integer identity"
            raise TypeError(message)
        person["visible_opaque_pixels"] = histogram[labels[person_id]]
        person["pixel_visible"] = histogram[labels[person_id]] > 0
    hearths = tuple(viewport.project(row.position) for row in layout.sprites if row.asset in {"hearth_1", "hearth_pots"})
    if settings.condition is Condition.NIGHT:
        image = night_value(image, NightLight(hearths, viewport.tile))
    with Image.open(inputs.hud_path) as hud:
        image.paste(hud.convert("RGB"), (0, 0))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, WINDOW[1] - 23, WINDOW[0], WINDOW[1]), fill=(25, 28, 26, 255))
    supplemental = settings.zoom < .5 or settings.condition is Condition.DROUGHT
    label = "SUPPLEMENTAL" if supplemental else "CANONICAL"
    draw.text((10, WINDOW[1] - 19), f"STRUCT {label} CANDIDATE | Frozen ENV1 real scene + same DECLARED overlay | Code walls/edges; owner verdict pending", font=FONT, fill=(211, 211, 197))
    evidence: dict[str, JsonValue] = {
        "scene_id": scene.scene_id, "tick": scene.tick, "source_display_day": scene.display_day,
        "source_clock_minute": scene.clock_minute, "source_season": scene.season,
        "condition": settings.condition.value, "zoom": settings.zoom, "lod": settings.lod, "supplemental": supplemental,
        "people": [row for row in people], "objects": [row for row in objects], "source_population": len(scene.people),
        "boundary_metrics": list(ground.boundary_metrics),
        "boundary_pass": all(bool(row["passed"]) for row in ground.boundary_metrics),
        "rooms": [{"id": room.id, "cells": [list(cell) for cell in sorted(room.cells)],
                   "door": list(room.door) if room.door is not None else None, "material": room.material} for room in layout.rooms],
        "hud_path": str(inputs.hud_path.relative_to(REPO)), "hud_sha256": sha256(inputs.hud_path),
        "overlay_path": str((SCENES / "m5_overlay.json").relative_to(REPO)), "overlay_sha256": sha256(SCENES / "m5_overlay.json"),
        "native_layer_sha256": sha256(DESTINATION), "provenance": scene.provenance,
        "asset_hashes": {key: value for key, value in catalog.hashes.items()}, "owner_approval": "PENDING",
        "depth_rule": "base y then x; standing base tile top+43; stable order on ties",
        "roof": "not drawn", "native_hut_count": sum(room.id.startswith("real_hut_") for room in layout.rooms),
        "cap_style": cap_style, "wall_parameters": (wall_params or WallParameters.read()).model_dump(mode="json"),
    }
    return RenderProduct(image.convert("RGB"), evidence)


def save_lossless(image: Image.Image, path: Path, enforce_cap: bool = True) -> None:
    """Try installed lossless PNG filters; verify decoded pixels before replacement."""
    image.save(path, optimize=True, compress_level=9)
    if enforce_cap and path.stat().st_size > 1_500_000:
        EXTERNAL.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="png-lossless-", dir=EXTERNAL) as temporary:
            compressed = Path(temporary) / "compressed.png"
            _ = subprocess.run(["xcrun", "pngcrush", "-q", "-brute", str(path), str(compressed)],
                               check=True, capture_output=True, timeout=180)
            with Image.open(compressed) as decoded:
                if decoded.convert("RGBA").tobytes() != image.convert("RGBA").tobytes():
                    message = f"Lossless compression changed pixels: {path}"
                    raise ValueError(message)
            if compressed.stat().st_size < path.stat().st_size:
                _ = shutil.copyfile(compressed, path)
    if enforce_cap and path.stat().st_size > 1_500_000:
        message = f"Lossless render exceeds 1.5MB: {path}: {path.stat().st_size}"
        raise ValueError(message)


def main(output: Path = EXTERNAL) -> None:
    catalog = struct_catalog()
    overlay = Overlay.read(SCENES / "m5_overlay.json")
    snapshot = dict(catalog.hashes)
    items_present = (DOCS / "item_assets.json").exists()
    input_paths = (*SCENES.glob("*.json"), *ENV1.glob("reference/*_hud.png"),
                   *Path(__file__).parent.glob("struct_*.py"), Path(__file__).parent / "walls.py",
                   Path(__file__).parent / "edges.py")
    snapshot.update((str(path.relative_to(REPO)), sha256(path)) for path in input_paths)
    output.mkdir(parents=True, exist_ok=True)
    EXTERNAL.mkdir(parents=True, exist_ok=True)
    rows: list[dict[str, JsonValue]] = []
    for name, condition in (("day6_noon", Condition.DAY), ("day6_night", Condition.NIGHT), ("winter_noon", Condition.WINTER), ("day6_noon", Condition.DROUGHT)):
        scene = Scene.read(SCENES / f"scene_{name}.json")
        for zoom in (1.0, .5, .25):
            suffix = "1x" if zoom == 1 else "half"
            hud = ENV1 / f"reference/current_{name}_{suffix}_hud.png"
            settings = RenderSettings(zoom, condition, True)
            product = compose(RenderInputs(scene, overlay, catalog, hud), settings)
            path = output / f"scene_{name}_{settings.suffix}.png"
            save_lossless(product.image, path, enforce_cap=path.is_relative_to(REPO))
            evidence = EXTERNAL / f"{path.stem}.json"
            _ = evidence.write_text(json.dumps(product.evidence, indent=2) + "\n")
            rows.append({"path": str(path.relative_to(REPO)) if path.is_relative_to(REPO) else str(path), "sha256": sha256(path), "bytes": path.stat().st_size,
                         "evidence": str(evidence), "evidence_sha256": sha256(evidence),
                         "boundary_pass": product.evidence["boundary_pass"], "supplemental": product.evidence["supplemental"]})
            print(f"STRUCT {path.name} {path.stat().st_size} bytes", flush=True)
    changed = [path for path, digest in snapshot.items() if sha256(REPO / path) != digest]
    if items_present != (DOCS / "item_assets.json").exists():
        changed.append("docs/v8/artstruct/item_assets.json appeared/disappeared during render")
    if changed:
        message = f"Input bytes changed during rendering; outputs are not a frozen set: {changed}"
        raise ValueError(message)
    _ = (DOCS / "render_manifest.json").write_text(json.dumps({"renders": rows, "input_snapshot": snapshot, "inputs_unchanged": True, "owner_approval": "PENDING"}, indent=2) + "\n")


if __name__ == "__main__":
    typer.run(main)
