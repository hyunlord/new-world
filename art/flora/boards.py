# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: python3 -m art.flora.boards
"""Create owner review boards with native scales and source-bound panel records."""
from __future__ import annotations

import io
import json
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import typer
from PIL import Image, ImageDraw, ImageFont
from pydantic import JsonValue

from art.compositor.assets import REPO
from art.compositor.render import sha256
from art.flora.board_io import save_lossless
from art.flora.resolve_flora import read_catalog, resolve_flora
from art.flora.scene_board import compose_scene, seasonal_image, shadow
from art.flora.schema import FloraCatalog, Season, Size, Species, SpeciesDocument

EXTERNAL: Final = Path("/Users/rexxa/work/arttrees-data/boards")
DESTINATION: Final = REPO / "art/boards"
SEASONS: Final[tuple[Season, ...]] = ("spring", "summer", "autumn", "winter")
SIZES: Final[tuple[Size, ...]] = ("small", "large")
REFERENCE_REF: Final = "d796fcacd23d03959e2bd1046f69581a9b1fc637"
REFERENCE_PATH: Final = "docs/images/mart2_compare_sheet.png"
REFERENCE_CROPS: Final = ((976, 8, 1936, 548), (976, 3068, 1936, 3608))
FONT: Final = ImageFont.truetype("/System/Library/Fonts/AppleSDGothicNeo.ttc", 16)
SCENE_ROI: Final = (784, 352, 1408, 784)
SEASON_LABELS: Final = {"spring": "봄", "summer": "여름", "autumn": "가을", "winter": "겨울"}
PERSON: Final = "art/character/route_i/adult_M_fur_cloak_bun/S/idle.png"


@dataclass(frozen=True, slots=True)
class OwnerCase:
    species_id: str
    size: Size
    variant: int
    season: Season
    scale: int


def owner_schedule(catalog: FloraCatalog) -> tuple[OwnerCase, ...]:
    """Cover every tree frame at both exact integer scales in every season."""
    return tuple(OwnerCase(key, size, index, season, scale)
                 for key, species in sorted(catalog.species.species.items()) if species.habit != "shrub"
                 for size in SIZES for index in range(len(species.sizes[size].variants))
                 for season in SEASONS for scale in (1, 2))


def owner_board(catalog: FloraCatalog) -> Path:
    """Place untouched frame sizes and nearest-neighbour doubles beside a Route I adult."""
    schedule = owner_schedule(catalog)
    frames = [resolve_flora(case.species_id, case.size, case.season, case.variant, catalog=catalog)
              for case in schedule]
    max_width = max(frame.canvas[0] for frame in frames)
    cell_w = max_width * 3 + 240
    row_heights = [max(frame.canvas[1] for frame in frames[index:index + 8]) * 2 + 64
                   for index in range(0, len(frames), 8)]
    board = Image.new("RGB", (4 * cell_w, sum(row_heights) + 80), (139, 155, 105))
    draw = ImageDraw.Draw(board)
    draw.text((16, 12), "ART-TREES / 사계절 / 원본 1배 + 최근접 2배 / Route I 성인 56px / 오너 판단 대기", font=FONT, fill=(37, 47, 29))
    with Image.open(REPO / PERSON) as source:
        person = source.convert("RGBA")
    records: list[dict[str, JsonValue]] = []
    for index, (case, resolved) in enumerate(zip(schedule, frames, strict=True)):
        row, column = index // 8, (index % 8) // 2
        cell_h = row_heights[row]
        left, top = column * cell_w, sum(row_heights[:row]) + 60
        base = (left + (max_width // 2 + 18 if case.scale == 1 else max_width * 2 + 100), top + cell_h - 24)
        if case.scale == 1:
            draw.text((left + 12, top + 4), f"{case.species_id} / {case.size} / v{case.variant + 1} / {SEASON_LABELS[case.season]}", font=FONT, fill=(37, 47, 29))
        image = seasonal_image(resolved)
        image = image.resize((image.width * case.scale, image.height * case.scale), Image.Resampling.NEAREST)
        origin = (base[0] - resolved.anchor[0] * case.scale, base[1] - resolved.anchor[1] * case.scale)
        shade = shadow((max(8, round(resolved.canvas[0] * .7 * case.scale)), 18 * case.scale))
        board.paste(shade, (base[0] - shade.width // 2, base[1] - shade.height // 2), shade)
        board.paste(image, origin, image)
        actor = person.resize((person.width * case.scale, person.height * case.scale), Image.Resampling.NEAREST)
        person_origin = (origin[0] + image.width - 15 * case.scale, base[1] - 104 * case.scale)
        board.paste(actor, person_origin, actor)
        draw.text((base[0] - 12, base[1] + 4), f"{case.scale}x", font=FONT, fill=(37, 47, 29))
        records.append({**asdict(case), "path": resolved.path, "mask": resolved.canopy_mask,
                        "origin": list(origin), "anchor": list(resolved.anchor), "person_origin": list(person_origin),
                        "source_sha256": sha256(REPO / resolved.path), "tint": resolved.tint.model_dump(mode="json")})
    path = DESTINATION / "trees_owner.png"
    save_lossless(board, path, optimize_oversize=True)
    _ = path.with_suffix(".json").write_text(json.dumps({"cases": records, "person": PERSON,
        "person_sha256": sha256(REPO / PERSON), "sha256": sha256(path), "size": board.size,
        "background": "calm uniform meadow RGB 139,155,105", "owner_approval": "PENDING"}, indent=2) + "\n")
    return path


def reference_panels() -> tuple[Image.Image, Image.Image]:
    """Read the actual committed M-ART2 sheet, retaining its 50% preview scale."""
    raw = subprocess.check_output(["git", "show", f"{REFERENCE_REF}:{REFERENCE_PATH}"], cwd=REPO)
    with Image.open(io.BytesIO(raw)) as sheet:
        panels = tuple(sheet.convert("RGB").crop(box) for box in REFERENCE_CROPS)
    return panels[0], panels[1]


def panel_board(panels: tuple[tuple[str, Image.Image], ...], columns: int) -> Image.Image:
    """Keep every panel at its supplied pixel size; never downsample tree scenes."""
    width = max(image.width for _, image in panels)
    height = max(image.height for _, image in panels)
    rows = (len(panels) + columns - 1) // columns
    board = Image.new("RGB", (columns * (width + 16) + 16, rows * (height + 42) + 16), (31, 35, 28))
    draw = ImageDraw.Draw(board)
    for index, (label, image) in enumerate(panels):
        x, y = 16 + index % columns * (width + 16), 16 + index // columns * (height + 42)
        draw.text((x, y), label, font=FONT, fill=(235, 236, 212))
        rgba = image.convert("RGBA")
        board.paste(rgba, (x, y + 25), rgba)
    return board


def fixture_catalog(catalog: FloraCatalog, fixture: Path, species_id: str) -> FloraCatalog:
    """Parse a standalone species record and merge only data into the existing catalog."""
    species = Species.model_validate_json(fixture.read_bytes())
    if species.id != species_id:
        message = f"Requested id {species_id!r} differs from fixture id {species.id!r}"
        raise ValueError(message)
    document = SpeciesDocument(schema_version=1, species={**catalog.species.species, species.id: species},
                               map_canopies=catalog.species.map_canopies)
    return FloraCatalog(species=document, defaults=catalog.defaults)


def proof_strip(fixture: Path, species_id: str, output: Path) -> None:
    """Render an arbitrary data-only species in all four seasons at native pixels."""
    catalog = fixture_catalog(read_catalog(REPO), fixture, species_id)
    frames = tuple(resolve_flora(species_id, "large", season, 0, catalog=catalog) for season in SEASONS)
    images = tuple(seasonal_image(frame) for frame in frames)
    board = panel_board(tuple((f"{species_id} / {SEASON_LABELS[season]} / 1배", image)
                              for season, image in zip(SEASONS, images, strict=True)), 4)
    output.parent.mkdir(parents=True, exist_ok=True)
    save_lossless(board, output)
    _ = output.with_suffix(".json").write_text(json.dumps({"fixture": str(fixture),
        "fixture_sha256": sha256(fixture), "species_id": species_id, "seasons": list(SEASONS),
        "resolved": [frame.model_dump(mode="json") for frame in frames], "sha256": sha256(output),
        "python_sha256": {str(path.relative_to(REPO)): sha256(path) for path in sorted((REPO / "art/flora").glob("*.py"))},
        "owner_approval": "PENDING"}, indent=2) + "\n")
    print(str(output))


def main(fixture: Path | None = None, species_id: str = "", output: Path | None = None) -> None:
    """Generate all three requested boards, external full scenes and provenance."""
    if fixture is not None:
        proof_strip(fixture, species_id, output or DESTINATION / f"{species_id}_strip.png")
        return
    catalog = read_catalog(REPO)
    EXTERNAL.mkdir(parents=True, exist_ok=True)
    DESTINATION.mkdir(parents=True, exist_ok=True)
    owner = owner_board(catalog)
    scenes = {season: compose_scene(catalog, season) for season in SEASONS}
    quarter = compose_scene(catalog, "summer", .25)
    for season, product in (*scenes.items(), ("summer_quarter", quarter)):
        product.image.save(EXTERNAL / f"{season}.png", optimize=True)
        _ = (EXTERNAL / f"{season}.json").write_text(json.dumps(product.evidence, indent=2) + "\n")
    before_1x, before_quarter = reference_panels()
    comparison = panel_board((("M-ART2 1배 게임 화면 (기존 보드의 50% 미리보기)", before_1x),
        ("ART-TREES 여름 1배 원본 픽셀 / 오너 판단 대기", scenes["summer"].image),
        ("M-ART2 ¼배 게임 화면 (기존 보드의 50% 미리보기)", before_quarter),
        ("ART-TREES ¼배 / 빽빽한 숲은 수관 덩어리 / 원본 픽셀", quarter.image)), 2)
    seasonal = panel_board(tuple((f"{SEASON_LABELS[season]} / 동일 6일째 장면 / 원본 1배", scenes[season].image)
                                 for season in SEASONS), 2)
    for name, full_board in (("trees_scene", comparison), ("trees_seasons_scene", seasonal)):
        buffer = io.BytesIO()
        full_board.save(buffer, format="PNG", optimize=True, compress_level=9)
        cropped = len(buffer.getvalue()) > 1_500_000
        board = full_board
        if cropped:
            if name == "trees_seasons_scene":
                board = panel_board(tuple((f"{SEASON_LABELS[season]} / 동쪽 8타일 숲·마을 영역 / 원본 1배",
                    scenes[season].image.crop(SCENE_ROI)) for season in SEASONS), 2)
            else:
                board = panel_board((("M-ART2 1배 화면 / 보드 50% 미리보기 영역", before_1x.crop((240, 80, 960, 520))),
                    ("ART-TREES 여름 / 동쪽 8타일 숲·마을 영역 / 원본 1배", scenes["summer"].image.crop(SCENE_ROI)),
                    ("M-ART2 ¼배 화면 / 보드 50% 미리보기 영역", before_quarter.crop((80, 40, 880, 500))),
                    ("ART-TREES ¼배 / 수관 덩어리 / 고정 영역 원본 픽셀", quarter.image.crop((320, 180, 1280, 720)))), 2)
        path = DESTINATION / f"{name}.png"
        save_lossless(board, path, optimize_oversize=True)
        _ = path.with_suffix(".json").write_text(json.dumps({"sha256": sha256(path), "dimensions": board.size,
            "reference_commit": REFERENCE_REF, "reference_path": REFERENCE_PATH, "reference_crops": REFERENCE_CROPS,
            "reference_panel_inner_crops": ((240, 80, 960, 520), (80, 40, 880, 500)) if cropped else None,
            "scene": "art/compositor/scenes/scene_day6_noon.json",
            "scene_sha256": sha256(REPO / "art/compositor/scenes/scene_day6_noon.json"),
            "renders": [product.evidence for product in scenes.values()] + [quarter.evidence],
            "native_scene_crop": SCENE_ROI if cropped else (0, 0, 1600, 900),
            "quarter_crop": (320, 180, 1280, 720) if cropped else (0, 0, 1600, 900),
            "crop_reason": "1.5MB cap; fixed forest/camp ROI; no native-pixel downsampling" if cropped else None,
            "full_native_renders": str(EXTERNAL), "owner_approval": "PENDING"}, indent=2) + "\n")
    print(json.dumps({"owner": str(owner), "boards": ["trees_scene.png", "trees_seasons_scene.png"]}))


if __name__ == "__main__":
    typer.run(main)
