"""Export native-course stone pieces beside immutable shipped W2 log pieces."""

from __future__ import annotations

import hashlib
import io
import json
import subprocess
from collections.abc import Mapping
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.batch_c3a.walls_pass2.drafts import write_drafts
from art.batch_c3a.walls_pass2.native import (
    compose_door,
    compose_piece,
    course_field,
    crop_scores,
)
from art.batch_c3a.walls_pass2.review_boards import JoinView, plans, write_zooms

ROOT: Final = Path(__file__).resolve().parents[3]
REFERENCE: Final = "d086d7c54d1b29421c9f7540f3a9358f0c054314"
PREFIX: Final = "assets/v8art/materials/textures/log/pieces/"
BITS: Final = ((1, (0, -1)), (2, (1, 0)), (4, (0, 1)), (8, (-1, 0)))
CAPS: Final = {
    "end_n": 1,
    "end_e": 2,
    "end_s": 4,
    "end_w": 8,
    "corner_ne": 3,
    "corner_se": 6,
    "corner_nw": 9,
    "corner_sw": 12,
}


def digest(path: Path) -> str:
    """Hash complete file bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def log_image(name: str) -> Image.Image:
    """Decode the actual shipped W2 bytes without changing another worktree."""
    raw = subprocess.check_output(
        ["git", "show", f"{REFERENCE}:{PREFIX}{name}.png"], cwd=ROOT
    )
    with Image.open(io.BytesIO(raw)) as image:
        return image.convert("RGBA")


def topology(cell: tuple[int, int], cells: frozenset[tuple[int, int]]) -> int:
    """N1/E2/S4/W8 occupancy includes door cells."""
    x, y = cell
    return sum(bit for bit, (dx, dy) in BITS if (x + dx, y + dy) in cells)


def room(
    pieces: Mapping[int, Image.Image],
    cells: frozenset[tuple[int, int]],
    doors: Mapping[tuple[int, int], Image.Image],
) -> Image.Image:
    """Place independent64px pieces at the shipped origin/anchor without resizing."""
    image = Image.new("RGBA", (352, 304), "#d0ccbb")
    for x, y in sorted(cells, key=lambda c: (c[1], c[0])):
        tile = doors.get((x, y), pieces[topology((x, y), cells)])
        image.alpha_composite(tile, (16 + x * 64, 16 + y * 64))
    return image


def comparison(
    stone: Mapping[int, Image.Image],
    log: Mapping[int, Image.Image],
    doors: Mapping[str, Image.Image],
    stone_doors: Mapping[str, Image.Image],
) -> Image.Image:
    """Show5×4, L and T/X plans against the shipped log rooms at identical pixel density."""
    board = Image.new("RGBA", (704, 984), "#d0ccbb")
    draw = ImageDraw.Draw(board)
    for row, plan in enumerate(plans()):
        placed_doors = (
            {plan.ew: doors[f"ew_{plan.state}"], (0, 1): doors[f"ns_{plan.state}"]}
            if plan.ew is not None
            else {}
        )
        for col, (material, pieces) in enumerate(
            (("shipped log", log), ("native stone", stone))
        ):
            board.alpha_composite(
                room(
                    pieces,
                    plan.cells,
                    {
                        cell: (stone_doors if col else doors)[
                            ("ew" if cell == plan.ew else "ns") + "_" + plan.state
                        ]
                        for cell in placed_doors
                    },
                ),
                (col * 352, row * 328 + 20),
            )
            draw.text(
                (col * 352 + 10, row * 328 + 5),
                f"{plan.name}: {material} / 1x",
                fill="#302923",
            )
    return board


def export(source: Path, output: Path, *, approved: bool = False) -> None:
    """Write a new immutable export; candidates require separate visual approval."""
    output.mkdir(parents=True, exist_ok=False)
    field, selected = course_field(source)
    field.save(output / "stone_course_native.png")
    log = {mask: log_image(f"mask_{mask:02}") for mask in range(16)}
    stone = {mask: compose_piece(log[mask], field) for mask in range(16)}
    for mask, image in stone.items():
        image.save(output / f"mask_{mask:02}.png")
    for name, mask in CAPS.items():
        stone[mask].save(output / f"{name}.png")
    doors = {
        f"{orientation}_{state}": log_image(f"door_{orientation}_{state}")
        for orientation in ("ew", "ns")
        for state in ("closed", "open")
    }
    stone_doors = {
        name: compose_door(f"door_{name}", image, field)
        for name, image in doors.items()
    }
    for name, image in stone_doors.items():
        image.save(output / f"door_{name}.png")
    board = comparison(stone, log, doors, stone_doors)
    board.convert("RGB").save(output / "rooms_1x.png")
    board.resize((board.width * 2, board.height * 2), Image.Resampling.NEAREST).convert(
        "RGB"
    ).save(output / "rooms_2x.png")
    views: list[JoinView] = []
    for plan in plans():
        placed = (
            {plan.ew: doors[f"ew_{plan.state}"], (0, 1): doors[f"ns_{plan.state}"]}
            if plan.ew is not None
            else {}
        )
        images = {
            "shipped log": room(log, plan.cells, placed),
            "native stone": room(
                stone,
                plan.cells,
                {
                    cell: stone_doors[
                        ("ew" if cell == plan.ew else "ns") + "_" + plan.state
                    ]
                    for cell in placed
                },
            ),
        }
        for cell in sorted(plan.cells, key=lambda c: (c[1], c[0])):
            mask = topology(cell, plan.cells)
            if mask in (3, 6, 9, 12, 7, 11, 13, 14, 15) and cell not in placed:
                views.append(JoinView(plan.name, cell, mask, images))
    write_zooms(tuple(views), output)
    contact = Image.new("RGBA", (512, 424), "#d0ccbb")
    draw = ImageDraw.Draw(contact)
    for mask, image in stone.items():
        x, y = mask % 4 * 128, mask // 4 * 92 + 16
        contact.alpha_composite(image, (x + 32, y))
        draw.text((x + 5, y - 14), f"mask{mask:02}", fill="#302923")
    for index, (name, image) in enumerate(doors.items()):
        contact.alpha_composite(stone_doors[name], (index * 128 + 32, 362))
        draw.text((index * 128 + 4, 347), name, fill="#302923")
    contact.convert("RGB").save(output / "contact_1x.png")
    contact.resize((1024, 848), Image.Resampling.NEAREST).convert("RGB").save(
        output / "contact_2x.png"
    )
    rows = [
        {"path": str(path.relative_to(output)), "sha256": digest(path)}
        for path in sorted(output.rglob("*.png"))
    ]
    manifest = {
        "status": "LEADER_AND_INDEPENDENT_VISUAL_APPROVED_NOT_IMPORTED"
        if approved
        else "HOLD_REFERENCE_DOOR_GAP_NOT_IMPORTED",
        "era": "iron",
        "style": "drystone",
        "material": "granite",
        "canvas": [64, 64],
        "anchor": [32, 59],
        "world_origin": [0, -16],
        "wall_width": 26,
        "width_policy": "Exact shipped alpha for every pixel; no added junction fill",
        "cap_height": 16,
        "face_height": 22,
        "face_policy": "Exact shipped stencil; native world-periodic RGB only",
        "bits": {"N": 1, "E": 2, "S": 4, "W": 8},
        "course": {
            "source": str(source.relative_to(ROOT)),
            "sha256": digest(source),
            "native_crop": asdict(selected),
            "operation": "crop64x32 at native1:1; repeat twice vertically; no resampling, blur, recolouring or seam blending",
            "selection_rule": "minimum worst horizontal/vertical edge RGB discontinuity, then sum,y,x; all phases in external evidence",
        },
        "reference": {
            "commit": subprocess.check_output(
                ["git", "rev-parse", REFERENCE], cwd=ROOT, text=True
            ).strip(),
            "paths": PREFIX + "{mask_00..15,door_ew/ns_open/closed}.png",
            "door_policy": "Shipped RGBA outside explicit fixed frame regions; frame RGB stone, alpha exact.",
            "caps_policy": "No separate cap PNG exists in source tree; cap exports use named mask stencils, verified pixel-identical to that selected source mask.",
        },
        "composition": "Shipped alpha unchanged. Native64px-period stone at world phase (tile*64+local);1px RGB edge shade, low-alpha contact/shadow RGBA retained.",
        "caps": CAPS,
        "acceptance_limit": "Exact alpha and floor parity; inherited64x60 NS door leaves4px gap at64px stride. Literal no-background criterion is HELD, never repaired by altering alpha.",
        "pass": 4,
        "join_zoom_count": len(views),
        "join_zoom_policy": "Every corner/T/X instance in all three rooms, with16px neighbour context, nearest4x",
        "outputs": rows,
        "capability": {
            "status": "VISUAL_APPROVED_DRAFT_NOT_APPLIED"
            if approved
            else "HOLD_REFERENCE_DOOR_GAP_NOT_APPLIED",
            "runtime_rows_to_append": [],
            "canvas": [64, 64],
            "anchor": [32, 59],
            "pieces": {str(mask): f"mask_{mask:02}.png" for mask in range(16)},
            "doors": {
                orientation: {
                    state: f"door_{orientation}_{state}.png"
                    for state in ("closed", "open")
                }
                for orientation in ("ew", "ns")
            },
        },
    }
    _ = (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    write_drafts(output, approved)
    evidence = Path("/Users/rexxa/work/art-batch/C3A/pass4")
    evidence.mkdir(parents=True, exist_ok=True)
    _ = (evidence / "wall_crop_scores.json").write_text(
        json.dumps(
            {
                "selected": asdict(selected),
                "all_phases": [asdict(score) for score in crop_scores(source)],
            },
            indent=2,
        )
        + "\n"
    )
