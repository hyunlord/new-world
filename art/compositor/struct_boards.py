# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic --with typer python -m art.compositor.struct_boards
"""Lossless same-crop comparisons and explicitly declared detail demonstrations."""
from __future__ import annotations

import json
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Final

from PIL import Image, ImageChops, ImageDraw, ImageFont
from pydantic import JsonValue
from typing_extensions import override

from art.compositor.assets import REPO, choose_look, frame_path, read_image
from art.compositor.drawing import ForegroundPainter
from art.compositor.geometry import Viewport
from art.compositor.ground import GroundPainter
from art.compositor.models import Activity, InputModel, Overlay, Scene
from art.compositor.placements import RenderPerson, Sprite, render_people
from art.compositor.render import sha256
from art.compositor.struct_render import (
    DOCS,
    ENV1,
    EXTERNAL,
    SCENES,
    save_lossless,
    struct_catalog,
)
from art.compositor.tone import Condition
from art.compositor.wall_style import CapStyle, WallParameters

FONT: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 18)
SMALL: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
BOARDS: Final = REPO / "art/boards"


@dataclass(frozen=True, slots=True)
class Panel:
    label: str
    path: Path
    crop: tuple[int, int, int, int]
    scale: int = 1


def make_board(destination: Path, panels: tuple[Panel, ...]) -> None:
    """Preserve 1x pixels, or enlarge exactly 2x without quantization."""
    widths = [(row.crop[2] - row.crop[0]) * row.scale for row in panels]
    height = max((row.crop[3] - row.crop[1]) * row.scale for row in panels)
    board = Image.new("RGB", (sum(widths), height + 72), (24, 28, 25))
    draw = ImageDraw.Draw(board)
    records: list[dict[str, JsonValue]] = []
    x = 0
    for panel, width in zip(panels, widths, strict=True):
        with Image.open(panel.path) as source:
            cropped = source.convert("RGB").crop(panel.crop)
        if panel.scale != 1:
            cropped = cropped.resize((width, cropped.height * panel.scale), Image.Resampling.NEAREST)
        board.paste(cropped, (x, 38))
        draw.text((x + 8, 10), panel.label, font=FONT, fill=(237, 229, 208))
        records.append({"path": str(panel.path), "sha256": sha256(panel.path), "crop": list(panel.crop),
                        "integer_scale": panel.scale, "board_origin": [x, 38]})
        x += width
    draw.text((10, height + 48), "CANDIDATE / OWNER VERDICT PENDING. Details may be declared demonstrations; see panel labels and sidecar provenance.", font=SMALL, fill=(212, 212, 194))
    destination.parent.mkdir(parents=True, exist_ok=True)
    save_lossless(board, destination)
    _ = destination.with_suffix(".json").write_text(json.dumps({"panels": records, "sha256": sha256(destination),
        "bytes": destination.stat().st_size, "owner_approval": "PENDING", "operation": "same-coordinate crops; integer nearest enlargement only"}, indent=2) + "\n")


class RenderAssets(InputModel):
    """Input hashes from the canonical image underlying every detail demonstration."""

    asset_hashes: dict[str, str]


class WorkshopPainter(ForegroundPainter):
    """Use Route I work poses at benches without the ENV1 chopping stump prop."""

    @override
    def chopping_context(self, actor: RenderPerson, foot: tuple[int, int]) -> None:
        return None


def detail_demos(*, external: Path = EXTERNAL, docs: Path = DOCS,
                 cap_style: CapStyle = "dark", wall_params: WallParameters | None = None) -> tuple[Path, Path, Path]:
    """Show requested occupants/workers separately from canonical scene comparisons."""
    source = external / "scene_day6_noon_day_1x_figures.png"
    scene = Scene.read(SCENES / "scene_day6_noon.json")
    overlay = Overlay.read(SCENES / "m5_overlay.json")
    catalog = struct_catalog()
    frozen = RenderAssets.model_validate_json(source.with_suffix(".json").read_bytes())
    if frozen.asset_hashes != catalog.hashes:
        message = "Detail assets differ from canonical render inputs; re-render before building boards"
        raise ValueError(message)
    actors = render_people(scene, overlay)
    paths: list[Path] = []
    records: list[dict[str, JsonValue]] = []
    for name, offsets, activity in (("house", ((-10, -3), (-8, -3), (-7, -3)), Activity.SLEEP),
                                    ("workshop", ((-2, -4), (0, -3), (1, -3)), Activity.CHOP)):
        with Image.open(source) as original:
            image = original.convert("RGBA")
        painter = WorkshopPainter(image, Viewport(scene.camp, 1), catalog, False)
        actor_records: list[dict[str, JsonValue]] = []
        extra_fixtures: list[dict[str, JsonValue]] = []
        if name == "workshop":
            fixture = Sprite("detail_middle_bench", "woodworking_bench", (scene.camp[0], scene.camp[1] - 4 + 11 / 64), True)
            record = painter.sprite(fixture)
            if record is not None:
                extra_fixtures.append(record)
        for actor, offset in zip(actors[:3], offsets, strict=True):
            sleep_lift = 3 / 64 if name == "house" else 0
            replacement = replace(actor, position=(scene.camp[0] + offset[0], scene.camp[1] + offset[1] - sleep_lift), activity=activity, declared=True, mark="")
            row = painter.person(replacement)
            if row is not None:
                if name == "house":
                    base = Viewport(scene.camp, 1).project((scene.camp[0] + offset[0], scene.camp[1] + offset[1]))
                    bed = catalog.image("bedroll")
                    anchor = catalog.asset("bedroll").anchor
                    if anchor is None:
                        message = "Occupied bed requires declared anchor"
                        raise ValueError(message)
                    bed_mask = Image.new("L", image.size)
                    bed_mask.paste(bed.getchannel("A"), (base[0] - round(anchor[0]), base[1] + 11 - round(anchor[1])))
                    person_mask = Image.new("L", image.size)
                    figure = read_image(frame_path(choose_look(actor.person, actor.top), "sleep"))
                    person_mask.paste(figure.getchannel("A"), (base[0] - 56, base[1] + 11 - 3 - 72))
                    threshold = [255 if value >= 128 else 0 for value in range(256)]
                    overlap = ImageChops.multiply(bed_mask.point(threshold), person_mask.point(threshold)).histogram()[255]
                    row["support_asset"] = "bedroll"
                    row["sleep_offset_from_bed_base_px"] = [0, -3]
                    row["opaque_overlap_with_bed_pixels"] = overlap
                    if overlap == 0:
                        message = "Declared sleeper does not overlap the bed"
                        raise ValueError(message)
                actor_records.append(row)
        path = external / f"detail_demo_{name}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        image.convert("RGB").save(path, optimize=True)
        paths.append(path)
        records.append({"name": name, "source_sha256": sha256(source), "path": str(path), "sha256": sha256(path),
                        "actors": [row for row in actor_records], "extra_fixtures": [row for row in extra_fixtures], "catalog_hashes": {key: value for key, value in catalog.hashes.items()}, "interpretation": "Additional illustrative copies of three real IDs; not a population observation or canonical overlay change"})
    hut = Image.new("RGBA", (224, 224), (42, 43, 35, 255))
    floor = GroundPainter(catalog, Viewport(scene.camp, 1), Condition.DAY).tile("floor_packed_earth")
    for y in range(3):
        for x in range(3):
            hut.alpha_composite(floor, (16 + x * 64, 32 + y * 64))
    look = choose_look(actors[0].person, actors[0].top)
    figure_path = frame_path(look, "idle")
    hut.alpha_composite(read_image(figure_path), (112 - 40, 139 - 104))
    from art.compositor.geometry import perimeter
    from art.compositor.struct_layers import load_wall_textures, wall_door
    from art.compositor.walls import Door, render_walls

    chunk_path = external / "detail_demo_hut_wall_chunk.png"
    chunk = render_walls(perimeter((3, 3)), load_wall_textures("log"),
                         (Door((1, 2), wall_door(catalog, "door_log_open")),),
                         cap_style=cap_style, params=wall_params)
    chunk.image.save(chunk_path, optimize=True)
    hut.alpha_composite(read_image(chunk_path), (16, 16))
    hut_path = external / "detail_demo_hut.png"
    hut.convert("RGB").save(hut_path, optimize=True)
    records.append({"name": "roofless_hut", "path": str(hut_path), "sha256": sha256(hut_path),
                    "chunk_path": str(chunk_path), "chunk_sha256": sha256(chunk_path),
                    "figure_path": str(figure_path), "figure_sha256": sha256(figure_path),
                    "floor_path": catalog.asset("floor_packed_earth").path,
                    "interpretation": "Declared3x3 hut fixture; no such completed hut exists in the three real scenes"})
    _ = (docs / "detail_demo_manifest.json").write_text(json.dumps({"demonstrations": records}, indent=2) + "\n")
    return paths[0], paths[1], hut_path


def main() -> None:
    crop = (0, 140, 960, 740)
    renders = EXTERNAL
    make_board(BOARDS / "struct_owner_board.png", (
        Panel("Current renderer / day6 noon / 1x", ENV1 / "reference/current_day6_noon_1x.png", crop),
        Panel("ENV1 / same scene / 1x", ENV1 / "renders/scene_day6_noon_day_1x_figures.png", crop),
        Panel("STRUCT / same scene / 1x", renders / "scene_day6_noon_day_1x_figures.png", crop),
    ))
    house, workshop, hut = detail_demos()
    day = renders / "scene_day6_noon_day_1x_figures.png"
    make_board(BOARDS / "struct_closeups_2x.png", (
        Panel("House / declared occupants / 2x", house, (0, 130, 448, 490), 2),
        Panel("Stone store / 2x", day, (0, 450, 320, 730), 2),
        Panel("Dug pocket / 2x", day, (448, 130, 640, 354), 2),
        Panel("River + ford / 2x", day, (544, 330, 736, 458), 2),
        Panel("Workshop / 3 declared workers / 2x", workshop, (608, 130, 928, 386), 2),
        Panel("3x3 hut / declared fixture / 2x", hut, (0, 0, 224, 224), 2),
    ))
    make_board(BOARDS / "struct_conditions.png", (
        Panel("Night / 1x", renders / "scene_day6_night_night_1x_figures.png", (0, 150, 720, 710)),
        Panel("Night / half + LOD", renders / "scene_day6_night_night_half_lod.png", (400, 328, 760, 608)),
        Panel("Winter / 1x", renders / "scene_winter_noon_winter_1x_figures.png", (0, 150, 720, 710)),
        Panel("Winter / half + LOD", renders / "scene_winter_noon_winter_half_lod.png", (400, 328, 760, 608)),
    ))


if __name__ == "__main__":
    main()
