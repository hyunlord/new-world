"""Shipped stencil parity, wooden leaves, native phase and room interiors."""

from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.batch_c3a.walls_pass2.build import CAPS, log_image, room, topology
from art.batch_c3a.walls_pass2.native import (
    compose_door,
    compose_piece,
    course_field,
    door_frame,
)
from art.batch_c3a.walls_pass2.review_boards import plans

NAMES = (
    [f"mask_{mask:02}" for mask in range(16)]
    + list(CAPS)
    + [
        f"door_{orientation}_{state}"
        for orientation in ("ew", "ns")
        for state in ("closed", "open")
    ]
)


def source_piece(name: str) -> Image.Image:
    """Caps explicitly reference actual shipped mask bytes, not nonexistent PNGs."""
    return log_image(f"mask_{CAPS[name]:02}" if name in CAPS else name)


@pytest.mark.parametrize("name", NAMES)
def test_all_28_piece_alpha_and_contact_are_exact(name: str) -> None:
    source = source_piece(name)
    field = Image.new("RGBA", (64, 64), (160, 130, 100, 255))
    result = (
        compose_door(name, source, field)
        if name.startswith("door_")
        else compose_piece(source, field)
    )
    assert result.getchannel("A").tobytes() == source.getchannel("A").tobytes()
    before, after = np.asarray(source), np.asarray(result)
    shadow = before[..., 3] < 96
    assert np.array_equal(before[shadow], after[shadow])
    if name.startswith("door_"):
        wooden = np.equal(
            np.asarray(door_frame(name).crop((0, 0, *source.size)), dtype=np.uint8), 0
        )
        assert np.array_equal(before[wooden], after[wooden])
        assert bool(np.any(np.not_equal(before[~wooden, :3], after[~wooden, :3])))
        # Fixed, independently inspected moving-leaf and hardware probes.
        probes = (
            ((30, 20), (35, 40))
            if "_ew_closed" in name
            else ((47, 20), (49, 40))
            if "_ew_open" in name
            else ((30, 28), (40, 38))
            if "_ns_closed" in name
            else ((32, 15), (45, 17))
        )
        for point in probes:
            assert result.getpixel(point) == source.getpixel(point)
        fixed = (
            (10, 25)
            if "_ew_" in name
            else (32, 9)
            if name.endswith("closed")
            else (32, 52)
        )
        assert result.getpixel(fixed) != source.getpixel(fixed)


def test_native_crop_and_world_phase() -> None:
    source = Path("art/materials/textures/drystone/face.png")
    field, selection = course_field(source)
    with Image.open(source) as original:
        crop = original.convert("RGBA").crop(
            (selection.x, selection.y, selection.x + 64, selection.y + 32)
        )
    assert field.crop((0, 0, 64, 32)).tobytes() == crop.tobytes()
    assert field.crop((0, 32, 64, 64)).tobytes() == crop.tobytes()
    stencil = Image.new("RGBA", (64, 64), (0, 0, 0, 255))
    a = compose_piece(stencil, field)
    b = compose_piece(stencil, field, tile=(7, -3))
    assert a.tobytes() == b.tobytes()
    assert a.getpixel((31, 9)) == field.getpixel((31, 9))
    assert a.getpixel((31, 25)) == field.getpixel((31, 25))


def test_every_room_retains_floor_and_all_join_alpha() -> None:
    field = Image.new("RGBA", (64, 64), (160, 130, 100, 255))
    log = {mask: log_image(f"mask_{mask:02}") for mask in range(16)}
    stone = {mask: compose_piece(image, field) for mask, image in log.items()}
    for plan in plans():
        before = Image.new("RGBA", (352, 304))
        after = before.copy()
        source_doors = (
            {}
            if plan.ew is None
            else {
                plan.ew: log_image(f"door_ew_{plan.state}"),
                (0, 1): log_image(f"door_ns_{plan.state}"),
            }
        )
        stone_doors = {
            cell: compose_door(
                "door_" + ("ew" if cell == plan.ew else "ns") + "_" + plan.state,
                image,
                field,
            )
            for cell, image in source_doors.items()
        }
        for x, y in sorted(plan.cells):
            mask = topology((x, y), plan.cells)
            before.alpha_composite(
                source_doors.get((x, y), log[mask]), (16 + x * 64, 16 + y * 64)
            )
            after.alpha_composite(
                stone_doors.get((x, y), stone[mask]), (16 + x * 64, 16 + y * 64)
            )
        assert before.getchannel("A").tobytes() == after.getchannel("A").tobytes()
        assert room(log, plan.cells, {}).size == (352, 304)


def test_invalid_canvas_is_rejected() -> None:
    with pytest.raises(ValueError, match="64×64"):
        _ = compose_piece(Image.new("RGBA", (32, 32)), Image.new("RGBA", (64, 64)))


def test_real_export_preserves_all_28_alpha(tmp_path: Path) -> None:
    from art.batch_c3a.walls_pass2.build import export

    output = tmp_path / "candidate"
    export(Path("art/materials/textures/drystone/face.png").resolve(), output)
    for name in NAMES:
        with Image.open(output / f"{name}.png") as result:
            source = source_piece(name)
            assert result.size == source.size
            assert result.getchannel("A").tobytes() == source.getchannel("A").tobytes()


def test_top_left_edge_light_changes_only_rgb() -> None:
    source = Image.new("RGBA", (64, 64))
    source.paste((100, 80, 60, 210), (10, 10, 54, 54))
    field = Image.new("RGBA", (64, 64), (160, 130, 100, 255))
    result = compose_piece(source, field)
    assert result.getpixel((30, 10)) == (136, 106, 76, 210)
    assert result.getpixel((30, 53)) == (112, 82, 52, 210)
    assert result.getpixel((30, 30)) == (160, 130, 100, 210)
    assert result.getchannel("A").tobytes() == source.getchannel("A").tobytes()


def test_unknown_door_is_rejected() -> None:
    with pytest.raises(ValueError, match="unknown shipped door"):
        _ = door_frame("door_diagonal_closed")
